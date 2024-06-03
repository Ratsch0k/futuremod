use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use std::{collections::HashMap, fs};
use futurecop_data::plugin::PluginError;
use log::*;
use mlua::{Lua, StdLib};
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;
use crate::plugins::plugin_info::load_plugin_info;
use regex::Regex;
use anyhow::anyhow;

use super::plugin::*;
use super::plugin_info::PluginInfoError;

static mut GLOBAL_PLUGIN_MANAGER: OnceLock<Arc<Mutex<PluginManager>>> = OnceLock::new();

/// Global plugin manager.
/// 
/// Global instance of the plugin manager that manages all
/// plugins and the script runtime environment and the plugin
/// context.
/// Instead of creating new instances of [`PluginManager`], use this
/// struct and its methods instead.
/// 
/// This struct is initialized (or should at least) the start of mod's lifecycle.
pub struct GlobalPluginManager;

impl GlobalPluginManager {
    pub fn get() -> Arc<Mutex<PluginManager>> {
        let plugin_manager;
        unsafe {plugin_manager = GLOBAL_PLUGIN_MANAGER.get().unwrap()};

        return plugin_manager.clone();
    }

    pub fn with_plugin_manager<F, R>(f: F) -> Result<R, anyhow::Error>
    where F: Fn(&PluginManager) -> Result<R, anyhow::Error> {
        match GlobalPluginManager::get().lock() {
            Ok(m) => f(&m),
            Err(e) => return Err(anyhow!("could not get lock to plugin manager: {:?}", e)),
        }
    }

    pub fn with_plugin_manager_mut<F, R>(f: F) -> Result<R, anyhow::Error>
    where F: Fn(&mut PluginManager) -> Result<R, anyhow::Error> {
        let plugin_manager;
        unsafe {plugin_manager = GLOBAL_PLUGIN_MANAGER.get().unwrap()}

        match plugin_manager.lock() {
            Ok(mut m) => f(&mut m),
            Err(e) => return Err(anyhow!("could not get mutable lock to plugin manager: {:?}", e)),
        }
    }

    /// Initialize the global plugin manager.
    /// 
    /// Should only be called once for the entire life of the mod.
    /// If its called a multiple time, calls after the first call will error.
    /// Additionally, if plugin initialization errors, this also returns an error.
    pub fn initialize(plugins_directory: PathBuf) -> Result<(), anyhow::Error> {
        let plugin_manager = match PluginManager::new(plugins_directory) {
            Ok(m) => m,
            Err(e) => {
                anyhow::bail!("{:?}", e)
            }
        };
        let p = Arc::new(Mutex::new(plugin_manager));
        unsafe { GLOBAL_PLUGIN_MANAGER.set(p).map_err(|_| anyhow!("global plugin manager already initialized")) }
    }
}

#[derive(Debug)]
pub enum PluginManagerError {
    Io(std::io::Error),
    PluginNotFound,
    Plugin(PluginError),
    Other(String),
    AlreadyLoaded,
}

#[derive(Debug)]
pub enum PluginInstallError {
    InfoFile(PluginInfoError),
    InvalidName,
    Copy(String),
    AlreadyInstalled,
    Plugin(String),
}

/// Persistence state of a plugin which indicates how a plugin should be loaded on the next start.
/// 
/// This doesn't reflect the actual plugin's state.
/// For example, if a plugin was loaded and enabled but threw an error during the loading process
/// and thus has now the state [`PluginState::Error`], it will have the state [`StoredPluginState::Disabled`].
/// Rather, this states whether the plugin manager will load and/or enable the plugin when it starts the next time.
/// This state is only updated due to the user's input.
#[derive(Debug, Serialize, Deserialize, Clone)]
enum PersistentPluginState {
    Unloaded,
    Disabled,
    Enabled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistentPluginStates {
    states: HashMap<String, PersistentPluginState>,
    path: PathBuf,
}

impl PersistentPluginStates {
    pub fn new(path: &Path) -> Result<PersistentPluginStates, anyhow::Error> {
        info!("Reading plugin states from '{}'", path.display());

        let states: HashMap<String, PersistentPluginState> = match fs::read_to_string(path) {
            Ok(content) => serde_json::from_str(&content).map_err(|e| anyhow!("could not parse the plugin states file: {}", e.to_string()))?,
            Err(_) => HashMap::new(),
        };

        Ok(PersistentPluginStates { states, path: path.to_path_buf() })
    }

    pub fn get_state(&self, name: &str) -> Option<&PersistentPluginState> {
        self.states.get(name)
    }

    pub fn insert(&mut self, name: &str, state: PersistentPluginState) -> Result<(), anyhow::Error>{
        self.states.insert(name.into(), state);

        self.write_to_file()
    }

