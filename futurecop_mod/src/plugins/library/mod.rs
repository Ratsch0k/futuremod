use serde::{Deserialize, Serialize};

pub mod dangerous;
pub mod game;


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PluginDependency {
  Dangerous,
  Game,
}