// use crate::memory::MemoryAccess;

use std::fmt::Debug;

use crate::gb_mode::GbMode;

use super::mbc::{self, Mbc};

pub const MBC_KIND_ADDR: usize = 0x147;
pub const TITLE_ADDR: usize = 0x134;
pub const GB_MODE_ADDR: usize = 0x143;
pub const RAM_SIZE_ADDR: usize = 0x149;

pub struct Cartridge {
    title: String,
    mbc: Box<dyn mbc::Mbc + 'static>,
    ram_size: usize,
}

impl Cartridge {
    pub fn new(rom: &[u8]) -> (Cartridge, GbMode) {
        fn ram_size(v: u8) -> usize {
            match v {
                1 => 0x800,
                2 => 0x2000,
                3 => 0x8000,
                4 => 0x20000,
                _ => 0,
            }
        }

        let mode = match rom[GB_MODE_ADDR] {
            // CGB only
            0xC0 => GbMode::Cgb,
            // game works on both CGB and DMG
            0x80 => GbMode::default(),
            _ => GbMode::Dmg,
        };

        let ram_size = ram_size(rom[RAM_SIZE_ADDR]);

        let mbc = match rom[MBC_KIND_ADDR] {
            0x00 => mbc::Mbc0::new(rom.to_owned()),
            0x01..=0x03 => mbc::Mbc1::new(rom.to_owned(), ram_size),
            0x0F..=0x13 => mbc::Mbc3::new(rom.to_owned(), ram_size, "savefile.gbsave"),
            0x19..=0x1B => mbc::Mbc5::new(rom.to_owned(), ram_size, "savefile.gbsave"),
            _ => panic!("Unsupported cartridge MBC"),
        };

        let title_len = if mode == GbMode::Dmg { 16 } else { 11 };
        let title = (0..title_len)
            .filter_map(|i| {
                rom.get(TITLE_ADDR + i)
                    .copied()
                    .filter(|byte| *byte != 0)
                    .map(|byte| byte as char)
            })
            .collect::<String>();

        (Cartridge { mbc, title, ram_size }, mode)
    }

    pub fn rom_name(&self) -> &str {
        &self.title
    }
}

impl Mbc for Cartridge {
    fn ram_read(&self, addr: u16) -> u8 {
        self.mbc.ram_read(addr)
    }
    fn rom_read(&self, addr: u16) -> u8 {
        self.mbc.rom_read(addr)
    }
    fn ram_write(&mut self, addr: u16, value: u8) {
        self.mbc.ram_write(addr, value)
    }
    fn rom_write(&mut self, addr: u16, value: u8) {
        self.mbc.rom_write(addr, value)
    }
}

impl Debug for Cartridge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Cartridge")
            .field("title", &self.title)
            .field("mbc", &self.mbc.kind())
            .field("ram_size", &self.ram_size)
            .finish()
    }
}
