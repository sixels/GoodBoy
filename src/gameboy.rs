use std::sync::mpsc;

use goodboy_core::vm::{Screen, Vm};

#[cfg(not(target_arch = "wasm32"))]
mod vm;

#[cfg(not(target_arch = "wasm32"))]
use crate::gameboy::vm::vm_loop;
use crate::io::IoEvent;
#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;

pub struct GameBoy {
    vm: Option<Vm>,
    #[cfg(target_arch = "wasm32")]
    screen_tx: Option<mpsc::SyncSender<Screen>>,
    #[cfg(target_arch = "wasm32")]
    io_rx: Option<mpsc::Receiver<IoEvent>>,
}

impl GameBoy {
    pub fn new() -> Self {
        GameBoy {
            vm: None,
            #[cfg(target_arch = "wasm32")]
            screen_tx: None,
            #[cfg(target_arch = "wasm32")]
            io_rx: None,
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn setup(
        mut self,
        io_rx: mpsc::Receiver<IoEvent>,
    ) -> (mpsc::Receiver<Screen>, Option<String>) {
        let game_title = self.game_title();

        let vm = self.vm.take();
        let (screen_tx, screen_rx) = mpsc::sync_channel(1);

        crate::utils::spawn(vm_loop(vm, screen_tx, io_rx));

        (screen_rx, game_title)
    }

    #[cfg(target_arch = "wasm32")]
    pub fn setup(&mut self, io_rx: mpsc::Receiver<IoEvent>) -> (mpsc::Receiver<Screen>, String) {
        let (screen_tx, screen_rx) = mpsc::sync_channel(1);

        self.screen_tx = Some(screen_tx);
        self.io_rx = Some(io_rx);

        (screen_rx, String::new())
    }

    #[cfg(target_arch = "wasm32")]
    pub fn take(
        mut self,
    ) -> (
        Option<Vm>,
        mpsc::SyncSender<Screen>,
        mpsc::Receiver<IoEvent>,
    ) {
        (
            self.vm,
            self.screen_tx.take().unwrap(),
            self.io_rx.take().unwrap(),
        )
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
}
