use std::sync::mpsc;

use goodboy_core::vm::VM;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};
use winit_input_helper::WinitInputHelper; 

use super::{common::{self, WgpuState}, ColorSchemeIter, IoEvent};

use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    type Date;

    #[wasm_bindgen(static_method_of = Date)]
    pub fn now() -> f64;
}

pub async fn run(window: Window, event_loop: EventLoop<()>, mut vm: VM) {
    let mut input = WinitInputHelper::new();

    let mut wgpu_state = WgpuState::new(&window).await;

    let (io_sender, io_receiver) = mpsc::channel();

    let mut color_schemes_iter: ColorSchemeIter = box super::COLOR_SCHEMES.iter().copied().cycle();
    vm.set_color_scheme(color_schemes_iter.next().unwrap());

    let mut clocks = 0;

    let mut start = Date::now();
    let mut fps_count = 0;
    let mut fps = 0;

    let mut screen = None;
    window.request_redraw();
    
    event_loop.run(move |event, _, control_flow| {

        // VM loop
        {
            let clocks_to_run = (4194304.0 / 1000.0 * 11.0f64).round() as u32;

            while clocks < clocks_to_run {
                clocks += vm.tick() as u32;

                if vm.check_vblank() {
                    screen = Some(vm.get_screen());
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

        let now = Date::now();
        if now > start + 1000.0 {
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
                if let Some(screen) = screen.take() {
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
