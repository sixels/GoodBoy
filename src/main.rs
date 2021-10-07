use std::{
    sync::mpsc::{SyncSender, TrySendError},
    thread,
};

use pixels::{PixelsBuilder, SurfaceTexture};
use sixels_gb::vm::{Screen, SCREEN_HEIGHT, SCREEN_WIDTH, VM};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    // Get the ROM path from the first argument
    let mut args = std::env::args().skip(1);
    let rom_path = args
        .next()
        .expect("You must pass the rom path as argument.");
    let _bios_path = args.next();

    let event_loop = EventLoop::new();

    let window = {
        let gb_screen_size = LogicalSize::new(SCREEN_WIDTH as f64, SCREEN_HEIGHT as f64);
        let window_size = LogicalSize::new(1.0 * SCREEN_WIDTH as f64, 1.0 * SCREEN_HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Good Boy 🐶")
            .with_min_inner_size(gb_screen_size)
            .with_inner_size(window_size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        PixelsBuilder::new(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32, surface_texture)
            .enable_vsync(true)
            .build()
            .unwrap()
    };

    let vm;
    // if let Some(bios_path) = bios_path {
        // println!("Loading with BIOS");
        // vm = VM::new_blank(bios_path, rom_path);
    // } else {
        println!("Loading without BIOS");
        vm = VM::new(rom_path);
    // }
    let vm = vm.unwrap();
    let (screen_sender, screen) = std::sync::mpsc::sync_channel(1);

    thread::spawn(move || {
        vm_loop(vm, screen_sender);
    });

    pixels.render().unwrap();

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }

            Event::RedrawRequested(_) | Event::MainEventsCleared => match screen.recv() {
                Ok(data) => {
                    pixels.get_frame().copy_from_slice(&*data);

                    pixels.render().unwrap();
                }
                Err(_) => {
                    *control_flow = ControlFlow::Exit;
                }
            },

            _ => (),
        };
    });
}

fn vm_loop(vm: VM, sender: SyncSender<Screen>) {
    let clocks_to_run = (4194304.0 / 1000.0 * 16f64).round() as u32;
    let mut clocks = 0;

    let mut vm = vm;

    loop {
        while clocks < clocks_to_run {
            clocks += vm.tick() as u32;

            if vm.check_vblank() {
                if let Err(TrySendError::Disconnected(..)) = sender.try_send(vm.get_screen()) {
                    return;
                }
            }
        }

        clocks -= clocks_to_run;
    }
}
