use std::sync::Arc;

use mlua::Lua;
use native::{create_native_function_function, create_native_struct_definition_fn, create_native_struct_fn, get_native_function};

mod types;
mod hook;
mod memory;
mod native;

use types::*;
use hook::hook_function;
use memory::*;


pub fn create_dangerous_library(lua: Arc<Lua>) -> Result<mlua::OwnedTable, mlua::Error> {
  let table = lua.create_table()?;

  let hook_fn = lua.create_function(hook_function)?;
  table.set("hook", hook_fn)?;

  let write_fn = lua.create_function(write_memory_function)?;
  table.set("writeMemory", write_fn)?;

  let read_fn = lua.create_function(read_memory_function)?;
  table.set("readMemory", read_fn)?;

  let create_native_function_fn = lua.create_function(create_native_function_function)?;
  table.set("createNativeFunction", create_native_function_fn)?;

  let get_native_function_fn = lua.create_function(get_native_function)?;
  table.set("getNativeFunction", get_native_function_fn)?;

  let create_native_struct_definition = lua.create_function(create_native_struct_definition_fn)?;
  table.set("createNativeStructDefinition", create_native_struct_definition)?;

  let create_native_struct = lua.create_function(create_native_struct_fn)?;
  table.set("createNativeStruct", create_native_struct)?;

  Ok(table.into_owned())
}






