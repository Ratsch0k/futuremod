use iced_aw::{card::StyleFn, style::badge};

use super::Theme;

impl badge::Catalog for Theme {
    type Class<'a> = StyleFn<'a, Theme, badge::Style>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(|theme, status| badge::primary(&theme.theme, status))
    }

    fn style(&self, class: &Self::Class<'_>, status: iced_aw::card::Status) -> badge::Style {
      class(self, status)
    }
}