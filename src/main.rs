use std::{
    sync::mpsc::{SyncSender, TrySendError},
    thread,
};

use pixels::{PixelsBuilder, SurfaceTexture};
use sixels_gb::vm::{Screen, SCREEN_HEIGHT, SCREEN_WIDTH, VM};
use winit::{
    dpi::{LogicalPosition, LogicalSize, PhysicalSize},
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;

fn main() {
    // Get the ROM path from the first argument
    let mut args = std::env::args().skip(1);
    let rom_path = args
        .next()
        .expect("You must pass the rom path as argument.");
    let _bios_path = args.next();

    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();

    let (window, p_width, p_height, _) = create_window("Good Boy 🐶", &event_loop);

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(p_width, p_height, &window);
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
            _ => (),
        };

        match screen.try_recv() {
            Ok(data) => {
                pixels.get_frame().copy_from_slice(&*data);
                pixels.render().unwrap();
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => (),
            Err(_) => {
                *control_flow = ControlFlow::Exit;
            }
        }

        if input.update(&event) {
            // Resize the window
            if let Some(size) = input.window_resized() {
                pixels.resize_surface(size.width, size.height);
            }
        }
    });
}

fn create_window(
    title: &str,
    event_loop: &EventLoop<()>,
) -> (winit::window::Window, u32, u32, f64) {
    // Create a hidden window so we can estimate a good default window size
    let window = winit::window::WindowBuilder::new()
        .with_visible(false)
        .with_title(title)
        .build(event_loop)
        .unwrap();
    let hidpi_factor = window.scale_factor();

    // Get dimensions
    let width = SCREEN_WIDTH as f64;
    let height = SCREEN_HEIGHT as f64;
    let (monitor_width, monitor_height) = {
        if let Some(monitor) = window.current_monitor() {
            let size = monitor.size().to_logical(hidpi_factor);
            (size.width, size.height)
        } else {
            (width, height)
        }
    };
    let scale = (monitor_height / height * 2.0 / 3.0).round().max(1.0);

    // Resize, center, and display the window
    let min_size: winit::dpi::LogicalSize<f64> =
        PhysicalSize::new(width, height).to_logical(hidpi_factor);
    let default_size = LogicalSize::new(width * scale, height * scale);
    let center = LogicalPosition::new(
        (monitor_width - width * scale) / 2.0,
        (monitor_height - height * scale) / 2.0,
    );
    window.set_inner_size(default_size);
    window.set_min_inner_size(Some(min_size));
    window.set_outer_position(center);
    window.set_visible(true);

    let size = default_size.to_physical::<f64>(hidpi_factor);

    (
        window,
        size.width.round() as u32,
        size.height.round() as u32,
        hidpi_factor,
    )
}

fn vm_loop(vm: VM, screen_sender: SyncSender<Screen>) {
    let clocks_to_run = (4194304.0 / 1000.0 * 16f64).round() as u32;
    let mut clocks = 0;

    let mut vm = vm;

    loop {
        while clocks < clocks_to_run {
            clocks += vm.tick() as u32;

            if vm.check_vblank() {
                if let Err(TrySendError::Disconnected(..)) = screen_sender.try_send(vm.get_screen()) {
                    return;
                }
            }
        }

        clocks -= clocks_to_run;
    }
}
