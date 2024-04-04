use std::{fs, path::PathBuf, sync::Arc};
use futurecop_data::plugin::{PluginError, PluginInfo};
use log::*;
use mlua::{OwnedFunction, Lua, Table, Function};
use serde::{ser::SerializeStruct, Serialize};
use super::plugin_environment::PluginEnvironment;


const MAIN_FILE_NAME: &str = "main";
const ALLOWED_EXTENSIONS: [&str; 2] = ["lua", "luau"];

/// Installed mod plugin.
/// 
/// Contains the plugin's information and current state.
/// Is typically managed by [`plugins::PluginManager`]
#[derive(Debug, Clone, Serialize)]
pub struct Plugin {
    /// Whether the plugin is enabled or not.
    /// 
    /// A disabled plugin is still in memory, but it's onUpdate function
    /// will not be called every frame.
    enabled: bool,

    /// Current state of the plugin.
    /// If loaded, it will contains the current lua state.
    pub state: PluginState,

    /// Plugin information such as the description and authors.
    pub info: PluginInfo,

    /// Reference to lua.
    #[serde(skip)]
    lua: Arc<Lua>,
}

impl Into<futurecop_data::plugin::Plugin> for Plugin {
    fn into(self) -> futurecop_data::plugin::Plugin {
        futurecop_data::plugin::Plugin {
            enabled: self.enabled,
            state: self.state.into(),
            info: self.info.into(),
        }
    }
}

/// Current state of the plugin.
#[derive(Debug, Serialize, Clone)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum PluginState {
    Error(PluginError),
    Unloaded,
    Loaded(PluginContext),
}

impl Into<futurecop_data::plugin::PluginState> for PluginState {
    fn into(self) -> futurecop_data::plugin::PluginState {
        match self {
            PluginState::Unloaded => futurecop_data::plugin::PluginState::Unloaded,
            PluginState::Error(e) => futurecop_data::plugin::PluginState::Error(e),
            PluginState::Loaded(c) => futurecop_data::plugin::PluginState::Loaded(c.into())
        }
    }
}

/// Plugin context.
/// 
/// Holds references to the plugin's globals and framework functions.
#[derive(Debug, Clone)]
pub struct PluginContext {
    environment: PluginEnvironment,
    on_load: Option<OwnedFunction>,
    on_unload: Option<OwnedFunction>,
    on_update: Option<OwnedFunction>,
    on_enable: Option<OwnedFunction>,
    on_disable: Option<OwnedFunction>,
    on_install: Option<OwnedFunction>,
    on_uninstall: Option<OwnedFunction>,
}


impl Into<futurecop_data::plugin::PluginContext> for PluginContext {
    fn into(self) -> futurecop_data::plugin::PluginContext {
        futurecop_data::plugin::PluginContext {
            on_load: self.on_load.is_some(),
            on_unload: self.on_unload.is_some(),
            on_update: self.on_update.is_some(),
            on_enable: self.on_enable.is_some(),
            on_disable: self.on_disable.is_some(),
            on_install: self.on_install.is_some(),
            on_uninstall: self.on_uninstall.is_some(),
        }
    }
}

fn optional_lua_function_to_string(fun: &Option<OwnedFunction>) -> &'static str {
    if fun.is_some() {
        "set"
    } else {
        "unset"
    }
}

impl Serialize for PluginContext {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        
        let mut s = serializer.serialize_struct("PluginContext", 7)?;
        s.serialize_field("onLoad", optional_lua_function_to_string(&self.on_load))?;
        s.serialize_field("onUnload", optional_lua_function_to_string(&self.on_unload))?;
        s.serialize_field("onUpdate", optional_lua_function_to_string(&self.on_update))?;
        s.serialize_field("onEnable", optional_lua_function_to_string(&self.on_enable))?;
        s.serialize_field("onDisable", optional_lua_function_to_string(&self.on_disable))?;
        s.serialize_field("onInstall", optional_lua_function_to_string(&self.on_install))?;
        s.serialize_field("onUninstall", optional_lua_function_to_string(&self.on_uninstall))?;

        s.end()
    }
}


impl Plugin {

