use std::sync::mpsc;

use eframe::egui;
use goodboy_core::io::JoypadButton;

#[allow(dead_code)]
pub enum IoEvent {
    ButtonPressed(JoypadButton),
    ButtonReleased(JoypadButton),
    InsertCartridge(Vec<u8>),
    // SetColorScheme(ColorScheme),
    ToggleFPSLimit,
    Exit,
}

#[rustfmt::skip]
pub fn handle_input(input: &egui::InputState, io_sender: mpsc::Sender<IoEvent>) {
    fn send_keys(input: &egui::InputState, io_sender: mpsc::Sender<IoEvent>) -> Result<(), mpsc::SendError<IoEvent>> {
        if input.key_pressed(egui::Key::Delete) { io_sender.send(IoEvent::ToggleFPSLimit)?; }

        if input.key_pressed(egui::Key::ArrowRight) { io_sender.send(IoEvent::ButtonPressed(JoypadButton::Right))?;  }
        if input.key_pressed(egui::Key::ArrowLeft)  { io_sender.send(IoEvent::ButtonPressed(JoypadButton::Left))?;   }
        if input.key_pressed(egui::Key::ArrowUp)    { io_sender.send(IoEvent::ButtonPressed(JoypadButton::Up))?;     }
        if input.key_pressed(egui::Key::ArrowDown)  { io_sender.send(IoEvent::ButtonPressed(JoypadButton::Down))?;   }
        if input.key_pressed(egui::Key::Z)          { io_sender.send(IoEvent::ButtonPressed(JoypadButton::A))?;      }
        if input.key_pressed(egui::Key::X)          { io_sender.send(IoEvent::ButtonPressed(JoypadButton::B))?;      }
        if input.key_pressed(egui::Key::Space)      { io_sender.send(IoEvent::ButtonPressed(JoypadButton::Select))?; }
        if input.key_pressed(egui::Key::Enter)      { io_sender.send(IoEvent::ButtonPressed(JoypadButton::Start))?;  }

        if input.key_released(egui::Key::ArrowRight) { io_sender.send(IoEvent::ButtonReleased(JoypadButton::Right))?;  }
        if input.key_released(egui::Key::ArrowLeft)  { io_sender.send(IoEvent::ButtonReleased(JoypadButton::Left))?;   }
        if input.key_released(egui::Key::ArrowUp)    { io_sender.send(IoEvent::ButtonReleased(JoypadButton::Up))?;     }
        if input.key_released(egui::Key::ArrowDown)  { io_sender.send(IoEvent::ButtonReleased(JoypadButton::Down))?;   }
        if input.key_released(egui::Key::Z)          { io_sender.send(IoEvent::ButtonReleased(JoypadButton::A))?;      }
        if input.key_released(egui::Key::X)          { io_sender.send(IoEvent::ButtonReleased(JoypadButton::B))?;      }
        if input.key_released(egui::Key::Space)      { io_sender.send(IoEvent::ButtonReleased(JoypadButton::Select))?; }
        if input.key_released(egui::Key::Enter)      { io_sender.send(IoEvent::ButtonReleased(JoypadButton::Start))?;  }
        Ok(())
    }
    send_keys(input, io_sender).ok();
}
