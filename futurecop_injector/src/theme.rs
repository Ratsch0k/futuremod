use iced::{advanced::widget::text, application::StyleSheet, border::Radius, color, theme::{self, Checkbox, Palette}, widget::{button, checkbox, container, rule, scrollable}, Border, Color, Shadow};
use iced_aw::{style::{card, modal}, CardStyles, ModalStyles};

use crate::util::{self, darken};

pub const PALETTE: Palette = Palette {
  background: color!(0x13111F),
  text: color!(0xEAEAEA),
  primary: color!(0x4012B3),
  success: color!(0x9AFF75),
  danger: color!(0xD04A4A),
};

/// Custom theme based on a default theme.
///
/// Based on the given default theme and implements custom styling only for some widgets.
#[derive(Debug, Default)]
pub struct Theme(pub iced::Theme);

impl StyleSheet for Theme {
    type Style = theme::Application;

    fn appearance(&self, style: &Self::Style) -> iced::application::Appearance {
        self.0.appearance(style)
    }
}

impl button::StyleSheet for Theme {
  type Style = theme::Button;
  
  fn active(&self, style: &Self::Style) -> button::Appearance {
    button::Appearance {
      border: Border::with_radius(6),
      ..self.0.active(style)
    }
  }
  
  fn hovered(&self, style: &Self::Style) -> button::Appearance {
    button::Appearance {
      border: Border::with_radius(6),
      ..self.0.hovered(style)
    }
  }
  
  fn pressed(&self, style: &Self::Style) -> button::Appearance {
        button::Appearance {
            shadow_offset: iced::Vector::default(),
            ..self.active(style)
        }
    }
  
  fn disabled(&self, style: &Self::Style) -> button::Appearance {
        let active = self.active(style);
  
        button::Appearance {
            shadow_offset: iced::Vector::default(),
            background: active.background.map(|background| match background {
                iced::Background::Color(color) => iced::Background::Color(Color {
                    a: color.a * 0.5,
                    ..color
                }),
                iced::Background::Gradient(gradient) => {
                    iced::Background::Gradient(gradient.mul_alpha(0.5))
                }
            }),
            text_color: Color {
                a: active.text_color.a * 0.5,
                ..active.text_color
            },
            ..active
        }
    }
}

#[derive(Default)]
#[allow(unused)]
pub enum Container {
  #[default]
  Transparent,
  Box,
  Error,
  Warning,
  Custom(Box<dyn iced::widget::container::StyleSheet<Style = Theme>>),
}

impl container::StyleSheet for Theme {
    type Style = Container;
    
    fn appearance(&self, style: &Self::Style) -> container::Appearance {
        match style {
            Container::Transparent => container::Appearance::default(),
            Container::Box => {
                let palette = self.0.extended_palette();

                container::Appearance {
                    text_color: None,
                    background: Some(palette.background.strong.color.into()),
                    border: Border::with_radius(2),
                    shadow: Shadow::default(),
                }
            }
            Container::Custom(custom) => custom.appearance(self),
            Container::Error => {
              let palette = self.0.palette();
              let danger = palette.danger;
              let text = util::mix(danger, color!(0xff0000), 0.2);
              let border = util::darken(danger, 0.4);
              let background = util::darken(danger, 0.45);

              container::Appearance {
                text_color: Some(text.into()),
                background: Some(background.into()),
                border: Border {
                  radius: Radius::from(8),
                  width: 1.0,
                  color: border,
                },
                shadow: Shadow::default(),
              }
            },
            Container::Warning => {
              let warning = color!(0xFFCC33);
              let text = util::lighten(warning, 0.1);
              let border = util::darken(warning, 0.4);
              let background = util::darken(warning, 0.45);

              container::Appearance {
                text_color: Some(text.into()),
                background: Some(background.into()),
                border: Border {
                  radius: Radius::from(8),
                  width: 1.0,
                  color: border,
                },
                shadow: Shadow::default(),
              }
              
            }
        }
    }

}

#[derive(Default, Clone, Copy)]
#[allow(unused)]
pub enum Text {
  #[default]
  Default,
  Color(Color),
  Warn,
  Error,
}

impl text::StyleSheet for Theme {
    type Style = Text;

    fn appearance(&self, style: Self::Style) -> text::Appearance {
      let palette = self.0.extended_palette();

      match style {
        Text::Default => text::Appearance::default(),
        Text::Color(c) => text::Appearance { color: Some(c) },
        Text::Warn => text::Appearance { color: Some(color!(0xFFD044).into()) },
        Text::Error => text::Appearance { color: Some(palette.danger.strong.color.into() )},
      }
    }
}

impl scrollable::StyleSheet for Theme {
  type Style = theme::Scrollable;
  
  fn active(&self, style: &Self::Style) -> scrollable::Appearance {
    self.0.active(style)
  }
  
  fn hovered(
        &self,
        style: &Self::Style,
        is_mouse_over_scrollbar: bool,
    ) -> scrollable::Appearance {
      self.0.hovered(style, is_mouse_over_scrollbar)
    }
}

#[derive(Default)]
#[allow(unused)]
pub enum Rule {
  #[default]
  Default,
  Custom(Box<dyn rule::StyleSheet<Style = Theme>>),
}

impl rule::StyleSheet for Theme {
    type Style = Rule;

    fn appearance(&self, style: &Self::Style) -> rule::Appearance {
      let palette = self.0.extended_palette();

      match style {
          Rule::Default => rule::Appearance {
              color: darken(palette.background.strong.color, 0.4),
              width: 1,
              radius: 0.0.into(),
              fill_mode: rule::FillMode::Full,
          },
          Rule::Custom(custom) => custom.appearance(self),
      }
    }
}


impl card::StyleSheet for Theme {
  type Style = CardStyles;
  
  fn active(&self, style: &Self::Style) -> card::Appearance {
    self.0.active(style)
  }
}

impl modal::StyleSheet for Theme {
    type Style = ModalStyles;

    fn active(&self, style: &Self::Style) -> modal::Appearance {
        self.0.active(style)
    }
}

impl checkbox::StyleSheet for Theme {
    type Style = Checkbox;

    fn active(&self, style: &Self::Style, is_checked: bool) -> checkbox::Appearance {
        self.0.active(style, is_checked)
    }

    fn hovered(&self, style: &Self::Style, is_checked: bool) -> checkbox::Appearance {
        self.0.hovered(style, is_checked)
    }
}