// use crate::memory::MemoryAccess;

use super::mbc::{self, Mbc};

pub const MBC_KIND_ADDR: usize = 0x147;

pub struct Cartridge {
    mbc: Box<dyn mbc::Mbc + 'static>,
    title: String,
}

impl Cartridge {
    pub fn new(rom: &[u8]) -> Cartridge {
        const TITLE_ADDR: usize = 0x134;
        const RAM_SIZE_ADDR: usize = 0x149;

        let ram_size = ram_size(rom[RAM_SIZE_ADDR]);

        let mbc = match rom[MBC_KIND_ADDR] {
            0x00 => mbc::Mbc0::new(rom.to_owned()),
            0x01..=0x03 => mbc::Mbc1::new(rom.to_owned(), ram_size),
            0x0F..=0x13 => mbc::Mbc3::new(rom.to_owned(), ram_size),
            _ => panic!("Unsupported cartridge MBC"),
        };

        let title = (0..16)
            .filter_map(|i| {
                let byte = mbc.rom_read(TITLE_ADDR as u16 + i);
                if byte != 0 {
                    Some(byte as char)
                } else {
                    None
                }
            })
            .collect::<String>();

        Cartridge { mbc, title }
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

fn ram_size(v: u8) -> usize {
    match v {
        1 => 0x800,
        2 => 0x2000,
        3 => 0x8000,
        4 => 0x20000,
        _ => 0,
    }
}
