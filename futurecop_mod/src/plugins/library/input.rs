use std::{str::FromStr, sync::Arc};

use device_query::Keycode;
use log::*;
use mlua::{Lua, OwnedTable};

use crate::input::KeyState;


/// List of supported key codes.
/// Copied from [`device_query::Keycode`]
const SUPPORTED_KEYCODES: [Keycode; 100] = [
  Keycode::Key0,
  Keycode::Key1,
  Keycode::Key2,
  Keycode::Key3,
  Keycode::Key4,
  Keycode::Key5,
  Keycode::Key6,
  Keycode::Key7,
  Keycode::Key8,
  Keycode::Key9,
  Keycode::A,
  Keycode::B,
  Keycode::C,
  Keycode::D,
  Keycode::E,
  Keycode::F,
  Keycode::G,
  Keycode::H,
  Keycode::I,
  Keycode::J,
  Keycode::K,
  Keycode::L,
  Keycode::M,
  Keycode::N,
  Keycode::O,
  Keycode::P,
  Keycode::Q,
  Keycode::R,
  Keycode::S,
  Keycode::T,
  Keycode::U,
  Keycode::V,
  Keycode::W,
  Keycode::X,
  Keycode::Y,
  Keycode::Z,
  Keycode::F1,
  Keycode::F2,
  Keycode::F3,
  Keycode::F4,
  Keycode::F5,
  Keycode::F6,
  Keycode::F7,
  Keycode::F8,
  Keycode::F9,
  Keycode::F10,
  Keycode::F11,
  Keycode::F12,
  Keycode::Escape,
  Keycode::Space,
  Keycode::LControl,
  Keycode::RControl,
  Keycode::LShift,
  Keycode::RShift,
  Keycode::LAlt,
  Keycode::RAlt,
  Keycode::Command,
  Keycode::LOption,
  Keycode::ROption,
  Keycode::LMeta,
  Keycode::RMeta,
  Keycode::Enter,
  Keycode::Up,
  Keycode::Down,
  Keycode::Left,
  Keycode::Right,
  Keycode::Backspace,
  Keycode::CapsLock,
  Keycode::Tab,
  Keycode::Home,
  Keycode::End,
  Keycode::PageUp,
  Keycode::PageDown,
  Keycode::Insert,
  Keycode::Delete,
  Keycode::Numpad0,
  Keycode::Numpad1,
  Keycode::Numpad2,
  Keycode::Numpad3,
  Keycode::Numpad4,
  Keycode::Numpad5,
  Keycode::Numpad6,
  Keycode::Numpad7,
  Keycode::Numpad8,
  Keycode::Numpad9,
  Keycode::NumpadSubtract,
  Keycode::NumpadAdd,
  Keycode::NumpadDivide,
  Keycode::NumpadMultiply,
  Keycode::Grave,
  Keycode::Minus,
  Keycode::Equal,
  Keycode::LeftBracket,
  Keycode::RightBracket,
  Keycode::BackSlash,
  Keycode::Semicolon,
  Keycode::Apostrophe,
  Keycode::Comma,
  Keycode::Dot,
  Keycode::Slash,
];


fn keycode_to_string(keycode: Keycode) -> String {
  format!("Key{}", keycode.to_string())
}


fn keycode_from_string(key: String) -> Result<Keycode, mlua::Error> {
  let code_name = key.replace("Key", "");

  Keycode::from_str(&code_name).map_err(|_| mlua::Error::RuntimeError("Invalid key code".into()))
}


fn insert_keycode(table: &mlua::Table, code: Keycode) -> Result<(), mlua::Error> {
  let code = keycode_to_string(code);
  table.set(code.clone(), code)
}


pub fn create_input_library(lua: Arc<Lua>) -> Result<OwnedTable, mlua::Error> {
  debug!("Creating input library");
  let library = lua.create_table()?;

  // Insert supported key codes into library table.
  for key in SUPPORTED_KEYCODES {
    insert_keycode(&library, key)?;
  }

  let key_state = KeyState::new();

  let is_key_pressed_function = lua.create_function(move |_, key: String| {
    let keycode = keycode_from_string(key)?;

    match key_state.is_key_pressed(keycode) {
      Ok(v) => Ok(v),
      Err(e) => {
        warn!("Error while checking if key {} is pressed: {}", keycode, e.to_string());

        Err(mlua::Error::RuntimeError("Error while checking key state".into()))
      }
    }
  })?;
  library.set("isKeyPressed", is_key_pressed_function)?;

  Ok(library.into_owned())
}