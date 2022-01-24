use std::{
    sync::mpsc,
};

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

pub async fn run(window: Window, event_loop: EventLoop<()>, mut vm: VM) -> ! {
    let mut input = WinitInputHelper::new();

    let mut wgpu_state = WgpuState::new(&window).await;

    let (screen_sender, screen_receiver) = mpsc::sync_channel(1);
    let (io_sender, io_receiver) = mpsc::channel();

    let mut color_schemes_iter: ColorSchemeIter = box super::COLOR_SCHEMES.iter().copied().cycle();
    vm.set_color_scheme(color_schemes_iter.next().unwrap());

    let mut vm_loop_handle = Some(thread::spawn(move || {
        common::vm_loop(vm, screen_sender, io_receiver);
    }));

    let mut start = Instant::now();
    let mut fps_count = 0;
    let mut fps = 0;

    window.request_redraw();
    event_loop.run(move |event, _, control_flow| {
        let now = Instant::now();
        if now > start + Duration::from_secs(1) {
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
            // if input.held_control() && input.key_pressed(VirtualKeyCode::Q) {
            //     *control_flow = ControlFlow::Exit;
            // }

            if common::handle_input(&mut input, &io_sender, Some(&mut color_schemes_iter)).is_err()
            {
                *control_flow = ControlFlow::Exit;
            }
        }

        if *control_flow == ControlFlow::Exit {
            io_sender.send(IoEvent::Exit).ok();
            vm_loop_handle.take().map(thread::JoinHandle::join);
        }
    });
}
