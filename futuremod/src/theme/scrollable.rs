use iced::widget::scrollable;

use crate::theme::Theme;

impl scrollable::Catalog for Theme {    
    type Class<'a> = scrollable::StyleFn<'a, Theme>;
    
    fn default<'a>() -> Self::Class<'a> {
        Box::new(|theme, status| scrollable::default(&theme.theme, status))
    }
    
    fn style(&self, class: &Self::Class<'_>, status: scrollable::Status) -> scrollable::Style {
        class(self, status)
    }
}
