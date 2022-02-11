mod vm;

use std::sync::mpsc;

#[cfg(not(target_arch = "wasm32"))]
use std::thread::{self, JoinHandle};

use goodboy_core::vm::{Screen, VM};
#[cfg(target_arch = "wasm32")]
pub use vm::update_vm;
#[cfg(not(target_arch = "wasm32"))]
use vm::vm_loop;

use crate::io::IoEvent;

pub struct GameBoy {
    pub screen_chan: (mpsc::SyncSender<Screen>, mpsc::Receiver<Screen>),
    pub io_chan: (mpsc::Sender<IoEvent>, Option<mpsc::Receiver<IoEvent>>),

    // fps: Fps,
    pub vm: Option<VM>,

    #[cfg(not(target_arch = "wasm32"))]
    vm_loop_handle: Option<JoinHandle<()>>,
}

impl GameBoy {
    pub fn new() -> Self {
        let screen_chan = mpsc::sync_channel(1);
        let io_chan = {
            let chan = mpsc::channel();
            (chan.0, Some(chan.1))
        };

        GameBoy {
            screen_chan,
            io_chan,

            // fps: Fps::default(),
            vm: None,

            #[cfg(not(target_arch = "wasm32"))]
            vm_loop_handle: None,
        }
    }

    pub fn prepare(&mut self) {
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
}
