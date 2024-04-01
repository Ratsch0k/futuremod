use iced::{widget::{column, text}, Alignment, Command, Length};
use log::debug;

use crate::{config::get_config, log_subscriber::{self, LogRecord}, theme::Button, widget::{button, Element}};

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
        debug!("Handling message: {:?}", message);

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
        match &self.view {
            None => {
                column![
                    text("FutureCop Mod").size(24),
                    column![
                        button("Logs").on_press(Message::ToLogs).style(Button::Primary),
                        button("Plugins").on_press(Message::ToPlugins).style(Button::Primary)
                    ].spacing(4)
                ]
                .width(Length::Fill)
                .spacing(16)
                .align_items(Alignment::Center)
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