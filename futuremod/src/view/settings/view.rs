use iced::{widget::text, Task};

use crate::widget::Element;

#[derive(Debug, Clone)]
pub struct Settings;

#[derive(Debug, Clone)]
pub enum Message {

}

impl Settings {
  pub fn new() -> Self {
    Settings
  }

  pub fn update(&self, _message: Message) -> Task<Message> {
    Task::none()
  }

  pub fn view(&self) -> Element<'_, Message> {
    text("Settings")
      .into()
  }
}