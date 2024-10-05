use iced::widget::toggler;

use super::Theme;

impl toggler::Catalog for Theme {
    type Class<'a> = toggler::StyleFn<'a, Theme>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(|theme, status| toggler::default(&theme.theme, status))
    }

    fn style(&self, class: &Self::Class<'_>, status: toggler::Status) -> toggler::Style {
      class(self, status)
    }
}