    /// Create a new Plugin instance from the plugin info.
    /// 
    /// This function only creates the plugin struct and doesn't load the actual plugin
    /// into memory.
    /// 
    /// To load the plugin into memory use [`Plugin::load`].
    pub fn new(lua: Arc<Lua>, info: PluginInfo) -> Self {
        Plugin { info, state: PluginState::Unloaded, enabled: false, lua: lua.clone() }
    }

    fn set_error(&mut self, e: PluginError) -> PluginError {
        self.state = PluginState::Error(e.clone());
        return e;
    }

    /// Load the plugin.
    /// 
    /// This method will load the plugin into memory, create its environment and execute the plugin's
    /// main file.
    pub fn load(&mut self) -> Result<(), PluginError> {
        let info = &self.info;
        let main_file = match discover_main_file(&info.path) {
            Ok(file) => file,
            Err(e) => {
                warn!("Couldn't get main file of plugin {:?}: {:?}", info.path, e);
    
                return Err(self.set_error(PluginError::NoMainFile));
            }
        };

        debug!("Check if file readable");
        let main_file_content = match fs::read_to_string(&main_file) {
            Ok(main_file_content) => main_file_content,
            Err(e) => {
                return Err(self.set_error(PluginError::Error(format!("Error while reading the main file: {:?}", e))));
            },
        };

        let environment = match PluginEnvironment::new(self.lua.clone(), &info) {
            Ok(env) => env,
            Err(e) => {
                return Err(self.set_error(PluginError::Error(format!("Could not create mod environment: {:?}", e))));
            }
        };

        match self.lua.load(main_file_content).set_environment(environment.table.clone()).exec() {
            Ok(_) => (),
            Err(e) => {
                return Err(self.set_error(PluginError::ScriptError(format!("Could not load module: {:?}", e))));
            },
        };

        let on_load = get_lua_function_or_none(&environment.table.to_ref(), "onLoad");
        let on_unload = get_lua_function_or_none(&environment.table.to_ref(), "onUnload");
        let on_update = get_lua_function_or_none(&environment.table.to_ref(), "onUpdate");
        let on_enable = get_lua_function_or_none(&environment.table.to_ref(), "onEnable");
        let on_disable = get_lua_function_or_none(&environment.table.to_ref(), "onDisable");
        let on_install = get_lua_function_or_none(&environment.table.to_ref(), "onInstall");
        let on_uninstall = get_lua_function_or_none(&environment.table.to_ref(), "onUninstall");

        let context = PluginContext {
            environment,
            on_load,
            on_unload,
            on_update,
            on_enable,
            on_disable,
            on_install,
            on_uninstall,
        };

        debug!("Execute onLoad function");
        match &context.on_load {
            Some(main) => match main.call::<_, ()>(()) {
                Ok(_) => debug!("Successfully called onLoad"),
                Err(e) => {
                    warn!("Main function threw error: {:?}", e);
                    return Err(self.set_error(PluginError::ScriptError(format!("Error while executing onLoad function: {:?}", e))));
                },
            },
            None => (),
        }

        self.state = PluginState::Loaded(context);

        Ok(())
    }

    /// Unload the plugin.
    /// 
    /// If the plugin is enabled, this function will additionally disable the plugin first.
    /// Unloading the plugin will remove the plugin from memory.
    /// *Should be tested to what extend this actually removes the plugin from memory.*
    pub fn unload(&mut self) -> Result<(), PluginError> {
        match &self.state {
            PluginState::Loaded(_) => (),
            _ => return Ok(()),
        };

        if self.enabled {
            if let Err(e) = self.disable() {
                warn!("Disabling plugin while unloading it threw error: {:?}", e);
            }
        }

        // This should drop `environment`, thus also dropping all functions and data stored
        // in the plugin's environment.
        self.state = PluginState::Unloaded;

        self.lua.gc_collect().map_err(|e| PluginError::ScriptError(format!("{:?}", e)))?;
        self.lua.gc_collect().map_err(|e| PluginError::ScriptError(format!("{:?}", e)))?;

        Ok(())
    }

    /// Reload the plugin.
    /// 
    /// Simply unloads the plugin and loads it again.
    pub fn reload(&mut self) -> Result<(), PluginError> {
        self.unload()?;

        self.load()
    }

