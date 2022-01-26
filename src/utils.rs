use std::future::Future;

#[cfg(target_arch = "wasm32")]
use eframe::wasm_bindgen::{self, prelude::*};
#[cfg(not(target_arch = "wasm32"))]
use std::time::{self, Instant};

trait TimeUnit<T> {
    fn now() -> Self
    where
        Self: Sized;

    fn one_sec() -> T
    where
        Self: Sized;
}

#[cfg(target_arch = "wasm32")]
type Time = f64;
#[cfg(not(target_arch = "wasm32"))]
type Time = Instant;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    type Date;

    #[wasm_bindgen(static_method_of = Date)]
    pub fn now() -> f64;
}

#[cfg(target_arch = "wasm32")]
impl TimeUnit<f64> for f64 {
    fn now() -> f64 {
        Date::now()
    }

    fn one_sec() -> f64 {
        1000.0
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl TimeUnit<time::Duration> for Instant {
    fn now() -> Self {
        Instant::now()
    }

    fn one_sec() -> time::Duration {
        time::Duration::from_secs(1)
    }
}

pub struct Fps {
    fps: usize,
    last_fps: usize,

    start: Time,
}

impl Fps {
    pub fn update(&mut self) -> usize {
        let now = Time::now();
        if now > self.start + Time::one_sec() {
            self.last_fps = self.fps;
            self.fps = 0;

            self.start = now;
        }

        self.fps += 1;
        self.last_fps
    }

    pub fn counter(&self) -> usize {
        self.last_fps
    }
}

impl Default for Fps {
    fn default() -> Self {
        Self {
            fps: Default::default(),
            last_fps: Default::default(),
            start: Time::now(),
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub fn spawn<F: Future<Output = ()> + 'static>(f: F) {
    wasm_bindgen_futures::spawn_local(f);
}
#[cfg(not(target_arch = "wasm32"))]
pub fn spawn<F: Future<Output = ()> + Send + 'static>(f: F) {
    std::thread::spawn(|| smol::future::block_on(f));
}
