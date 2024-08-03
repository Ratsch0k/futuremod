use serde::{Serialize, Deserialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerConfig {
    pub port: u32,
    pub host: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SprintConfig {
    pub player_one: u32,
    pub player_two: u32,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    #[serde(default = "default_server")]
    pub server: ServerConfig,

    #[serde(default = "default_log_level")]
    pub log_level: String,

    /// Fixed path to the plugins directory.
    /// By default this option is None.
    /// 
    /// If this is None, it will load plugins from the directory "plugins" within
    /// the games root directory. For example: `C:\\Program Files (x86)\\Electronic Arts\\Future Cop\\plugins`
    pub plugins_directory: Option<String>,

    /// Optional sprint config that specifies for both players their sprint key.
    /// 
    /// As the sprint mod should be shifted to an actual plugin this will be removed in the future.
    pub sprint_config: Option<SprintConfig>,
}

fn default_server() -> ServerConfig {
    ServerConfig {
        port: 8000,
        host: "0.0.0.0".to_string(),
    }
}

fn default_log_level() -> String {
    "INFO".to_string()
}