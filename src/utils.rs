#![allow(dead_code)]

use std::future::Future;

use instant::{Duration, Instant};

pub struct Fps {
    fps: usize,
    last_fps: usize,

    start: Instant,
}

impl Fps {
    pub fn update(&mut self) -> usize {
        let now = Instant::now();
        if now > self.start + Duration::from_secs(1) {
            self.last_fps = self.fps;
            self.fps = 0;

            self.start = now;
        }

        self.fps += 1;
        self.last_fps
    }

    pub fn current_rate(&self) -> usize {
        self.last_fps
    }
}

impl Default for Fps {
    fn default() -> Self {
        Self {
            fps: Default::default(),
            last_fps: Default::default(),
            start: Instant::now(),
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub fn spawn<F: Future<Output = ()> + 'static>(f: F) {
    wasm_bindgen_futures::spawn_local(f);
}
#[cfg(not(target_arch = "wasm32"))]
pub fn spawn<F: Future<Output = ()> + Send + 'static>(f: F) {
    std::thread::spawn(|| pollster::block_on(f));
}
