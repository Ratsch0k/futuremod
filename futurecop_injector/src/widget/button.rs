use crate::theme::Theme;
use super::Element;

pub fn button<'a, Message>(content: impl Into<Element<'a, Message>>) -> iced::widget::Button<'a, Message, Theme> {
  iced::widget::button(content).padding([6.0, 12.0])
}