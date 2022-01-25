mod app;
mod utils;
// mod gameboy;
// mod utils;

pub use app::App;

#[cfg(target_arch = "wasm32")]
use eframe::wasm_bindgen::{self, prelude::*};

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn start(canvas_id: &str) -> Result<(), eframe::wasm_bindgen::JsValue> {
    let app = App::new();
    eframe::start_web(canvas_id, Box::new(app))
}
