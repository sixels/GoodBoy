use std::{
    sync::mpsc::{self, TryRecvError},
    thread,
    time::Duration,
};

use pixels::{PixelsBuilder, SurfaceTexture};
use goodboy_core::vm::VM;
use winit::{
    event::{Event, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};
use winit_input_helper::WinitInputHelper;

use super::{common, ColorSchemeIter, IoEvent};
use crate::utils::fps_counter_middleware;

pub fn run(window: Window, event_loop: EventLoop<()>) -> ! {
    use goodboy_core::vm::{SCREEN_HEIGHT, SCREEN_WIDTH};


    let mut input = WinitInputHelper::new();


    let mut pixels = {
        let size = window.inner_size();
        let surface_texture = SurfaceTexture::new(size.width, size.height, &window);
        PixelsBuilder::new(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32, surface_texture)
            .enable_vsync(true)
            .build()
            .unwrap()
    };

    // Get the ROM path from the first argument
    let mut args = std::env::args().skip(1);
    let rom_path = args
        .next()
        .expect("You must pass the rom path as argument.");

    let vm = VM::new(rom_path).unwrap();

    let (screen_sender, screen_receiver) = mpsc::sync_channel(1);
    let (io_sender, io_receiver) = mpsc::channel();

    let screen_receiver = fps_counter_middleware(screen_receiver);

    thread::spawn(move || {
        common::vm_loop(vm, screen_sender, io_receiver);
    });

    let mut color_schemes_iter: ColorSchemeIter = box super::COLOR_SCHEMES.iter().copied().cycle();

    pixels.render().unwrap();
    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            _ => (),
        };

        match screen_receiver.try_recv() {
            Ok(data) => {
                pixels.get_frame().copy_from_slice(&*data);
                if pixels.render().is_err() {
                    *control_flow = ControlFlow::Exit;
                }
            }
            Err(TryRecvError::Empty) => (),
            Err(_) => {
                *control_flow = ControlFlow::Exit;
            }
        }

        if *control_flow != ControlFlow::Exit && input.update(&event) {
            if let Some(size) = input.window_resized() {
                pixels.resize_surface(size.width, size.height);
            }

            if input.held_control() && input.key_pressed(VirtualKeyCode::Q) {
                *control_flow = ControlFlow::Exit;
            }

            if common::handle_input(&mut input, &io_sender, Some(&mut color_schemes_iter)).is_err()
            {
                *control_flow = ControlFlow::Exit;
            }
        }

        // Drop the vm before exit
        if *control_flow == ControlFlow::Exit {
            io_sender.send(IoEvent::Exit).ok();
            thread::sleep(Duration::from_millis(100));
        }
    });
}
