use iced::{widget::markdown, Task};

use crate::widget::Element;

use super::components::plugin_details_view;


#[derive(Debug, Clone)]
pub struct Plugin {
  pub name: String,
  pub description: Vec<markdown::Item>,
}

#[derive(Debug, Clone)]
pub enum Message {
  GoBack,
  Enable(String),
  Disable(String),
  Reload(String),
  UninstallPrompt(String),
  Empty(reqwest::Url)
}

impl Plugin {
  pub fn new(plugin: &futuremod_data::plugin::Plugin) -> Self {
    let description = markdown::parse(&plugin.info.description).collect();

    Plugin { name: plugin.info.name.clone(), description }
  }

  #[allow(unused)]
  pub fn update(&mut self, plugin: &mut futuremod_data::plugin::Plugin, message: Message) -> Task<Message> {
    Task::none()
  }

  pub fn view<'a>(&'a self, plugin: &futuremod_data::plugin::Plugin) -> Element<'a, Message> {
    plugin_details_view(self, &plugin, false)
  }
}