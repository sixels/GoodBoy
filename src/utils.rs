use eframe::egui;
use goodboy_core::io::JoypadButton;
use std::sync::mpsc;

#[cfg(target_arch = "wasm32")]
use eframe::wasm_bindgen::{self, prelude::*};
#[cfg(not(target_arch = "wasm32"))]
use std::time::{self, Instant};

use crate::app::IoEvent;

#[rustfmt::skip]
pub fn handle_input(input: &egui::InputState, io_sender: mpsc::Sender<IoEvent>) {
    fn send_keys(input: &egui::InputState, io_sender: mpsc::Sender<IoEvent>) -> Result<(), mpsc::SendError<IoEvent>> {
        if input.key_pressed(egui::Key::Delete) { io_sender.send(IoEvent::ToggleFPSLimit)?; }

        if input.key_pressed(egui::Key::ArrowRight) { io_sender.send(IoEvent::ButtonPressed(JoypadButton::Right))?;  }
        if input.key_pressed(egui::Key::ArrowLeft)  { io_sender.send(IoEvent::ButtonPressed(JoypadButton::Left))?;   }
        if input.key_pressed(egui::Key::ArrowUp)    { io_sender.send(IoEvent::ButtonPressed(JoypadButton::Up))?;     }
        if input.key_pressed(egui::Key::ArrowDown)  { io_sender.send(IoEvent::ButtonPressed(JoypadButton::Down))?;   }
        if input.key_pressed(egui::Key::Z)          { io_sender.send(IoEvent::ButtonPressed(JoypadButton::A))?;      }
        if input.key_pressed(egui::Key::X)          { io_sender.send(IoEvent::ButtonPressed(JoypadButton::B))?;      }
        if input.key_pressed(egui::Key::Space)      { io_sender.send(IoEvent::ButtonPressed(JoypadButton::Select))?; }
        if input.key_pressed(egui::Key::Enter)      { io_sender.send(IoEvent::ButtonPressed(JoypadButton::Start))?;  }

        if input.key_released(egui::Key::ArrowRight) { io_sender.send(IoEvent::ButtonReleased(JoypadButton::Right))?;  }
        if input.key_released(egui::Key::ArrowLeft)  { io_sender.send(IoEvent::ButtonReleased(JoypadButton::Left))?;   }
        if input.key_released(egui::Key::ArrowUp)    { io_sender.send(IoEvent::ButtonReleased(JoypadButton::Up))?;     }
        if input.key_released(egui::Key::ArrowDown)  { io_sender.send(IoEvent::ButtonReleased(JoypadButton::Down))?;   }
        if input.key_released(egui::Key::Z)          { io_sender.send(IoEvent::ButtonReleased(JoypadButton::A))?;      }
        if input.key_released(egui::Key::X)          { io_sender.send(IoEvent::ButtonReleased(JoypadButton::B))?;      }
        if input.key_released(egui::Key::Space)      { io_sender.send(IoEvent::ButtonReleased(JoypadButton::Select))?; }
        if input.key_released(egui::Key::Enter)      { io_sender.send(IoEvent::ButtonReleased(JoypadButton::Start))?;  }
        Ok(())
    }
    send_keys(input, io_sender).ok();
}

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
