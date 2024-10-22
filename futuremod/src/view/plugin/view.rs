use iced::Task;

use crate::widget::Element;

use super::components::plugin_details_view;


#[derive(Debug, Clone)]
pub struct Plugin {
  pub name: String,
}

#[derive(Debug, Clone)]
pub enum Message {
  GoBack,
  Enable(String),
  Disable(String),
  Reload(String),
  UninstallPrompt(String),
}

impl Plugin {
  pub fn new(name: String) -> Self {
    Plugin { name }
  }

  #[allow(unused)]
  pub fn update(&mut self, plugin: &mut futuremod_data::plugin::Plugin, message: Message) -> Task<Message> {
    Task::none()
  }

  pub fn view<'a>(&self, plugin: &futuremod_data::plugin::Plugin) -> Element<'a, Message> {
    plugin_details_view(&plugin, false)
  }
}