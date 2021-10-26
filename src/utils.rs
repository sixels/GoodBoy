#![allow(dead_code)]

use std::{
    iter,
    sync::mpsc::{self, Receiver, TryRecvError},
    thread,
    time::{Duration, Instant},
};

use goodboy_core::vm::{Screen, SCREEN_HEIGHT, SCREEN_WIDTH};
use winit::{
    dpi::{LogicalPosition, LogicalSize, PhysicalSize},
    event_loop::EventLoop,
};

pub fn create_window(
    title: &str,
    event_loop: &EventLoop<()>,
) -> (winit::window::Window, u32, u32, f64) {
    // Create a hidden window so we can estimate a good default window size
    let window = winit::window::WindowBuilder::new()
        .with_visible(false)
        .with_title(title)
        .build(event_loop)
        .unwrap();

    let width = SCREEN_WIDTH as f64;
    let height = SCREEN_HEIGHT as f64;

    let hidpi_factor = window.scale_factor();

    // Get dimensions
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

    return (
        window,
        size.width.round() as u32,
        size.height.round() as u32,
        hidpi_factor,
    );
}

pub fn fps_counter_middleware(screen_receiver: Receiver<Screen>) -> Receiver<Screen> {
    let (new_screen_sender, new_screen_receiver) = mpsc::sync_channel(1);

    // FPS Counter middleware
    thread::spawn(move || {
        let mut start = Instant::now();
        let mut fps = 0u64;

        let mut overlay = fps_overlay(fps);

        loop {
            let now = Instant::now();
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
    let digits = vec![
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
    let mut overlay: Vec<u8> = iter::repeat(0u8).take(SCREEN_WIDTH * 8 * 4).collect();

    if fps >= 100 {
        return overlay;
    }

    let first_number = fps / 10;
    let second_number = fps % 10;

    for (row, bits) in digits[first_number as usize].iter().enumerate() {
        for (col, bit) in bits.iter().enumerate() {
            if *bit > 0u8 {
                overlay[(2 + col) * 4 + (2 + row) * SCREEN_WIDTH * 4 + 0] = 1;
                overlay[(2 + col) * 4 + (2 + row) * SCREEN_WIDTH * 4 + 1] = 255;
                overlay[(2 + col) * 4 + (2 + row) * SCREEN_WIDTH * 4 + 2] = 1;
                overlay[(2 + col) * 4 + (2 + row) * SCREEN_WIDTH * 4 + 3] = 255;
            }
        }
    }

    for (row, bits) in digits[second_number as usize].iter().enumerate() {
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
