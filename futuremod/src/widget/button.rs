use iced::{widget::row, Alignment, Pixels};
use iced_aw::BootstrapIcon;

use crate::theme::{self, Theme};
use super::{icon_with_size, Element};

pub fn button<'a, Message>(content: impl Into<Element<'a, Message>>) -> iced::widget::Button<'a, Message, Theme> {
  iced::widget::button(content).padding([8.0, 16.0])
}

#[allow(unused)]
pub fn hover_button<'a, Message>(content: impl Into<Element<'a, Message>>) -> iced::widget::Button<'a, Message, Theme> {
  button(content).style(theme::Button::HoverHighlight)
}

#[allow(unused)]
pub fn icon_button<'a, Message: 'a>(icon: BootstrapIcon) -> iced::widget::Button<'a, Message, Theme> {
  button(crate::widget::icon(icon)).style(theme::Button::HoverHighlight)
}

pub fn icon_text_button<'a, Message: 'a>(icon_content: BootstrapIcon, content: impl Into<Element<'a, Message>>) -> iced::widget::Button<'a, Message, Theme> {
  icon_text_button_advanced(icon_content, content, IconTextButtonOptions::default())
}

pub struct IconTextButtonOptions {
  pub icon_size: f32,
  pub spacing: f32,
}

impl Default for IconTextButtonOptions {
    fn default() -> Self {
        Self { icon_size: 16.0, spacing: 8.0 }
    }
}

#[allow(unused)]
impl IconTextButtonOptions {
  pub fn new() -> Self {
    IconTextButtonOptions::default()
  }

  pub fn with_icon_size(mut self, size: impl Into<Pixels>) -> Self {
    self.icon_size = size.into().0;
    self
  }

  pub fn with_spacing(mut self, spacing: impl Into<Pixels>) -> Self {
    self.spacing = spacing.into().0;
    self
  }
}

pub fn icon_text_button_advanced<'a, Message: 'a>(icon_content: BootstrapIcon, content: impl Into<Element<'a, Message>>, options: IconTextButtonOptions) -> iced::widget::Button<'a, Message, Theme> {
  button(
    row![
      icon_with_size(icon_content, options.icon_size),
      content.into(),
    ]
      .spacing(options.spacing)
      .align_items(Alignment::Center)
  )
    .style(theme::Button::HoverHighlight)
}