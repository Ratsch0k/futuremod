use std::{fmt::Display, path::PathBuf};

use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum PluginDependency {
  Dangerous,
  Game,
  Input,
  #[serde(rename = "ui")]
  UI,
  System,
  Matrix,

  // The following libraries are from the standard library
  Math,
  Table,
  Bit32,
  String,
  Utf8,
}

impl Display for PluginDependency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      match self {
        PluginDependency::Dangerous => f.write_str("Dangerous"),
        PluginDependency::Game => f.write_str("Game"),
        PluginDependency::Input => f.write_str("Input"),
        PluginDependency::UI => f.write_str("UI"),
        PluginDependency::System => f.write_str("System"),
        PluginDependency::Math => f.write_str("Math"),
        PluginDependency::Table => f.write_str("Table"),
        PluginDependency::Bit32 => f.write_str("Bit32"),
        PluginDependency::String => f.write_str("String"),
        PluginDependency::Utf8 => f.write_str("Utf8"),
        PluginDependency::Matrix => f.write_str("Matrix"),
      }
    }
}


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

#[derive(Debug, Serialize, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum PluginError {
    Error(String),
    NotEnabledError,
    NoMainFile,
    ScriptError(String),
    NotLoaded,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum PluginState {
    Error(PluginError),
    Unloaded,
    Loaded(PluginContext),
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
#[serde(rename_all = "camelCase")]
pub struct PluginContext {
    pub on_load: bool,
    pub on_unload: bool,
    pub on_update: bool,
    pub on_enable: bool,
    pub on_disable: bool,
    pub on_install: bool,
    pub on_uninstall: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Plugin {
  pub enabled: bool,
  pub state: PluginState,
  pub info: PluginInfo,
}