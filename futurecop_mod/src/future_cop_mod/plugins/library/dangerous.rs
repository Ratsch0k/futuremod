use std::{arch::asm, sync::Arc};

use anyhow::bail;
use log::{debug, error, info, warn};
use mlua::{Function, Lua, MultiValue};

use crate::future_cop_mod::util::Hook;



#[derive(Debug, Clone, Copy)]
enum Type {
  String,
  Integer,
  Float,
  Void,
}

impl Type {
  fn try_from_str(name: &str) -> Option<Type> {
    let type_value = match name {
      "string" => Type::String,
      "int" => Type::Integer,
      "float" => Type::Float,
      "void" => Type::Void,
      _ => return None,
    };

    Some(type_value)
  }
}

const MAX_STRING: u16 = 1024;

unsafe fn raw_to_lua<'a>(lua: &'a Lua, lua_type: Type, raw_value: u32) -> Result<mlua::Value<'a>, mlua::Error> {
  let value = match lua_type {
    Type::Integer => mlua::Value::Integer(raw_value as i32),
    Type::String => {
      let mut string_bytes: Vec<u8> = Vec::new();
      let string_pointer = raw_value as *const u8;

      for i in 0..MAX_STRING {
        let current_value = *(string_pointer.add(i.into()));
        
        if current_value == 0 {
          break;
        }

        string_bytes.push(current_value);
      }

      mlua::Value::String(lua.create_string(string_bytes.as_slice())?)
    },
    Type::Float => mlua::Value::Number(f64::from(raw_value as f32)),
    Type::Void => mlua::Value::Nil,
  };

  Ok(value)
}

unsafe fn lua_to_raw<'a>(lua_type: Type, lua_value: &'a mlua::Value) -> Result<Vec<u32>, anyhow::Error> {
  let value: Vec<u32> = match lua_type {
    Type::Integer => match lua_value.as_u32() {
      Some(value) => vec![value],
      None => bail!("value is not an integer"),
    },
    Type::Float => match lua_value.as_f32() {
      Some(value) => vec![value as u32],
      None => bail!("value is not a float"),
    }
    Type::Void => vec![0u32],
    Type::String => match lua_value.as_str() {
      Some(value) => {
        vec![value.as_ptr() as u32]
      },
      None => bail!("value is not a string"),
    }
  };

  Ok(value)
}


pub fn create_dangerous_library(lua: Arc<Lua>) -> Result<mlua::OwnedTable, mlua::Error> {
  let table = lua.create_table()?;

  let hook_fn = lua.create_function(|lua, (address, arg_type_names, return_type_name, callback): (u32, Vec<String>, String, Function)| {
    info!("Creating hook on {:#08x} with type {:?} -> {}", address, arg_type_names, return_type_name);

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

    unsafe {
      let mut hook = Hook::new(address);

      let hook_closure = move |original_fn: u32, args: u32| {
        debug!("Called closure for hook of {:#08x}", address);

        let wrapper_return_type = hook_return_type.clone();
        let hook_return_type = hook_return_type.clone();
        let wrapper_argument_types = hook_arg_types.clone();

        let original_fn_clone = original_fn.clone() as *const u32;

        let original_wrapper = match lua.create_function::<_, mlua::Value, _>(move |lua, args: MultiValue| {
          debug!("Lua called original function");

          let lua_args = args.into_vec();

          let mut converted_lua_args: Vec<u32> = Vec::new();

          for arg_idx in (0..wrapper_argument_types.len()).rev() {
            let lua_arg = &lua_args[arg_idx];
            let arg_type = &wrapper_argument_types[arg_idx];

            let mut converted_arg = match lua_to_raw(*arg_type, lua_arg) {
              Ok(value) => value,
              Err(e) => return Err(mlua::Error::RuntimeError(format!("could not converted argument {} into {:?}: {:?}", arg_idx, *arg_type, e))),
            };

            converted_lua_args.append(&mut converted_arg);
          }

          let raw_args = converted_lua_args.as_ptr();
          let arg_len = converted_lua_args.len();

          #[allow(unused_assignments)]
          let mut original_fn_return: u32 = 0;

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
            "js 2b",
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

          drop(lua_args);

          raw_to_lua(lua, wrapper_return_type, original_fn_return as u32)
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

          match raw_to_lua(lua, arg_type, *arg_pointer.add(i)) {
            Ok(value) => callback_args.push(value),
            Err(e) => {
              warn!("could not convert {} argument to lua value: {:?}. Panicking...", i, e);
              panic!("could not convert a raw argument to a lua value: {:?}", e);
            }
          }
        }

        let return_value = match callback.call::<_, mlua::Value>(mlua::MultiValue::from_vec(callback_args)) {
          Ok(value) => value,
          Err(e) => {
            warn!("Lua hook threw error: {:?}. Panicking...", e);
            panic!("Lua hook threw an error: {:?}", e);
          }
        };

        let raw_value = match lua_to_raw(hook_return_type, &return_value) {
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

        return raw_value;
      };

      let boxed_closure: Box<dyn FnMut(u32, u32) -> u32> = Box::new(hook_closure);

      match hook.set_closure(boxed_closure) {
        Err(e) => warn!("Couldn't hook {:#08x}: {:?}", address, e),
        _ => (),
      }
    }
    
    Ok(())
  })?;
  
  table.set("hook", hook_fn)?;

  Ok(table.into_owned())
}