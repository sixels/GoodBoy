#![feature(stmt_expr_attributes)]
#![feature(box_syntax)]
#![feature(try_blocks)]

use std::{
    sync::mpsc::{Receiver, SyncSender, TryRecvError, TrySendError},
    thread,
    time::Duration,
};

use pixels::{PixelsBuilder, SurfaceTexture};
use sixels_gb::{
    io::JoypadButton,
    ppu::ColorScheme,
    vm::{Screen, SCREEN_HEIGHT, SCREEN_WIDTH, VM},
};
use winit::{
    dpi::{LogicalPosition, LogicalSize, PhysicalSize},
    event::{Event, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};
use winit_input_helper::WinitInputHelper;

enum IoEvent {
    ButtonPressed(JoypadButton),
    ButtonReleased(JoypadButton),
    SetColorScheme(ColorScheme),
    ToggleFPSLimit,
    Exit,
}

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
        let surface_texture = SurfaceTexture::new(p_width, p_height, &window);
        PixelsBuilder::new(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32, surface_texture)
            .enable_vsync(true)
            .build()
            .unwrap()
    };

    let vm;
    // if let Some(bios_path) = bios_path {
    //     println!("Loading with BIOS");
    //     vm = VM::new_with_bios(bios_path, rom_path);
    // } else {
    println!("Loading without BIOS");
    vm = VM::new(rom_path);
    // }
    let vm = vm.unwrap();
    let (screen_sender, screen_receiver) = std::sync::mpsc::sync_channel(1);
    let (io_sender, io_receiver) = std::sync::mpsc::channel();

    let screen_receiver = fps_counter_middleware(screen_receiver);

    thread::spawn(move || {
        vm_loop(vm, screen_sender, io_receiver);
    });

    let color_schemes = [
        ColorScheme::GRAY,
        ColorScheme::BLUE_ALT,
        ColorScheme::GREEN,
        ColorScheme::BLUE,
        ColorScheme::RED,
    ]
    .to_vec();

    let mut color_schemes_iter: Box<dyn Iterator<Item = ColorScheme>> =
        box color_schemes.clone().into_iter().cycle();

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

        // IO handlers
        if *control_flow != ControlFlow::Exit {
            if input.update(&event) {
                // Resize the window
                if let Some(size) = input.window_resized() {
                    pixels.resize_surface(size.width, size.height);
                }

                #[rustfmt::skip]
                let result: Result<(), Box<dyn std::error::Error>> = try {
                    if input.held_control() && input.key_pressed(VirtualKeyCode::Q) {
                        *control_flow = ControlFlow::Exit;
                    }

                    if input.key_pressed(VirtualKeyCode::Tab) { io_sender.send(IoEvent::SetColorScheme(color_schemes_iter.next().unwrap()))?; }
                    if input.held_shift() && input.key_pressed(VirtualKeyCode::Tab) {
                        for _ in 3..color_schemes.len() {
                            color_schemes_iter.next().unwrap();
                        }
                        io_sender.send(IoEvent::SetColorScheme(color_schemes_iter.next().unwrap()))?;
                    }

                    if input.key_pressed(VirtualKeyCode::F1) { io_sender.send(IoEvent::ToggleFPSLimit)?; }

                    if input.key_pressed(VirtualKeyCode::Right)  { io_sender.send(IoEvent::ButtonPressed(JoypadButton::Right))?;  }
                    if input.key_pressed(VirtualKeyCode::Left)   { io_sender.send(IoEvent::ButtonPressed(JoypadButton::Left))?;   }
                    if input.key_pressed(VirtualKeyCode::Up)     { io_sender.send(IoEvent::ButtonPressed(JoypadButton::Up))?;     }
                    if input.key_pressed(VirtualKeyCode::Down)   { io_sender.send(IoEvent::ButtonPressed(JoypadButton::Down))?;   }
                    if input.key_pressed(VirtualKeyCode::Z)      { io_sender.send(IoEvent::ButtonPressed(JoypadButton::A))?;      }
                    if input.key_pressed(VirtualKeyCode::X)      { io_sender.send(IoEvent::ButtonPressed(JoypadButton::B))?;      }
                    if input.key_pressed(VirtualKeyCode::Space)  { io_sender.send(IoEvent::ButtonPressed(JoypadButton::Select))?; }
                    if input.key_pressed(VirtualKeyCode::Return) { io_sender.send(IoEvent::ButtonPressed(JoypadButton::Start))?;  }

                    if input.key_released(VirtualKeyCode::Right)  { io_sender.send(IoEvent::ButtonReleased(JoypadButton::Right))?;  }
                    if input.key_released(VirtualKeyCode::Left)   { io_sender.send(IoEvent::ButtonReleased(JoypadButton::Left))?;   }
                    if input.key_released(VirtualKeyCode::Up)     { io_sender.send(IoEvent::ButtonReleased(JoypadButton::Up))?;     }
                    if input.key_released(VirtualKeyCode::Down)   { io_sender.send(IoEvent::ButtonReleased(JoypadButton::Down))?;   }
                    if input.key_released(VirtualKeyCode::Z)      { io_sender.send(IoEvent::ButtonReleased(JoypadButton::A))?;      }
                    if input.key_released(VirtualKeyCode::X)      { io_sender.send(IoEvent::ButtonReleased(JoypadButton::B))?;      }
                    if input.key_released(VirtualKeyCode::Space)  { io_sender.send(IoEvent::ButtonReleased(JoypadButton::Select))?; }
                    if input.key_released(VirtualKeyCode::Return) { io_sender.send(IoEvent::ButtonReleased(JoypadButton::Start))?;  }
                };

                if result.is_err() {
                    *control_flow = ControlFlow::Exit;
                }
            }
        }

        // Drop the vm before exit
        if *control_flow == ControlFlow::Exit {
            // drop(screen);
            io_sender.send(IoEvent::Exit).ok();
            thread::sleep(Duration::from_millis(100));
        }
    });
}

