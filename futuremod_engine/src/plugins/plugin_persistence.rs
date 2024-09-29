use std::{collections::HashMap, fs, path::{Path, PathBuf}};

use log::debug;
use serde::{Deserialize, Serialize};
use anyhow::{bail, anyhow};

/// Persistence state of a plugin which indicates how a plugin should be loaded on the next start.
/// 
/// This doesn't reflect the actual plugin's state.
/// For example, if a plugin was loaded and enabled but threw an error during the loading process
/// and thus has now the state [`PluginState::Error`], it will have the state [`StoredPluginState::Disabled`].
/// Rather, this states whether the plugin manager will load and/or enable the plugin when it starts the next time.
/// This state is only updated due to the user's input.
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum PersistentPluginState {
    Unloaded,
    Disabled,
    Enabled,
}

/// Persistent plugin information.
/// 
/// Contains all the information necessary for the plugin manager to load a plugin
/// from the plugin folder.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedPlugin {
    pub state: PersistentPluginState,
    pub in_dev_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedPlugins {
    states: HashMap<String, PersistedPlugin>,
    path: PathBuf,
}

impl PersistedPlugins {
    pub fn new(path: &Path) -> Result<PersistedPlugins, anyhow::Error> {
        debug!("Reading plugin states from '{}'", path.display());

        let states: HashMap<String, PersistedPlugin> = match fs::read_to_string(path) {
            Ok(content) => serde_json::from_str(&content).map_err(|e| anyhow!("could not parse the plugin states file: {}", e.to_string()))?,
            Err(_) => HashMap::new(),
        };

        Ok(PersistedPlugins { states, path: path.to_path_buf() })
    }

    pub fn get_state(&self, name: &str) -> Option<&PersistedPlugin> {
        self.states.get(name)
    }

    pub fn insert(&mut self, name: &str, state: PersistedPlugin) -> Result<(), anyhow::Error>{
        self.states.insert(name.into(), state);

        self.write_to_file()
    }

    pub fn update_state(&mut self, name: &str, state: PersistentPluginState) -> Result<(), anyhow::Error> {
        let plugin_state = match self.states.get_mut(name) {
            Some(p) => p,
            None => bail!("Plugin doesn't exist"),
        };

        plugin_state.state = state;

        self.write_to_file()
    }

    pub fn write_to_file(&self) -> Result<(), anyhow::Error> {
        let content = serde_json::to_string(&self.states).map_err(|e| anyhow!("could not serialize plugin states to string: {}", e.to_string()))?;

        fs::write(&self.path, content).map_err(|e| anyhow!("could not persist change: {}", e.to_string()))
    }

    pub fn remove(&mut self, name: &str) -> Result<(), anyhow::Error> {
        self.states.remove(name);

        self.write_to_file()
    }
}