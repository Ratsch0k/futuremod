use futures::TryFutureExt;
use iced::Task;
use log::info;

use crate::api::send_settings_event;

use super::{view::FutureResult, Message};

pub(super) fn update(view: &mut super::Plugin, plugin: &futuremod_data::plugin::Plugin, message: Message) -> Task<Message> {
    match message {
        Message::HandlePluginSettingsResponse(result) => match result {
            Ok(settings) => view.settings = FutureResult::Finished(settings),
            Err(e) => view.settings = FutureResult::Error(e),
        },
        Message::SettingsEvent(id, event) => {
            return Task::perform(send_settings_event(plugin.info.name.clone(), id, event).map_err(|e| e.to_string()), Message::EventResponse);
        },
        Message::EventResponse(response) => {
            info!("Event response: {:?}", response);
        }
        _ => (),
    }

    Task::none()
}