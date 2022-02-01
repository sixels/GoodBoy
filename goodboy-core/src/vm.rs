use std::{fs, io, path::Path};

use crate::{cpu::Cpu, io::JoypadButton, mmu::Bus, ppu::ColorScheme};

pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;

pub type Screen = Box<[u8; SCREEN_WIDTH * SCREEN_HEIGHT * 4]>;

pub struct VM {
    cpu: Cpu,
}

impl VM {
    pub fn new<P: AsRef<Path>>(rom_path: P) -> io::Result<Self> {
        log::info!("Creating a new VM from path: {:?}", rom_path.as_ref());

        let rom_buffer = fs::read(rom_path)?;
        let bus = Bus::new(&rom_buffer);

        Ok(Self { cpu: Cpu::new(bus) })
    }

    pub fn new_with_buffer(rom_buffer: &[u8]) -> Self {
        log::info!("Creating a new VM from file buffer");

        let bus = Bus::new(rom_buffer);
        Self { cpu: Cpu::new(bus) }
    }

    pub fn tick(&mut self) -> u32 {
        self.cpu.run()
    }

    pub fn check_vblank(&mut self) -> bool {
        let vblanked = self.cpu.bus.gpu.vblanked;
        self.cpu.bus.gpu.vblanked = false;
        vblanked
    }

    pub fn get_screen(&self) -> Screen {
        self.cpu.bus.gpu.screen_buffer.clone()
    }

    pub fn press_button(&mut self, button: JoypadButton) {
        log::info!("Button pressed: {button:?}");
        self.cpu.bus.joypad.press_button(button);
    }

    pub fn release_button(&mut self, button: JoypadButton) {
        log::info!("Button released: {button:?}");
        self.cpu.bus.joypad.release_button(button);
    }

    pub fn set_color_scheme(&mut self, color_scheme: ColorScheme) {
        log::info!("Setting color scheme: {color_scheme:?}");
        self.cpu.bus.gpu.set_color_scheme(color_scheme);
    }
}
