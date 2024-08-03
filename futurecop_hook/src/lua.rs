use std::arch::asm;

use log::{debug, error, info, warn};
use mlua::{Function, Lua, MultiValue, UserData};
use windows::Win32::System::Memory::{VirtualAlloc, MEM_COMMIT, MEM_RESERVE, PAGE_EXECUTE_READWRITE};

use crate::types::{lua_to_native, lua_to_native_implied, native_to_lua, Type};
use crate::native::{memory_copy, Hook};

/// Create a hook on any function with a given lua function.
pub fn hook_function<'lua>(lua: &'lua Lua, (address, arg_type_names, return_type_name, callback): (u32, Vec<String>, String, Function)) -> Result<(), mlua::Error> {
  debug!("Creating hook on {:#08x} with type {:?} -> {}", address, arg_type_names, return_type_name);

  // Parse parameter and return types
  let return_type = match Type::try_from_str(return_type_name.as_str()) {
    Some(value) => value,
    None => return Err(mlua::Error::RuntimeError(format!("return type invalid: type '{}' doesn't exist", return_type_name)))
  };

  let mut argument_types: Vec<Type> = Vec::new();
  for arg_type_name in arg_type_names {
    let arg_type = match Type::try_from_str(arg_type_name.as_str()) {
      Some(value) => value,
      None => return Err(mlua::Error::RuntimeError(format!("argument type invalid: type '{}' doesn't exist", arg_type_name)))
    };

    argument_types.push(arg_type);
  }

  let hook_return_type = return_type.clone();
  let hook_arg_types = argument_types.clone();

  // Create the native hook.
  // This hook is called instead of the actual address.
  // It receives the original arguments, converts them into lua values and passes them
  // to the lua hook. It converts the lua hook's return value back into its native representation
  // and returns.
  // In addition to the original arguments, the lua hook also gets a function to call the original (hooked)
  // function.
  unsafe {
    let mut hook = Hook::new(address);

    let hook_closure = move |original_fn: u32, args: u32| {
      debug!("Called closure for hook of {:#08x}", address);

      let wrapper_return_type = hook_return_type.clone();
      let hook_return_type = hook_return_type.clone();
      let wrapper_argument_types = hook_arg_types.clone();

      let original_fn_clone = original_fn.clone() as *const u32;

      // Create a lua function to call the original function (the function that was hooked)
      // This lua will do three things.
      // 1. Convert the arguments from lua values into native values
      // 2. Call the original function with the arguments
      // 3. Convert the return value back to a lua value and return it
      let original_wrapper = match lua.create_function::<_, mlua::Value, _>(move |lua, args: MultiValue| {
        debug!("Lua called original function");

        // Convert the arguments from lua values into actual native values.
        let lua_args = args.into_vec();

        let mut converted_lua_args: Vec<u32> = Vec::new();

        for arg_idx in (0..wrapper_argument_types.len()).rev() {
          let lua_arg = &lua_args[arg_idx];
          let arg_type = &wrapper_argument_types[arg_idx];

          let mut converted_arg = match lua_to_native(*arg_type, lua_arg) {
            Ok(value) => value,
            Err(e) => return Err(mlua::Error::RuntimeError(format!("could not converted argument {} into {:?}: {:?}", arg_idx, *arg_type, e))),
          };

          converted_lua_args.append(&mut converted_arg);
        }

        let raw_args = converted_lua_args.as_ptr();
        let arg_len = converted_lua_args.len();

        // This variable will hold the return value of the original function
        #[allow(unused_assignments)]
        let mut original_fn_return: u32 = 0;

        // Unfortunately I couldn't find a way force rust to behave as I wanted to.
        // Therefore, ugly assembly code.
        // The following assembly code acts the trampoline to the original function.
        // It takes all the converted arguments given by the lua function that called this closure and passes them all
        // to the original function. As we don't know the amount of arguments and cannot use a tuple to represent variadic arguments,
        // we use the assembly code to manually push all arguments to the stack and call the function.
        asm!(
          "push ebx",
          "push ecx",
          "push edx",
          "push esi",
          "push edi",
          "mov {tmp}, {len}",
          "2:",
          "mov eax, [{args}]",
          "push eax",
          "add {args}, 4",
          "sub {tmp}, 1",
          "ja 2b",
          "call {address}",
          "mov {tmp}, {len}",
          "shl {tmp}, 2",
          "add esp, {tmp}",
          "pop edi",
          "pop esi",
          "pop edx",
          "pop ecx",
          "pop ebx",
          address = in(reg) original_fn_clone,
          args = in(reg) raw_args,
          len = in(reg) arg_len,
          tmp = out(reg) _,
          out("eax") original_fn_return,
        );

        // Don't know if this necessary, but it fixed some weird issue.
        drop(lua_args);

        // Convert the return value of the original function into a lua value
        native_to_lua(lua, wrapper_return_type, original_fn_return as u32)
      }) {
        Ok(w) => w,
        Err(e) => {
          warn!("Error while creating wrapper for the original function: {:?}. Panicking...", e);
          panic!("Could not create a wrapper for the original function of a hook: {:?}", e);
        }
      };

      let mut callback_args: Vec<mlua::Value> = vec![mlua::Value::Function(original_wrapper)];
      let arg_pointer = &args as *const u32;

      for i in 0..argument_types.len() {
        let arg_type = argument_types[i];

        match native_to_lua(lua, arg_type, *arg_pointer.byte_offset(i as isize * 4)) {
          Ok(value) => callback_args.push(value),
          Err(e) => {
            warn!("could not convert {} argument to lua value: {:?}. Panicking...", i, e);
            panic!("could not convert a raw argument to a lua value: {:?}", e);
          }
        }
      }

      // Call the lua hook
      let return_value = match callback.call::<_, mlua::Value>(mlua::MultiValue::from_vec(callback_args)) {
        Ok(value) => value,
        Err(e) => {
          warn!("Lua hook threw error: {:?}. Panicking...", e);
          panic!("Lua hook threw an error: {:?}", e);
        }
      };

      // Convert the return value of the lua hook into a native value
      let raw_value = match lua_to_native(hook_return_type, &return_value) {
        Ok(raw_value) => {
          if raw_value.len() < 1 {
            error!("Lua hook returned an invalid value: return value could not be converted to a full word. Cannot handle this error panicking...");
            panic!("Lua hook returned an invalid value: could not be converted to a full word");
          } else if raw_value.len() > 1 {
            warn!("Lua hook returned an invalid value: return value too large. Handling by truncating the value. May lead to undesired results");
            raw_value[0]
          } else {
            raw_value[0]
          }
        },
        Err(e) => {
          error!("Could not convert lua hook return value into: {:?}. Panicking...", e);
          panic!("Error while converting the return value of a lua hook: {:?}", e);
        },
      };

      // Return the lua return value
      return raw_value;
    };

    let boxed_closure: Box<dyn FnMut(u32, u32) -> u32> = Box::new(hook_closure);

    match hook.set_closure(boxed_closure) {
      Err(e) => warn!("Couldn't hook {:#08x}: {:?}", address, e),
      _ => (),
    }
  }
  
  Ok(())
}

pub struct NativeFunction {
  // Generic native closure that wraps a lua function
  address: u32,
  #[allow(dead_code)]
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
    memory_copy(&arg_len_in_bytes as *const u32 as u32, store_args_in_eax_addr.add(1) as u32, 4);
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
    memory_copy(&data as *const u32 as u32, push_data_addr.add(1) as u32, 4);
    offset += 5;

    // call native_closure
    let jmp_src = closure_wrapper.add(offset + 5) as u32;
    let jmp_dst = native_address;
    let jmp_delta = jmp_dst as i32 - jmp_src as i32;
    let call_closure_addr = closure_wrapper.add(offset) as *mut u8;
    *call_closure_addr = 0xe8u8;
    memory_copy(&jmp_delta as *const i32 as u32, call_closure_addr.add(1) as u32, 4);
    offset += 5;

    // mov ecx, {arg_len+0x4}
    let mov_arg_len_in_ecx_addr = closure_wrapper.add(offset) as *mut u8;
    *mov_arg_len_in_ecx_addr = 0xb9u8;
    let args_with_data_len = arg_len_in_bytes + 4;
    memory_copy(&args_with_data_len as *const u32 as u32, mov_arg_len_in_ecx_addr.add(1) as u32, 4);
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