fn vm_loop(mut vm: VM, screen_sender: SyncSender<Screen>, io: Receiver<IoEvent>) {
    let mut clocks = 0;
    let clocks_to_run = (4194304.0 / 1000.0 * 12f64).round() as u32;

    let timer = speed_limit(Duration::from_millis(11));
    let mut respect_timer = true;

    'vm_loop: loop {
        while clocks < clocks_to_run {
            clocks += vm.tick() as u32;

            if vm.check_vblank() {
                if let Err(TrySendError::Disconnected(..)) = screen_sender.try_send(vm.get_screen())
                {
                    break;
                }
            }
        }

        loop {
            match io.try_recv() {
                Ok(event) => match event {
                    IoEvent::ButtonPressed(button) => vm.press_button(button),
                    IoEvent::ButtonReleased(button) => vm.release_button(button),
                    IoEvent::SetColorScheme(color_scheme) => vm.set_color_scheme(color_scheme),
                    IoEvent::ToggleFPSLimit => respect_timer ^= true,

                    IoEvent::Exit => break 'vm_loop,
                },
                Err(TryRecvError::Empty) => break,
                Err(_) => break 'vm_loop,
            }
        }

        clocks -= clocks_to_run;

        if respect_timer {
            if timer.recv().is_err() {
                break;
            }
        }
    }
}

fn speed_limit(wait_time: Duration) -> Receiver<()> {
    let (time_sender, time_receiver) = std::sync::mpsc::sync_channel(1);
    std::thread::spawn(move || loop {
        std::thread::sleep(wait_time);
        if time_sender.send(()).is_err() {
            break;
        };
    });

    time_receiver
}

fn fps_counter_middleware(screen_receiver: Receiver<Screen>) -> Receiver<Screen> {
    let (new_screen_sender, new_screen_receiver) = std::sync::mpsc::sync_channel(1);

    // FPS Counter middleware
    thread::spawn(move || {
        let mut start = std::time::Instant::now();
        let mut fps = 0u64;

        let mut overlay = fps_overlay(fps);

        loop {
            let now = std::time::Instant::now();
            if now > start + Duration::from_secs(1) {
                overlay = fps_overlay(fps);
                start = now;
                fps = 0;
            }

            match screen_receiver.try_recv() {
                Ok(mut screen) => {
                    for (s, o) in screen.iter_mut().zip(&overlay) {
                        if *o > 0 {
                            *s = *o
                        }
                    }
                    new_screen_sender.try_send(screen).ok();
                    fps += 1;
                }
                Err(TryRecvError::Empty) => (),
                _ => break,
            };
        }
    });

    new_screen_receiver
}

