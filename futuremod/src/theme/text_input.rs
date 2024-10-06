use iced::widget::text_input;

use super::Theme;

impl text_input::Catalog for Theme {
    type Class<'a> = text_input::StyleFn<'a, Theme>;

    fn default<'a>() -> Self::Class<'a> {
      Box::new(|theme, status| text_input::default(&theme.theme, status))
    }

    fn style(&self, class: &Self::Class<'_>, status: text_input::Status) -> text_input::Style {
      class(self, status)
    }
}