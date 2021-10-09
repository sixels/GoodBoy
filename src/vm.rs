use std::{fs, io, path::Path};

use crate::{cpu::Cpu, io::{JoypadButton}, mmu::Bus};

pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;

pub type Screen = Box<[u8; SCREEN_WIDTH * SCREEN_HEIGHT * 4]>;

pub struct VM {
    cpu: Cpu,
}

impl VM {
    pub fn new<P: AsRef<Path>>(rom_path: P) -> io::Result<Self> {
        let rom_buffer = fs::read(rom_path)?;
        let bus = Bus::new(&rom_buffer);

        Ok(Self { cpu: Cpu::new(bus) })
    }

    // pub fn new_blank<B, P>(bios_path: B, rom_path: P) -> io::Result<Self>
    // where
    //     B: AsRef<Path>,
    //     P: AsRef<Path>,
    // {
    //     let mut bios = fs::read(bios_path)?;
    //     let rom_buffer = fs::read(rom_path)?;

    //     bios.extend(&rom_buffer[0x100..]);

    //     let bus = Bus::new_blank(&bios);

    //     Ok(Self { cpu: Cpu::new_blank(bus) })
    // }

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
        self.cpu.bus.joypad.press_button(button);
    }

    pub fn release_button(&mut self, button: JoypadButton) {
        self.cpu.bus.joypad.release_button(button);
    }
}
