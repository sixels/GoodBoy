use std::{
    sync::mpsc::{self, TryRecvError, TrySendError},
    thread,
    time::Duration,
};

use sixels_gb::vm::{SCREEN_HEIGHT, SCREEN_WIDTH, VM};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::web::WindowExtWebSys,
    window::Window,
};
use winit_input_helper::WinitInputHelper;

use super::{common, ColorSchemeIter, IoEvent};

pub async fn run(window: Window, event_loop: EventLoop<()>) {
    use wasm_bindgen::JsCast;

    let mut input = WinitInputHelper::new();

    let canvas = window.canvas();
    web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.body())
        .and_then(|body| body.append_child(&canvas).ok())
        .expect("Could not create the canvas");

    let context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();

    log::info!("canvas context created");

    let mut vm = VM::new_with_buffer(include_bytes!("../../../../assets/roms/zelda.gb"));

    let (screen_sender, screen_receiver) = mpsc::sync_channel(1);
    let (io_sender, io_receiver) = mpsc::channel();

    // let screen_receiver = fps_counter_middleware(screen_receiver);

    let mut color_schemes_iter: ColorSchemeIter = box super::COLOR_SCHEMES.iter().copied().cycle();
    let mut clocks = 0;

    context.clear_rect(0.0, 0.0, SCREEN_WIDTH as _, SCREEN_HEIGHT as _);

    log::info!("Starting the event loop");
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

        // VM loop
        {
            let clocks_to_run = (4194304.0 / 1000.0f64).round() as u32;

            // let timer = speed_limit(Duration::from_millis(11));
            let mut _respect_timer = true;

            while clocks < clocks_to_run {
                clocks += vm.tick() as u32;

                if vm.check_vblank() {
                    if let Err(TrySendError::Disconnected(..)) =
                        screen_sender.try_send(vm.get_screen())
                    {
                        break;
                    }
                }
            }
            clocks -= clocks_to_run;

            loop {
                match io_receiver.try_recv() {
                    Ok(event) => match event {
                        IoEvent::ButtonPressed(button) => vm.press_button(button),
                        IoEvent::ButtonReleased(button) => vm.release_button(button),
                        IoEvent::SetColorScheme(color_scheme) => vm.set_color_scheme(color_scheme),
                        IoEvent::ToggleFPSLimit => _respect_timer ^= true,

                        IoEvent::Exit => break,
                    },
                    Err(TryRecvError::Empty) => break,
                    Err(_) => break,
                }
            }
        }

        match screen_receiver.try_recv() {
            Ok(data) => {
                use wasm_bindgen::Clamped;

                let image = web_sys::ImageData::new_with_u8_clamped_array_and_sh(
                    Clamped(data.as_ref()),
                    SCREEN_WIDTH as _,
                    SCREEN_HEIGHT as _,
                )
                .expect("could not create an image from the game boy screen");

                context
                    .put_image_data(&image, 0.0, 0.0)
                    .expect("could not render the screen")
            }
            Err(TryRecvError::Empty) => (),
            Err(_) => {
                *control_flow = ControlFlow::Exit;
            }
        }

        if *control_flow != ControlFlow::Exit && input.update(&event) {
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
