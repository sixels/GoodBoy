use std::{
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

// use pixels::{PixelsBuilder, SurfaceTexture};
use goodboy_core::vm::{SCREEN_HEIGHT, SCREEN_WIDTH, VM};
use wgpu_glyph::{ab_glyph, GlyphBrushBuilder, Section, Text};
use winit::{
    event::{Event, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};
use winit_input_helper::WinitInputHelper;

use super::{
    common::{self, WgpuState},
    ColorSchemeIter, IoEvent,
};

pub async fn run(window: Window, event_loop: EventLoop<()>, vm: VM) -> ! {
    let mut input = WinitInputHelper::new();

    let wgpu_state = WgpuState::new(&window).await;

    // let font =
    //     ab_glyph::FontArc::try_from_slice(include_bytes!("../../assets/fonts/ReturnofGanon.ttf"))
    //         .expect("Could not open the font");

    // let mut glyph_brush = GlyphBrushBuilder::using_font(font).build(&device, render_format);

    let (screen_sender, screen_receiver) = mpsc::sync_channel(1);
    let (io_sender, io_receiver) = mpsc::channel();

    // let screen_receiver = fps_counter_middleware(screen_receiver);

    thread::spawn(move || {
        common::vm_loop(vm, screen_sender, io_receiver);
    });

    let mut color_schemes_iter: ColorSchemeIter = box super::COLOR_SCHEMES.iter().copied().cycle();

    let mut start = Instant::now();
    let mut text_fps = String::from("FPS: 0");
    let mut fps = 0;

    window.request_redraw();
    event_loop.run(move |event, _, control_flow| {
        let now = Instant::now();
        if now > start + Duration::from_secs(1) {
            start = now;
            text_fps = format!("FPS: {}", fps);
            fps = 0
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
                    wgpu_state.render_frame(screen.as_ref());
                    fps += 1;
                }
            }
            _ => (),
        };

        if *control_flow != ControlFlow::Exit && input.update(&event) {
            if input.held_control() && input.key_pressed(VirtualKeyCode::Q) {
                *control_flow = ControlFlow::Exit;
            }

            if common::handle_input(&mut input, &io_sender, Some(&mut color_schemes_iter)).is_err()
            {
                *control_flow = ControlFlow::Exit;
            }
        }

        if *control_flow == ControlFlow::Exit {
            io_sender.send(IoEvent::Exit).ok();
            // Drop the vm before exit
            thread::sleep(Duration::from_millis(100));
        }
    });
}
