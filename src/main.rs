#![feature(stmt_expr_attributes)]
#![feature(box_syntax)]
#![feature(try_blocks)]
// #![feature(extern_types)]


mod gameboy;

mod utils;

use goodboy_core::vm::VM;
use winit::event_loop::EventLoop;

pub fn main() {
    let event_loop = EventLoop::new();
    let (window, _, _, _) = utils::create_window("Good Boy 🐶", &event_loop);

    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();

        // Get the ROM path from the first argument
        let mut args = std::env::args().skip(1);
        let rom_path = args
            .next()
            .expect("You must pass the rom path as argument.");

        let vm = VM::new(rom_path).unwrap();

        pollster::block_on(gameboy::runtime::run(window, event_loop, vm));
    }
    #[cfg(target_arch = "wasm32")]
    {


        std::panic::set_hook(box |error| log::error!("Panicked: {}", error));
        console_log::init().unwrap();

        use winit::platform::web::WindowExtWebSys;
        // On wasm, append the canvas to the document body
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| doc.body())
            .and_then(|body| {
                body.append_child(&web_sys::Element::from(window.canvas()))
                    .ok()
            })
            .unwrap();

        let vm = VM::new_with_buffer(include_bytes!("../assets/roms/zelda.gb"));

        wasm_bindgen_futures::spawn_local(gameboy::runtime::run(window, event_loop, vm));
    }
}
