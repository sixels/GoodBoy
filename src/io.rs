#![allow(dead_code)]

use std::{
    ops::{Deref, DerefMut},
    sync::mpsc,
};

use goodboy_core::{io::JoypadButton, mmu::cartridge::Cartridge};
use winit::event::VirtualKeyCode;
use winit_input_helper::WinitInputHelper;

use crate::utils;

#[allow(dead_code)]
pub enum IoEvent {
    ButtonPressed(JoypadButton),
    ButtonReleased(JoypadButton),
    InsertCartridge(Cartridge),
    // SetColorScheme(ColorScheme),
    SwitchSpeedNext,
    SwitchSpeedPrev,
    Exit,
}

#[derive(Clone)]
pub struct IoHandler {
    input: WinitInputHelper,
    pub sender: mpsc::Sender<IoEvent>,
}

impl IoHandler {
    pub fn new() -> (Self, mpsc::Receiver<IoEvent>) {
        let (io_tx, io_rx) = mpsc::channel();
        let input = WinitInputHelper::new();

        (
            Self {
                input,
                sender: io_tx,
            },
            io_rx,
        )
    }

    pub fn handle_input(&self, title_sender: &mpsc::Sender<String>) {
        let Self {
            input,
            sender: io_tx,
        } = self.clone();

        let send_keys = move || -> Result<(), mpsc::SendError<IoEvent>> {
            if input.key_pressed(VirtualKeyCode::Escape) {
                self::insert_cartridge(io_tx.clone(), title_sender.clone());
            }
            if input.key_pressed(VirtualKeyCode::Tab) {
                io_tx.send(if !input.held_shift() {
                    IoEvent::SwitchSpeedNext
                } else {
                    IoEvent::SwitchSpeedPrev
                })?
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
        };

        send_keys().ok();
    }
}

pub fn insert_cartridge(io_ev_sender: mpsc::Sender<IoEvent>, title_sender: mpsc::Sender<String>) {
    let dialog = rfd::AsyncFileDialog::new()
        .add_filter("ROM", &["gb", "gbc"])
        .pick_file();

    utils::spawn({
        let io_tx = io_ev_sender.clone();

        async move {
            let file = dialog.await;

            if let Some(file) = file {
                log::info!("Loading file: {file:?}");
                let buffer = file.read().await;

                let cartridge = Cartridge::new(&buffer);
                title_sender.send(cartridge.rom_name().to_string()).ok();

                if io_tx.send(IoEvent::InsertCartridge(cartridge)).is_err() {
                    log::error!("Error sending the file buffer");
                }
            } else {
                log::info!("No file selected");
            }
        }
    });
}

impl Deref for IoHandler {
    type Target = WinitInputHelper;

    fn deref(&self) -> &Self::Target {
        &self.input
    }
}

impl DerefMut for IoHandler {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.input
    }
}
