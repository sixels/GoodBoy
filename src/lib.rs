use std::{rc::Rc, sync::mpsc};

pub use goodboy_core::vm::{SCREEN_HEIGHT as HEIGHT, SCREEN_WIDTH as WIDTH};
use pixels::{Pixels, SurfaceTexture};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

pub mod gameboy;
mod io;
mod utils;

pub use crate::gameboy::GameBoy;
use crate::io::IoHandler;
use crate::utils::Fps;

#[cfg(target_arch = "wasm32")]
use crate::io::IoEvent;

pub async fn run(gameboy: GameBoy) {
    #[cfg(target_arch = "wasm32")]
    let mut gameboy = gameboy;

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
    let (mut io_handler, io_rx) = IoHandler::new();

    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsCast;
        use winit::platform::web::WindowExtWebSys;

        // Retrieve current width and height dimensions of browser client window
        let get_window_size = move || {
            let document = web_sys::window().and_then(|win| win.document()).unwrap();
            let screen = document.get_element_by_id("screen-container").unwrap();
            LogicalSize::new(screen.client_width() as f64, screen.client_height() as f64)
        };

        let window = Rc::clone(&window);
        // Initialize winit window with current dimensions of browser client
        window.set_inner_size(get_window_size());

        let client_window = web_sys::window().unwrap();

        // Attach winit canvas to body element
        let document = client_window.document().unwrap();
        document
            .get_element_by_id("screen")
            .and_then(|scr| {
                scr.append_child(&web_sys::Element::from(window.canvas()))
                    .ok()
            })
            .expect("couldn't append canvas to document body");

        // Listen for resize event on browser client. Adjust winit window dimensions
        // on event trigger
        {
            let window = Rc::clone(&window);
            let closure =
                wasm_bindgen::closure::Closure::wrap(Box::new(move |_e: web_sys::Event| {
                    let size = get_window_size();
                    window.set_inner_size(size)
                }) as Box<dyn FnMut(_)>);
            client_window
                .add_event_listener_with_callback("resize", closure.as_ref().unchecked_ref())
                .unwrap();
            closure.forget();
        }
        {
            use goodboy_core::io::JoypadButton;

            let press_button = |btn: JoypadButton| {
                let sender = io_handler.sender();
                return wasm_bindgen::closure::Closure::wrap(Box::new(move |ev: web_sys::Event| {
                    sender.send(IoEvent::ButtonPressed(btn)).ok();
                })
                    as Box<dyn FnMut(_)>);
            };

            let release_button = |btn: JoypadButton| {
                let sender = io_handler.sender();
                return wasm_bindgen::closure::Closure::wrap(Box::new(move |_: web_sys::Event| {
                    sender.send(IoEvent::ButtonReleased(btn)).ok();
                })
                    as Box<dyn FnMut(_)>);
            };

            macro_rules! bind_button {
                ($id:expr, $button:expr) => {{
                    let btn = document
                        .get_element_by_id($id)
                        .expect(&format!("Couldn't find #{:?}", $id));
                    let btn_press = press_button($button);
                    let btn_release = release_button($button);
                    btn.add_event_listener_with_callback(
                        "mousedown",
                        btn_press.as_ref().unchecked_ref(),
                    )
                    .unwrap();
                    btn.add_event_listener_with_callback(
                        "touchstart",
                        btn_press.as_ref().unchecked_ref(),
                    )
                    .unwrap();
                    btn.add_event_listener_with_callback(
                        "mouseup",
                        btn_release.as_ref().unchecked_ref(),
                    )
                    .unwrap();
                    btn.add_event_listener_with_callback(
                        "mouseout",
                        btn_release.as_ref().unchecked_ref(),
                    )
                    .unwrap();
                    btn.add_event_listener_with_callback(
                        "touchend",
                        btn_release.as_ref().unchecked_ref(),
                    )
                    .unwrap();
                    btn.add_event_listener_with_callback(
                        "touchcancel",
                        btn_release.as_ref().unchecked_ref(),
                    )
                    .unwrap();

                    btn_press.forget();
                    btn_release.forget();
                }};
            }

            {
                use goodboy_core::mmu::cartridge::Cartridge;

                let sender = io_handler.sender();
                let game_title = io_handler.game_title.clone();

                let cb = wasm_bindgen::closure::Closure::wrap(Box::new(move |_: web_sys::Event| {
                    let dialog = rfd::AsyncFileDialog::new()
                        .add_filter("ROM", &["gb", "gbc"])
                        .pick_file();

                    utils::spawn({
                        let sender = sender.clone();
                        let game_title = game_title.clone();

                        async move {
                            let file = dialog.await;

                            if let Some(file) = file {
                                log::info!("Loading file: {file:?}");
                                let buffer = file.read().await;

                                let cartridge = Cartridge::new(&buffer);

                                let _ = game_title.set_title(cartridge.rom_name());

                                if sender.send(IoEvent::InsertCartridge(cartridge)).is_err() {
                                    log::error!("Error sending the file buffer");
                                }
                            } else {
                                log::info!("No file selected");
                            }
                        }
                    });
                })
                    as Box<dyn FnMut(_)>);

                let btn = document.get_element_by_id("btn-start").unwrap();
                btn.add_event_listener_with_callback("mousedown", cb.as_ref().unchecked_ref())
                    .unwrap();
                cb.forget();
            }
            bind_button!("btn-a", JoypadButton::A);
            bind_button!("btn-b", JoypadButton::B);
            bind_button!("btn-up", JoypadButton::Up);
            bind_button!("btn-down", JoypadButton::Down);
            bind_button!("btn-left", JoypadButton::Left);
            bind_button!("btn-right", JoypadButton::Right);
        }
    }

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

    #[cfg(not(target_arch = "wasm32"))]
    let (game_title, screen_rx) = (gameboy.game_title(), gameboy.run(io_rx));
    #[cfg(target_arch = "wasm32")]
    let screen_rx = gameboy.screen_rx.take().unwrap();

    #[cfg(not(target_arch = "wasm32"))]
    io_handler
        .set_game_title(game_title.unwrap_or_default())
        .unwrap();

    let mut fps = Fps::default();

    #[cfg(target_arch = "wasm32")]
    let mut clocks = 0;
    #[cfg(target_arch = "wasm32")]
    let mut frame_start = wasm_timer::Instant::now();

    event_loop.run(move |ev, _, control_flow| {
        #[cfg(target_arch = "wasm32")]
        {
            use goodboy_core::vm::Vm;
            use std::time::Duration;
            use wasm_timer::Instant;

            let frame_now = Instant::now();
            let frame_next = frame_start + Duration::from_micros(2000);

            if frame_now >= frame_next {
                let total_clocks = (4194304.0 / 1000.0 * 16f64).round() as u32;

                if let Some(vm) = gameboy.vm.as_mut() {
                    while clocks < total_clocks {
                        clocks += vm.tick();

                        if vm.check_vblank() {
                            if let Err(mpsc::TrySendError::Disconnected(..)) =
                                gameboy.screen_tx.try_send(vm.get_screen())
                            {
                                *control_flow = ControlFlow::Exit;
                                return;
                            }
                        }
                    }
                    clocks -= total_clocks;
                }

                loop {
                    match io_rx.try_recv() {
                        Ok(event) => match event {
                            IoEvent::ButtonPressed(button) => {
                                gameboy.vm.as_mut().map(|vm| vm.press_button(button));
                            }
                            IoEvent::ButtonReleased(button) => {
                                gameboy.vm.as_mut().map(|vm| vm.release_button(button));
                            }
                            IoEvent::InsertCartridge(cart) => {
                                let _ = gameboy.vm.insert(Vm::from_cartridge(cart));
                                break;
                            }
                            // IoEvent::SetColorScheme(color_scheme) => vm.set_color_scheme(color_scheme),
                            // IoEvent::SwitchSpeedNext => {
                            //     time_cycle.nth(0);
                            // }
                            // IoEvent::SwitchSpeedPrev => {
                            //     time_cycle.nth(2);
                            // }
                            IoEvent::Exit => {
                                *control_flow = ControlFlow::Exit;
                                return;
                            }
                            _ => {}
                        },
                        Err(mpsc::TryRecvError::Empty) => break,
                        Err(_) => {
                            *control_flow = ControlFlow::Exit;
                            return;
                        }
                    }
                }

                frame_start = Instant::now();
            }
        } // cfg wasm32

        match &ev {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                log::info!("Exit event received");
                *control_flow = ControlFlow::Exit;
                return;
            }
            Event::RedrawRequested(..) => {
                let frame = pixels.get_frame();
                fps.update();

                match screen_rx.try_recv() {
                    Ok(screen) => {
                        frame.copy_from_slice(screen.as_slice());

                        let title = io_handler.get_game_title().unwrap().clone();
                        if title.len() > 0 {
                            window.set_title(&format!("{} - {} FPS", title, fps.current_rate()));
                        } else {
                            window.set_title(&format!("GoodBoy"));
                        }
                    }
                    Err(mpsc::TryRecvError::Disconnected) => {
                        log::warn!("Screen channel was dropped");
                        *control_flow = ControlFlow::Exit;
                        return;
                    }
                    _ => {}
                }

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

        if io_handler.update(&ev) {
            io_handler.handle_input();

            // Resize the window
            if let Some(size) = io_handler.window_resized() {
                pixels.resize_surface(size.width, size.height);
            }

            window.request_redraw();
        }
    });
}

#[cfg(target_arch = "wasm32")]
pub mod wasm {
    use futures_executor as executor;
    use futures_util::task::LocalSpawnExt;
    use wasm_bindgen::prelude::*;

    use crate::GameBoy;

    #[wasm_bindgen]
    pub fn start() {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init().ok();

        let mut pool = executor::LocalPool::new();
        let spawner = pool.spawner();

        spawner.spawn_local(crate::run(GameBoy::new())).ok();
        pool.run();
    }
}
