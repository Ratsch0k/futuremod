#![allow(dead_code)]

use iced::{advanced::widget::text, application::StyleSheet, border::Radius, color, overlay::menu, theme::{self, palette::Pair, Checkbox, Menu, PickList, Toggler}, widget::{button, checkbox, container, pick_list, rule, scrollable, toggler}, Background, Border, Color, Shadow, Vector};
use iced_aw::{style::{card, modal, MenuBarStyle}, CardStyles, ModalStyles};

use crate::{palette::ColorRange, util};


/// Custom theme based on a default theme.
///
/// Based on the given default theme and implements custom styling only for some widgets.
#[derive(Debug, Default)]
pub struct Theme {
  theme: iced::Theme,
  palette: crate::palette::Palette,
}

impl Theme {
  pub fn new(palette: crate::palette::Palette) -> Self {
    Theme {
      theme: palette.to_theme(),
      palette,
    }
  }
}

impl StyleSheet for Theme {
    type Style = theme::Application;

    fn appearance(&self, style: &Self::Style) -> iced::application::Appearance {
        self.theme.appearance(style)
    }
}

/// Custom button styles.
/// 
/// Based on the iced button style but with additional variants.
#[derive(Default)]
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
  Custom(Box<dyn button::StyleSheet<Style = Theme>>),
}

impl button::StyleSheet for Theme {
  type Style = Button;
  
