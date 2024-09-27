use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use std::{collections::HashMap, fs};
use futuremod_data::plugin::PluginError;
use log::*;
use mlua::{Lua, StdLib};
use walkdir::WalkDir;
use crate::plugins::plugin_info::load_plugin_info;
use regex::Regex;
use anyhow::anyhow;

use super::plugin::*;
use super::plugin_info::PluginInfoError;
use super::plugin_persistence::{PersistedPlugins, PersistentPluginState, PersistedPlugin};

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
    InvalidPluginFolder,
    IO(String),
}


fn add_plugin_to_persistence(persistence: &mut PersistedPlugins, plugin: &Plugin, state: PersistentPluginState) {
    debug!("Adding plugin '{}' to persistence", plugin.info.name);

    let persisted_plugin = PersistedPlugin {
        state,
        in_dev_mode: plugin.in_dev_mode,
    };

    if let Err(e) = persistence.insert(&plugin.info.name, persisted_plugin) {
        warn!("Could not persist change: {}", e);
    }
}

fn persist_plugin_state_change(persistence: &mut PersistedPlugins, plugin: &Plugin, state: PersistentPluginState) {
    debug!("Changing persistence state of plugin {} to {:?}", plugin.info.name, state);

    if let Err(e) = persistence.update_state(&plugin.info.name, state) {
        warn!("Could not persist change: {}", e);
    }
}

