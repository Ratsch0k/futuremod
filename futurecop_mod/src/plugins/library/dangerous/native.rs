use std::{arch::asm, collections::HashMap};

use anyhow::{anyhow, bail};
use log::{debug, warn};
use mlua::{Lua, MetaMethod, UserData};
use windows::Win32::System::Memory::{VirtualAlloc, MEM_COMMIT, MEM_RESERVE, PAGE_EXECUTE_READWRITE};

use crate::plugins::library::dangerous::{lua_to_native, lua_to_native_implied, native_to_lua};

use super::Type;

pub struct NativeFunction {
  // Generic native closure that wraps a lua function
  address: u32,
  arg_types: Vec<Type>,
  return_type: Type,
}

impl NativeFunction {
  pub fn new(address: u32, arg_types: Vec<Type>, return_type: Type) -> NativeFunction {
    NativeFunction {
      address,
      arg_types,
      return_type,
    }
  }

  pub fn call<'lua>(&self, lua: &'lua Lua, args: mlua::MultiValue) -> Result<mlua::Value<'lua>, mlua::Error> {
    let args = args.into_vec();

    debug!("Calling function at address {:x} with ({:?}), expecting return type {:?}", self.address, args, self.return_type);

    let mut arg_bytes: Vec<u32> = Vec::new();

    for arg in args.iter().rev() {
      let mut arg_byte = unsafe {lua_to_native_implied(&arg).map_err(|e| mlua::Error::RuntimeError(format!("could not convert lua value into bytes: {}", e.to_string())))?};
      arg_bytes.append(&mut arg_byte);
    }

    let native_fn_address = self.address;

    let raw_args = arg_bytes.as_ptr();
    let arg_len = args.len();

    unsafe {
      #[allow(unused_assignments)]
      let mut raw_response: u32 = 0;

        // Call native function with arguments
        // Use raw assembly because we don't know how many arguments we have at compile time
        asm!(
          "mov {tmp}, {len}",
          "2:",
          "mov eax, [{args}]",
          "push eax",
          "add {args}, 0x4",
          "sub {tmp}, 0x1",
          "ja 2b",  // Jumps if tmp is above zero
          "call {address}",
          "mov {tmp}, {len}",
          "shl {tmp}, 0x2",
          "add esp, {tmp}",
          address = in(reg) native_fn_address,
          len = in(reg) arg_len,
          args = in(reg) raw_args,
          tmp = out(reg) _,
          out("eax") raw_response,
        );

      let lua_response = native_to_lua(lua, self.return_type, raw_response);

      lua_response.map_err(|e| mlua::Error::RuntimeError(format!("could not convert return value into lua value: {}", e.to_string())))
    }
  }
}

impl UserData for NativeFunction {
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
      methods.add_method("getAddress", |_, native_function, ()| {
        return Ok(native_function.address);
      });

      methods.add_method("call", |lua, native_function, args| {
        debug!("Calling native function: 0x{:x}", native_function.address);
        native_function.call(lua, args)
      })
    }
}

