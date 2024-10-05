use iced::widget::rule::{self, FillMode};

use super::Theme;

impl rule::Catalog for Theme {
    type Class<'a> = rule::StyleFn<'a, Theme>;
    
    fn default<'a>() -> Self::Class<'a> {
        Box::new(default)
    }
    
    fn style(&self, class: &Self::Class<'_>) -> rule::Style {
        class(self)
    }
}

pub fn default(theme: &Theme) -> rule::Style {
    rule::Style {
        color: theme.palette.background.medium.color,
        width: 1,
        radius: 0.0.into(),
        fill_mode: FillMode::Full,
    }
}