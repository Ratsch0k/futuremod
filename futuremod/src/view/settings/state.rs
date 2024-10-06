use iced::Task;
use rfd::FileDialog;

use crate::config;

use super::{Message, Settings};

pub fn update(settings: &mut Settings, message: Message) -> Task<Message> {
  match message {
    Message::ModPathChanged(mod_path) => {
      settings.mod_path = mod_path;
    },
    Message::SaveChanges => {
      if let Err(e) = config::update(|config| {
        config.mod_path = settings.mod_path.clone();
        config.mod_address = settings.mod_address.clone();
        config.process_name = settings.process_name.clone();
        config.require_admin = settings.require_admin.clone();
      }) {
        return Task::done(Message::SetError(e.to_string()));
      }
    },
    Message::ModAddressChanged(value) => {
      settings.mod_address = value;
    },
    Message::ProcessNameChanged(value) => {
      settings.process_name = value;
    },
    Message::RequireAdminChanged(value) => {
      settings.require_admin = value;
    },
    Message::ClearError => {
      settings.error = None;
    },
    Message::SetError(error) => {
      settings.error = Some(error);
    },
    Message::GoBack => (),
    Message::SelectModPath => {
      return match FileDialog::new()
        .add_filter("FutureMod engine", &["dll"])
        .set_title("Select the FutureMod engine DLL")
        .pick_file()
      {
        Some(path) => match path.to_str() {
          Some(v) => Task::done(Message::ModPathChanged(v.to_string())),
          None => Task::done(Message::SetError("Selected an invalid path".into())),
        },
        None => Task::none()
      }
    },
    Message::Reset => {
      let config = config::get();

      settings.error = None;
      settings.mod_address = config.mod_address.clone();
      settings.mod_path = config.mod_path.clone();
      settings.process_name = config.process_name.clone();
      settings.require_admin = config.require_admin.clone();
    },
    Message::ResetToDefaults => {
      match config::create_default_config() {
        Ok(default_config) => {
          settings.error = None;
          settings.mod_address = default_config.mod_address;
          settings.mod_path = default_config.mod_path;
          settings.process_name = default_config.process_name;
          settings.require_admin = default_config.require_admin;

          return Task::done(Message::SaveChanges);
        },
        Err(e) => {
          return Task::done(Message::SetError(format!("Could not get default config: {}", e)));
        }
      }
    }
  }

  Task::none()
}