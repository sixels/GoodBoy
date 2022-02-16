#![allow(dead_code)]

use std::{
    ops::{Deref, DerefMut},
    sync::{mpsc, Arc, PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard},
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

pub struct IoHandler {
    input: WinitInputHelper,
    io_tx: mpsc::Sender<IoEvent>,
    /// the current game's title
    game_title: GameTitle,
}

#[derive(Clone)]
pub struct GameTitle(Arc<RwLock<String>>);

impl GameTitle {
    pub fn new(title: impl Into<String>) -> Self {
        Self(Arc::new(RwLock::new(title.into())))
    }
    pub fn set_title(
        &self,
        title: impl Into<String>,
    ) -> Result<(), PoisonError<RwLockWriteGuard<String>>> {
        self.write()
            .map(|mut game_title| *game_title = title.into())
    }

    pub fn get_title(&self) -> Result<String, PoisonError<RwLockReadGuard<String>>> {
        self.read().map(|title| title.clone())
    }
}

impl Deref for GameTitle {
    type Target = Arc<RwLock<String>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl IoHandler {
    pub fn new() -> (Self, mpsc::Receiver<IoEvent>) {
        let (io_tx, io_rx) = mpsc::channel();
        let input = WinitInputHelper::new();

        (
            Self {
                input,
                io_tx,
                game_title: GameTitle::new(""),
            },
            io_rx,
        )
    }

    pub fn handle_input(&self) {
        let input = self.input.clone();
        let io_tx = self.io_tx.clone();
        let game_title = self.game_title.clone();

        let send_keys = move || -> Result<(), mpsc::SendError<IoEvent>> {
            if input.key_pressed(VirtualKeyCode::Escape) {
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

                            let cartridge = Cartridge::new(&buffer);

                            let _ = game_title.set_title(cartridge.rom_name());

                            if io_tx.send(IoEvent::InsertCartridge(cartridge)).is_err() {
                                log::error!("Error sending the file buffer");
                            }
                        } else {
                            log::info!("No file selected");
                        }
                    }
                });
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

    pub fn set_game_title(
        &self,
        title: impl Into<String>,
    ) -> Result<(), PoisonError<RwLockWriteGuard<String>>> {
        self.game_title.set_title(title)
    }

    pub fn get_game_title(&self) -> Result<String, PoisonError<RwLockReadGuard<String>>> {
        self.game_title.get_title()
    }
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
