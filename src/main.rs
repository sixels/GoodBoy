use goodboy::App;

#[cfg(not(target_arch = "wasm32"))]
pub fn main() {
    env_logger::init();

    let app = App::new();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
}
