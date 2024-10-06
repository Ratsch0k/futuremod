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
    }
  }

  Task::none()
}