use std::sync::mpsc;
#[cfg(not(target_arch = "wasm32"))]
use std::{
    thread,
    time::{Duration, Instant},
};


use goodboy_core::vm::VM;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

use winit_input_helper::WinitInputHelper;

use super::{
    common::{self, WgpuState},
    ColorSchemeIter, IoEvent,
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    type Date;

    #[wasm_bindgen(static_method_of = Date)]
    pub fn now() -> f64;
}

#[cfg(target_arch = "wasm32")]
fn now() -> f64 {
    Date::now()
}
#[cfg(target_arch = "wasm32")]
fn one_sec() -> f64 {
    1000.0
}
#[cfg(not(target_arch = "wasm32"))]
fn now() -> Instant {
    Instant::now()
}
#[cfg(not(target_arch = "wasm32"))]
fn one_sec() -> Duration {
    Duration::from_secs(1)
}

pub async fn run(window: Window, event_loop: EventLoop<()>, mut vm: VM) {
    let mut input = WinitInputHelper::new();

    let mut wgpu_state = WgpuState::new(&window).await;

    let (screen_sender, screen_receiver) = mpsc::sync_channel(1);
    let (io_sender, io_receiver) = mpsc::channel();

    let mut color_schemes_iter: ColorSchemeIter = box super::COLOR_SCHEMES.iter().copied().cycle();
    vm.set_color_scheme(color_schemes_iter.next().unwrap());

    #[cfg(not(target_arch = "wasm32"))]
    let mut vm_loop_handle = Some(thread::spawn(move || {
        common::vm_loop(vm, screen_sender, io_receiver);
    }));

    let mut clocks = 0;

    let mut start = now();
    let mut fps_count = 0;
    let mut fps = 0;

    window.request_redraw();
    event_loop.run(move |event, _, control_flow| {
        // VM loop
        #[cfg(target_arch = "wasm32")]
        {
            let clocks_to_run = (4194304.0 / 1000.0 * 11.0f64).round() as u32;

            while clocks < clocks_to_run {
                clocks += vm.tick() as u32;

                if vm.check_vblank() {
                    let _ = screen_receiver.try_recv();
                    screen_sender.send(vm.get_screen()).ok();
                }
            }
            clocks -= clocks_to_run;

            loop {
                match io_receiver.try_recv() {
                    Ok(event) => match event {
                        IoEvent::ButtonPressed(button) => vm.press_button(button),
                        IoEvent::ButtonReleased(button) => vm.release_button(button),
                        IoEvent::SetColorScheme(color_scheme) => vm.set_color_scheme(color_scheme),

                        _ => (),
                    },
                    Err(_) => break,
                }
            }
        }

        let now = now();
        if now > start + one_sec() {
            start = now;
            fps = fps_count;
            fps_count = 0
        }

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }

            Event::WindowEvent {
                event: WindowEvent::Resized(new_size),
                ..
            } => {
                wgpu_state.resize(new_size.width, new_size.height);
            }

            Event::RedrawRequested(..) | Event::MainEventsCleared => {
                let screen = match screen_receiver.try_recv() {
                    Ok(data) => Some(data),
                    Err(mpsc::TryRecvError::Empty) => None,
                    Err(_) => {
                        *control_flow = ControlFlow::Exit;
                        None
                    }
                };

                if let Some(screen) = &screen {
                    fps_count += 1;
                    wgpu_state.render_frame(screen.as_ref(), fps);
                }
            }

            _ => (),
        };

        if *control_flow != ControlFlow::Exit && input.update(&event) {
            if common::handle_input(&mut input, &io_sender, Some(&mut color_schemes_iter)).is_err()
            {
                *control_flow = ControlFlow::Exit;
            }
        }
    });
}
