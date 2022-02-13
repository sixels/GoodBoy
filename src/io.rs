use std::sync::mpsc;

use goodboy_core::io::JoypadButton;
use winit::event::VirtualKeyCode;
use winit_input_helper::WinitInputHelper;

use crate::utils;

#[allow(dead_code)]
pub enum IoEvent {
    ButtonPressed(JoypadButton),
    ButtonReleased(JoypadButton),
    InsertCartridge(Vec<u8>),
    // SetColorScheme(ColorScheme),
    ToggleFPSLimit,
    Exit,
}

pub fn handle_input(input: WinitInputHelper, io_sender: mpsc::Sender<IoEvent>) {
    fn send_keys(
        input: WinitInputHelper,
        io_tx: mpsc::Sender<IoEvent>,
    ) -> Result<(), mpsc::SendError<IoEvent>> {
        if input.key_pressed(VirtualKeyCode::F1) {
            let dialog = rfd::AsyncFileDialog::new()
                .add_filter("ROM", &["gb", "gbc"])
                .pick_file();

            utils::spawn({
                let io_tx = io_tx.clone();

                async move {
                    let file = dialog.await;

                    if let Some(file) = file {
                        log::info!("Loading file: {file:?}");
                        let buffer = file.read().await;
                        if io_tx.send(IoEvent::InsertCartridge(buffer)).is_err() {
                            log::error!("Error sending the file buffer");
                        }
                    } else {
                        log::info!("No file selected");
                    }
                }
            });
        }
        if input.key_pressed(VirtualKeyCode::Tab) {
            io_tx.send(IoEvent::ToggleFPSLimit)?
        }

        if input.key_pressed(VirtualKeyCode::Right) {
            io_tx.send(IoEvent::ButtonPressed(JoypadButton::Right))?;
        }
        if input.key_pressed(VirtualKeyCode::Left) {
            io_tx.send(IoEvent::ButtonPressed(JoypadButton::Left))?;
        }
        if input.key_pressed(VirtualKeyCode::Up) {
            io_tx.send(IoEvent::ButtonPressed(JoypadButton::Up))?;
        }
        if input.key_pressed(VirtualKeyCode::Down) {
            io_tx.send(IoEvent::ButtonPressed(JoypadButton::Down))?;
        }
        if input.key_pressed(VirtualKeyCode::Z) {
            io_tx.send(IoEvent::ButtonPressed(JoypadButton::A))?;
        }
        if input.key_pressed(VirtualKeyCode::X) {
            io_tx.send(IoEvent::ButtonPressed(JoypadButton::B))?;
        }
        if input.key_pressed(VirtualKeyCode::Space) {
            io_tx.send(IoEvent::ButtonPressed(JoypadButton::Select))?;
        }
        if input.key_pressed(VirtualKeyCode::Return) {
            io_tx.send(IoEvent::ButtonPressed(JoypadButton::Start))?;
        }

        if input.key_released(VirtualKeyCode::Right) {
            io_tx.send(IoEvent::ButtonReleased(JoypadButton::Right))?;
        }
        if input.key_released(VirtualKeyCode::Left) {
            io_tx.send(IoEvent::ButtonReleased(JoypadButton::Left))?;
        }
        if input.key_released(VirtualKeyCode::Up) {
            io_tx.send(IoEvent::ButtonReleased(JoypadButton::Up))?;
        }
        if input.key_released(VirtualKeyCode::Down) {
            io_tx.send(IoEvent::ButtonReleased(JoypadButton::Down))?;
        }
        if input.key_released(VirtualKeyCode::Z) {
            io_tx.send(IoEvent::ButtonReleased(JoypadButton::A))?;
        }
        if input.key_released(VirtualKeyCode::X) {
            io_tx.send(IoEvent::ButtonReleased(JoypadButton::B))?;
        }
        if input.key_released(VirtualKeyCode::Space) {
            io_tx.send(IoEvent::ButtonReleased(JoypadButton::Select))?;
        }
        if input.key_released(VirtualKeyCode::Return) {
            io_tx.send(IoEvent::ButtonReleased(JoypadButton::Start))?;
        }
        Ok(())
    }

    send_keys(input, io_sender).ok();
}
