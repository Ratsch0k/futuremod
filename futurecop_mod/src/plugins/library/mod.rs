use serde::{Deserialize, Serialize};

pub mod dangerous;
pub mod game;
pub mod input;


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PluginDependency {
  Dangerous,
  Game,
  Input,
}