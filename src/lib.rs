mod app;
mod gameboy;
mod io;
mod utils;
#[cfg(target_arch = "wasm32")]
mod web;

pub use crate::app::App;
pub use crate::gameboy::GameBoy;

#[cfg(target_arch = "wasm32")]
pub mod wasm {
    use futures_executor as executor;
    use futures_util::task::LocalSpawnExt;
    use wasm_bindgen::prelude::*;

    use crate::{GameBoy, App};

    #[wasm_bindgen]
    pub fn start() {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init().ok();

        let mut pool = executor::LocalPool::new();
        let spawner = pool.spawner();

        let app = App::new(GameBoy::new()).unwrap();
        spawner.spawn_local(app.run()).unwrap();

        pool.run();
    }
}
