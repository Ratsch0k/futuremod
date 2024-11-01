use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::get_data_directory;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerConfig {
    pub port: u32,
    pub host: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SprintConfig {
    pub player_one: u32,
    pub player_two: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    #[serde(default = "default_server")]
    pub server: ServerConfig,

    #[serde(default = "default_log_level")]
    pub log_level: String,

    /// Fixed path to the plugins directory.
    /// By default this option is set to `<data_directory>\\plugins`.
    /// This should be `C:\\Users\\<user>\\AppData\\Roaming\\futuremod\\plugins`.
    #[serde(default = "default_plugins_directory")]
    pub plugins_directory: String,

    /// Optional sprint config that specifies for both players their sprint key.
    ///
    /// As the sprint mod should be shifted to an actual plugin this will be removed in the future.
    pub sprint_config: Option<SprintConfig>,
}

fn default_server() -> ServerConfig {
    ServerConfig {
        port: 8000,
        host: "127.0.0.1".to_string(),
    }
}

fn default_log_level() -> String {
    "INFO".to_string()
}

fn default_plugins_directory() -> String {
    get_data_directory()
        .map(|path| Path::join(&path, "plugins"))
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
}

impl Default for Config {
    fn default() -> Self {
        Config {
            server: default_server(),
            log_level: default_log_level(),
            plugins_directory: default_plugins_directory(),
            sprint_config: None,
        }
    }
}
