use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::futurecop::{self, RenderCharacterFunction, RENDER_CHARACTER_FUNCTION_ADDRESS};


/// Renders a character onto the screen at the position with a palette.
/// 
/// This function returns the the y position where the next character should be rendered.
/// 
/// Directly calls an internal game function to accomplish the rendering.
/// **For now, this function does not perform any runtime checks to avoid crashes, so be careful.**
pub fn render_character(character: u32, pos_x: u32, pos_y: u32, palette: u32) -> u32 {
    let fn_ptr = RENDER_CHARACTER_FUNCTION_ADDRESS as *const();
    unsafe {
        let render_character_fn = {std::mem::transmute::<_, RenderCharacterFunction>(fn_ptr)};
        render_character_fn(character, pos_x, pos_y, palette)
    }
}

/// Render text at a position with a specific palette.
/// 
/// Renders the string in `text` at the position specified with `pos_x` and `pos_y` using the palette
/// specified in `palette`.
/// The position is absolute.
/// 
/// The text palette mainly determines the text's color. Refer to [`TextPalette`] for more details.
/// 
/// **Important: Only use characters that are supported by the game's font texture. Usually, this should include
/// all numbers, characters in the alphabet, and some special characters. However, be careful as it doesn't support
/// all ASCII special characters.**
pub fn render_text(pos_x: u32, pos_y: u32, palette: TextPalette, text: &str) {
    let characters = [text.as_bytes(), &[0x00]].concat();
    futurecop::render_text(characters.as_ptr(), pos_x, pos_y, palette.into());
}

/// Palette for text.
/// 
/// Each item represents one palette.
/// A palette gives a text a specific color.
/// This enum also provides the item [`TextPalette::Unknown(u32)`] that allows you to specify
/// any palette you want. Invalid palettes will lead to invisible text.
/// 
/// Based on internal game logic, a palette is identified with a number.
#[derive(Debug, Clone, Copy)]
pub enum TextPalette {
    Black,
    LightGreen,
    LightRed,
    LightBlue,
    Gray,
    Red,
    Green,
    Blue,
    White,
    Yellow,
    Pink,
    SkyBlue,
    Amber,
    Purple,
    Seal,
    DarkGray,
    Unknown(u32),
}

pub const TEXT_PALETTES: [TextPalette; 16] = [
    TextPalette::Black,
    TextPalette::LightGreen,
    TextPalette::LightRed,
    TextPalette::LightBlue,
    TextPalette::Gray,
    TextPalette::Red,
    TextPalette::Green,
    TextPalette::Blue,
    TextPalette::White,
    TextPalette::Yellow,
    TextPalette::Pink,
    TextPalette::SkyBlue,
    TextPalette::Amber,
    TextPalette::Purple,
    TextPalette::Seal,
    TextPalette::DarkGray,
];

impl Display for TextPalette {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}


impl Into<u32> for TextPalette {
    fn into(self) -> u32 {
        match self {
            TextPalette::Black => 0,
            TextPalette::LightRed => 1,
            TextPalette::LightGreen => 2,
            TextPalette::LightBlue => 3,
            TextPalette::Gray => 4,
            TextPalette::Red => 5,
            TextPalette::Green => 6,
            TextPalette::Blue => 7,
            TextPalette::White => 8,
            TextPalette::Yellow => 9,
            TextPalette::Pink => 10,
            TextPalette::SkyBlue => 11,
            TextPalette::Amber => 12,
            TextPalette::Purple => 13,
            TextPalette::Seal => 14,
            TextPalette::DarkGray => 15,
            TextPalette::Unknown(x) => x,
        }
    }
}

impl From<u32> for TextPalette {
    fn from(value: u32) -> Self {
        match value {
            0 => TextPalette::Black,
            1 => TextPalette::LightGreen,
            2 => TextPalette::LightRed,
            3 => TextPalette::LightBlue,
            4 => TextPalette::Gray,
            5 => TextPalette::Red,
            6 => TextPalette::Green,
            7 => TextPalette::Blue,
            8 => TextPalette::White,
            9 => TextPalette::Yellow,
            10 => TextPalette::Pink,
            11 => TextPalette::SkyBlue,
            12 => TextPalette::Amber,
            13 => TextPalette::Purple,
            14 => TextPalette::Seal,
            15 => TextPalette::DarkGray,
            x => TextPalette::Unknown(x),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Into<u32> for Color {
    fn into(self) -> u32 {
        // Restrict all color values to 5 bit
        let capped_r: u32 = self.red as u32 & 0x1f;
        let capped_g: u32 = self.green as u32 & 0x1f;
        let capped_b: u32 = self.blue as u32 & 0x1f;

        // Construct color integer
        (capped_r << 10) | (capped_g << 5) | capped_b
    }
}

pub fn render_rectangle(color: Color, pos_x: u16, pos_y: u16, width: u16, height: u16, semi_transparent: bool) {
    let converted_color: u32 = color.into();
    let converted_semi_transparent = match semi_transparent {
        true => 0x3d,
        false => 0x35,
    };

    futurecop::render_rectangle(converted_color, pos_x, pos_y, width, height, converted_semi_transparent)
}