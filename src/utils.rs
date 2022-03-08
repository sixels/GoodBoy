#![allow(dead_code)]

use std::fmt::Display;
use std::future::Future;
use std::time::Duration;

use wasm_timer::Instant;

pub struct Fps {
    fps: usize,
    current_rate: usize,

    start: Instant,
}

impl Fps {
    pub fn update(&mut self) -> usize {
        let now = Instant::now();
        if now > self.start + Duration::from_secs(1) {
            self.current_rate = self.fps;
            self.fps = 0;

            self.start = now;
        }

        self.fps += 1;
        self.current_rate
    }

    pub fn current_rate(&self) -> usize {
        self.current_rate
    }
}

impl Default for Fps {
    fn default() -> Self {
        Self {
            fps: Default::default(),
            current_rate: Default::default(),
            start: Instant::now(),
        }
    }
}

impl Display for Fps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let current_rate = self.current_rate();
        write!(f, "{current_rate} FPS")?;
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
pub fn spawn<F: Future<Output = ()> + 'static>(f: F) {
    wasm_bindgen_futures::spawn_local(f);
}
#[cfg(not(target_arch = "wasm32"))]
pub fn spawn<F: Future<Output = ()> + Send + 'static>(f: F) {
    std::thread::spawn(move || pollster::block_on(f));
}
