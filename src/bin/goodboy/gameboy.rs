mod common;

#[cfg(not(target_arch = "wasm32"))]
#[path = "gameboy/native_runtime.rs"]
pub mod runtime;
#[cfg(target_arch = "wasm32")]
#[path = "gameboy/wasm_runtime.rs"]
pub mod runtime;

use sixels_gb::{io::JoypadButton, ppu::ColorScheme};

pub enum IoEvent {
    ButtonPressed(JoypadButton),
    ButtonReleased(JoypadButton),
    SetColorScheme(ColorScheme),
    ToggleFPSLimit,
    Exit,
}

lazy_static::lazy_static! {
    pub static ref COLOR_SCHEMES: Vec<ColorScheme> = vec![
        ColorScheme::GRAY,
        ColorScheme::BLUE_ALT,
        ColorScheme::GREEN,
        ColorScheme::BLUE,
        ColorScheme::RED,
    ];
}
pub type ColorSchemeIter = Box<dyn Iterator<Item = ColorScheme>>;
