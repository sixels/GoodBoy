use std::{rc::Rc, sync::mpsc};

use gameboy::GameBoy;
pub use goodboy_core::vm::{SCREEN_HEIGHT as HEIGHT, SCREEN_WIDTH as WIDTH};
use pixels::{Pixels, SurfaceTexture};
use winit::{
    dpi::LogicalSize,
    event::Event,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{self, prelude::*};
#[cfg(target_arch = "wasm32")]
use winit::platform::web::WindowExtWebSys;

mod framework;
mod gameboy;
mod io;
mod utils;

use framework::Framework;
use winit_input_helper::WinitInputHelper;

pub async fn run() {
    let event_loop = EventLoop::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("GoodBoy")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .expect("WindowBuilder error")
    };

    let window = Rc::new(window);

    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsCast;
        use winit::platform::web::WindowExtWebSys;

        // Retrieve current width and height dimensions of browser client window
        let get_window_size = || {
            let client_window = web_sys::window().unwrap();
            LogicalSize::new(
                client_window.inner_width().unwrap().as_f64().unwrap(),
                client_window.inner_height().unwrap().as_f64().unwrap(),
            )
        };

        let window = Rc::clone(&window);

        // Initialize winit window with current dimensions of browser client
        window.set_inner_size(get_window_size());

        let client_window = web_sys::window().unwrap();

        // Attach winit canvas to body element
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| doc.get_element_by_id("screen"))
            .and_then(|scr| {
                scr.append_child(&web_sys::Element::from(window.canvas()))
                    .ok()
            })
            .expect("couldn't append canvas to document body");

        // Listen for resize event on browser client. Adjust winit window dimensions
        // on event trigger
        let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move |_e: web_sys::Event| {
            let size = get_window_size();
            window.set_inner_size(size)
        }) as Box<dyn FnMut(_)>);
        client_window
            .add_event_listener_with_callback("resize", closure.as_ref().unchecked_ref())
            .unwrap();
        closure.forget();
    }

    let mut input = WinitInputHelper::new();
    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture =
            SurfaceTexture::new(window_size.width, window_size.height, window.as_ref());
        let pixels = Pixels::new_async(WIDTH as u32, HEIGHT as u32, surface_texture)
            .await
            .expect("Pixels error");

        // let framework = Framework::new(
        //     WIDTH as u32,
        //     HEIGHT as u32,
        //     scale_factor as f32,
        //     &pixels,
        //     gameboy.io_chan.0.clone(),
        // );

        pixels
    };
    let mut gameboy = GameBoy::new();
    let mut framework = {
        let scale_factor = window.scale_factor();
        Framework::new(
            WIDTH as u32,
            HEIGHT as u32,
            scale_factor as f32,
            &pixels,
            gameboy.io_chan.0.clone(),
        )
    };
    gameboy.prepare();

    event_loop.run(move |ev, _, control_flow| {
        #[cfg(target_arch = "wasm32")]
        {
            let screen_sender = gameboy.screen_chan.0.clone();
            let io_receiver = gameboy.io_chan.1.as_ref().unwrap();

            gameboy::update_vm(&mut gameboy.vm, screen_sender, io_receiver, 0, None).ok();
        }

        match &ev {
            Event::WindowEvent { event, .. } => {
                framework.handle_event(&event);
            }
            Event::RedrawRequested(..) | Event::MainEventsCleared => {
                let frame = pixels.get_frame();

                match gameboy.screen_chan.1.try_recv() {
                    Ok(screen) => frame.copy_from_slice(screen.as_slice()),
                    Err(mpsc::TryRecvError::Disconnected) => *control_flow = ControlFlow::Exit,
                    _ => {}
                }

                framework.prepare(&window);

                if let Err(_) = pixels.render_with(|encoder, render_target, context| {
                    // Render the world texture
                    context.scaling_renderer.render(encoder, render_target);

                    // Render egui
                    framework.render(encoder, render_target, context)?;

                    Ok(())
                }) {
                    *control_flow = ControlFlow::Exit;
                    return;
                }
            }
            _ => {}
        };

        if input.update(&ev) {
            io::handle_input(&mut input, gameboy.io_chan.0.clone());

            // Update the scale factor
            if let Some(scale_factor) = input.scale_factor() {
                framework.scale_factor(scale_factor);
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                pixels.resize_surface(size.width, size.height);
                framework.resize(size.width, size.height);
            }

            window.request_redraw();
        }
    });
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn start() {
    console_log::init().ok();

    wasm_bindgen_futures::spawn_local(run());
}
