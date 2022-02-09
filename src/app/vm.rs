use std::sync::mpsc;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Duration;

use goodboy_core::vm::{Screen, VM};

use crate::io::IoEvent;

#[cfg(not(target_arch = "wasm32"))]
pub fn vm_loop(
    mut vm: Option<VM>,
    screen_sender: mpsc::SyncSender<Screen>,
    io: mpsc::Receiver<IoEvent>,
) {
    let mut clocks = 0;

    let sleep_time: Vec<u64> = vec![16, 8, 4, 0];
    let mut time_cycle = sleep_time.iter().copied().cycle();

    let (timer, sleep_sender) = speed_limit(Duration::from_millis(time_cycle.next().unwrap()));

    loop {
        let mut switch_speed = false;

        match update_vm(
            &mut vm,
            screen_sender.clone(),
            &io,
            clocks,
            Some(&mut switch_speed),
        ) {
            Ok(rem_clocks) => clocks = rem_clocks,
            Err(_) => break,
        }

        if switch_speed {
            sleep_sender
                .send(Duration::from_millis(time_cycle.next().unwrap()))
                .ok();
        }

        if timer.recv().is_err() {
            // timer stopped
            break;
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn speed_limit(wait_time: Duration) -> (mpsc::Receiver<()>, mpsc::Sender<Duration>) {
    use std::thread;

    let (time_sender, time_receiver) = mpsc::sync_channel(1);
    let (sleep_sender, sleep_receiver) = mpsc::channel();

    let mut wait_time = wait_time;
    thread::spawn(move || loop {
        while let Ok(time) = sleep_receiver.try_recv() {
            wait_time = time
        }

        thread::sleep(wait_time);
        if time_sender.send(()).is_err() {
            break;
        };
    });

    (time_receiver, sleep_sender)
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
        clocks += vm.tick();

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
