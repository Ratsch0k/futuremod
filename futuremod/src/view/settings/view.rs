use iced::Task;

use crate::{config::{self, Config}, widget::Element};

use super::{components::settings_overview, state::update};

#[derive(Debug, Clone)]
pub struct Settings {
  pub(super) mod_path: String,
  pub(super) mod_address: String,
  pub(super) process_name: String,
  pub(super) require_admin: bool,
  pub(super) error: Option<String>,
  pub(super) back_button: bool,
}

impl PartialEq<Config> for Settings {
    fn eq(&self, other: &Config) -> bool {
        self.mod_path == other.mod_path && self.mod_address == other.mod_address && self.process_name == other.process_name && self.require_admin == other.require_admin
    }
}

#[derive(Debug, Clone)]
pub enum Message {
  SelectModPath,
  Reset,
  ModPathChanged(String),
  ModAddressChanged(String),
  ProcessNameChanged(String),
  RequireAdminChanged(bool),
  SaveChanges,
  SetError(String),
  ClearError,
  GoBack,
  ResetToDefaults,
}

impl Settings {
  pub fn new() -> Self {
    let config = config::get();

    Settings {
      mod_path: config.mod_path.clone(),
      mod_address: config.mod_address.clone(),
      process_name: config.process_name.clone(),
      require_admin: config.require_admin,
      error: None,
      back_button: false,
    }
  }

  pub fn with_back_button(mut self, value: bool) -> Self {
    self.back_button = value;
    self
  }

  pub fn update(&mut self, message: Message) -> Task<Message> {
    update(self, message)
  }

  pub fn view(&self) -> Element<'_, Message> {
    let config = config::get();

    settings_overview(&self, config.clone())
  }
}