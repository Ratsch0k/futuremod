use std::{fs, path::{Path, PathBuf}};

use serde::{Deserialize, Serialize};

use super::library::PluginDependency;


/// Plugin information struct used during serialization.
/// 
/// See [`PluginInfo`] for information about the individual fields.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginInfoContent {
  pub name: String,
  pub authors: Vec<String>,
  pub version: String,
  #[serde(default)]
  pub dependencies: Vec<PluginDependency>,
  #[serde(default)]
  pub description: String,
}

/// Plugin information.
/// 
/// Contains all information about a plugin, such as name and authors.
/// These information are loaded from the plugin `info.toml` file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
  /// Path to the plugin
  pub path: PathBuf,

  /// The plugin's name
  pub name: String,

  /// The list of authors
  pub authors: Vec<String>,

  /// The plugin's version
  pub version: String,

  /// List of libraries requested by the plugin.
  ///
  /// A plugin only is granted access to the library it requests.
  pub dependencies: Vec<PluginDependency>,

  /// Plugin description.
  /// 
  /// A short plugin description that explains what the plugin does.
  pub description: String,
}



#[derive(Debug)]
pub enum PluginInfoError {
  /// The plugin directory doesn't contain a `info.toml` file
  FileNotFound,

  /// Some unexpected error occurred
  Other(String),

  /// The format of the `into.toml` file in incorrect
  Format(String),
}


impl PluginInfo {
  pub fn from_folder(path: PathBuf) -> Result<PluginInfo, PluginInfoError> {
    let path = path.canonicalize().map_err(|e| PluginInfoError::Other(format!("Could not access plugin folder: {:?}", e)))?;

    let info_file_path = Path::join(&path, "info.toml");

    if !info_file_path.exists() {
      return Err(PluginInfoError::FileNotFound);
    }

    let content = match fs::read_to_string(info_file_path) {
      Ok(c) => c,
      Err(e) => return Err(PluginInfoError::Other(format!("Could not read the plugin's info file: {:?}", e)))
    };

    let plugin_info: PluginInfoContent = match toml::from_str(content.as_str()) {
      Ok(v) => v,
      Err(e) => return Err(PluginInfoError::Format(format!("Format of info file is incorrect: {:?}", e))),
    };

    Ok(PluginInfo{
      path,
      name: plugin_info.name,
      authors: plugin_info.authors,
      version: plugin_info.version,
      dependencies: plugin_info.dependencies,
      description: plugin_info.description,
    })
  }
}

pub fn load_plugin_info(path: PathBuf) -> Result<futurecop_data::plugin::PluginInfo, PluginInfoError> {
    let path = path.canonicalize().map_err(|e| PluginInfoError::Other(format!("Could not access plugin folder: {:?}", e)))?;

    let info_file_path = Path::join(&path, "info.toml");

    if !info_file_path.exists() {
      return Err(PluginInfoError::FileNotFound);
    }

    let content = match fs::read_to_string(info_file_path) {
      Ok(c) => c,
      Err(e) => return Err(PluginInfoError::Other(format!("Could not read the plugin's info file: {:?}", e)))
    };

    let plugin_info: futurecop_data::plugin::PluginInfoContent = match toml::from_str(content.as_str()) {
      Ok(v) => v,
      Err(e) => return Err(PluginInfoError::Format(format!("Format of info file is incorrect: {:?}", e))),
    };

    Ok(futurecop_data::plugin::PluginInfo{
      path,
      name: plugin_info.name,
      authors: plugin_info.authors,
      version: plugin_info.version,
      dependencies: plugin_info.dependencies,
      description: plugin_info.description,
    })
  }