    pub fn write_to_file(&self) -> Result<(), anyhow::Error> {
        let content = serde_json::to_string(&self.states).map_err(|e| anyhow!("could not serialize plugin states to string: {}", e.to_string()))?;

        fs::write(&self.path, content).map_err(|e| anyhow!("could not persist change: {}", e.to_string()))
    }

    pub fn remove(&mut self, name: &str) {
        self.states.remove(name);
    }
}

fn persist_plugin_state_change(states: &mut PersistentPluginStates, plugin: &Plugin, state: PersistentPluginState) {
    if let Err(e) = states.insert(&plugin.info.name, PersistentPluginState::Enabled) {
        warn!("Could not persist change '{}' -> {:?}: {:?}", plugin.info.name, state, e);
    }
}

/// Manages plugins.
/// 
/// **Should never be instantiated manually. [`GlobalPluginManager`] should be used to
/// get the global plugin manager instance.**
/// Loads and manages all plugins
pub struct PluginManager {
  /// All plugins
  pub plugins: HashMap<String, Plugin>,
  //// Directory where the plugins are stored
  pub plugins_directory: PathBuf,
  /// Persistence state
  persistent_states: PersistentPluginStates,
  /// Reference to lua
  lua: Arc<Lua>,
}

impl PluginManager {
  /// Load all plugins from the given folder and create a PluginManager that
  /// with the contained plugins.
  /// Before loading any plugins from the directory, it will first load the state persistence file from the directory
  /// if it exists. This file persists whether the user enabled or disabled a plugin.
  /// For plugins not in the persistence file, they will be loaded but disabled.
  pub fn new(plugins_directory: PathBuf) -> Result<Self, PluginManagerError> {
      let lua = Arc::new(Lua::new());
      if let Err(e) = lua.load_from_std_lib(StdLib::STRING | StdLib::BIT | StdLib::MATH | StdLib::TABLE) {
        error!("Could not load subset of standard library: {}", e);
        return Err(PluginManagerError::Other(format!("Standard library error import: {}", e)));
      }

      if !plugins_directory.is_dir() {
        info!("Plugin directory doesn't exist, creating it.");
        if let Err(e) = fs::create_dir_all(&plugins_directory) {
            error!("Error while creating the plugin directory: {}", e.to_string());
            return Err(PluginManagerError::Io(e));
        }
      }

      let plugin_states_file = Path::join(&plugins_directory, "plugins.json");
      let mut persistent_states = PersistentPluginStates::new(&plugin_states_file).map_err(|e| PluginManagerError::Other(e.to_string()))?;

      info!("Loading plugins from {:?}", plugins_directory);
      let plugin_directories = plugins_directory.read_dir().map_err(PluginManagerError::Io)?
          .filter_map(|path| {
              match path {
                  Ok(path) => match path.path().is_dir() {
                      true => Some(path),
                      false => {
                          info!("Found file '{:?}' in plugins directory, skipping...", path);
                          None
                      },
                  },
                  Err(e) => {
                      warn!("Error while trying to get a plugin directory: {:?}", e);
                      None
                  }
              }
          });

      let mut plugins: HashMap<String, Plugin> = HashMap::new();

      info!("Loading plugin list");
      for plugin_folder in plugin_directories {
          debug!("Discovered plugin folder {:?}", plugin_folder);

          let plugin_folder_path = plugin_folder.path();

          let plugin_info = match load_plugin_info(plugin_folder_path) {
            Ok(v) => v,
            Err(e) => {
                warn!("Error while loading the plugin's info file: {:?}", e);
                continue;
            }
          };

          if plugins.contains_key(&plugin_info.name) {
            debug!("Already found a plugin with the same name");
            continue;
          }
              
          debug!("Creating plugin {}", plugin_info.name);
          let plugin: Plugin = Plugin::new(lua.clone(), plugin_info);
  
          match plugin.state {
              PluginState::Error(ref e) => {
                  warn!("Error while creating plugin {}: {:?}", plugin.info.name, e)
              },
              _ => info!("Successfully created plugin: {}", plugin.info.name),
          }
          
          plugins.insert(plugin.info.name.to_string(), plugin);
      }

      debug!("Discovered {} plugins", plugins.len());

      let mut successfully_loads = 0;
      let mut errored_loads = 0;

      info!("Loading plugins");
      for (name, plugin) in plugins.iter_mut() {
        info!("Loading plugin {}", name);

        let state = match persistent_states.get_state(name) {
            None => {
                info!("Plugin was not in persistence file, adding it as disabled");
                persistent_states.insert(&name, PersistentPluginState::Disabled).map_err(|e| PluginManagerError::Other(e.to_string()))?;

                PersistentPluginState::Disabled
            },
            Some(state) => state.clone(),
        };

        let success = match plugin.load() {
            Ok(_) => {
                info!("Successfully loaded plugin {}", name);
                successfully_loads += 1;
                true
            }
            Err(e) => {
                warn!("Error while loading plugin {}: {:?}", name, e);
                errored_loads += 1;
                false
            },
        };

        if success {
            match state {
                PersistentPluginState::Enabled => {
                    info!("Plugin was persisted as enabled, enabling plugin");

                    if let Err(e) = plugin.enable() {
                        warn!("Error while enabling plugin: {:?}", e);
                    }
                }
                _ => (),
            }
        }
      }

      info!("Loaded {} plugins, {} errored", successfully_loads, errored_loads);

      info!("Loaded the following plugins:");

      for (name, plugin) in plugins.iter() {
        info!("- {}: {:?}", name, plugin.state);
      }

      debug!("Detailed plugin overview");
      for (name, game_plugin) in plugins.iter() {
          debug!("Plugin '{}'\n---------", name);
          debug!("{:#?}", game_plugin);
          debug!("\n\n");
      }

      Ok(
          PluginManager { plugins, plugins_directory, lua, persistent_states }
      )
  }