pub fn create_native_function_function<'lua>(lua: &'lua Lua, (arg_types, return_type, lua_fn): (Vec<String>, String, mlua::Function)) -> Result<NativeFunction, mlua::Error> {
  debug!("Creating native function with signature ({:?}) -> {:?}. Calls lua function: {:?}", arg_types, return_type, lua_fn);

  let args_len = arg_types.len();

  // Convert lua argument types
  let mut lua_arg_types: Vec<Type> = Vec::new();

  for arg_type in arg_types {
    match Type::try_from_str(&arg_type) {
      Some(arg_type) => lua_arg_types.push(arg_type),
      None => return Err(mlua::Error::RuntimeError("unsupported argument type".to_string())),
    }
  }

  let lua_arg_types_clone = lua_arg_types.clone();

  // Convert lua return type
  let lua_ret_type = match Type::try_from_str(&return_type) {
    Some(value) => value,
    None => return Err(mlua::Error::RuntimeError("unsupported return type".to_string())),
  };

  let lua_ret_type_clone = lua_ret_type.clone();

  // Type must be explicitly set, otherwise, rust doesn't know what to when splitting the fat pointer
  let native_closure: Box<dyn FnMut(u32) -> u32> = Box::new(move |args: u32| -> u32 {
    debug!("Called native function");

    let arg_pointer = &args as *const u32;

    let mut lua_args: Vec<mlua::Value> = Vec::new();

    for i in 0..lua_arg_types.len() {
      let arg_type = lua_arg_types[i];

      unsafe {
        match native_to_lua(lua, arg_type, *arg_pointer.add(i)) {
          Ok(value) => lua_args.push(value),
          Err(e) => {
            warn!("could not convert {} argument into lua value: {:?}", i, e);
            panic!("could not convert raw argument into lua value: {:?}", e);
          }
        }
      }
    }

    let return_value = match lua_fn.call::<_, mlua::Value>(mlua::MultiValue::from_vec(lua_args)) {
      Ok(value) => value,
      Err(e) => {
        warn!("Lua function threw unexpected error: {:?}. Panicking...", e);
        panic!("Lua function in native wrapper threw unexpected error: {:?}", e);
      }
    };

    
    let native_return_value = unsafe {
      match lua_to_native(lua_ret_type, &return_value) {
        Ok(value) => value,
        Err(e) => {
          warn!("could not convert lua return value into native value: {:?}. Panicking...", e);
          panic!("could not convert lua return value into native value: {:?}", e);
        }
      }
    };

    native_return_value[0]
  });

  unsafe {
  // Get the data and function pointer from the native closure
    let raw_native_closure = Box::into_raw(native_closure);

    let (data, vtable) = std::mem::transmute_copy::<_, (u32, *const u32)>(&raw_native_closure);
    let native_address = *vtable.add(4);
  
    // This wrapper function handles the calling the native closure.
    // The wrapper acts similar to a trampoline when hooking, therefore we must manually allocate and write the function
    let closure_wrapper = VirtualAlloc(None, 100, MEM_COMMIT | MEM_RESERVE, PAGE_EXECUTE_READWRITE);

    // Write the following assembly into the closure wrapper
    // mov eax, {arg_len}
    // mov ecx, esp
    // mov edx, esp
    // add ecx, eax
    // loop:
    // push dword [ecx]
    // sub ecx, 0x4
    // cmp ecx, edx
    // jb loop
    // push data
    // call native_closure
    // mov ecx, {arg_len + 0x4}
    // add esp, ecx
    // ret

    let arg_len_in_bytes: u32 = args_len as u32 * 4;

    let mut offset = 0;

    // mov eax, {arg_len}
    let store_args_in_eax_addr = closure_wrapper as *mut u8;
    *store_args_in_eax_addr = 0xb8;
    std::ptr::copy_nonoverlapping(&arg_len_in_bytes, store_args_in_eax_addr.add(1) as *mut u32, 1);
    offset += 5;

    // Insert static instructions
    // mov ecx, esp
    // add ecx, eax
    // mov edx, esp
    // loop:
    // push dword [ecx]
    // sub ecx, 0x4
    // cmp edx, ecx
    // jb loop
    let start_and_loop_instructions: &[u8; 15] = &[
      0x89, 0xe1, // mov ecx, esp
      0x01, 0xc1, // add ecx, eax
      0x89, 0xe2, // mov edx, esp 
      0xff, 0x31, // loop: push dword [ecx]
      0x83, 0xe9, 0x04, // sub ecx, 0x4
      0x39, 0xca, // cmp edx, ecx
      0x72, 0xf7  // jb loop
    ];

    let start_and_loop_addr = closure_wrapper.add(offset) as *mut u8;

    for i in 0..start_and_loop_instructions.len() {
      *(start_and_loop_addr.add(i)) = start_and_loop_instructions[i];
    }
    offset += start_and_loop_instructions.len();

    // push data
    let push_data_addr = closure_wrapper.add(offset) as *mut u8;
    *push_data_addr = 0x68u8;
    std::ptr::copy_nonoverlapping(&data, push_data_addr.add(1) as *mut u32, 1);
    offset += 5;

    // call native_closure
    let jmp_src = closure_wrapper.add(offset + 5) as u32;
    let jmp_dst = native_address;
    let jmp_delta = jmp_dst as i32 - jmp_src as i32;
    let call_closure_addr = closure_wrapper.add(offset) as *mut u8;
    *call_closure_addr = 0xe8u8;
    std::ptr::copy_nonoverlapping(&jmp_delta, call_closure_addr.add(1) as *mut i32, 1);
    offset += 5;

    // mov ecx, {arg_len+0x4}
    let mov_arg_len_in_ecx_addr = closure_wrapper.add(offset) as *mut u8;
    *mov_arg_len_in_ecx_addr = 0xb9u8;
    let args_with_data_len = arg_len_in_bytes + 4;
    std::ptr::copy_nonoverlapping(&args_with_data_len, mov_arg_len_in_ecx_addr.add(1) as *mut u32, 1);
    offset += 5;

    // End
    // add esp, ecx,
    // ret
    let end_instructions: &[u8; 3] = &[
      0x01, 0xcc, // add esp, ecx
      0xc3, // ret
    ];

    let end_addr = closure_wrapper.add(offset) as *mut u8;

    for i in 0..end_instructions.len() {
      *(end_addr.add(i)) = end_instructions[i];
    }


    Ok(NativeFunction {
      address: closure_wrapper as u32,
      arg_types: lua_arg_types_clone,
      return_type: lua_ret_type_clone,
    })
  }
}

pub fn get_native_function<'lua>(_: &'lua Lua, (address, arg_types, return_type): (u32, Vec<String>, String)) -> Result<NativeFunction, mlua::Error> {
  let mut lua_arg_types: Vec<Type> = Vec::new();
  for arg_type in arg_types {
    match Type::try_from_str(&arg_type) {
      Some(value) => lua_arg_types.push(value),
      None => return Err(mlua::Error::RuntimeError("unsupported argument type".to_string())),
    }
  }

  let lua_ret_type = match Type::try_from_str(&return_type) {
    Some(ret) => ret,
    None => return Err(mlua::Error::RuntimeError("invalid return type".to_string())),
  };

  let native_function = NativeFunction::new(address, lua_arg_types, lua_ret_type);

  Ok(native_function)
}


