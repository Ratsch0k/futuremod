use std::{fs, path::{Path, PathBuf}};

#[derive(Debug)]
pub enum PluginInfoError {
  /// The plugin directory doesn't contain a `info.toml` file
  FileNotFound,

  /// Some unexpected error occurred
  Other(String),

  /// The format of the `into.toml` file in incorrect
  Format(String),
}

/// Load the plugin info file from the given plugin folder.
/// If no plugin info file exists, returns an error.
pub fn load_plugin_info(path: &PathBuf) -> Result<futuremod_data::plugin::PluginInfo, PluginInfoError> {
    let path = path.canonicalize().map_err(|e| PluginInfoError::Other(format!("Could not access plugin folder: {:?}", e)))?;

    let info_file_path = Path::join(&path, "info.toml");

    if !info_file_path.exists() {
      return Err(PluginInfoError::FileNotFound);
    }

    let content = match fs::read_to_string(info_file_path) {
      Ok(c) => c,
      Err(e) => return Err(PluginInfoError::Other(format!("Could not read the plugin's info file: {:?}", e)))
    };

    let plugin_info: futuremod_data::plugin::PluginInfoContent = match toml::from_str(content.as_str()) {
      Ok(v) => v,
      Err(e) => return Err(PluginInfoError::Format(format!("Format of info file is incorrect: {:?}", e))),
    };

    Ok(futuremod_data::plugin::PluginInfo{
      path: path.clone(),
      name: plugin_info.name,
      authors: plugin_info.authors,
      version: plugin_info.version,
      dependencies: plugin_info.dependencies,
      description: plugin_info.description,
    })
  }