#[cfg(not(target_arch = "wasm32"))]
pub fn main() {
    use goodboy::{run, GameBoy};

    env_logger::init();

    pollster::block_on(run(GameBoy::new()))
}
