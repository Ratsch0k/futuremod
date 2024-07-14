use iced::{alignment::{Horizontal, Vertical}, widget::{column, container, text}, Alignment, Command, Length};
use log::trace;

use crate::{config::get_config, log_subscriber::{self, LogRecord}, theme::{Button, Theme}, widget::{button, Element}};

use super::{logs, plugins};

#[derive(Debug, Clone)]
pub enum View {
    Plugins(plugins::Plugins),
    Logs(logs::Logs),
}

#[derive(Debug, Clone)]
pub enum Message {
    ToLogs,
    ToPlugins,
    Plugins(plugins::Message),
    Logs(logs::Message),
    LogEvent(log_subscriber::Event)
}

#[derive(Debug, Clone)]
#[allow(unused)]
pub enum LogState {
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct Logs {
    pub state: LogState,
    pub logs: Vec<LogRecord>,
}

#[derive(Debug, Clone)]
pub struct Main {
    logs: Logs,
    view: Option<View>,
}

impl Main {
    pub fn new() -> Self {
        Main {
            logs: Logs { state: LogState::Disconnected, logs: Vec::new() },
            view: None,
        }
    }

    pub fn update(&mut self, message: Message) -> iced::Command<Message> {
        trace!("Handling message: {:?}", message);

        match message {
            Message::LogEvent(message) => {

                match message {
                    log_subscriber::Event::Connected => {
                        self.logs.state = LogState::Connected;
                    },
                    log_subscriber::Event::Disconnected => {
                        self.logs.state = LogState::Error(format!("Got disconnected"));
                        self.logs.logs.clear();
                    },
                    log_subscriber::Event::Message(message) => {
                        self.logs.logs.push(message);
                    },
                };

                return Command::none();
            }
            _ => (),
        }

        match &mut self.view {
            Some(view) => match view {
                View::Plugins(plugins) => match message {
                    Message::Plugins(plugins::Message::GoBack) => {
                        self.view = None;
                        Command::none()
                    },
                    Message::Plugins(message) => return plugins.update(message).map(Message::Plugins),
                    _ => Command::none(),
                }
                View::Logs(logs) => match message {
                    Message::Logs(logs::Message::GoBack) => {
                        self.view = None;
                        Command::none()
                    },
                    Message::Logs(msg) => {
                        logs.update(msg).map(Message::Logs)
                    },
                    _ => Command::none(),
                },
            },
            None => match message {
                Message::ToPlugins => {
                    let (view, message) = plugins::Plugins::new();

                    self.view = Some(View::Plugins(view));
                    message.map(Message::Plugins)
                },
                Message::ToLogs => {
                    let (view, message) = logs::Logs::new();
                    self.view = Some(View::Logs(view));
                    message.map(Message::Logs)
                },
                _ => Command::none()
            },
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        fn menu_button(label: &str) -> iced::widget::Button<'_, Message, Theme> {
            button(text(label).horizontal_alignment(Horizontal::Center).width(Length::Fill)).width(Length::Fill).height(36)
        }

        match &self.view {
            None => {
                container(
                    column![
                        text("FutureCop Mod").size(48),
                        column![
                            menu_button("Plugins").on_press(Message::ToPlugins).style(Button::Primary),
                            menu_button("Logs").on_press(Message::ToLogs)
                        ]
                        .spacing(8)
                        .width(Length::Fill)
                        .max_width(200)
                        .align_items(Alignment::Center)
                    ].spacing(24)
                    .align_items(Alignment::Center)
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Horizontal::Center)
                .align_y(Vertical::Center)
                .into()
            },
            Some(view) => match view {
                View::Plugins(plugins) => plugins.view().map(Message::Plugins),
                View::Logs(logs) => logs.view(&self.logs).map(Message::Logs),
            }
        }
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        let config = get_config();
        
        log_subscriber::connect(config.mod_address.clone()).map(Message::LogEvent)
    }
}