#![allow(dead_code)]

use std::{sync::mpsc::{self, Receiver, Sender, SyncSender, TryRecvError, TrySendError}, thread, time::Duration};

use goodboy_core::{io::JoypadButton, vm::{Screen, VM}};
use winit::event::VirtualKeyCode;
use winit_input_helper::WinitInputHelper;

use super::{IoEvent, ColorSchemeIter};

#[rustfmt::skip]
pub fn handle_input(input: &mut WinitInputHelper, io_sender: &Sender<IoEvent>, color_schemes_iter: Option<&mut ColorSchemeIter>) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(color_schemes_iter) = color_schemes_iter {
        if input.key_pressed(VirtualKeyCode::Tab) { io_sender.send(IoEvent::SetColorScheme(color_schemes_iter.next().unwrap()))?; }
        if input.held_shift() && input.key_pressed(VirtualKeyCode::Tab) {
            for _ in 3..super::COLOR_SCHEMES.len() {
                color_schemes_iter.next().unwrap();
            }
            io_sender.send(IoEvent::SetColorScheme(color_schemes_iter.next().unwrap()))?;
        }
    }

    if input.key_pressed(VirtualKeyCode::F1) { io_sender.send(IoEvent::ToggleFPSLimit)?; }

    if input.key_pressed(VirtualKeyCode::Right)  { io_sender.send(IoEvent::ButtonPressed(JoypadButton::Right))?;  }
    if input.key_pressed(VirtualKeyCode::Left)   { io_sender.send(IoEvent::ButtonPressed(JoypadButton::Left))?;   }
    if input.key_pressed(VirtualKeyCode::Up)     { io_sender.send(IoEvent::ButtonPressed(JoypadButton::Up))?;     }
    if input.key_pressed(VirtualKeyCode::Down)   { io_sender.send(IoEvent::ButtonPressed(JoypadButton::Down))?;   }
    if input.key_pressed(VirtualKeyCode::Z)      { io_sender.send(IoEvent::ButtonPressed(JoypadButton::A))?;      }
    if input.key_pressed(VirtualKeyCode::X)      { io_sender.send(IoEvent::ButtonPressed(JoypadButton::B))?;      }
    if input.key_pressed(VirtualKeyCode::Space)  { io_sender.send(IoEvent::ButtonPressed(JoypadButton::Select))?; }
    if input.key_pressed(VirtualKeyCode::Return) { io_sender.send(IoEvent::ButtonPressed(JoypadButton::Start))?;  }

    if input.key_released(VirtualKeyCode::Right)  { io_sender.send(IoEvent::ButtonReleased(JoypadButton::Right))?;  }
    if input.key_released(VirtualKeyCode::Left)   { io_sender.send(IoEvent::ButtonReleased(JoypadButton::Left))?;   }
    if input.key_released(VirtualKeyCode::Up)     { io_sender.send(IoEvent::ButtonReleased(JoypadButton::Up))?;     }
    if input.key_released(VirtualKeyCode::Down)   { io_sender.send(IoEvent::ButtonReleased(JoypadButton::Down))?;   }
    if input.key_released(VirtualKeyCode::Z)      { io_sender.send(IoEvent::ButtonReleased(JoypadButton::A))?;      }
    if input.key_released(VirtualKeyCode::X)      { io_sender.send(IoEvent::ButtonReleased(JoypadButton::B))?;      }
    if input.key_released(VirtualKeyCode::Space)  { io_sender.send(IoEvent::ButtonReleased(JoypadButton::Select))?; }
    if input.key_released(VirtualKeyCode::Return) { io_sender.send(IoEvent::ButtonReleased(JoypadButton::Start))?;  }

    Ok(())
}

pub fn vm_loop(mut vm: VM, screen_sender: SyncSender<Screen>, io: Receiver<IoEvent>) {
    println!("a");
    let mut clocks = 0;
    let clocks_to_run = (4194304.0 / 1000.0 * 12f64).round() as u32;

    let timer = speed_limit(Duration::from_millis(11));
    let mut respect_timer = true;

    'vm_loop: loop {
        while clocks < clocks_to_run {
            clocks += vm.tick() as u32;

            if vm.check_vblank() {
                // println!("b");
                if let Err(TrySendError::Disconnected(..)) = screen_sender.try_send(vm.get_screen())
                {
                    break;
                }
            }
        }

        loop {
            match io.try_recv() {
                Ok(event) => match event {
                    IoEvent::ButtonPressed(button) => vm.press_button(button),
                    IoEvent::ButtonReleased(button) => vm.release_button(button),
                    IoEvent::SetColorScheme(color_scheme) => vm.set_color_scheme(color_scheme),
                    IoEvent::ToggleFPSLimit => respect_timer ^= true,

                    IoEvent::Exit => break 'vm_loop,
                },
                Err(TryRecvError::Empty) => break,
                Err(_) => break 'vm_loop,
            }
        }

        clocks -= clocks_to_run;

        if respect_timer {
            if timer.recv().is_err() {
                break;
            }
        }
    }
}

fn speed_limit(wait_time: Duration) -> Receiver<()> {
    let (time_sender, time_receiver) = mpsc::sync_channel(1);
    thread::spawn(move || loop {
        thread::sleep(wait_time);
        if time_sender.send(()).is_err() {
            break;
        };
    });

    time_receiver
}
