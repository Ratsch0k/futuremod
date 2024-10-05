use iced::{Subscription, Task};
use log::debug;

use crate::palette::Palette;
use crate::view::dashboard;
use crate::{theme, widget::Element};

use super::view::loading;

/// State of the entire gui.
///
/// The state contains some global information as well as
/// the current view.
#[derive(Debug)]
pub struct ModInjector{
    /// Wether the GUI is in developer mode
    is_developer: bool,

    /// The current view.
    current_view: View,
}

#[derive(Debug)]
pub enum View {
    Loading(loading::Loading),
    Dashboard(dashboard::Dashboard),
}

#[derive(Debug)]
pub enum Message {
    Loading(loading::Message),
    Dashboard(dashboard::Message),
}

pub fn title(gui: &ModInjector) -> String {
    if gui.is_developer {
        String::from("FutureMod - Developer Mode")
    } else {
        String::from("FutureMod")
    }
}

pub fn theme(_gui: &ModInjector) -> theme::Theme {
    theme::Theme::new(Palette::default())
}

pub fn update(gui: &mut ModInjector, message: Message) -> Task<Message> {
    debug!("Handling message: {:?}", message);

    match &mut gui.current_view {
        View::Loading(loading) => {
            if let Message::Loading(loading::Message::GotPlugins(plugins)) = message {
                gui.current_view = View::Dashboard(dashboard::Dashboard::new(plugins, gui.is_developer));
                return Task::none()
            }

            if let Message::Loading(message) = message {
                return loading.update(message).map(Message::Loading);
            }

            Task::none()
        },
        View::Dashboard(dashboard) => match message {
            Message::Dashboard(message) => {
                dashboard.update(message).map(Message::Dashboard)
            },
            _ => Task::none(),
        },
    }
}

pub fn view(gui: &ModInjector) -> Element<Message> {
    match &gui.current_view {
        View::Loading(loading) => loading.view().map(Message::Loading),
        View::Dashboard(main) => main.view().map(Message::Dashboard),
    }
}

pub fn subscription(gui: &ModInjector) -> iced::Subscription<Message> {
    match &gui.current_view {
        View::Dashboard(main) => main.subscription().map(Message::Dashboard),
        _ => Subscription::none(),
    }
}

impl ModInjector {
    pub fn new(is_developer: bool) -> (Self, Task<Message>) {
        let (loading, message) = loading::Loading::new();

        // TOOD: Reenable flags
        (
            ModInjector {
                is_developer,
                current_view: View::Loading(loading)
            },
            message.map(Message::Loading)
        )
    }
}
