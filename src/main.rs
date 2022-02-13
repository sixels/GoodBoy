#[cfg(not(target_arch = "wasm32"))]
pub fn main() {
    use std::env;

    use goodboy::{run, GameBoy};

    env_logger::init();

    let mut gameboy = GameBoy::new();
    let args = env::args();

    if let Some(path) = args.skip(1).next() {
        match gameboy.load_game_file(&path) {
            Ok(()) => {}
            Err(e) => {
                panic!("Could not read the file \"{path}\": {e:?}")
            }
        }
    }

    pollster::block_on(run(gameboy))
}