fn remove_plugin_from_persistence(persistence: &mut PersistedPlugins, plugin_name: &str) {
    debug!("Removing plugin {} from persistence", plugin_name);
    if let Err(e) = persistence.remove(&plugin_name) {
        warn!("Could not write plugin persistence to file: {}", e);
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
  persistent_states: PersistedPlugins,
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
      let mut persisted_plugins = PersistedPlugins::new(&plugin_states_file).map_err(|e| PluginManagerError::Other(e.to_string()))?;

      info!("Loading plugins from {:?}", plugins_directory);
      let plugin_directories = plugins_directory.read_dir().map_err(PluginManagerError::Io)?
          .filter_map(|path| {
              match path {
                  Ok(path) => match path.path().is_dir() || junction::exists(path.path()).unwrap_or(false) {
                      true => Some(path),
                      false => {
                          debug!("Found file '{:?}' in plugins directory, skipping...", path);
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

      debug!("Loading plugin list");
      for plugin_folder in plugin_directories {
          debug!("Discovered plugin folder {}", plugin_folder.path().display());

          let plugin_folder_path = plugin_folder.path();

          let mut plugin_info = match load_plugin_info(&plugin_folder_path) {
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
          let in_dev_mode = match persisted_plugins.get_state(&plugin_info.name) {
            Some(v) => v.in_dev_mode,
            None => false,
          };

          // If plugin is in dev mode, adjust the plugin's path
          if in_dev_mode {
            plugin_info.path = plugin_folder.path();
          }

          let plugin: Plugin = Plugin::new(lua.clone(), plugin_info, in_dev_mode);
  
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
        debug!("Loading plugin {}", name);

        let persisted_plugin = match persisted_plugins.get_state(name) {
            None => {
                info!("Plugin was not in persistence file, adding it as disabled");
                persisted_plugins.insert(&name, PersistedPlugin{ state: PersistentPluginState::Disabled, in_dev_mode: false }).map_err(|e| PluginManagerError::Other(e.to_string()))?;

                PersistedPlugin {state: PersistentPluginState::Disabled, in_dev_mode: false }
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
            match persisted_plugin.state {
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
          PluginManager { plugins, plugins_directory, lua, persistent_states: persisted_plugins }
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
    let plugin_info = load_plugin_info(&folder).map_err(PluginInstallError::InfoFile)?;

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

    debug!("Copying files from plugin package to destination");
    copy_plugin_directory_to_plugins_folder(&folder, &destination)?;
    
    debug!("Copying finished, loading plugin");
    
    self.add_and_load_plugin_from_folder(&destination, false)?;

    Ok(())
  }

  pub fn install_plugin_in_dev_mode(&mut self, folder: &PathBuf) -> Result<(), PluginInstallError> {
    info!("Installing plugin in developer mode from '{}'", folder.display());

    // Try to load the plugin information from the folder.
    // If we can't load it, for whatever reason, only report the folder as being an invalid plugin folder.
    // It should not be possible to detect what folders exists based on the response.
    let plugin_info = load_plugin_info(&folder)
        .map_err(|_| PluginInstallError::InvalidPluginFolder)?;

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

    debug!("Creating a softlink from the plugin folder to the destination in the plugins folder");
    softlink_plugin_directory_to_plugins_folder(&folder, &destination)?;
    
    debug!("Softlink successfully created");

    info!("Installing plugin");
    self.add_and_load_plugin_from_folder(&destination, true)?;

    Ok(())
  }

  /// Add the plugin at in the given folder to the installed plugins and load it.
  fn add_and_load_plugin_from_folder(&mut self, folder: &PathBuf, in_dev_mode: bool) -> Result<(), PluginInstallError> {
    debug!("Adding plugin from '{}' to plugins", folder.display());
    let mut plugin_info = load_plugin_info(folder).map_err(PluginInstallError::InfoFile)?;

    // If the plugin is in dev mode, a junction is used.
    // In this case, we cannot use the canonic path of the plugin since it would
    // point to the original folder instead of the junction.
    // The `load_plugin_info` uses the canonic path however.
    // In this case, we change the path afterwards to fix this issue.
    if in_dev_mode {
        plugin_info.path = folder.clone();
        debug!("Plugin installed in dev mode. Adjusting path to '{}'", plugin_info.path.display());
    }

    debug!("Plugin info: {:?}", plugin_info);
    let plugin_name = plugin_info.name.clone();

    if self.plugins.contains_key(&plugin_name) {
        info!("Cannot add plugin '{}' since its already installed", plugin_name);
        return Err(PluginInstallError::AlreadyInstalled);
    }

    debug!("Create the plugin");
    // Create and load the plugin
    let plugin = Plugin::new(self.lua.clone(), plugin_info, in_dev_mode);
    add_plugin_to_persistence(&mut self.persistent_states, &plugin, PersistentPluginState::Disabled);
    self.plugins.insert(plugin_name.clone(), plugin);

    debug!("Load the plugin");
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

    let plugin = match self.plugins.get_mut(name) {
        None => return Err(PluginManagerError::PluginNotFound),
        Some(p) => p,
    };

    // Persist change
    debug!("Remove the plugin from persistence");
    remove_plugin_from_persistence(&mut self.persistent_states, &plugin.info.name);

    // We will execute the plugin's disable function just that it has a chance to be uninstalled cleanly.
    // However, we won't care if the plugin's disable function will throw an error and still remove it afterwards.
    debug!("Disable plugin");
    if let Err(e) = plugin.disable() {
        warn!("Plugin {} threw an error while it was disabled: {:?}", name, e);
    };

    // Unload the plugin.
    // This should drop all references to lua objects and also run lua's garbage collector.
    // However, it this call fails, removing the Plugin from the map should still work
    debug!("Unload plugin");
    if let Err(e) = plugin.unload() {
        warn!("Plugin {} threw an error while unloading: {:?}", name, e);
    }

    // Delete the plugin folder
    debug!("Delete plugin folder");
    delete_plugin_folder(&plugin)?;
    
    // Remove the plugin from the plugin map.
    // This should only return None due to race conditions.
    // In such cases, log it.
    debug!("Remove plugin from list");
    if let None = self.plugins.remove(name) {
        warn!("Could not find plugin '{}' while removing it from the internal map", name);
    }

    // Ensure that all lua references and objects are destroyed properly.
    debug!("Let lua garbage collect");
    let _ = self.lua.gc_collect();
    let _ = self.lua.gc_collect();

    Ok(())
  }
}

fn copy_plugin_directory_to_plugins_folder(source: &PathBuf, destination: &PathBuf) -> Result<(), PluginInstallError> {
    debug!("Copying files from plugin package to destination");
    for file in WalkDir::new(source).into_iter().filter_map(|e| e.ok()) {
        let path = file.path();

        let relative_path = match path.strip_prefix(source) {
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

    Ok(())
}

fn softlink_plugin_directory_to_plugins_folder(source: &PathBuf, destination: &PathBuf) -> Result<(), PluginInstallError> {
    junction::create(&source, &destination)
        .map_err(|e| PluginInstallError::IO(format!("Could not create softlink: {}", e).to_string()))
}

/// Delete the given plugin's folder.
/// 
/// If the plugin is in developer mode, we expect the folder to be a junction.
/// Thus, we only remove the junction.
/// Otherwise, we completely remove the plugin folder.
fn delete_plugin_folder(plugin: &Plugin) -> Result<(), PluginManagerError> {
    match plugin.in_dev_mode {
        false => {
            fs::remove_dir_all(&plugin.info.path).map_err(PluginManagerError::Io)
        },
        true => {
            debug!("Remove plugin folder by deleting the junction");
            junction::delete(&plugin.info.path).map_err(PluginManagerError::Io)?;
            if plugin.info.path.exists() {
                fs::remove_dir_all(&plugin.info.path).map_err(PluginManagerError::Io)?;
            }

            Ok(())
        },
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
