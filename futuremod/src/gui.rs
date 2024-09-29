use iced::{executor, font, Application, Command, Subscription};
use log::debug;

use crate::palette::Palette;
use crate::{theme, widget::Element};

use super::view::{main, loading};

#[derive(Debug)]
pub struct Flags {
    pub is_developer: bool
}

impl Default for Flags {
    fn default() -> Self {
        Self { is_developer: false }
    }
}

/// State of the entire gui.
///
/// The state contains some global information as well as
/// the current view.
#[derive(Debug)]
pub struct ModInjector {
    /// Wether the GUI is in developer mode
    is_developer: bool,

    /// The current view.
    current_view: View,
}

#[derive(Debug)]
pub enum View {
    Loading(loading::Loading),
    Main(main::Main),
}

#[derive(Debug)]
pub enum Message {
    Loading(loading::Message),
    FontLoaded(Result<(), font::Error>),
    Main(main::Message),
}


impl Application for ModInjector {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = theme::Theme;
    type Flags = Flags;

    fn new(flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        let (loading, message) = loading::Loading::new();

        (
            ModInjector {
                is_developer: flags.is_developer,
                current_view: View::Loading(loading)
            },
            Command::batch(vec![
                font::load(iced_aw::BOOTSTRAP_FONT_BYTES).map(Message::FontLoaded),
                message.map(Message::Loading)
            ])
        )
    }

    fn title(&self) -> String {
        if self.is_developer {
            String::from("FutureMod - Developer Mode")
        } else {
            String::from("FutureMod")
        }

    }

    fn theme(&self) -> Self::Theme {
        theme::Theme::new(Palette::default())
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        debug!("Handling message: {:?}", message);

        match &mut self.current_view {
            View::Loading(loading) => {
                if let Message::Loading(loading::Message::IsModActive(true)) = message {
                    let main = main::Main::new(self.is_developer);
                    self.current_view = View::Main(main);
                    return Command::none()
                }

                if let Message::Loading(message) = message {
                    return loading.update(message).map(Message::Loading);
                }

                Command::none()
            },
            View::Main(main) => match message {
                Message::Main(message) => {
                    main.update(message).map(Message::Main)
                },
                _ => Command::none(),
            },
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        match &self.current_view {
            View::Loading(loading) => loading.view().map(Message::Loading),
            View::Main(main) => main.view().map(Message::Main),
        }
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        match &self.current_view {
            View::Main(main) => main.subscription().map(Message::Main),
            _ => Subscription::none(),
        }
    }
}