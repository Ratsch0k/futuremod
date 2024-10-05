use iced::widget::checkbox;

use super::Theme;

impl checkbox::Catalog for Theme {
    type Class<'a> = checkbox::StyleFn<'a, Theme>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(|theme, status| checkbox::primary(&theme.theme, status))
    }

    fn style(&self, class: &Self::Class<'_>, status: checkbox::Status) -> checkbox::Style {
      class(self, status)
    }
}