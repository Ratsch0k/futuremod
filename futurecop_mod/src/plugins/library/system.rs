use std::{sync::Arc, time::{SystemTime, UNIX_EPOCH}};

use mlua::{Lua, OwnedTable};

pub fn create_system_library(lua: Arc<Lua>) -> Result<OwnedTable, mlua::Error> {
  let library = lua.create_table()?;

  let get_time_fn = lua.create_function(|_, ()| {
    let time = SystemTime::now();

    match time.duration_since(UNIX_EPOCH) {
      Ok(timestamp) => Ok(timestamp.as_millis()),
      Err(e) => Err(mlua::Error::RuntimeError(format!("could not get time: {}", e))),
    }
  })?;
  library.set("getTime", get_time_fn)?;

  Ok(library.into_owned())
}