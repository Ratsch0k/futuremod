use iced::{color, theme::palette::Pair, widget::button::{Catalog, Status, Style}, Background, Border, Color};

use crate::{palette::ColorRange, util};

use super::theme::Theme;

/// Custom button styles.
/// 
/// Based on the iced button style but with additional variants.
#[derive(Default)]
#[allow(unused)]
pub enum Button {
  #[default]
  /// Default style
  /// 
  /// Based on the background color
  Default,
  Primary,
  Secondary,
  Positive,
  Destructive,
  Text,
  HoverHighlight,
}

impl Catalog for Theme {
    type Class<'a> = Button;

    fn default<'a>() -> Self::Class<'a> {
        Button::Default
    }

    fn style(&self, class: &Self::Class<'_>, status: iced::widget::button::Status) -> Style {
        match status {
          Status::Active => active(self, class),
            Status::Hovered => hovered(self, class),
            Status::Pressed => pressed(self, class),
            Status::Disabled => disabled(self, class),
        }
    }
}

fn active(theme: &Theme, style: &Button) -> Style {
  let appearance = Style {
      border: Border::default().rounded(6),
      ..Style::default()
  };

  let from_pair = |pair: Pair| Style {
      background: Some(pair.color.into()),
      text_color: pair.text,
      ..appearance
  };

  let from_color_range = |range: &ColorRange| Style {
    background: Some(range.medium.color.into()),
    text_color: range.medium.text,
    ..appearance
  };

  match style {
      Button::Primary => from_pair(theme.palette.primary.strong),
      Button::Secondary => from_pair(theme.palette.secondary.base),
      Button::Positive => from_pair(theme.palette.success.medium),
      Button::Destructive => from_pair(theme.palette.danger.medium),
      Button::Text | Button::HoverHighlight => Style {
          text_color: theme.palette.background.darkest.text,
          ..appearance
      },
      Button::Default => from_color_range(&theme.palette.background),
  }
}

fn hovered(theme: &Theme, style: &Button) -> Style {
    let active = theme.style(style, Status::Active);

    let background = match style {
        Button::Primary => Some(theme.palette.primary.base.color),
        Button::Secondary => Some(theme.palette.secondary.base.color),
        Button::Positive => Some(theme.palette.success.dark.color),
        Button::Destructive => Some(theme.palette.danger.dark.color),
        Button::Default => Some(theme.palette.background.light.color),
        Button::Text  => Some(util::alpha(color!(0xffffff), 0.01)),
        Button::HoverHighlight => Some(theme.palette.primary.strong.color),
    };

    Style {
        background: background.map(Background::from),
        ..active
    }
}

fn pressed(theme: &Theme, style: &Button) -> Style {
  let active = theme.style(style, Status::Active);

  let background = match style {
    Button::HoverHighlight => Some(Background::from(theme.palette.primary.base.color)),
    _ => active.background,
  };

  Style {
      background,
      ..active
  }
}

fn disabled(theme: &Theme, style: &Button) -> Style {
    let active = theme.style(style, Status::Active);

    Style {
        background: active.background.map(|background| match background {
            Background::Color(color) => Background::Color(Color {
                a: color.a * 0.5,
                ..color
            }),
            Background::Gradient(gradient) => {
                Background::Gradient(gradient.scale_alpha(0.5))
            }
        }),
        text_color: Color {
            a: active.text_color.a * 0.5,
            ..active.text_color
        },
        ..active
    }
}