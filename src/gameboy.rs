#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;
use std::sync::mpsc::{self, Receiver, SyncSender};

use goodboy_core::vm::{Screen, Vm};

#[cfg(not(target_arch = "wasm32"))]
use crate::io::IoEvent;

pub struct GameBoy {
    pub vm: Option<Vm>,
    pub screen_tx: SyncSender<Screen>,
    pub screen_rx: Option<Receiver<Screen>>,
}

impl GameBoy {
    pub fn new() -> Self {
        let (screen_tx, screen_rx) = mpsc::sync_channel(1);

        GameBoy {
            vm: None,
            screen_tx,
            screen_rx: Some(screen_rx),
        }
    }
    pub fn game_title(&self) -> Option<String> {
        self.vm
            .as_ref()
            .map_or(None, |vm| Some(vm.game_title().to_string()))
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_game(&mut self, game_data: &[u8]) {
        let new_vm = Vm::new(game_data);
        let _ = self.vm.insert(new_vm);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_game_file(&mut self, path: impl AsRef<Path>) -> std::io::Result<()> {
        let game_data = std::fs::read(path)?;
        self.load_game(&game_data);
        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn run(mut self, io_rx: Receiver<IoEvent>) -> Receiver<Screen> {
        let screen_rx = self.screen_rx.take().unwrap();
        crate::utils::spawn(self.run_loop(io_rx));
        screen_rx
    }

    #[cfg(not(target_arch = "wasm32"))]
    async fn run_loop(mut self, io_rx: Receiver<IoEvent>) {
        use std::time::Duration;

        let mut clocks = 0;

        let sleep_time = vec![16, 8, 4, 0];
        let mut time_cycle = sleep_time
            .into_iter()
            .map(Duration::from_millis)
            .cycle()
            .peekable();

        'vm: loop {
            let timer = wasm_timer::Delay::new(*time_cycle.peek().unwrap());
            let total_clocks = (4194304.0 / 1000.0 * 16f64).round() as u32;

            if let Some(vm) = self.vm.as_mut() {
                while clocks < total_clocks {
                    clocks += vm.tick();

                    if vm.check_vblank() {
                        if let Err(mpsc::TrySendError::Disconnected(..)) =
                            self.screen_tx.try_send(vm.get_screen())
                        {
                            break 'vm;
                        }
                    }
                }
                clocks -= total_clocks;
            }

            loop {
                match io_rx.try_recv() {
                    Ok(event) => match event {
                        IoEvent::ButtonPressed(button) => {
                            self.vm.as_mut().map(|vm| vm.press_button(button));
                        }
                        IoEvent::ButtonReleased(button) => {
                            self.vm.as_mut().map(|vm| vm.release_button(button));
                        }
                        IoEvent::InsertCartridge(cart) => {
                            let _ = self.vm.insert(Vm::from_cartridge(cart));
                            clocks = 0;
                            break;
                        }
                        // IoEvent::SetColorScheme(color_scheme) => vm.set_color_scheme(color_scheme),
                        IoEvent::SwitchSpeedNext => {
                            time_cycle.nth(0);
                        }
                        IoEvent::SwitchSpeedPrev => {
                            time_cycle.nth(2);
                        }
                        IoEvent::Exit => break 'vm,
                        // _ => {}
                    },
                    Err(mpsc::TryRecvError::Empty) => break,
                    Err(_) => break 'vm,
                }
            }

            timer.await.ok();
        }
    }
}
