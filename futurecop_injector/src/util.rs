use iced::Color;
use palette::{Hsl, FromColor, rgb::Rgb, Mix};

// Yoinked from https://github.com/iced-rs/iced/blob/master/style/src/theme/palette.rs because functions are not public

pub fn to_hsl(color: Color) -> Hsl {
    Hsl::from_color(Rgb::from(color))
}

pub fn from_hsl(hsl: Hsl) -> Color {
    Rgb::from_color(hsl).into()
}

pub fn darken(color: Color, amount: f32) -> Color {
    let mut hsl = to_hsl(color);

    hsl.lightness = if hsl.lightness - amount < 0.0 {
        0.0
    } else {
        hsl.lightness - amount
    };

    from_hsl(hsl)
}

pub fn lighten(color: Color, amount: f32) -> Color {
    let mut hsl = to_hsl(color);

    hsl.lightness = if hsl.lightness + amount > 1.0 {
        1.0
    } else {
        hsl.lightness + amount
    };

    from_hsl(hsl)
}

pub fn is_dark(color: Color) -> bool {
    to_hsl(color).lightness < 0.6
}

#[allow(unused)]
pub fn deviate(color: Color, amount: f32) -> Color {
    if is_dark(color) {
        lighten(color, amount)
    } else {
        darken(color, amount)
    }
}

pub fn mix(a: Color, b: Color, factor: f32) -> Color {
    let a_lin = Rgb::from(a).into_linear();
    let b_lin = Rgb::from(b).into_linear();

    let mixed = a_lin.mix(b_lin, factor);
    Rgb::from_linear(mixed).into()
}
