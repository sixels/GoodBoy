use std::{sync::mpsc, thread, time::Duration};

use eframe::{egui, epi};

use goodboy_core::{
    io::JoypadButton,
    vm::{Screen, SCREEN_HEIGHT, SCREEN_WIDTH, VM},
};

use crate::utils::{self, Fps};

#[allow(dead_code)]
pub enum IoEvent {
    ButtonPressed(JoypadButton),
    ButtonReleased(JoypadButton),
    // SetColorScheme(ColorScheme),
    ToggleFPSLimit,
    Exit,
}

pub struct App {
    screen_chan: (mpsc::SyncSender<Screen>, mpsc::Receiver<Screen>),
    io_chan: (mpsc::Sender<IoEvent>, Option<mpsc::Receiver<IoEvent>>),

    display: Option<egui::TextureId>,
    fps: Fps,

    vm: Option<VM>,
    #[cfg(not(target_arch = "wasm32"))]
    vm_loop_handle: Option<thread::JoinHandle<()>>,
}

impl App {
    pub fn new() -> Self {
        let screen_chan = mpsc::sync_channel(1);
        let io_chan = {
            let chan = mpsc::channel();
            (chan.0, Some(chan.1))
        };

        Self {
            screen_chan,
            io_chan,
            display: None,
            fps: Default::default(),

            vm: None,
            #[cfg(not(target_arch = "wasm32"))]
            vm_loop_handle: None,
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
        self.vm = Some(VM::new_with_buffer(include_bytes!(
            "../assets/roms/zelda.gb"
        )));

        #[cfg(not(target_arch = "wasm32"))]
        {
            let screen_sender = self.screen_chan.0.clone();
            let io_receiver = self.io_chan.1.take().unwrap();

            let vm = self.vm.take();

            self.vm_loop_handle = Some(thread::spawn(move || {
                vm_loop(vm.unwrap(), screen_sender, io_receiver);
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
            let io = self.io_chan.1.as_ref().unwrap();

            let mut clocks = 0;
            let clocks_to_run = (4194304.0 / 1000.0 * 16f64).round() as u32;

            //     // let timer = speed_limit(Duration::from_millis(15));
            //     // let mut respect_timer = true;

            let vm = self.vm.as_mut().unwrap();
            while clocks < clocks_to_run {
                clocks += vm.tick() as u32;

                if vm.check_vblank() {
                    if let Err(mpsc::TrySendError::Disconnected(..)) =
                        screen_sender.try_send(vm.get_screen())
                    {
                        // break 'vm_loop;
                        break;
                    }
                }
            }

            loop {
                match io.try_recv() {
                    Ok(event) => match event {
                        IoEvent::ButtonPressed(button) => vm.press_button(button),
                        IoEvent::ButtonReleased(button) => vm.release_button(button),
                        // IoEvent::SetColorScheme(color_scheme) => vm.set_color_scheme(color_scheme),
                        // IoEvent::ToggleFPSLimit => respect_timer ^= true,
                        _ => (),
                    },
                    // Err(mpsc::TryRecvError::Empty) => break,
                    Err(_) => break,
                }
            }
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Load ROM File").clicked() {
                        println!("Load ROM")
                    }
                    if ui.button("Load ROM File").clicked() {
                        println!("Load ROM")
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        ui.separator();
                        if ui.button("Exit").clicked() {
                            frame.quit();
                        }
                    }
                });
                ui.menu_button("Save", |ui| {
                    if ui.button("Quick Save").clicked() {
                        ();
                    }
                    if ui.button("Quick Load").clicked() {
                        ();
                    }
                    ui.menu_button("Select Slot", |ui| {
                        for i in 0..10 {
                            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                                if ui.button(format!("Slot {i}")).clicked() {
                                    ();
                                }
                            });
                        }
                    });
                })
            });
        });

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

            utils::handle_input(ui.input(), self.io_chan.0.clone());
        });
        ctx.request_repaint();
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn vm_loop(mut vm: VM, screen_sender: mpsc::SyncSender<Screen>, io: mpsc::Receiver<IoEvent>) {
    let mut clocks = 0;
    let clocks_to_run = (4194304.0 / 1000.0 * 16f64).round() as u32;

    let timer = speed_limit(Duration::from_millis(15));
    let mut respect_timer = true;

    'vm_loop: loop {
        while clocks < clocks_to_run {
            clocks += vm.tick() as u32;

            if vm.check_vblank() {
                if let Err(mpsc::TrySendError::Disconnected(..)) =
                    screen_sender.try_send(vm.get_screen())
                {
                    break 'vm_loop;
                }
            }
        }

        loop {
            match io.try_recv() {
                Ok(event) => match event {
                    IoEvent::ButtonPressed(button) => vm.press_button(button),
                    IoEvent::ButtonReleased(button) => vm.release_button(button),
                    // IoEvent::SetColorScheme(color_scheme) => vm.set_color_scheme(color_scheme),
                    IoEvent::ToggleFPSLimit => respect_timer ^= true,
                    IoEvent::Exit => break 'vm_loop,
                },
                Err(mpsc::TryRecvError::Empty) => break,
                Err(_) => break 'vm_loop,
            }
        }

        clocks -= clocks_to_run;

        if respect_timer && timer.recv().is_err() {
            break;
        }
    }
}

fn speed_limit(wait_time: Duration) -> mpsc::Receiver<()> {
    let (time_sender, time_receiver) = mpsc::sync_channel(1);
    thread::spawn(move || loop {
        thread::sleep(wait_time);
        if time_sender.send(()).is_err() {
            break;
        };
    });

    time_receiver
}
