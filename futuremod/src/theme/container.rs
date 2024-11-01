use iced::{
    border::Radius,
    widget::container::{Catalog, Style},
    Border, Color, Shadow, Vector,
};

use crate::{
    palette::{BaseColor, Shade},
    util::{self, lighten},
    widget::Theme,
};

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

    /// Box with colored background and border based on given base color.
    BoxWithBase(BaseColor),

    Shade(Shade),

    Custom(Box<dyn Fn(&Theme) -> Style>),
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

pub fn appearance(theme: &Theme, style: &Container) -> Style {
    let rounded_border = |color: Color| Border {
        color,
        width: 1.0,
        radius: Radius::from(12),
    };

    match style {
        Container::Transparent => Style::default(),
        Container::Dialog => Style {
            text_color: None,
            background: Some(theme.palette.background.dark.color.into()),
            border: Border::default().rounded(12),
            shadow: Shadow {
                color: util::darken(theme.palette.background.darkest.color, 0.05),
                offset: Vector::new(0.0, -8.0),
                blur_radius: 48.0,
            },
        },
        Container::Box => Style {
            text_color: None,
            background: Some(theme.palette.background.dark.color.into()),
            border: rounded_border(theme.palette.background.medium.color),
            shadow: Shadow::default(),
        },
        Container::Danger => Style {
            text_color: Some(theme.palette.danger.lightest.color),
            background: Some(theme.palette.danger.darkest.color.into()),
            border: Border {
                radius: Radius::from(8),
                width: 1.0,
                color: theme.palette.danger.dark.color,
            },
            shadow: Shadow::default(),
        },
        Container::Warning => Style {
            text_color: Some(theme.palette.warning.lightest.color.into()),
            background: Some(theme.palette.warning.darkest.color.into()),
            border: Border {
                radius: Radius::from(8),
                width: 1.0,
                color: theme.palette.warning.dark.color,
            },
            shadow: Shadow::default(),
        },
        Container::Backdrop => Style {
            background: Some(
                Color {
                    a: 0.3,
                    ..Color::BLACK
                }
                .into(),
            ),
            text_color: None,
            border: Border::default(),
            shadow: Shadow::default(),
        },
        Container::BoxWithBase(base) => {
            let base_color = theme.palette.background.get_base(base);
            let background_color = base_color.color;
            let text_color = base_color.text;
            let border_color = lighten(background_color, 0.1);

            Style {
                background: Some(background_color.into()),
                text_color: Some(text_color),
                border: rounded_border(border_color),
                shadow: Shadow::default(),
            }
        }
        Container::Shade(shade) => {
            let shade_color = theme.palette.base.get_shade(shade);
            let border_color = theme.palette.base.get_shade(&shade.slight_contrast());
            let text_color = theme.palette.base.get_shade(&shade.get_contrast());

            Style {
                background: Some(shade_color.into()),
                text_color: Some(text_color),
                border: rounded_border(border_color),
                shadow: Shadow::default(),
            }
        }
        Container::Custom(class) => class(theme),
    }
}
