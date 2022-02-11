use std::sync::mpsc;

use goodboy_core::io::JoypadButton;
use winit::event::VirtualKeyCode;
use winit_input_helper::WinitInputHelper;

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
pub fn handle_input(input: &mut WinitInputHelper, io_sender: mpsc::Sender<IoEvent>) {
    fn send_keys(input: &mut WinitInputHelper, io_sender: mpsc::Sender<IoEvent>) -> Result<(), mpsc::SendError<IoEvent>> {
        if input.key_pressed(VirtualKeyCode::F1) { io_sender.send(IoEvent::ToggleFPSLimit)?; }

        if input.key_pressed(VirtualKeyCode::Right) { io_sender.send(IoEvent::ButtonPressed(JoypadButton::Right))?;  }
        if input.key_pressed(VirtualKeyCode::Left)  { io_sender.send(IoEvent::ButtonPressed(JoypadButton::Left))?;   }
        if input.key_pressed(VirtualKeyCode::Up)    { io_sender.send(IoEvent::ButtonPressed(JoypadButton::Up))?;     }
        if input.key_pressed(VirtualKeyCode::Down)  { io_sender.send(IoEvent::ButtonPressed(JoypadButton::Down))?;   }
        if input.key_pressed(VirtualKeyCode::Z)          { io_sender.send(IoEvent::ButtonPressed(JoypadButton::A))?;      }
        if input.key_pressed(VirtualKeyCode::X)          { io_sender.send(IoEvent::ButtonPressed(JoypadButton::B))?;      }
        if input.key_pressed(VirtualKeyCode::Space)      { io_sender.send(IoEvent::ButtonPressed(JoypadButton::Select))?; }
        if input.key_pressed(VirtualKeyCode::Return)      { io_sender.send(IoEvent::ButtonPressed(JoypadButton::Start))?;  }

        if input.key_released(VirtualKeyCode::Right) { io_sender.send(IoEvent::ButtonReleased(JoypadButton::Right))?;  }
        if input.key_released(VirtualKeyCode::Left)  { io_sender.send(IoEvent::ButtonReleased(JoypadButton::Left))?;   }
        if input.key_released(VirtualKeyCode::Up)    { io_sender.send(IoEvent::ButtonReleased(JoypadButton::Up))?;     }
        if input.key_released(VirtualKeyCode::Down)  { io_sender.send(IoEvent::ButtonReleased(JoypadButton::Down))?;   }
        if input.key_released(VirtualKeyCode::Z)          { io_sender.send(IoEvent::ButtonReleased(JoypadButton::A))?;      }
        if input.key_released(VirtualKeyCode::X)          { io_sender.send(IoEvent::ButtonReleased(JoypadButton::B))?;      }
        if input.key_released(VirtualKeyCode::Space)      { io_sender.send(IoEvent::ButtonReleased(JoypadButton::Select))?; }
        if input.key_released(VirtualKeyCode::Return)      { io_sender.send(IoEvent::ButtonReleased(JoypadButton::Start))?;  }
        Ok(())
    }
    send_keys(input, io_sender).ok();
}
