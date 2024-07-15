use std::arch::asm;

use log::{debug, error, info, warn};
use mlua::{Function, Lua, MultiValue};

use crate::{plugins::library::dangerous::{lua_to_native, native_to_lua, Type}, util::Hook};

/// Create a hook on any function with a given lua function.
pub fn hook_function<'lua>(lua: &'lua Lua, (address, arg_type_names, return_type_name, callback): (u32, Vec<String>, String, Function)) -> Result<(), mlua::Error> {
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

        match native_to_lua(lua, arg_type, *arg_pointer.add(i)) {
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