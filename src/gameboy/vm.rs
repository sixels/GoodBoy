use std::sync::mpsc;
use std::time::Duration;

use goodboy_core::vm::{Screen, Vm};

use crate::io::IoEvent;

pub async fn vm_loop(
    mut vm: Option<Vm>,
    screen_sender: mpsc::SyncSender<Screen>,
    io: mpsc::Receiver<IoEvent>,
) {
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

        if let Some(vm) = vm.as_mut() {
            while clocks < total_clocks {
                clocks += vm.tick();

                if vm.check_vblank() {
                    if let Err(mpsc::TrySendError::Disconnected(..)) =
                        screen_sender.try_send(vm.get_screen())
                    {
                        break 'vm;
                    }
                }
            }
            clocks -= total_clocks;
        }

        loop {
            match io.try_recv() {
                Ok(event) => match event {
                    IoEvent::ButtonPressed(button) => {
                        vm.as_mut().map(|vm| vm.press_button(button));
                    }
                    IoEvent::ButtonReleased(button) => {
                        vm.as_mut().map(|vm| vm.release_button(button));
                    }
                    IoEvent::InsertCartridge(cart) => {
                        let _ = vm.insert(Vm::from_cartridge(cart));
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
