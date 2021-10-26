#![feature(stmt_expr_attributes)]
#![feature(box_syntax)]
#![feature(try_blocks)]

mod gameboy;

mod utils;

use utils::create_window;
use winit::event_loop::EventLoop;

pub fn main() {
    let event_loop = EventLoop::new();
    let (window, _, _, _) = create_window("Good Boy 🐶", &event_loop);

    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(box |error| log::error!("Panicked: {}", error));
        console_log::init().unwrap();
        wasm_bindgen_futures::spawn_local(gameboy::runtime::run(window, event_loop));
    }
    #[cfg(not(target_arch = "wasm32"))]
    gameboy::runtime::run(window, event_loop)
}
