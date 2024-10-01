/// Big inspiration for custom theming was from: https://github.com/squidowl/halloy and https://github.com/B0ney/xmodits
mod button;
pub use button::{button, hover_button};

mod icon;
pub use icon::*;

mod font;
pub use font::*;

pub type Renderer = iced::Renderer;
pub type Theme = crate::theme::Theme;

pub type Element<'a, Message> = iced::Element<'a, Message, Theme, Renderer>;
pub type Column<'a, Message> = iced::widget::Column<'a, Message, Theme, Renderer>;

#[allow(unused)]
pub type Row<'a, Message> = iced::widget::Row<'a, Message, Theme, Renderer>;
pub type Text<'a> = iced::widget::Text<'a, Theme>;