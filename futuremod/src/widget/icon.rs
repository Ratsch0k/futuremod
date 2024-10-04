use iced::alignment::{Horizontal, Vertical};
use iced::widget::{container, row, text, Container, Row};
use iced::{Alignment, Color, Element, Padding, Pixels};
use iced_fonts::{Bootstrap, BOOTSTRAP_FONT};

use crate::theme;

use super::Text;

pub fn icon_text<'a>(content: Bootstrap) -> Text<'a> {
  text(content.to_string()).font(BOOTSTRAP_FONT).shaping(text::Shaping::Advanced).align_x(Horizontal::Center).align_y(Vertical::Center).size(24)
}

/// Convenient function to create a text element that contains a Bootstrap icon.
pub fn icon<'a, Message>(content: Bootstrap) -> Container<'a, Message, theme::Theme> {
  icon_with_size(content, 24)
}

pub fn icon_with_size<'a, Message>(content: Bootstrap, size: impl Into<Pixels>) -> Container<'a, Message, theme::Theme> {
  align_icon(icon_text(content).size(size))
}

pub fn align_icon<'a, Message>(content: impl Into<Element<'a, Message, theme::Theme>>) -> Container<'a, Message, theme::Theme> {
  container(content).padding(Padding{top: 2.0, right: 0.0, bottom: 0.0, left: 0.0})
}

#[allow(unused)]
pub fn icon_with_color<'a, Message>(content: Bootstrap, color: Color) -> Container<'a, Message, theme::Theme> {
  align_icon(icon_text(content).class(theme::text::Text::Color(color)))
}

#[allow(unused)]
pub fn icon_with_style<'a, Message>(content: Bootstrap, style: theme::text::Text<'a>) -> Container<'a, Message, theme::Theme> {
  align_icon(icon_text(content).class(style))
}

#[allow(unused)]
pub fn icon_with_text<'a, Message: 'a>(icon_content: Bootstrap, content: impl Into<Element<'a, Message, theme::Theme>>) -> Row<'a, Message, theme::Theme> {
  row![
    icon(icon_content),
    content.into(),
  ]
    .align_y(Alignment::Center)
    .spacing(4.0)
}
