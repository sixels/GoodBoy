// use crate::memory::MemoryAccess;

use crate::gb_mode::GbMode;

use super::mbc::{self, Mbc};

pub const MBC_KIND_ADDR: usize = 0x147;

pub struct Cartridge {
    mbc: Box<dyn mbc::Mbc + 'static>,
    title: String,
}

impl Cartridge {
    pub fn new(rom: &[u8]) -> (Cartridge, GbMode) {
        const TITLE_ADDR: usize = 0x134;
        const GB_MODE_ADDR: usize = 0x143;
        const RAM_SIZE_ADDR: usize = 0x149;

        let mode = match rom[GB_MODE_ADDR] {
            // CGB only
            0xC0 => GbMode::CGB,
            // game works on both CGB and DMG
            0x80 => GbMode::default(),
            _ => GbMode::DMG,
        };
        log::info!("Gameboy mode: {:?}", mode);

        let ram_size = ram_size(rom[RAM_SIZE_ADDR]);
        log::debug!("Cartridge RAM size: {}", ram_size);

        let mbc = match rom[MBC_KIND_ADDR] {
            0x00 => mbc::Mbc0::new(rom.to_owned()),
            0x01..=0x03 => mbc::Mbc1::new(rom.to_owned(), ram_size),
            0x0F..=0x13 => mbc::Mbc3::new(rom.to_owned(), ram_size, "savefile.gbsave"),
            0x19..=0x1B => mbc::Mbc5::new(rom.to_owned(), ram_size),
            _ => panic!("Unsupported cartridge MBC"),
        };

        if let Some(info) = mbc.kind() {
            log::info!("MBC info: {:?}", info);
        } else {
            log::info!("No information supplied for this MBC")
        }

        let title_len = if mode == GbMode::DMG { 16 } else { 11 };
        let title = (0..title_len)
            .filter_map(|i| {
                rom.get(TITLE_ADDR + i)
                    .copied()
                    .filter(|byte| *byte != 0)
                    .map(|byte| byte as char)
            })
            .collect::<String>();
        log::info!("Game title: \"{}\"", title);

        (Cartridge { mbc, title }, mode)
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
