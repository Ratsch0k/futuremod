use std::{fs, path::Path};
use anyhow::anyhow;
use log::debug;
use serde::{Deserialize, Serialize};
use tokio::sync::OnceCell;


#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub mod_path: String,
    pub mod_address: String,
    pub process_name: String,
    pub require_admin: bool,
}

static CONFIG: OnceCell<Config> = OnceCell::<Config>::const_new();

fn create_config_error(path: &str, reason: &str) -> anyhow::Error {
  anyhow!("could not load config from '{}': {}", path, reason)
}

pub fn init(config_path_str: &str) -> Result<Config, anyhow::Error> {
  let config_path = Path::new(config_path_str);

  debug!("Loading config content");
  let config_content = match config_path.exists() {
    false => return Err(create_config_error(config_path_str, "file doesn't exist")),
    true => match fs::read_to_string(config_path) {
      Err(e) => {
        return Err(create_config_error(config_path_str, format!("{:?}", e).as_str()))
      },
      Ok(config_content) => config_content,
    }
  };

  debug!("Parsing config");
  let config: Config = match serde_json::from_str(&config_content) {
    Ok(config) => config,
    Err(e) => return Err(create_config_error(config_path_str, format!("{:?}", e).as_str())),
  };

  debug!("Setting config global");
  match CONFIG.set(config) {
    Ok(_) => debug!("set config"),
    Err(_) => {
      debug!("didn't set config");
      return Err(anyhow!("config is already loaded"));
    }
  }

  assert!(CONFIG.get().is_some(), "config wasn't set");

  Ok(get_config())
}

pub fn get_config() -> Config {
  match CONFIG.get() {
    Some(config) => config.clone(),
    None => panic!("config was not initialized")
  }
}