    /// Disable the plugin.
    /// 
    /// Will only disable it and not unload it.
    /// Meaning, it is kept in memory but it's `onUpdate` function is not called anymore.
    /// Will call the plugin's `onDisable` function.
    pub fn disable(&mut self) -> Result<(), PluginError> {
        if !self.enabled {
            return Ok(());
        }

        match &self.state {
            PluginState::Loaded(context) => {
                self.enabled = false;

                if let Some(on_disabled) = &context.on_disable {
                    on_disabled.call(()).map_err(|e| PluginError::ScriptError(e.to_string()))?;
                }
            },
            _ => (),
        }

        Ok(())
    }

    /// Enable the plugin.
    /// 
    /// Also calls the plugin's `onEnable` function.
    pub fn enable(&mut self) -> Result<(), PluginError> {
        if self.enabled {
            return Ok(());
        }

        match &self.state {
            PluginState::Loaded(context) => {
                self.enabled = true;

                if let Some(on_enabled) = &context.on_enable {
                    on_enabled.call(()).map_err(|e| PluginError::ScriptError(e.to_string()))?;
                }
            },
            _ => {
                warn!("Do not enable mod because it is not loaded");
                return Err(PluginError::NotLoaded);
            }
        }

        Ok(())
    }

    /// Call the plugin's `onUpdate` function.
    /// 
    /// Returns an error if the plugin is not enabled.
    /// Will not call the function if the plugin is in an error state.
    pub fn on_update(&self) -> Result<(), PluginError> {
        if !self.enabled {
            return Err(PluginError::NotEnabledError);
        }

        match &self.state {
            PluginState::Loaded(context) => {
                if let Some(on_update) = &context.on_update {
                    debug!("Plugin '{}': Calling on_update", self.info.name);
                    on_update.call(()).map_err(|e| PluginError::ScriptError(e.to_string()))?;
                    debug!("Plugin '{}: Called on_update", self.info.name);
                } else {
                    debug!("Plugin '{}': on_update not set", self.info.name);
                }
            }
            _ => debug!("Plugin '{}': not calling on_update since mod is not loaded", self.info.name),
        }

        Ok(())
    }

    /// Whether the plugin is enabled or not.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

fn get_lua_function_or_none<'lua>(module: &'lua Table, name: &str) -> Option<OwnedFunction> {
    match module.get::<&str, Function>(name) {
        Ok(function) => {
            debug!("Module {:?} has attribute '{}'", module, name);
            
            Some(function.into_owned())
        },
        Err(_) => {
            debug!("Module {:?} has no attribute '{}'", module, name);
  
            None
        },
    }
}

/// Searches for the main file of a plugin within a directory.
/// 
/// If it cannot identify any main, it will return an error.
fn discover_main_file(directory: &PathBuf) -> Result<PathBuf, PluginError> {
    let files = directory.read_dir()
        .map_err(|e| PluginError::Error(format!("Error while reading mod directory '{:?}': {:?}", directory, e)))?
        .filter_map(|file| match file {
            Ok(file) => {
                if file.path().is_dir() {
                    debug!("Skipping directory '{:?}'", file);
                    return None
                }
  
                Some(file)
            },
            Err(e) => {
                warn!("Error while trying to read a file from mod directory '{:?}': {:?}", directory, e);
                None
            }
        });
  
    for file in files {
        let file_path = file.path();
  
        debug!("Checking file '{:?}'", file_path);
  
        let file_stem = match file_path.file_stem() {
            Some(stem) => match stem.to_str() {
                Some(stem) => stem,
                None => {
                    warn!("Couldn't convert file stem '{:?}' to string", stem);
                    continue;
                },
            },
            None => {
                warn!("Couldn't get file stem of '{:?}'", file);
                continue;
            }
        };
  
        let file_extension = match file_path.extension() {
            Some(extension) => match extension.to_str() {
                Some(stem) => stem,
                None => {
                    warn!("Couldn't convert file extension '{:?}' to string", extension);
                    continue;
                },
            },
            None => {
                warn!("Couldn't get file extension of {:?}", file);
                continue;
            }
        };
  
        debug!("Stem: {}, Extension: {}", file_stem, file_extension);
  
        if file_stem == MAIN_FILE_NAME && ALLOWED_EXTENSIONS.contains(&file_extension) {
            return Ok(file.path())
        }
    }
  
    Err(PluginError::NoMainFile)
}
  