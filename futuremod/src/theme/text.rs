use iced::{widget::text, Color};

use super::theme::Theme;

#[derive(Default, Clone, Copy)]
#[allow(unused)]
pub enum Text {
  #[default]
  Default,
  Color(Color),
  Warn,
  Danger,
}

impl text::Catalog for Theme {
    type Class<'a> = Text;

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
  }
}
