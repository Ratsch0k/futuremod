use iced::{border::Radius, widget::container::{Catalog, Style}, Border, Color, Shadow, Vector};

use crate::{util, widget::Theme};

/// Container Styles
#[derive(Default)]
#[allow(unused)]
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

  /// Box used as the backdrop for dialogs
  Backdrop,
}

impl Catalog for Theme {
    type Class<'a> = Container;

    fn default<'a>() -> Self::Class<'a> {
        Container::default()
    }

    fn style(&self, class: &Self::Class<'_>) -> Style {
        appearance(self, class)
    }
}

fn appearance(theme: &Theme, style: &Container) -> Style {
  match style {
      Container::Transparent => Style::default(),
      Container::Dialog => {
          Style {
              text_color: None,
              background: Some(theme.palette.background.dark.color.into()),
              border: Border::default().rounded(12),
              shadow: Shadow {
                color: util::darken(theme.palette.background.darkest.color, 0.05),
                offset: Vector::new(0.0, -8.0),
                blur_radius: 48.0,
              },
          }
      },
      Container::Box => {
          Style {
              text_color: None,
              background: Some(theme.palette.background.dark.color.into()),
              border: Border {
                color: theme.palette.background.medium.color,
                width: 1.0,
                radius: Radius::from(12),
              },
              shadow: Shadow::default(),
          }
      },
      Container::Danger => {
        Style {
          text_color: Some(theme.palette.danger.lightest.color),
          background: Some(theme.palette.danger.darkest.color.into()),
          border: Border {
            radius: Radius::from(8),
            width: 1.0,
            color: theme.palette.danger.dark.color,
          },
          shadow: Shadow::default(),
        }
      },
      Container::Warning => {
        Style {
          text_color: Some(theme.palette.warning.lightest.color.into()),
          background: Some(theme.palette.warning.darkest.color.into()),
          border: Border {
            radius: Radius::from(8),
            width: 1.0,
            color: theme.palette.warning.dark.color,
          },
          shadow: Shadow::default(),
        }
      },
      Container::Backdrop => {
        Style {
          background: Some(Color {
            a: 0.3,
            ..Color::BLACK
          }.into()),
          text_color: None,
          border: Border::default(),
          shadow: Shadow::default(),
        }
      }
  }
}