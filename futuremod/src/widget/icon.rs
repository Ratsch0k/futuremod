use iced::alignment::{Horizontal, Vertical};
use iced::widget::{container, row, text, Container, Row};
use iced::{Alignment, Color, Element, Pixels};
use iced_aw::{BootstrapIcon, BOOTSTRAP_FONT};
use iced_aw::graphics::icons::bootstrap::icon_to_string;

use crate::theme;

use super::Text;

pub fn icon_text<'a>(content: BootstrapIcon) -> Text<'a> {
  text(icon_to_string(content)).font(BOOTSTRAP_FONT).shaping(text::Shaping::Advanced).horizontal_alignment(Horizontal::Center).vertical_alignment(Vertical::Center).size(24)
}

/// Convenient function to create a text element that contains a Bootstrap icon.
pub fn icon<'a, Message>(content: BootstrapIcon) -> Container<'a, Message, theme::Theme> {
  icon_with_size(content, 24)
}

pub fn icon_with_size<'a, Message>(content: BootstrapIcon, size: impl Into<Pixels>) -> Container<'a, Message, theme::Theme> {
  align_icon(icon_text(content).size(size))
}

pub fn align_icon<'a, Message>(content: impl Into<Element<'a, Message, theme::Theme>>) -> Container<'a, Message, theme::Theme> {
  container(content).padding([2.0, 0.0, 0.0, 0.0])
}

#[allow(unused)]
pub fn icon_with_color<'a, Message>(content: BootstrapIcon, color: Color) -> Container<'a, Message, theme::Theme> {
  align_icon(icon_text(content).style(theme::Text::Color(color)))
}

#[allow(unused)]
pub fn icon_with_style<'a, Message>(content: BootstrapIcon, style: theme::Text) -> Container<'a, Message, theme::Theme> {
  align_icon(icon_text(content).style(style))
}

#[allow(unused)]
pub fn icon_with_text<'a, Message: 'a>(icon_content: BootstrapIcon, content: impl Into<Element<'a, Message, theme::Theme>>) -> Row<'a, Message, theme::Theme> {
  row![
    icon(icon_content),
    content.into(),
  ]
    .align_items(Alignment::Center)
    .spacing(4.0)
}
