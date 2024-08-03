use std::collections::HashMap;

use chrono::{DateTime, Utc};
use futuremod_data::plugin::Plugin;
use iced::{alignment::{Horizontal, Vertical}, widget::{checkbox, column, container, row, scrollable::{Alignment, Direction, Properties, Scrollable}, text}, Command, Length, Renderer};
use iced_aw::{menu::{Item, Menu}, menu_bar, menu_items, BootstrapIcon};

use crate::{api::get_plugins, theme::{Button, Theme}, widget::bold};
use crate::{log_subscriber::LogRecord, theme, view::main::LogState, widget::{button, icon, Element}};

use super::main;

const MAX_HISTORY: isize = 250;

#[derive(Debug, Clone)]
pub enum Message {
    GoBack,
    ToggleHistory(bool),
    ToggleLevelDebug(bool),
    ToggleLevelInfo(bool),
    ToggleLevelWarn(bool),
    ToggleLevelError(bool),
    GetPluginResponse(Result<HashMap<String, Plugin>, String>),
    ChangeOriginSelection(LogOrigin, bool),
    None,
}

#[derive(Debug, Clone)]
pub struct SelectedLogLevels {
  debug: bool,
  info: bool,
  warn: bool,
  error: bool,
}

impl Default for SelectedLogLevels {
    fn default() -> Self {
        Self { debug: false, info: true, warn: true, error: true }
    }
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub enum LogOrigin {
  System,
  Plugin(String),
}

#[derive(Debug, Clone, Default)]
pub struct LogsState {
  unlimited_history: bool,
  selected_log_levels: SelectedLogLevels,
  selected_origins: HashMap<LogOrigin, bool>,
  plugins: HashMap<String, Plugin>,
}

#[derive(Debug, Clone)]
pub enum Logs {
  Loading,
  View(LogsState),
  Error(String),
}

fn log_level_to_text(level: &str) -> Element<Message> {
    let message = text(format!("[{}]", level));

    let color: iced::Color = match level {
        "DEBUG" => iced::Color::from_rgb8(150, 150, 150),
        "INFO" => iced::Color::from_rgb8(81, 113, 240),
        "WARN" => iced::Color::from_rgb8(238, 203, 63),
        "ERROR" => iced::Color::from_rgb8(241, 83, 75),
        _ => iced::Color::BLACK,
    };


    message
    .style(theme::Text::Color(color))
    .font(iced::Font{
        weight: iced::font::Weight::Bold,
        ..iced::Font::default()
    })
    .into()
}

impl Logs {
  pub fn new() -> (Self, Command<Message>) {
    (
      Logs::Loading,
      Command::perform(get_plugins(), Message::GetPluginResponse),
    )
  }

  pub fn view<'a>(&self, log: &'a main::Logs) -> Element<'a, Message> {
    match self {
      Logs::Loading => {
        container(text("Loading..."))
          .height(Length::Fill)
          .width(Length::Fill)
          .align_x(Horizontal::Center)
          .align_y(Vertical::Center)
          .into()
      },
      Logs::View(loaded_logs) => {
        let content: Element<_> = match &log.state {
          LogState::Disconnected => text("Disconnected").into(),
          LogState::Connecting => text("Connecting").into(),
          LogState::Connected => {
              let mut filtered: Vec<&LogRecord> = Vec::new();

              for message in &log.logs {
                let valid = match &message.level.as_str() {
                  &"DEBUG" => loaded_logs.selected_log_levels.debug,
                  &"INFO" => loaded_logs.selected_log_levels.info,
                  &"WARN" => loaded_logs.selected_log_levels.warn,
                  &"ERROR" => loaded_logs.selected_log_levels.error,
                  _ => false,
                };

                if !valid {
                  continue
                }

                let is_selected_origin = match &message.plugin {
                  Some(origin) => {
                    let origin_key = LogOrigin::Plugin(origin.clone());

                    *loaded_logs.selected_origins.get(&origin_key).unwrap_or(&true)
                  },
                  None => {
                    *loaded_logs.selected_origins.get(&LogOrigin::System).unwrap_or(&true)
                  }
                };

                if !is_selected_origin {
                  continue
                }

                filtered.push(message)
              }

              let mut lines: Vec<Element<Message>> = Vec::new();

              let end = filtered.len();
              let start =  if loaded_logs.unlimited_history {
                0
              } else {
                0.max(end as isize - MAX_HISTORY) as usize
              };

              for message in &filtered[start..end] {
                let origin_text = match &message.plugin {
                  Some(plugin) => {
                    text(format!("[{}]", plugin))
                      .font(bold())
                  },
                  None => {
                    text(&message.target.replace("futuremod_engine::", ""))
                  }
                };

                let time_text = text(
                  message.timestamp.parse::<DateTime<Utc>>()
                    .map_or(message.timestamp.clone(), |v| v.format("%Y-%m-%d %H:%M:%S%.3f").to_string())
                );
                  

                let line = row![
                    time_text,
                    log_level_to_text(message.level.as_str()),
                    origin_text,
                    text(&message.message),
                ]
                .spacing(8);

                lines.push(line.into());
              }

              Scrollable::new(
                column(
                lines
                ).padding([0.0, 8.0])
              )
              .direction(Direction::Vertical(Properties::new().alignment(Alignment::End)))
              .width(Length::Fill)
              .into()
          },
          LogState::Error(e) => text(format!("Error: {:?}", e)).into(),
      };
      container(
          column![
            header(loaded_logs.unlimited_history, &loaded_logs.selected_log_levels, &loaded_logs.plugins, &loaded_logs.selected_origins),
            content,
          ]
      )
      .into()
    },
    Logs::Error(msg) => {
      container(text(format!("Could not load plugins: {}", msg)))
        .height(Length::Fill)
        .width(Length::Fill)
        .align_x(Horizontal::Center)
        .align_y(Vertical::Center)
        .into()
    }
  }
}

  pub fn update(&mut self, message: Message) -> Command<Message> {
    match self {
      Logs::Loading => {
        match message {
          Message::GetPluginResponse(response) => {
            match response {
              Ok(plugins) => {
                *self = Logs::View(LogsState {
                  plugins,
                  ..LogsState::default()
                });
              }
              Err(e) => {
                *self = Logs::Error(e);
              }
            };

            Command::none()
          },
          _ => Command::none(),
        }
      },
      Logs::View(logs) => {
        match message {
          Message::ToggleHistory(unlimited_history) => {
            logs.unlimited_history = unlimited_history;
              Command::none()
          },
          Message::ToggleLevelDebug(value) => {
            logs.selected_log_levels.debug = value;
  
            Command::none()
          },
          Message::ToggleLevelInfo(value) => {
            logs.selected_log_levels.info = value;
  
            Command::none()
          },
          Message::ToggleLevelWarn(value) => {
            logs.selected_log_levels.warn = value;
  
            Command::none()
          },
          Message::ToggleLevelError(value) => {
            logs.selected_log_levels.error = value;
  
            Command::none()
          },
          Message::ChangeOriginSelection(origin, value) => {
            logs.selected_origins.insert(origin, value);
            Command::none()
          }
          _ => Command::none(),
        }
      },
      Logs::Error(_) => Command::none(),
    }
  }
}