#[derive(Debug, Clone)]
struct NativeStructField {
  pub offset: u32,
  pub native_type: Type,
}

#[derive(Debug, Clone)]
pub struct NativeStruct {
  fields: HashMap<String, NativeStructField>,
  address: u32,
}

impl UserData for NativeStruct {
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {      
      methods.add_meta_method(MetaMethod::Index, |lua, native_struct, field: String| -> Result<mlua::Value, mlua::Error> {
        debug!("Getting field {} from native struct at 0x{:x}", field, native_struct.address);
        let native_field = match native_struct.fields.get(&field) {
          Some(field) => field,
          None => {
            debug!("Native struct at 0x{:x} doesn't have field {}", native_struct.address, field);
            return Ok(mlua::Nil);
          },
        };

        let field_ptr = (native_struct.address + native_field.offset) as *const u32;

        unsafe {
          let value = *field_ptr;
          native_to_lua(lua, native_field.native_type, value)
        }
      });

      methods.add_meta_method(MetaMethod::NewIndex, |_, native_struct, (field, value): (String, mlua::Value)| -> Result<(), mlua::Error> {
        debug!("Set field {} of struct at 0x{:x} to {:?}", field, native_struct.address, value);

        let native_field = match native_struct.fields.get(&field) {
          Some(field) => field,
          None => {
            debug!("Struct at 0x{:x} doesn't have field {}", native_struct.address, field);
            return Err(mlua::Error::RuntimeError("Field doesn't exist".to_string()))
          }
        };

        let native_value = unsafe {
          lua_to_native(native_field.native_type, &value)
            .map_err(|e| mlua::Error::RuntimeError(format!("Could not convert lua value into native: {}", e)))?
        };

        // Report if the lua value was converted into more bytes than expected
        if native_value.len() > 1 {
          debug!("Converted lua value is larger than one double word: {:?}", value);
        }

        let field_addr = native_struct.address + native_field.offset;

        match native_field.native_type {
          Type::Byte => {
            let field_ptr = field_addr as *mut u8;

            unsafe {
              *field_ptr = native_value[0] as u8;
            }
          },
          Type::Short => {
            let field_ptr = field_addr as *mut u16;

            unsafe {
              *field_ptr = native_value[0] as u16;
            }
          },
          _ => {
            let field_ptr = field_addr as *mut u32;

            // Only copy the first double word into the field
            unsafe {
              *field_ptr = native_value[0];
            }
          }
        }

        Ok(())
      });
    }
}

#[derive(Debug, Clone)]
pub struct NativeStructDefinition {
  fields: HashMap<String, NativeStructField>
}

impl NativeStructDefinition {
  pub fn new(fields: mlua::Table) -> Result<NativeStructDefinition, anyhow::Error> {
    debug!("Create new native struct definition");

    let mut native_fields: HashMap<String, NativeStructField> = HashMap::new();
  
    for pair in fields.pairs::<String, mlua::Table>() {
      let (key, field_definition) = match pair {
        Ok(pair) => pair,
        Err(e) => {
          debug!("Field definition has invalid type");
          bail!("Field definition must be a table: {}", e);
        }
      };
  
      let offset: u32 = field_definition.get("offset").map_err(|_| anyhow!("Field definition of {} is missing 'offset'", key))?;
      let native_type = {
        let native_type_str: String = field_definition.get("type")
          .map_err(|_| mlua::Error::RuntimeError(format!("Field definition of {} is missing 'type'", key)))?;
  
        match Type::try_from_str(&native_type_str) {
          Some(value) => value,
          None => {
            debug!("Field {} has an invalid type", key);
            bail!("Field {} has an invalid type", key);
          },
        }
      };
  
      native_fields.insert(key, NativeStructField {
        offset,
        native_type,
      });
    }
  
    Ok(NativeStructDefinition{
      fields: native_fields,
    })
  }
}

impl UserData for NativeStructDefinition {
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
      methods.add_method("cast", |_, definition, address: u32| -> Result<NativeStruct, mlua::Error> {
        Ok(NativeStruct {
          fields: definition.fields.clone(),
          address,
        })
      });
    }
}

pub fn create_native_struct_definition_fn<'lua>(_: &'lua Lua, fields: mlua::Table) -> Result<NativeStructDefinition, mlua::Error> {
  NativeStructDefinition::new(fields).map_err(|e| mlua::Error::RuntimeError(e.to_string()))
}

pub fn create_native_struct_fn<'lua>(_: &'lua Lua, (address, fields): (u32, mlua::Table)) -> Result<NativeStruct, mlua::Error> {
  debug!("Create new native struct at 0x{:x}", address);

  let definition = NativeStructDefinition::new(fields)
    .map_err(|e| mlua::Error::RuntimeError(format!("Could not create the struct definition: {}", e)))?;

  Ok(NativeStruct {
    fields: definition.fields.clone(),
    address,
  })
}