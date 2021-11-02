mod common;

#[cfg(not(target_arch = "wasm32"))]
#[path = "gameboy/native_runtime.rs"]
pub mod runtime;
#[cfg(target_arch = "wasm32")]
#[path = "gameboy/wasm_runtime.rs"]
pub mod runtime;

use goodboy_core::{
    io::JoypadButton,
    ppu::{color::Color, ColorScheme},
};

#[allow(dead_code)]
pub enum IoEvent {
    ButtonPressed(JoypadButton),
    ButtonReleased(JoypadButton),
    SetColorScheme(ColorScheme),
    ToggleFPSLimit,
    Exit,
}

lazy_static::lazy_static! {
    pub static ref COLOR_SCHEMES: Vec<ColorScheme> = vec![
        // black and white
        ColorScheme::new(
            Color::rgb(0x000000),
            Color::rgb(0x525252),
            Color::rgb(0x949494),
            Color::rgb(0xFFFFFF),
        ),

        // pink
        ColorScheme::new(
            Color::rgb(0x21193C),
            Color::rgb(0x932F7B),
            Color::rgb(0xE67B8B),
            Color::rgb(0xF5D2B8),
        ),

        // green
        ColorScheme::new(
            Color::rgb(0x372A51),
            Color::rgb(0x3A5068),
            Color::rgb(0x5A8F78),
            Color::rgb(0xF5F6DF),
        ),

        ColorScheme::RED,

        // blue
        ColorScheme::new(
            Color::rgb(0x1C1530),
            Color::rgb(0x2A308B),
            Color::rgb(0x367DD8),
            Color::rgb(0x8DE2F6),
        ),

        // copper
        ColorScheme::new(
            Color::rgb(0x1B1829),
            Color::rgb(0x3F7A63),
            Color::rgb(0xF4A374),
            Color::rgb(0xFFFBD1),
        ),

];
}
pub type ColorSchemeIter = Box<dyn Iterator<Item = ColorScheme>>;
