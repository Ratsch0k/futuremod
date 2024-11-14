use futuremod_data::plugin::settings::{Event, PluginSettings};
use futures::TryFutureExt;
use iced::{widget::markdown, Task};

use crate::{api::get_plugin_settings, widget::Element};

use super::{components::plugin_details_view, state::update};

#[derive(Debug, Clone)]
pub enum FutureResult<T, E> {
    Loading,
    Finished(T),
    Error(E),
}

#[derive(Debug, Clone)]
pub struct Plugin {
    pub name: String,
    pub description: Vec<markdown::Item>,
    pub settings: FutureResult<PluginSettings, String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    GoBack,
    Enable(String),
    Disable(String),
    Reload(String),
    UninstallPrompt(String),
    OpenUrl(reqwest::Url),
    HandlePluginSettingsResponse(Result<PluginSettings, String>),
    SettingsEvent(String, Event),
    EventResponse(Result<(), String>),
}

impl Plugin {
    pub fn new(plugin: &futuremod_data::plugin::Plugin) -> (Self, Task<Message>) {
        let description = markdown::parse(&plugin.info.description).collect();

        (
            Plugin {
                name: plugin.info.name.clone(),
                description,
                settings: FutureResult::Loading,
            },
            Task::perform(get_plugin_settings(plugin.info.name.clone()).map_err(|e| e.to_string()), Message::HandlePluginSettingsResponse),
        )
    }

    pub fn update(&mut self, plugin: &futuremod_data::plugin::Plugin, message: Message) -> Task<Message> {
        update(self, plugin, message)
    }

    pub fn view<'a>(&'a self, plugin: &futuremod_data::plugin::Plugin) -> Element<'a, Message> {
        plugin_details_view(self, &plugin, false)
    }
}
