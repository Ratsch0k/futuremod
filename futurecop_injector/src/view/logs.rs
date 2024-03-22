use iced::{theme::Button, widget::{checkbox, column, container, row, scrollable::{Alignment, Direction, Properties, Scrollable}, text}, Command, Length};
use iced_aw::{menu::{Item, Menu}, menu_bar, menu_items, BootstrapIcon};

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


#[derive(Debug, Clone, Default)]
pub struct Logs {
  unlimited_history: bool,
  selected_log_levels: SelectedLogLevels,
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
    (Logs::default(), Command::none())
  }

  pub fn view<'a>(&self, log: &'a main::Logs) -> Element<'a, Message> {
      let content: Element<_> = match &log.state {
          LogState::Disconnected => text("Disconnected").into(),
          LogState::Connecting => text("Connecting").into(),
          LogState::Connected => {
              let mut filtered: Vec<&LogRecord> = Vec::new();

              for message in &log.logs {
                let valid = match &message.level.as_str() {
                  &"DEBUG" => self.selected_log_levels.debug,
                  &"INFO" => self.selected_log_levels.info,
                  &"WARN" => self.selected_log_levels.warn,
                  &"ERROR" => self.selected_log_levels.error,
                  _ => false,
                };

                if valid {
                  filtered.push(message);
                }
              }

              let mut lines: Vec<Element<Message>> = Vec::new();

              let end = filtered.len();
              let start =  if self.unlimited_history {
                0
              } else {
                0.max(end as isize - MAX_HISTORY) as usize
              };

              for message in &filtered[start..end] {
                let line = row![
                    text(&message.timestamp),
                    log_level_to_text(message.level.as_str()),
                    text(&message.target.replace("futurecop_mod::future_cop_mod::", "")).font(iced::Font{
                        weight: iced::font::Weight::Bold,
                        ..iced::Font::default()
                    }),
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
            header(self.unlimited_history, &self.selected_log_levels),
            content,
          ]
      )
      .into()
  }

  pub fn update(&mut self, message: Message) -> Command<Message> {
    match message {
        Message::ToggleHistory(unlimited_history) => {
            self.unlimited_history = unlimited_history;
            Command::none()
        },
        Message::ToggleLevelDebug(value) => {
          self.selected_log_levels.debug = value;

          Command::none()
        },
        Message::ToggleLevelInfo(value) => {
          self.selected_log_levels.info = value;

          Command::none()
        },
        Message::ToggleLevelWarn(value) => {
          self.selected_log_levels.warn = value;

          Command::none()
        },
        Message::ToggleLevelError(value) => {
          self.selected_log_levels.error = value;

          Command::none()
        },
        _ => Command::none(),
    }
  }
}

fn header<'a>(unlimited_history: bool, selected_levels: &SelectedLogLevels) -> Element<'a, Message> {
    row![
        button(icon(BootstrapIcon::ArrowLeft)).style(Button::Text)
            .on_press(Message::GoBack),
        container(text("Logs").size(24)).width(Length::Fill),
        level_picker(&selected_levels),
        checkbox("Unlimited history", unlimited_history).on_toggle(Message::ToggleHistory),
    ].spacing(16).padding([4.0, 16.0]).align_items(iced::Alignment::Center)
    .into()
}

fn level_picker<'a>(log_levels: &SelectedLogLevels) -> Element<'a, Message> {
  let filter_button = button("Filter").on_press(Message::None);

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
          .spacing(5.0)
          .width(140.0)
        }
      )
  )
  .padding(4.0).into()
}