  /// Call `onUpdate` function of all enabled plugins.
  pub fn on_update(&self) {
      for (_, plugin) in &self.plugins {
          
          if plugin.is_enabled() {
              debug!("Calling on_update for plugin '{}'", plugin.info.name);

              match plugin.on_update() {
                  Err(e) => warn!("Plugin '{}' main function threw error: {:?}", plugin.info.name, e),
                  _ => debug!("Called on_update of plugin '{}'", plugin.info.name),
              }
          } else {
              debug!("Not calling on_update for plugin '{}', plugin not enabled", plugin.info.name);
          }
      }
  }

  /// Enable the plugin
  pub fn enable_plugin(&mut self, name: &String) -> Result<(), PluginManagerError> {
      info!("Enable plugin '{}'", name);
      let plugin = match self.plugins.get_mut(name) {
          Some(plugin) => plugin,
          None => {
            warn!("Plugin doesn't exist");
            return Err(PluginManagerError::PluginNotFound)
          }
      };

      plugin.enable().map_err(PluginManagerError::Plugin)?;
      persist_plugin_state_change(&mut self.persistent_states, plugin, PersistentPluginState::Enabled);

      Ok(())
    }

  /// Disable the plugin
  pub fn disable_plugin(&mut self, name: &String) -> Result<(), PluginManagerError> {
      info!("Disable plugin '{}'", name);
      match self.plugins.get_mut(name) {
          Some(game_plugin) => {
              game_plugin.disable().map_err(PluginManagerError::Plugin)?;
                persist_plugin_state_change(&mut self.persistent_states, game_plugin, PersistentPluginState::Disabled);

              Ok(())
          },
          None => {
            warn!("Plugin doesn't exist");
            Err(PluginManagerError::PluginNotFound)
          },
      }
  }

  /// Reload the plugin
  pub fn reload_plugin(&mut self, name: &str) -> Result<(), PluginManagerError> {
    info!("Reloading plugin '{}'", name);

    let plugin = match self.plugins.get_mut(name) {
        None => return Err(PluginManagerError::PluginNotFound),
        Some(p) => p,
    };

    plugin.reload().map_err(PluginManagerError::Plugin)
  }

  pub fn get_plugins(&self) -> &HashMap<String, Plugin> {
    return &self.plugins;
  }

  /// Install a plugin from a folder.
  ///
  /// This method will install the plugin stored at the specified `folder`.
  /// Installation simply means, copying the plugin's file into the plugin folder, creating a [`Plugin`] struct
  /// for the plugin, loading it, and then storing it.
  /// This means, that the plugin is loaded when installing, which will execute the plugin and it's main function.
  pub fn install_plugin_from_folder(&mut self, folder: &PathBuf) -> Result<(), PluginInstallError> {
    info!("Installing plugin from {}", folder.display());
    let plugin_info = load_plugin_info(folder.clone()).map_err(PluginInstallError::InfoFile)?;

    if self.plugins.contains_key(&plugin_info.name) {
        warn!("Plugin '{}' already installed", plugin_info.name);
        return Err(PluginInstallError::AlreadyInstalled);
    }

    let plugin_folder_name = match sanitize_name(&plugin_info.name) {
        None => return Err(PluginInstallError::InvalidName),
        Some(v) => v,
    };
    debug!("Plugin name '{}' sanitized to '{}'", plugin_info.name, plugin_folder_name);

    let destination = self.plugins_directory.clone().join(plugin_folder_name);
    debug!("Plugin folder will be '{}'", destination.display());

    info!("Copying files from plugin package to destination");
    for file in WalkDir::new(folder).into_iter().filter_map(|e| e.ok()) {
        let path = file.path();

        let relative_path = match path.strip_prefix(folder) {
            Ok(v) => v,
            Err(err) => return Err(PluginInstallError::Copy(format!("Could not get relative path of {}: {}", path.display(), err.to_string()))),
        };
        let destination_path = Path::join(&destination.clone(), &relative_path);

        if path.is_dir() {
            match fs::create_dir_all(&destination_path) {
                Err(err) => return Err(PluginInstallError::Copy(format!("Could not destination directory {}: {}", destination_path.display(), err.to_string()))),
                _ => (),
            }
        } else if path.is_file() {
        debug!("Copy {} to {}", path.display(), destination_path.display());
            match fs::copy(path, destination_path) {
                Err(err) => return Err(PluginInstallError::Copy(format!("Could not copy {}: {}", path.display(), err.to_string()))),
                _ => (),
            }
        }
    }
    
    info!("Copying finished, loading plugin");
    // Create a new plugin info struct based on the freshly copied plugin.
    // Since the plugin info contains the current location of the plugin, reusing the original plugin
    // info is not possible.
    let plugin_info = load_plugin_info(destination).map_err(PluginInstallError::InfoFile)?;
    let plugin_name = plugin_info.name.clone();

    // Create and load the plugin
    let plugin = Plugin::new(self.lua.clone(), plugin_info);
    persist_plugin_state_change(&mut self.persistent_states, &plugin, PersistentPluginState::Disabled);
    self.plugins.insert(plugin_name.clone(), plugin);

    let plugin = self.plugins.get_mut(&plugin_name).unwrap();
    plugin.load().map_err(|e| PluginInstallError::Plugin(format!("{:?}", e)))?;

    Ok(())
  }

