use std::{rc::Rc, sync::mpsc};

use goodboy_core::vm::{SCREEN_HEIGHT as HEIGHT, SCREEN_WIDTH as WIDTH};
use pixels::{Pixels, SurfaceTexture};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

#[cfg(target_arch = "wasm32")]
use crate::io::IoEvent;
use crate::io::IoHandler;
use crate::utils::Fps;
use crate::GameBoy;

pub struct App {
    window: Rc<Window>,
    event_loop: EventLoop<()>,
    gameboy: GameBoy,
}

impl App {
    pub fn new(gameboy: GameBoy) -> Result<App, winit::error::OsError> {
        let event_loop = EventLoop::new();
        let window = {
            let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
            WindowBuilder::new()
                .with_title("GoodBoy")
                .with_inner_size(size)
                .with_min_inner_size(size)
                .build(&event_loop)
        }?;

        window.set_visible(true);

        Ok(Self {
            window: Rc::new(window),
            event_loop,
            gameboy,
        })
    }

    #[allow(unused_mut)]
    pub async fn run(mut self) {
        let (mut io_handler, io_rx) = IoHandler::new();
        let (title_sender, title_receiver) = mpsc::channel();

        #[cfg(target_arch = "wasm32")]
        crate::web::init(Rc::clone(&self.window), &io_handler, &title_sender);

        let mut pixels = {
            let window_size = self.window.inner_size();
            // let scale_factor = window.scale_factor();
            let surface_texture =
                SurfaceTexture::new(window_size.width, window_size.height, self.window.as_ref());

            let pixels = Pixels::new_async(WIDTH as u32, HEIGHT as u32, surface_texture)
                .await
                .expect("Pixels error");

            pixels
        };

        #[cfg(target_arch = "wasm32")]
        let (mut game_title, screen_rx) = (Option::<String>::None, self.gameboy.screen_rx.take().unwrap());
        #[cfg(not(target_arch = "wasm32"))]
        let (mut game_title, screen_rx) = (self.gameboy.game_title(), self.gameboy.run(io_rx));

        if let Some(ref title) = game_title {
            self.window.set_title(title);
        }

        let mut fps = Fps::default();

        #[cfg(target_arch = "wasm32")]
        let mut clocks = 0;
        #[cfg(target_arch = "wasm32")]
        let mut frame_start = wasm_timer::Instant::now();

        let Self {
            window,
            event_loop,
            #[cfg(target_arch = "wasm32")]
            mut gameboy,
            ..
        } = self;

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

                            if let Some(ref title) = game_title {
                                window.set_title(&format!("{title} - {fps}"));
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
                io_handler.handle_input(&title_sender);

                while let Ok(title) = title_receiver.try_recv() {
                    game_title = Some(title)
                }

                // Resize the window
                if let Some(size) = io_handler.window_resized() {
                    pixels.resize_surface(size.width, size.height);
                }

                window.request_redraw();
            }
        });
    }
}
