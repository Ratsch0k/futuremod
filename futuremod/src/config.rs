use std::{env, fs, path::Path};
use anyhow::anyhow;
use log::{debug, info};
use serde::{Deserialize, Serialize};
use tokio::sync::OnceCell;


#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    #[serde(default = "default_mod_path")]
    pub mod_path: String,

    #[serde(default = "default_mod_address")]
    pub mod_address: String,

    #[serde(default = "default_process_name")]
    pub process_name: String,

    #[serde(default = "default_require_admin")]
    pub require_admin: bool,
}

/// Get the default path to the mod dll.
/// 
/// We expect the dll to be inside the same directory as the injector.
fn default_mod_path() -> String {
  let mut current_dir_path = match env::current_dir() {
    Ok(v) => v,
    Err(e) => {
      panic!("Could not get the current directory: {}", e);
    }
  };

  current_dir_path.push("futuremod_engine.dll");

  let current_dir = current_dir_path.to_str().expect("Could not convert the path to the current directory to a string");

  String::from(current_dir)
}

fn default_mod_address() -> String {
  "127.0.0.1:8000".to_string()
}

fn default_process_name() -> String {
  "FCopLAPD.exe".to_string()
}

fn default_require_admin() -> bool {
  false
}

static CONFIG: OnceCell<Config> = OnceCell::<Config>::const_new();

fn create_default_config() -> Result<Config, serde_json::Error> {
  serde_json::from_str("{}")
}

fn get_config_from_path(path: &Path) -> Result<Config, anyhow::Error> {
  if path.exists() {
    info!("Reading the config");

    // If the file exists, read its contents and parse it
    let config_content = fs::read_to_string(path)
      .map_err(|e| anyhow!("Could not read the config: {}", e))?;

    let config: Config = serde_json::from_str(&config_content).
      map_err(|e| anyhow!("Could not parse the config: {}", e))?;

    Ok(config)
  } else {
    info!("Config file doesn't exist, creating the default config");

    // If the file doesn't exist, create a default config and create the file
    let config = create_default_config()
      .map_err(|e| anyhow!("Could not create the default config: {}", e))?;

    // Use pretty string. A human should be able to read and change the config
    let config_as_str = serde_json::to_string_pretty(&config)
      .map_err(|e| anyhow!("Could not convert the default config to string: {}", e))?;

    fs::write(path, config_as_str)
      .map_err(|e| anyhow!("Could not write the default config to file: {}", e))?;

    Ok(config)
  }
}

pub fn init(config_path_str: &str) -> Result<Config, anyhow::Error> {
  debug!("Initializing the config from '{}'", config_path_str);

  let config_path = Path::new(config_path_str);

  let config = get_config_from_path(config_path)?;

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