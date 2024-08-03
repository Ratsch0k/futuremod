use iced::font::Weight;

/// Return the default font in bold.
pub fn bold() -> iced::Font {
  iced::Font {
    weight: Weight::Bold,
    ..iced::Font::default()
  }
}