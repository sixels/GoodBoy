use std::sync::mpsc;
#[cfg(not(target_arch = "wasm32"))]
use std::thread;

use eframe::{egui, epi};

use goodboy_core::vm::{Screen, SCREEN_HEIGHT, SCREEN_WIDTH, VM};

mod vm;
mod widgets;

use crate::io::{self, IoEvent};
use crate::utils::Fps;

#[cfg(target_arch = "wasm32")]
use vm::update_vm;
#[cfg(not(target_arch = "wasm32"))]
use vm::vm_loop;

#[cfg(target_arch = "wasm32")]
use eframe::wasm_bindgen::{self, prelude::*};

use self::widgets::MenuBar;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

pub struct App {
    screen_chan: (mpsc::SyncSender<Screen>, mpsc::Receiver<Screen>),
    io_chan: (mpsc::Sender<IoEvent>, Option<mpsc::Receiver<IoEvent>>),

    display: Option<egui::TextureId>,
    fps: Fps,

    vm: Option<VM>,
    #[cfg(not(target_arch = "wasm32"))]
    vm_loop_handle: Option<thread::JoinHandle<()>>,

    menu_bar: MenuBar,
}

impl App {
    pub fn new() -> Self {
        let screen_chan = mpsc::sync_channel(1);
        let io_chan = {
            let chan = mpsc::channel();
            (chan.0, Some(chan.1))
        };

        let menu_bar = MenuBar::new(io_chan.0.clone());

        Self {
            screen_chan,
            io_chan,
            display: None,
            fps: Default::default(),

            vm: None,
            #[cfg(not(target_arch = "wasm32"))]
            vm_loop_handle: None,

            menu_bar,
        }
    }
}

impl epi::App for App {
    fn name(&self) -> &str {
        "Good Boy 🐶"
    }

    fn setup(
        &mut self,
        _ctx: &egui::CtxRef,
        _frame: &epi::Frame,
        _storage: Option<&dyn epi::Storage>,
    ) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let screen_sender = self.screen_chan.0.clone();
            let io_receiver = self.io_chan.1.take().unwrap();

            let vm = self.vm.take();

            self.vm_loop_handle = Some(thread::spawn(move || {
                let screen_sender_clone = screen_sender.clone();
                thread::spawn(move || vm_loop(vm, screen_sender_clone, io_receiver));
            }));
        }
    }

    fn on_exit(&mut self) {
        self.io_chan.0.send(IoEvent::Exit).ok();

        #[cfg(not(target_arch = "wasm32"))]
        self.vm_loop_handle.take().map(thread::JoinHandle::join);
    }

    fn update(&mut self, ctx: &egui::CtxRef, frame: &epi::Frame) {
        #[cfg(target_arch = "wasm32")]
        {
            let screen_sender = self.screen_chan.0.clone();
            let io_receiver = self.io_chan.1.as_ref().unwrap();

            update_vm(&mut self.vm, screen_sender, io_receiver, 0, None).ok();
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| self.menu_bar.render(ui, frame));

        egui::CentralPanel::default().show(ctx, |ui| {
            match self.screen_chan.1.try_recv() {
                Ok(screen) => {
                    self.fps.update();

                    let image = epi::Image::from_rgba_unmultiplied(
                        [SCREEN_WIDTH as _, SCREEN_HEIGHT as _],
                        screen.as_ref(),
                    );

                    self.display
                        .replace(frame.alloc_texture(image))
                        .map(|id| frame.free_texture(id));
                }
                Err(mpsc::TryRecvError::Disconnected) => frame.quit(),
                _ => {}
            }

            let fps = self.fps.counter();
            ui.label(format!("FPS: {fps}"));

            self.display.map(|display| {
                ui.with_layout(
                    egui::Layout::centered_and_justified(egui::Direction::TopDown),
                    |ui| {
                        ui.image(display, [(SCREEN_WIDTH * 3) as _, (SCREEN_HEIGHT * 3) as _]);
                    },
                );
            });

            io::handle_input(ui.input(), self.io_chan.0.clone());
        });
        ctx.request_repaint();
    }
}
