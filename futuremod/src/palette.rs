use iced::{color, theme::palette::{Pair, Primary, Secondary}, Color};


pub const BACKGROUND: Color = color!(0x161517);

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

#[derive(Debug)]
pub enum BaseColor {
  Lightest,
  Light,
  Medium,
  Dark,
  Darkest,
}

impl ColorRange {
  /// Get a specific base color from the color range.
  pub fn get_base(&self, base: &BaseColor) -> Pair {
    match base {
        BaseColor::Lightest => self.lightest,
        BaseColor::Light => self.light,
        BaseColor::Medium => self.medium,
        BaseColor::Dark => self.dark,
        BaseColor::Darkest => self.darkest,
    }
  }
}

#[derive(Debug, Clone)]
pub enum Shade {
  S50,
  S100,
  S200,
  S300,
  S400,
  S500,
  S600,
  S700,
  S800,
  S900,
  S950,
}

impl Shade {
  pub fn lighter(&self) -> Shade {
    match self {
        Shade::S50 => Shade::S50,
        Shade::S100 => Shade::S50,
        Shade::S200 => Shade::S100,
        Shade::S300 => Shade::S200,
        Shade::S400 => Shade::S300,
        Shade::S500 => Shade::S400,
        Shade::S600 => Shade::S500,
        Shade::S700 => Shade::S600,
        Shade::S800 => Shade::S700,
        Shade::S900 => Shade::S800,
        Shade::S950 => Shade::S900,
    }
  }

  pub fn darker(&self) -> Shade {
    match self {
      Shade::S50 => Shade::S100,
      Shade::S100 => Shade::S200,
      Shade::S200 => Shade::S300,
      Shade::S300 => Shade::S400,
      Shade::S400 => Shade::S500,
      Shade::S500 => Shade::S600,
      Shade::S600 => Shade::S700,
      Shade::S700 => Shade::S800,
      Shade::S800 => Shade::S900,
      Shade::S900 => Shade::S950,
      Shade::S950 => Shade::S950,
    }
  }

  pub fn get_contrast(&self) -> Shade {
    match self {
        Shade::S50 => Shade::S950,
        Shade::S100 => Shade::S950,
        Shade::S200 => Shade::S950,
        Shade::S300 => Shade::S950,
        Shade::S400 => Shade::S950,
        Shade::S500 => Shade::S50,
        Shade::S600 => Shade::S50,
        Shade::S700 => Shade::S50,
        Shade::S800 => Shade::S50,
        Shade::S900 => Shade::S50,
        Shade::S950 => Shade::S50,
    }
  }

  pub fn slight_contrast(&self) -> Shade {
    match self {
      Shade::S50 => Shade::S100,
      Shade::S100 => Shade::S200,
      Shade::S200 => Shade::S300,
      Shade::S300 => Shade::S400,
      Shade::S400 => Shade::S500,
      Shade::S500 => Shade::S600,
      Shade::S600 => Shade::S500,
      Shade::S700 => Shade::S600,
      Shade::S800 => Shade::S700,
      Shade::S900 => Shade::S800,
      Shade::S950 => Shade::S900,
  }
  }
}

#[derive(Debug, Clone)]
pub struct ColorShades {
  s50: Color,
  s100: Color,
  s200: Color,
  s300: Color,
  s400: Color,
  s500: Color,
  s600: Color,
  s700: Color,
  s800: Color,
  s900: Color,
  s950: Color,
}

impl ColorShades {
  pub fn get_shade(&self, shade: &Shade) -> Color {
    match shade {
        Shade::S50 => self.s50,
        Shade::S100 => self.s100,
        Shade::S200 => self.s200,
        Shade::S300 => self.s300,
        Shade::S400 => self.s400,
        Shade::S500 => self.s500,
        Shade::S600 => self.s600,
        Shade::S700 => self.s700,
        Shade::S800 => self.s800,
        Shade::S900 => self.s900,
        Shade::S950 => self.s950,
    }
  }

  pub fn get_base(&self, base: &BaseColor) -> Color {
    match base {
        BaseColor::Lightest => self.s100,
        BaseColor::Light => self.s300,
        BaseColor::Medium => self.s500,
        BaseColor::Dark => self.s700,
        BaseColor::Darkest => self.s900,
    }
  }
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
  pub success: ColorRange,
  pub warning: ColorRange,
  pub danger: ColorRange,
  pub base: ColorShades,
}



impl Default for Palette {
    fn default() -> Self {
      Palette {
        background: ColorRange {
          lightest: Pair {
            color: color!(0x3D3A40),
            text: color!(0xFAFAFA),
          },
          light: Pair {
            color: color!(0x363339),
            text: color!(0xFAFAFA),
          },
          medium: Pair {
            color: color!(0x2B292D),
            text: color!(0xFAFAFA),
          },
          dark: Pair {
            color: color!(0x1F1E20),
            text: color!(0xFAFAFA),
          },
          darkest: Pair {
            color: color!(0x161517),
            text: color!(0xFAFAFA),
          },
        },
        primary: Primary::generate(color!(0xAE6BFF), BACKGROUND, color!(0xFAFAFA)),
        secondary: Secondary::generate(color!(0x13111F), color!(0xffffff)),
        success: ColorRange {
            lightest: Pair { color: color!(0xCDFFBA), text: color!(0x000000) },
            light: Pair { color: color!(0xB7FF9D), text: color!(0x000000) },
            medium: Pair { color: color!(0x9AFF75), text: color!(0xffffff) },
            dark: Pair { color: color!(0x155000), text: color!(0xffffff) },
            darkest: Pair { color: color!(0x0E3500), text: color!(0xffffff) },
        },
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
        base: ColorShades {
            s50: color!(0xEEEEEF),
            s100: color!(0xDEDCE0),
            s200: color!(0xCDCBD0),
            s300: color!(0xBDB9C0),
            s400: color!(0xACA8B0),
            s500: color!(0x8B8591),
            s600: color!(0x59555E),
            s700: color!(0x49454C),
            s800: color!(0x38353B),
            s900: color!(0x28262A),
            s950: color!(0x161517),
        }
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
      success: self.success.medium.color,
      danger: self.danger.medium.color,
      text: self.background.darkest.text,
    }
  }
}


