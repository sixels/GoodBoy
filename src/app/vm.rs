#[cfg(not(target_arch = "wasm32"))]
use std::time::Duration;
use std::{ops::Deref, sync::mpsc};

use goodboy_core::vm::{Screen, VM};

use crate::io::IoEvent;

#[cfg(not(target_arch = "wasm32"))]
pub fn vm_loop(
    mut vm: Option<VM>,
    screen_sender: mpsc::SyncSender<Screen>,
    io: mpsc::Receiver<IoEvent>,
) {
    let mut clocks = 0;

    let timer = speed_limit(Duration::from_millis(15));
    let mut respect_timer = true;

    loop {
        match update_vm(
            &mut vm,
            screen_sender.clone(),
            &io,
            clocks,
            Some(&mut respect_timer),
        ) {
            Ok(rem_clocks) => clocks = rem_clocks,
            Err(_) => break,
        }

        if respect_timer && timer.recv().is_err() {
            // timer stopped
            break;
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn speed_limit(wait_time: Duration) -> mpsc::Receiver<()> {
    use std::thread;

    let (time_sender, time_receiver) = mpsc::sync_channel(1);
    thread::spawn(move || loop {
        thread::sleep(wait_time);
        if time_sender.send(()).is_err() {
            break;
        };
    });

    time_receiver
}

pub fn update_vm(
    vm: &mut Option<VM>,
    screen_sender: mpsc::SyncSender<Screen>,
    io: &mpsc::Receiver<IoEvent>,
    clocks: u32,
    timer_controller: Option<&mut bool>,
) -> Result<u32, ()> {
    let mut clocks = clocks;
    let total_clocks = (4194304.0 / 1000.0 * 16f64).round() as u32;

    if let Some(vm) = vm.as_mut() {
        while clocks < total_clocks {
            clocks += vm.tick() as u32;

            if vm.check_vblank() {
                if let Err(mpsc::TrySendError::Disconnected(..)) =
                    screen_sender.try_send(vm.get_screen())
                {
                    return Err(());
                }
            }
        }
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
                IoEvent::InsertCartridge(buffer) => {
                    let _ = vm.insert(VM::new_with_buffer(buffer.as_slice()));
                    return Ok(0);
                }
                // IoEvent::SetColorScheme(color_scheme) => vm.set_color_scheme(color_scheme),
                IoEvent::ToggleFPSLimit => {
                    timer_controller.map(|tc| *tc ^= true);
                    break;
                }
                IoEvent::Exit => return Err(()),
                // _ => (),
            },
            Err(mpsc::TryRecvError::Empty) => break,
            Err(_) => return Err(()),
        }
    }

    Ok(clocks - total_clocks)
}
