use serde_derive::{Deserialize, Serialize};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameMode {
  CrimeWar,
  PrecinctAssault,
}