fn header<'a>(unlimited_history: bool, selected_levels: &SelectedLogLevels, plugins: &HashMap<String, Plugin>, selected_origins: &HashMap<LogOrigin, bool>) -> Element<'a, Message> {
    row![
        button(icon(BootstrapIcon::ArrowLeft)).style(Button::Text)
            .on_press(Message::GoBack),
        container(text("Logs").size(24)).width(Length::Fill),
        origin_picker(plugins, selected_origins),
        level_picker(&selected_levels),
        checkbox("Unlimited history", unlimited_history).on_toggle(Message::ToggleHistory),
    ].spacing(16).padding([4.0, 16.0]).align_items(iced::Alignment::Center)
    .into()
}

fn level_picker<'a>(log_levels: &SelectedLogLevels) -> Element<'a, Message> {
  let filter_button = button("Log Level").on_press(Message::None).style(Button::Text);

  menu_bar!(
      (
        filter_button,
        {
          Menu::new(menu_items!(
            (checkbox("Debug", log_levels.debug).on_toggle(Message::ToggleLevelDebug).width(Length::Fill))
            (checkbox("Info", log_levels.info).on_toggle(Message::ToggleLevelInfo).width(Length::Fill))
            (checkbox("Warn", log_levels.warn).on_toggle(Message::ToggleLevelWarn).width(Length::Fill))
            (checkbox("Error", log_levels.error).on_toggle(Message::ToggleLevelError).width(Length::Fill))
          ))
          .spacing(8.0)
          .width(140.0)
        }
      )
  )
  .into()
}

fn origin_picker<'a>(plugins: &HashMap<String, Plugin>, selected_origins: &HashMap<LogOrigin, bool>) -> Element<'a, Message> {
  let filter_button = button("Origin").on_press(Message::None).style(Button::Text);

  let mut items: Vec<Item<Message, Theme, Renderer>> = Vec::new();

  items.push(
    Item::new(
      checkbox(
        "System",
        *selected_origins.get(&LogOrigin::System).unwrap_or(&true),
      )
      .on_toggle(|value| Message::ChangeOriginSelection(LogOrigin::System, value))
      .width(Length::Fill)
    )
  );

  for (name, _) in plugins.iter() {
    let name = name.clone();
    let plugin_key = LogOrigin::Plugin(name.clone());

    let value = *selected_origins.get(&plugin_key).unwrap_or(&true);

    items.push(Item::new(
      checkbox(format!("[P] {}", name), value)
        .on_toggle(move |value| Message::ChangeOriginSelection(plugin_key.clone(), value))
        .width(Length::Fill)
    ));
  }

  menu_bar!(
    (
      filter_button,
      {
        Menu::new(items)
          .spacing(8.0)
          .width(148.0)
      }
    )
  )
  .into()
}