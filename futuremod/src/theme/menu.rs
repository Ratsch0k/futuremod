use iced_aw::{card::StyleFn, menu};

use super::Theme;

impl menu::Catalog for Theme {
    type Class<'a> = StyleFn<'a, Theme, menu::Style>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(|theme, status| menu::primary(&theme.theme, status))
    }

    fn style(&self, class: &Self::Class<'_>, status: iced_aw::card::Status) -> menu::Style {
        class(self, status)
    }
}