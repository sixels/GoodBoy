use std::{fs, path::Path};

use crate::{bus::Bus, cpu::Cpu};

pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;

pub type Screen = Box<[u8; SCREEN_WIDTH * SCREEN_HEIGHT * 4]>;

pub struct VM {
    cpu: Cpu,
}

impl VM {
    pub fn new<P: AsRef<Path>>(rom_path: P) -> Self {
        let rom_buffer = fs::read(rom_path).unwrap();
        let bus = Bus::new(&rom_buffer);

        Self { cpu: Cpu::new(bus) }
    }

    pub fn tick(&mut self) -> u8 {
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
}
