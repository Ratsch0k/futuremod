use iced::alignment::{Horizontal, Vertical};
use iced::widget::{container, text, Container};
use iced::{Color, Element};
use iced_aw::{BootstrapIcon, BOOTSTRAP_FONT};
use iced_aw::graphics::icons::bootstrap::icon_to_string;

use crate::theme;

use super::Text;

pub fn icon_text<'a>(content: BootstrapIcon) -> Text<'a> {
text(icon_to_string(content)).font(BOOTSTRAP_FONT).shaping(text::Shaping::Advanced).horizontal_alignment(Horizontal::Center).vertical_alignment(Vertical::Center).size(24)
}

/// Convenient function to create a text element that contains a Bootstrap icon.
pub fn icon<'a, Message>(content: BootstrapIcon) -> Container<'a, Message, theme::Theme> {
  align_icon(icon_text(content))
}

pub fn align_icon<'a, Message>(content: impl Into<Element<'a, Message, theme::Theme>>) -> Container<'a, Message, theme::Theme> {
  container(content).padding([2.0, 0.0, 0.0, 0.0])
}

#[allow(unused)]
pub fn icon_with_color<'a, Message>(content: BootstrapIcon, color: Color) -> Container<'a, Message, theme::Theme> {
  align_icon(icon_text(content).style(theme::Text::Color(color)))
}

pub fn icon_with_style<'a, Message>(content: BootstrapIcon, style: theme::Text) -> Container<'a, Message, theme::Theme> {
  align_icon(icon_text(content).style(style))
}