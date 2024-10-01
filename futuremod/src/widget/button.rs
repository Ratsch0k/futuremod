use crate::theme::{self, Theme};
use super::Element;

pub fn button<'a, Message>(content: impl Into<Element<'a, Message>>) -> iced::widget::Button<'a, Message, Theme> {
  iced::widget::button(content).padding([6.0, 12.0])
}

pub fn hover_button<'a, Message>(content: impl Into<Element<'a, Message>>) -> iced::widget::Button<'a, Message, Theme> {
  button(content).style(theme::Button::HoverHighlight)
}