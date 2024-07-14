use iced::{executor, font, Application, Command, Subscription};
use log::trace;

use crate::palette::Palette;
use crate::{theme, widget::Element};

use super::view::{main, loading};


#[derive(Debug)]
pub enum ModInjector {
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
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        let (loading, message) = loading::Loading::new();

        (
            ModInjector::Loading(loading),
            Command::batch(vec![
                font::load(iced_aw::BOOTSTRAP_FONT_BYTES).map(Message::FontLoaded),
                message.map(Message::Loading)
            ])
        )
    }

    fn title(&self) -> String {
        String::from("FutureCop Mod")
    }

    fn theme(&self) -> Self::Theme {
        theme::Theme::new(Palette::default())
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        trace!("Handling message: {:?}", message);

        match self {
            ModInjector::Loading(loading) => {
                if let Message::Loading(loading::Message::IsModActive(true)) = message {
                    let main = main::Main::new();
                    *self = ModInjector::Main(main);
                    return Command::none()
                }

                if let Message::Loading(message) = message {
                    return loading.update(message).map(Message::Loading);
                }

                Command::none()
            },
            ModInjector::Main(main) => match message {
                Message::Main(message) => {
                    main.update(message).map(Message::Main)
                },
                _ => Command::none(),
            },
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        match self {
            ModInjector::Loading(loading) => loading.view().map(Message::Loading),
            ModInjector::Main(main) => main.view().map(Message::Main),
        }
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        match self {
            ModInjector::Main(main) => main.subscription().map(Message::Main),
            _ => Subscription::none(),
        }
    }
}