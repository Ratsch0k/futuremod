use iced::widget::markdown;

use crate::palette::BaseColor;

use super::{Container, Theme};

impl markdown::Catalog for Theme {
    fn code_block<'a>() -> <Self as iced::widget::container::Catalog>::Class<'a> {
        Container::BoxWithBase(BaseColor::Medium)
    }
}
