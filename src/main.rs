// use goodboy::App;

#[cfg(not(target_arch = "wasm32"))]
pub fn main() {
    env_logger::init();

    pollster::block_on(async move {
        goodboy::run().await;
    })
}
