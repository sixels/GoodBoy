use std::{rc::Rc, sync::mpsc};

pub use goodboy_core::vm::{SCREEN_HEIGHT as HEIGHT, SCREEN_WIDTH as WIDTH};
use pixels::{Pixels, SurfaceTexture};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;

// mod framework;
pub mod gameboy;
mod io;
mod utils;

pub use gameboy::GameBoy;
// use framework::Framework;

pub async fn run(mut gameboy: GameBoy) {
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
        // let scale_factor = window.scale_factor();
        let surface_texture =
            SurfaceTexture::new(window_size.width, window_size.height, window.as_ref());

        let pixels = Pixels::new_async(WIDTH as u32, HEIGHT as u32, surface_texture)
            .await
            .expect("Pixels error");

        pixels
    };

    gameboy.prepare();

    event_loop.run(move |ev, _, control_flow| {
        #[cfg(target_arch = "wasm32")]
        {
            let screen_sender = gameboy.screen_chan.0.clone();
            let io_receiver = gameboy.io_chan.1.as_ref().unwrap();

            gameboy::update_vm(&mut gameboy.vm, screen_sender, io_receiver, 0, None).ok();
        }

        let GameBoy {
            io_chan: (io_tx, _),
            screen_chan: (_, screen_rx),
            ..
        } = &gameboy;

        match &ev {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                log::info!("Exit event received");
                *control_flow = ControlFlow::Exit;
                return;
            }
            // Event::WindowEvent { event, .. } => {
            //     framework.handle_event(&event);
            // }
            Event::RedrawRequested(..) => {
                let frame = pixels.get_frame();

                match screen_rx.try_recv() {
                    Ok(screen) => frame.copy_from_slice(screen.as_slice()),
                    Err(mpsc::TryRecvError::Disconnected) => {
                        log::warn!("Screen channel was dropped");
                        *control_flow = ControlFlow::Exit;
                        return;
                    }
                    _ => {}
                }

                // framework.prepare(&window);

                // if let Err(_) = pixels.render_with(|encoder, render_target, context| {
                //     // Render game frame
                //     context.scaling_renderer.render(encoder, render_target);

                //     // Render egui
                //     // framework.render(encoder, render_target, context)?;

                //     Ok(())
                // })
                if let Err(e) = pixels.render() {
                    log::error!("Pixel render failed: {:?}", e);
                    *control_flow = ControlFlow::Exit;
                    return;
                }
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => {}
        };

        if input.update(&ev) {
            io::handle_input(input.clone(), io_tx.clone());

            // Update the scale factor
            // if let Some(scale_factor) = input.scale_factor() {
            // framework.scale_factor(scale_factor);
            // }

            // Resize the window
            if let Some(size) = input.window_resized() {
                pixels.resize_surface(size.width, size.height);
                // framework.resize(size.width, size.height);
                window.request_redraw();
            }
        }
    });
}

#[cfg(target_arch = "wasm32")]
pub mod wasm {
    use wasm_bindgen::prelude::*;

    use futures::executor;
    use futures::task::LocalSpawnExt;

    use crate::GameBoy;

    #[wasm_bindgen]
    pub fn start() {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init().ok();

        let mut pool = executor::LocalPool::new();
        let spawner = pool.spawner();

        let mut gameboy = GameBoy::new();

        spawner.spawn_local(crate::run(gameboy)).ok();
        pool.run();
    }
}