fn fps_overlay(fps: u64) -> Vec<u8> {
    let numbers = vec![
        // 0
        [
            vec![1, 1, 1],
            vec![1, 0, 1],
            vec![1, 0, 1],
            vec![1, 0, 1],
            vec![1, 0, 1],
            vec![1, 1, 1],
        ],
        // 1
        [
            vec![0, 1, 0],
            vec![1, 1, 0],
            vec![0, 1, 0],
            vec![0, 1, 0],
            vec![0, 1, 0],
            vec![1, 1, 1],
        ],
        // 2
        [
            vec![1, 1, 1],
            vec![0, 0, 1],
            vec![1, 1, 1],
            vec![1, 0, 0],
            vec![1, 0, 0],
            vec![1, 1, 1],
        ],
        // 3
        [
            vec![1, 1, 1],
            vec![0, 0, 1],
            vec![1, 1, 1],
            vec![0, 0, 1],
            vec![0, 0, 1],
            vec![1, 1, 1],
        ],
        // 4
        [
            vec![1, 0, 1],
            vec![1, 0, 1],
            vec![1, 1, 1],
            vec![0, 0, 1],
            vec![0, 0, 1],
            vec![0, 0, 1],
        ],
        // 5
        [
            vec![1, 1, 1],
            vec![1, 0, 0],
            vec![1, 1, 1],
            vec![0, 0, 1],
            vec![0, 0, 1],
            vec![1, 1, 1],
        ],
        // 6
        [
            vec![1, 1, 1],
            vec![1, 0, 0],
            vec![1, 1, 1],
            vec![1, 0, 1],
            vec![1, 0, 1],
            vec![1, 1, 1],
        ],
        // 7
        [
            vec![1, 1, 1],
            vec![0, 0, 1],
            vec![0, 0, 1],
            vec![0, 0, 1],
            vec![0, 0, 1],
            vec![0, 0, 1],
        ],
        // 8
        [
            vec![1, 1, 1],
            vec![1, 0, 1],
            vec![1, 1, 1],
            vec![1, 0, 1],
            vec![1, 0, 1],
            vec![1, 1, 1],
        ],
        // 9
        [
            vec![1, 1, 1],
            vec![1, 0, 1],
            vec![1, 1, 1],
            vec![0, 0, 1],
            vec![0, 0, 1],
            vec![1, 1, 1],
        ],
    ];

    // 3x6 number + 6 gap + 3x6 number
    let mut overlay: Vec<u8> = std::iter::repeat(0u8).take(SCREEN_WIDTH * 8 * 4).collect();

    if fps > 100 {
        return overlay;
    }

    let first_number = fps / 10;
    let second_number = fps % 10;

    for (row, bits) in numbers[first_number as usize].iter().enumerate() {
        for (col, bit) in bits.iter().enumerate() {
            if *bit > 0u8 {
                overlay[(2 + col) * 4 + (2 + row) * SCREEN_WIDTH * 4 + 0] = 1;
                overlay[(2 + col) * 4 + (2 + row) * SCREEN_WIDTH * 4 + 1] = 255;
                overlay[(2 + col) * 4 + (2 + row) * SCREEN_WIDTH * 4 + 2] = 1;
                overlay[(2 + col) * 4 + (2 + row) * SCREEN_WIDTH * 4 + 3] = 255;
            }
        }
    }

    for (row, bits) in numbers[second_number as usize].iter().enumerate() {
        for (col, bit) in bits.iter().enumerate() {
            if *bit > 0u8 {
                overlay[(8 + col) * 4 + (2 + row) * SCREEN_WIDTH * 4 + 0] = 1;
                overlay[(8 + col) * 4 + (2 + row) * SCREEN_WIDTH * 4 + 1] = 255;
                overlay[(8 + col) * 4 + (2 + row) * SCREEN_WIDTH * 4 + 2] = 1;
                overlay[(8 + col) * 4 + (2 + row) * SCREEN_WIDTH * 4 + 3] = 255;
            }
        }
    }

    overlay
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
