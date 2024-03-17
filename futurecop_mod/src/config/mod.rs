use serde::{Serialize, Deserialize};

#[derive(Default, Debug, Clone, PartialEq, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerConfig {
    pub sprint_key: u32,
    pub invincible: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerConfig {
    pub port: u32,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub player_one: PlayerConfig,
    pub player_two: PlayerConfig,
    pub server: ServerConfig,
    pub log_level: String,

    /// Fixed path to the plugins directory.
    /// By default this option is None.
    /// 
    /// If this is None, it will load plugins from the directory "plugins" within
    /// the games root directory. For example: `C:\\Program Files (x86)\\Electronic Arts\\Future Cop\\plugins`
    pub plugins_directory: Option<String>,
}
