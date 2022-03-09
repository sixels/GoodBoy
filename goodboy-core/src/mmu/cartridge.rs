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
    pub gb_mode: GbMode,
    mbc: Box<dyn mbc::Mbc + 'static>,
    ram_size: usize,
}

impl Cartridge {
    pub fn new(rom: &[u8]) -> Cartridge {
        let mode = match rom[GB_MODE_ADDR] {
            // CGB only
            0xC0 => GbMode::Cgb,
            // game works on both CGB and DMG
            0x80 => {
                let default_gb_mode = GbMode::default();
                log::info!("No specific mode specified, using the default ({default_gb_mode:?})");
                default_gb_mode
            }
            _ => GbMode::Dmg,
        };

        let ram_size = match rom[RAM_SIZE_ADDR] {
            0x00 => 0x00000, // No RAM
            0x01 => 0x00800, // Undocumented
            0x02 => 0x02000, // 8 KB
            0x03 => 0x08000, // 32 KB
            0x04 => 0x20000, // 128 KB
            0x05 => 0x16000, // 64 KB
            _ => unreachable!(),
        };

        let title_len = if mode == GbMode::Dmg { 16 } else { 11 };
        let title = (0..title_len)
            .filter_map(|i| {
                rom.get(TITLE_ADDR + i)
                    .copied()
                    .filter(|byte| *byte != 0)
                    .map(char::from)
            })
            .collect::<String>();

        let save_path = format!("{}.gbsave", title.to_ascii_lowercase());

        let mbc = match rom[MBC_KIND_ADDR] {
            0x00 => mbc::Mbc0::new(rom.to_owned()),
            0x01..=0x03 => mbc::Mbc1::new(rom.to_owned(), ram_size, &save_path),
            0x0F..=0x13 => mbc::Mbc3::new(rom.to_owned(), ram_size, &save_path),
            0x19..=0x1B => mbc::Mbc5::new(rom.to_owned(), ram_size, &save_path),
            _ => panic!("Unsupported cartridge MBC"),
        };

        Cartridge {
            title,
            gb_mode: mode,
            mbc,
            ram_size,
        }
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
            .field("mbc", &self.mbc.description())
            .field("ram_size", &self.ram_size)
            .finish()
    }
}
