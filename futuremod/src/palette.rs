use iced::{color, theme::palette::{Pair, Primary, Secondary, Success}, Color};


pub const BACKGROUND: Color = color!(0x13111F);

/// Color range.
/// 
/// Provides five colors from the lightest to the darkest.
#[derive(Debug, Clone)]
pub struct ColorRange {
  pub lightest: Pair,
  pub light: Pair,
  pub medium: Pair,
  pub dark: Pair,
  pub darkest: Pair,
}

/// Custom palette struct with much more color selection.
/// 
/// Is based on the extended palette
/// but adds more color options for each color category and
/// additional color categories.
#[derive(Debug, Clone)]
pub struct Palette {
  pub background: ColorRange,
  pub primary: Primary,
  pub secondary: Secondary,
  pub success: Success,
  pub warning: ColorRange,
  pub danger: ColorRange,
  pub is_dark: bool,
}

impl Default for Palette {
    fn default() -> Self {
      Palette {
        background: ColorRange {
          lightest: Pair {
            color: color!(0xC3BFDC),
            text: color!(0x151515),
          },
          light: Pair {
            color: color!(0x877FB8),
            text: color!(0x151515),
          },
          medium: Pair {
            color: color!(0x2F2A4C),
            text: color!(0xFAFAFA),
          },
          dark: Pair {
            color: color!(0x201C33),
            text: color!(0xFAFAFA),
          },
          darkest: Pair {
            color: color!(0x171324),
            text: color!(0xFAFAFA),
          },
        },
        primary: Primary::generate(color!(0x4926A1), BACKGROUND, color!(0xffffff)),
        secondary: Secondary::generate(color!(0x13111F), color!(0xffffff)),
        success: Success::generate(color!(0x9AFF75), BACKGROUND, color!(0xffffff)),
        danger: ColorRange {
            lightest: Pair { color: color!(0xF3D1D1), text: color!(0x000000) },
            light: Pair { color: color!(0xD35656), text: color!(0x000000) },
            medium: Pair { color: color!(0xBF3131), text: color!(0xffffff) },
            dark: Pair { color: color!(0x7B2020), text: color!(0xffffff) },
            darkest: Pair { color: color!(0x360E0E), text: color!(0xffffff) },
        },
        warning: ColorRange {
          lightest: Pair {
            color: color!(0xFFF8E2),
            text: color!(0x000000),
          },
          light: Pair {
            color: color!(0xFFDB6D),
            text: color!(0x000000),
          },
          medium: Pair {
            color: color!(0xFFCC33),
            text: color!(0x000000),
          },
          dark: Pair {
            color: color!(0xDBA400),
            text: color!(0x000000),
          },
          darkest: Pair {
            color: color!(0x6D5200),
            text: color!(0xFFFFFF),
          },
        },
        is_dark: true,
      }
    }
}

impl Palette {
  pub fn to_theme(&self) -> iced::Theme {
    iced::Theme::custom("Custom".to_string(), self.to_palette())
  }

  pub fn to_palette(&self) -> iced::theme::Palette {
    iced::theme::Palette {
      background: self.background.darkest.color,
      primary: self.primary.base.color,
      success: self.success.base.color,
      danger: self.danger.medium.color,
      text: self.background.darkest.text,
    }
  }
}

