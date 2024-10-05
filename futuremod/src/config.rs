use std::{env, fmt::Display, fs, io, path::{Path, PathBuf}, sync::{Arc, OnceLock, RwLock, RwLockReadGuard}};
use anyhow::anyhow;
use log::{debug, info};
use serde::{Deserialize, Serialize};


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

pub fn init(config_path_str: &str) -> Result<(), anyhow::Error> {
  debug!("Initializing the config from '{}'", config_path_str);

  let config_path = Path::new(config_path_str);

  let config = get_config_from_path(config_path)?;

  debug!("Initializing global config");

  let global_config = GlobalConfig {
    path: PathBuf::from(config_path_str),
    config: Arc::new(RwLock::new(config)),
  };

  match GLOBAL_CONFIG.set(global_config) {
    Ok(_) => debug!("Initialized config"),
    Err(_) => {
      debug!("Could not initialize config");
      return Err(anyhow!("Config already initialized"))
    },
  }

  assert!(GLOBAL_CONFIG.get().is_some(), "Config wasn't initialized");

  Ok(())
}

#[derive(Default, Clone)]
pub struct GlobalConfig {
  path: PathBuf,
  config: Arc<RwLock<Config>>,
}

static GLOBAL_CONFIG: OnceLock<GlobalConfig> = OnceLock::new();

pub fn get<'a>() -> RwLockReadGuard<'a, Config> {
  let config = GLOBAL_CONFIG.get().expect("Config not initialized");

  config.config.read().expect("Cannot read config, it is locked")
}

pub enum ConfigWriteError {
  Serialization(serde_json::Error),
  Io(io::Error),
}

impl Display for ConfigWriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      let content = match self {
        ConfigWriteError::Serialization(e) => format!("Could not serialize config: {}", e),
        ConfigWriteError::Io(e) => format!("Could not write config to disk: {}", e),
      };

      f.write_str(&content)
    }
}

pub fn update(f: impl Fn(&mut Config) -> ()) -> Result<(), ConfigWriteError> {
  let config = GLOBAL_CONFIG.get().expect("Config not initialized");

  let mut writable_config = config.config.write().expect("Cannot write to config, it is locked");

  // Let the provided function update the config
  f(&mut writable_config);
  
  // Write the changed config to the config file
  let config_content = serde_json::to_string::<Config>(&writable_config).map_err(ConfigWriteError::Serialization)?;

  fs::write(&config.path, config_content).map_err(ConfigWriteError::Io)
}