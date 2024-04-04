use std::{collections::HashMap, fmt::Debug, fs, path::{Path, PathBuf}, sync::{Arc, Mutex}};

use anyhow::bail;
use log::*;
use mlua::{Lua, OwnedTable};
use futurecop_data::plugin::{PluginInfo, PluginDependency};
use super::library::{dangerous::create_dangerous_library, game::create_game_library, input::create_input_library};

/// Holds the entire plugin environment.
/// 
/// Holds a reference to the plugin's globals, such as functions and global
/// variables.
/// Also contains the plugin's package cache which contains libraries and other
/// files the plugin required
#[derive(Clone)]
pub struct PluginEnvironment {
  /// Plugin globals.
  pub table: OwnedTable,
  /// Plugin package cache
  package_cache: Arc<Mutex<HashMap<PathBuf, mlua::OwnedTable>>>,
}

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

/// Prepare available libraries based on the plugin information.
/// 
/// For each library mentioned in the plugin's information, this function
/// will initialize the library and add it to the library list.
fn prepare_libraries(lua: Arc<Lua>, info: &PluginInfo) -> Result<HashMap<&'static str, mlua::OwnedTable>, mlua::Error> {
  let mut libraries = HashMap::new();

  for library in info.dependencies.iter() {
    match library {
      PluginDependency::Dangerous => libraries.insert("dangerous", create_dangerous_library(lua.clone())?),
      PluginDependency::Game => libraries.insert("game", create_game_library(lua.clone())?),
      PluginDependency::Input => libraries.insert("input", create_input_library(lua.clone())?),
    };
  }

  Ok(libraries)
}

impl PluginEnvironment {
  /// Create a new plugin environment for a plugin with the given information.
  pub fn new(lua: Arc<Lua>, plugin_info: &PluginInfo) -> Result<Self, mlua::Error> {
    let table = lua.create_table()?;

    // Set constants
    table.set("NAME", plugin_info.name.clone())?;

    // Create and set functions
    let print_target = plugin_info.name.to_string();
    let print_fn = lua.create_function(move |_, msg: mlua::Value| {
      info!(target: format!("plugin::{}", print_target).as_str(), "{:?}", msg);

      Ok(())
    })?;

    let libraries = prepare_libraries(lua.clone(), &plugin_info)?;
    let package_cache: Arc<Mutex<HashMap<PathBuf, OwnedTable>>> = Arc::new(Mutex::new(HashMap::new()));
    let require_fn_package_cache = Arc::downgrade(&package_cache);
    let plugin_info_clone = plugin_info.clone();
    let plugin_path = plugin_info.path.clone();
    let plugin_name = plugin_info.name.clone();
    let lua_ref = lua.clone();

    let require_fn = lua.create_function(move |lua, name: String| {
      info!("Plugin '{}' required {}", plugin_name, name);

      // Check if a library with the given name exists
      if let Some(library) = libraries.get(name.as_str()) {
        debug!("Required name is a library");
        return Ok(library.clone());
      }

      debug!("Library doesn't exist, treating require statement as requiring a local file");

      // Check if the require statement should load another lua file
      // Normalize the require path such that referencing the same file with a slightly different path
      // will not load the same file multiple times.
      // We enforce here that every require statement of a lua file is the relative path to that file
      // starting from the root of the plugin.
      let require_path = Path::new(&name).to_path_buf().with_extension("lua");

      debug!("Requiring file '{:?}'", require_path);

      let absolute_require_path = Path::join(&plugin_path, require_path.clone()).canonicalize().map_err(|e| mlua::Error::RuntimeError(format!("Could not load library: {:?}", e)))?;

      let require_package_cache = match require_fn_package_cache.upgrade() {
        Some(c) => c,
        None => return Err(mlua::Error::RuntimeError("Require is forbidden: Plugin is destroyed".into())),
      };

      let mut require_package_cache = require_package_cache.lock().map_err(|e| mlua::Error::RuntimeError(format!("Couldn't get lock to cache: {:?}", e)))?;

      if let Some(cached_file) = require_package_cache.get(&require_path) {        
        debug!("Found required file in cache");
        return Ok(cached_file.clone());
      }

      if !absolute_require_path.starts_with(&plugin_path) {
        warn!("Plugin {} required {:?} which is outside it's plugin folder", plugin_name, absolute_require_path);
        return Err(mlua::Error::RuntimeError("Permission denied: Requiring a file outside of the plugin folder is not allowed".into()));
      }

      if !absolute_require_path.exists() {
        warn!("Plugin {} required non-existing file {:?}", plugin_name, absolute_require_path);
        return Err(mlua::Error::RuntimeError("Required file doesn't exist".into()));
      }

      debug!("Preparing plugin environment for required file");
      let file_environment = PluginEnvironment::new(lua_ref.clone(), &plugin_info_clone)?;

      // Read the file content
      let content = fs::read_to_string(&absolute_require_path).map_err(|e| mlua::Error::RuntimeError(format!("Could not require file: {:?}", e)))?;
      let file_chunk = lua.load(content).set_environment(file_environment.table.clone());

      debug!("Executing required file");
      file_chunk.exec()?;

      let file_globals = file_environment.table.clone();

      let _ = require_package_cache.insert(absolute_require_path, file_globals.clone());

      Ok(file_globals)
    })?;
    
    table.set("print", print_fn)?;
    table.set("require", require_fn)?;

    Ok(PluginEnvironment { table: table.into_owned(), package_cache })
  }

}

impl Debug for PluginEnvironment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginEnvironment").finish()
    }
}