  /// Load the plugin with the specified name.
  /// 
  /// Refer to [`Plugin.load()`] for information about what loading a plugin means.
  pub fn load_plugin(&mut self, name: &str) -> Result<(), PluginManagerError> {    
    info!("Load plugin: {}", name);

    let plugin = match self.plugins.get_mut(name) {
        None => return Err(PluginManagerError::PluginNotFound),
        Some(p) => p,
    };

    persist_plugin_state_change(&mut self.persistent_states, &plugin, PersistentPluginState::Disabled);
    plugin.load().map_err(PluginManagerError::Plugin)
  }

  /// Unload the plugin with the specified name.
  pub fn unload_plugin(&mut self, name: &str) -> Result<(), PluginManagerError> {
    info!("Unload plugin: {}", name);

    let plugin = match self.plugins.get_mut(name) {
        None => return Err(PluginManagerError::PluginNotFound),
        Some(p) => p,
    };

    persist_plugin_state_change(&mut self.persistent_states, &plugin, PersistentPluginState::Unloaded);
    plugin.unload().map_err(PluginManagerError::Plugin)
  }

  // Uninstall the plugin.
  pub fn uninstall_plugin(&mut self, name: &str) -> Result<(), PluginManagerError> {
    info!("Uninstalling plugin: {}", name);

    self.persistent_states.remove(name);

    let plugin = match self.plugins.get_mut(name) {
        None => return Err(PluginManagerError::PluginNotFound),
        Some(p) => p,
    };

    // We will execute the plugin's disable function just that it has a chance to be uninstalled cleanly.
    // However, we won't care if the plugin's disable function will throw an error and still remove it afterwards.
    if let Err(e) = plugin.disable() {
        warn!("Plugin {} threw an error while it was disabled: {:?}", name, e);
    };

    // Unload the plugin.
    // This should drop all references to lua objects and also run lua's garbage collector.
    // However, it this call fails, removing the Plugin from the map should still work
    if let Err(e) = plugin.unload() {
        warn!("Plugin {} threw an error while unloading: {:?}", name, e);
    }

    let plugin_path = plugin.info.path.clone();

    // Remove the plugin from the plugin map.
    // This should only return None due to race conditions.
    // In such cases, log it.
    if let None = self.plugins.remove(name) {
        warn!("Could not find plugin '{}' while removing it from the internal map", name);
    }

    // Ensure that all lua references and objects are destroyed properly.
    let _ = self.lua.gc_collect();
    let _ = self.lua.gc_collect();

    // Lastly, remove the plugin's file from the plugin folder
    fs::remove_dir_all(plugin_path).map_err(PluginManagerError::Io)?;

    Ok(())
  }
}

/// Sanitizes the given name to be used as a folder name.
/// 
/// This function returns `Some` if the name can be sanitized and
/// and `None` if it can't.
/// If it returns `Some`, this will contain the sanitized name.
/// 
/// A name can only be sanitized if the it solely consists of
/// ASCII characters.
fn sanitize_name(name: &str) -> Option<String> {
    let name = name.to_ascii_lowercase();
    let re = Regex::new(r"^[a-z0-9 \.]*$").unwrap();

    if !re.is_match(&name) {
        return None;
    }

    Some(
        name.replace(" ", "_")
            .replace(".", "-")
    )
}
