use std::collections::HashMap;

use futuremod_data::plugin::Plugin;
use iced::Task;

use crate::widget::Element;

use super::components::plugin_overview;


#[derive(Debug, Clone)]
pub struct PluginList;

#[derive(Debug, Clone)]
pub enum Message {
  Install,
  DevInstall,
  Enable(String),
  Disable(String),
  ToPlugin(String),
}

impl PluginList {
  pub fn new() -> Self {
    PluginList
  }

  pub fn update(&self, _message: Message) -> Task<Message> {
    Task::none()
  }

  pub fn view<'a>(&self, plugins: &'a HashMap<String, Plugin>, is_developer: bool) -> Element<'a, Message> {
    plugin_overview(plugins, is_developer)
  }
}