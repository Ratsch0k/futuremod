use iced::{widget::text, Color};

use super::theme::Theme;

#[derive(Default)]
#[allow(unused)]
pub enum Text<'a> {
  #[default]
  Default,
  Color(Color),
  Warn,
  Danger,
  Custom(Box<dyn Fn(&Theme) -> text::Style + 'a>),
}

impl text::Catalog for Theme {
    type Class<'a> = Text<'a>;

    fn default<'a>() -> Self::Class<'a> {
        Text::default()
    }

    fn style(&self, style: &Self::Class<'_>) -> text::Style {
        appearance(self, style)
    }
}

fn appearance(theme: &Theme, style: &Text) -> text::Style {
  match style {
    Text::Default => text::Style::default(),
    Text::Color(c) => text::Style { color: Some(*c) },
    Text::Warn => text::Style { color: Some(theme.palette.warning.medium.color) },
    Text::Danger => text::Style { color: Some(theme.palette.danger.medium.color )},
    Text::Custom(class) => class(theme),
  }
}
