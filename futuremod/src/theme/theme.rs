/// Custom theme based on a default theme.
///
/// Based on the given default theme and implements custom styling only for some widgets.
#[derive(Debug, Default)]
pub struct Theme {
  pub theme: iced::Theme,
  pub palette: crate::palette::Palette,
}

impl Theme {
  pub fn new(palette: crate::palette::Palette) -> Self {
    Theme {
      theme: palette.to_theme(),
      palette,
    }
  }
}

impl iced::application::DefaultStyle for Theme {
    fn default_style(&self) -> iced::daemon::Appearance {
        iced::daemon::Appearance {
            background_color: self.palette.background.darkest.color,
            text_color: self.palette.background.darkest.text,
        }
    }
}
