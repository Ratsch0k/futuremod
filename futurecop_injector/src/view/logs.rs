use iced::{theme::Button, widget::{checkbox, column, container, row, scrollable::{Alignment, Direction, Properties, Scrollable}, text}, Command, Length};
use iced_aw::BootstrapIcon;

use crate::{theme, view::main::LogState, widget::{button, icon, Element}};

use super::main;

const MAX_HISTORY: isize = 250;

#[derive(Debug, Clone)]
pub enum Message {
    GoBack,
    ToggleHistory(bool),
}

#[derive(Debug, Clone)]
pub struct Logs(bool);

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
    (Logs(false), Command::none())
  }

  pub fn view<'a>(&self, log: &'a main::Logs) -> Element<'a, Message> {
      let content: Element<_> = match &log.state {
          LogState::Disconnected => text("Disconnected").into(),
          LogState::Connecting => text("Connecting").into(),
          LogState::Connected => {
              let mut lines: Vec<Element<Message>> = Vec::new();
              let end = log.logs.len();
              let start =  if self.0 {
                0
              } else {
                0.max(end as isize - MAX_HISTORY) as usize
              };

              for message in &log.logs[start..end] {
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
            header(self.0),
            content,
          ]
      )
      .into()
  }

  pub fn update(&mut self, message: Message) -> Command<Message> {
    match message {
        Message::ToggleHistory(unlimited_history) => {
            self.0 = unlimited_history;
            Command::none()
        }
        _ => Command::none(),
    }
  }
}

fn header<'a>(unlimited_history: bool) -> Element<'a, Message> {
    row![
        button(icon(BootstrapIcon::ArrowLeft)).style(Button::Text)
            .on_press(Message::GoBack),
        container(text("Logs").size(24)).width(Length::Fill),
        checkbox("Unlimited history", unlimited_history).on_toggle(Message::ToggleHistory),
    ].spacing(16).padding([4.0, 16.0]).align_items(iced::Alignment::Center)
    .into()
}