  fn active(&self, style: &Self::Style) -> button::Appearance {
    let appearance = button::Appearance {
        border: Border::with_radius(6),
        ..button::Appearance::default()
    };

    let from_pair = |pair: Pair| button::Appearance {
        background: Some(pair.color.into()),
        text_color: pair.text,
        ..appearance
    };

    let from_color_range = |range: &ColorRange| button::Appearance {
      background: Some(range.medium.color.into()),
      text_color: range.medium.text,
      ..appearance
    };

    match style {
        Button::Primary => from_pair(self.palette.primary.strong),
        Button::Secondary => from_pair(self.palette.secondary.base),
        Button::Positive => from_pair(self.palette.success.base),
        Button::Destructive => from_pair(self.palette.danger.medium),
        Button::Text => button::Appearance {
            text_color: self.palette.background.darkest.text,
            ..appearance
        },
        Button::Default => from_color_range(&self.palette.background),
        Button::Custom(custom) => custom.active(self),
    }
}

fn hovered(&self, style: &Self::Style) -> button::Appearance {
    if let Button::Custom(custom) = style {
        return custom.hovered(self);
    }

    let active = self.active(style);

    let background = match style {
        Button::Primary => Some(self.palette.primary.base.color),
        Button::Secondary => Some(self.palette.secondary.base.color),
        Button::Positive => Some(self.palette.success.strong.color),
        Button::Destructive => Some(self.palette.danger.dark.color),
        Button::Default => Some(self.palette.background.light.color),
        Button::Text  => Some(util::alpha(color!(0xffffff), 0.01)),
        Button::Custom(_) => None,
    };

    button::Appearance {
        background: background.map(Background::from),
        ..active
    }
}

fn pressed(&self, style: &Self::Style) -> button::Appearance {
    if let Button::Custom(custom) = style {
        return custom.pressed(self);
    }

    button::Appearance {
        shadow_offset: Vector::default(),
        ..self.active(style)
    }
}

fn disabled(&self, style: &Self::Style) -> button::Appearance {
    if let Button::Custom(custom) = style {
        return custom.disabled(self);
    }

    let active = self.active(style);

    button::Appearance {
        shadow_offset: Vector::default(),
        background: active.background.map(|background| match background {
            Background::Color(color) => Background::Color(Color {
                a: color.a * 0.5,
                ..color
            }),
            Background::Gradient(gradient) => {
                Background::Gradient(gradient.mul_alpha(0.5))
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

/// Container Styles
#[derive(Default)]
pub enum Container {
  #[default]
  Transparent,
  /// Box with a dark background and slightly lighter border.
  Box,
  /// Same as Box as with danger colors
  Danger,
  /// Same as Box as with warning colors
  Warning,
  /// Box used for dialogs
  Dialog,
  Custom(Box<dyn iced::widget::container::StyleSheet<Style = Theme>>),
}

impl container::StyleSheet for Theme {
    type Style = Container;
    
    fn appearance(&self, style: &Self::Style) -> container::Appearance {
        match style {
            Container::Transparent => container::Appearance::default(),
            Container::Dialog => {
                container::Appearance {
                    text_color: None,
                    background: Some(self.palette.background.dark.color.into()),
                    border: Border::with_radius(12),
                    shadow: Shadow {
                      color: util::darken(self.palette.background.darkest.color, 0.05),
                      offset: Vector::new(0.0, -8.0),
                      blur_radius: 48.0,
                    },
                }
            },
            Container::Box => {
                container::Appearance {
                    text_color: None,
                    background: Some(self.palette.background.dark.color.into()),
                    border: Border {
                      color: self.palette.background.medium.color,
                      width: 1.0,
                      radius: Radius::from(12),
                    },
                    shadow: Shadow::default(),
                }
            },
            Container::Custom(custom) => custom.appearance(self),
            Container::Danger => {
              container::Appearance {
                text_color: Some(self.palette.danger.lightest.color),
                background: Some(self.palette.danger.darkest.color.into()),
                border: Border {
                  radius: Radius::from(8),
                  width: 1.0,
                  color: self.palette.danger.dark.color,
                },
                shadow: Shadow::default(),
              }
            },
            Container::Warning => {
              container::Appearance {
                text_color: Some(self.palette.warning.lightest.color.into()),
                background: Some(self.palette.warning.darkest.color.into()),
                border: Border {
                  radius: Radius::from(8),
                  width: 1.0,
                  color: self.palette.warning.dark.color,
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
  Danger,
}

impl text::StyleSheet for Theme {
    type Style = Text;

    fn appearance(&self, style: Self::Style) -> text::Appearance {
      match style {
        Text::Default => text::Appearance::default(),
        Text::Color(c) => text::Appearance { color: Some(c) },
        Text::Warn => text::Appearance { color: Some(self.palette.warning.medium.color) },
        Text::Danger => text::Appearance { color: Some(self.palette.danger.medium.color )},
      }
    }
}

impl scrollable::StyleSheet for Theme {
  type Style = theme::Scrollable;
  
  fn active(&self, style: &Self::Style) -> scrollable::Appearance {
    self.theme.active(style)
  }
  
  fn hovered(
        &self,
        style: &Self::Style,
        is_mouse_over_scrollbar: bool,
    ) -> scrollable::Appearance {
      self.theme.hovered(style, is_mouse_over_scrollbar)
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
      match style {
          Rule::Default => rule::Appearance {
              color: self.palette.background.medium.color,
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
    self.theme.active(style)
  }
}

impl modal::StyleSheet for Theme {
    type Style = ModalStyles;

    fn active(&self, style: &Self::Style) -> modal::Appearance {
        self.theme.active(style)
    }
}

impl checkbox::StyleSheet for Theme {
    type Style = Checkbox;

    fn active(&self, style: &Self::Style, is_checked: bool) -> checkbox::Appearance {
        self.theme.active(style, is_checked)
    }

    fn hovered(&self, style: &Self::Style, is_checked: bool) -> checkbox::Appearance {
        self.theme.hovered(style, is_checked)
    }
}

impl pick_list::StyleSheet for Theme {
  type Style = PickList;
  
  fn active(&self, style: &<Self as pick_list::StyleSheet>::Style) -> pick_list::Appearance {
      self.theme.active(style)
    }
  
  fn hovered(&self, style: &<Self as pick_list::StyleSheet>::Style) -> pick_list::Appearance {
      self.theme.hovered(style)
    }
}

impl menu::StyleSheet for Theme {
  type Style = Menu;

  fn appearance(&self, style: &Self::Style) -> menu::Appearance {
      menu::StyleSheet::appearance(&self.theme, style)
  }
}

impl iced_aw::menu::StyleSheet for Theme {
    type Style = iced_aw::style::MenuBarStyle;

    fn appearance(&self, style: &Self::Style) -> iced_aw::menu::Appearance {
      let palette = self.theme.extended_palette();

      match style {
        MenuBarStyle::Default => iced_aw::menu::Appearance {
          menu_background: util::lighten(palette.background.base.color, 0.05).into(),
          bar_background: Color::from_rgba(0.0, 0.0, 0.0, 0.0).into(),
            ..iced_aw::menu::Appearance::default()
        },
        _ => iced_aw::menu::StyleSheet::appearance(&self.theme, style),
      }
    }
}

impl toggler::StyleSheet for Theme {
    type Style = Toggler;

    fn active(&self, style: &Self::Style, is_active: bool) -> toggler::Appearance {
        self.theme.active(style, is_active)
    }

    fn hovered(&self, style: &Self::Style, is_active: bool) -> toggler::Appearance {
        self.theme.hovered(style, is_active)
    }
}