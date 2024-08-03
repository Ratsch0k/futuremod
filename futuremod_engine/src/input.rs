use std::{collections::HashSet, sync::{Arc, Mutex}};

use device_query::{DeviceQuery, DeviceState, Keycode};

lazy_static! {
  static ref KEY_STATE: Arc<Mutex<HashSet<Keycode>>> = Arc::new(Mutex::new(HashSet::new()));
}

/// Globally shared key state.
/// 
/// Keeps track of all keys pressed by the user in the current frame.
/// Must be manually updated every frame but only one time.
/// The key state is globally stored and new instances will be automatically updated without
/// the need to call [`input::KeyState.update`].
pub struct KeyState {
  state: Arc<Mutex<HashSet<Keycode>>>,
}

impl KeyState {
  pub fn new() -> Self {
    KeyState {state: KEY_STATE.clone()}
  }

  /// Update the key state.
  /// 
  /// **Only call this function once per frame**
  pub fn update(&self) -> Result<(), anyhow::Error> {
    let device_state = DeviceState::new();
    let pressed_keys = device_state.get_keys();

    match self.state.lock() {
        Ok(mut key_state) => {
          key_state.clear();

          for key in pressed_keys {
            key_state.insert(key);
          }

          Ok(())
        },
        Err(e) => anyhow::bail!("Could not get lock to key state global: {}", e.to_string()),
    }
  }

  /// Get all currently pressed keys.
  /// 
  /// The returned hashset will contain all keys that are currently pressed.
  /// Every key not in the set are currently not pressed.
  pub fn get_state(&self) -> Result<HashSet<Keycode>, anyhow::Error> {
    match self.state.lock() {
      Ok(key_state) => {
        Ok(key_state.clone())
      },
      Err(e) => {
        anyhow::bail!("Could not get lock to key state global: {}", e.to_string())
      },
    }
  }

  /// Check if the given key is pressed.
  pub fn is_key_pressed(&self, code: Keycode) -> Result<bool, anyhow::Error> {
    match self.state.lock() {
      Ok(key_state) => Ok(key_state.contains(&code)),
      Err(e) => {
        anyhow::bail!("Could not get lock to key state: {}", e.to_string())
      }
    }
  }
}

