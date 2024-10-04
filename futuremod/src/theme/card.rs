use iced_aw::{card::StyleFn, style::card};

use super::Theme;

impl card::Catalog for Theme {
    type Class<'a> = StyleFn<'a, Theme, card::Style>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(|theme, status| card::primary(&theme.theme, status))
    }

    fn style(&self, class: &Self::Class<'_>, status: iced_aw::card::Status) -> card::Style {
        class(self, status)
    }
}