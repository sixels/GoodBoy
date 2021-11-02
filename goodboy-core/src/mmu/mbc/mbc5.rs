use std::{fs, path::PathBuf};

use crate::mmu::{cartridge::MBC_KIND_ADDR, mbc::MbcCapability};

use super::{Mbc, MbcKind};

pub struct Mbc5 {
    capabilities: Vec<MbcCapability>,

    rom: Vec<u8>,
    ram: Vec<u8>,
    save: Option<PathBuf>,

    rom_bank: usize,
    ram_bank: usize,
    // banking_mode: u8,
    ram_enabled: bool,
}

impl Mbc5 {
    pub fn new(rom: Vec<u8>, ram_size: usize) -> Box<dyn Mbc + 'static> {
        // TODO: Specify capabilities
        let (ram, save, capabilities) = match rom[MBC_KIND_ADDR] {
            b @ 0x1B | b @ 0x1E => {
                let mut capabilities = match b {
                    0x18 => vec![MbcCapability::Timer],
                    0x1E | _ => vec![MbcCapability::Timer, MbcCapability::RAM],
                };
                capabilities.push(MbcCapability::Battery);

                let path = PathBuf::from("save_file.gbsave");
                let ram = match fs::read(&path) {
                    Ok(buffer) => Some(buffer),
                    // Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
                    Err(_) => None,
                };

                (ram, Some(path), capabilities)
            }
            _ => (
                Some(std::iter::repeat(0).take(ram_size).collect()),
                None,
                vec![],
            ),
        };

        let ram = ram.unwrap_or(std::iter::repeat(0).take(ram_size).collect());

        Box::new(Mbc5 {
            capabilities,

            rom,
            ram,
            save,

            rom_bank: 1,
            ram_bank: 0,
            // 00h simple ROM banking mode (default)
            // 01h RAM banking mode / advanced rom banking mode
            // banking_mode: 0,
            // 00h disable RAM (default)
            // 0Ah enable RAM
            ram_enabled: false,
        })
    }
}

impl Mbc for Mbc5 {
    fn kind<'a>(&'a self) -> Option<super::MbcKind<'a>> {
        Some(MbcKind::MBC5(&self.capabilities))
    }

    fn rom_read(&self, addr: u16) -> u8 {
        let addr = addr as usize;

        let addr = if addr < 0x4000 {
            addr
        } else {
            (addr & 0x3FFF) | self.rom_bank as usize * 0x4000
        };
        self.rom[addr]
    }
    fn ram_read(&self, addr: u16) -> u8 {
        if !self.ram_enabled {
            return 0;
        }
        self.ram[self.ram_bank * 0x2000 | ((addr as usize) & 0x1FFF)]
    }
    fn rom_write(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x1FFF => self.ram_enabled = value == 0x0A,
            0x2000..=0x2FFF => self.rom_bank = (self.rom_bank & 0x100) | value as usize,
            0x3000..=0x3fff => {
                self.rom_bank = (self.rom_bank & 0x0FF) | ((value as usize & 0x1) << 8)
            }
            0x4000..=0x5FFF => self.ram_bank = (value & 0x0F) as usize,
            0x6000..=0x7FFF => (),
            _ => panic!("Invalid MBC1 ROM addr: 0x{:04X}", addr),
        }
    }

    fn ram_write(&mut self, addr: u16, value: u8) {
        if self.ram_enabled == false {
            return;
        }
        self.ram[self.ram_bank * 0x2000 | ((addr as usize) & 0x1FFF)] = value;
    }
}

impl Drop for Mbc5 {
    fn drop(&mut self) {
        match self.save {
            None => (),
            Some(ref path) => {
                log::info!("Saving game to file {:?}", path);
                fs::write(path, &self.ram).ok();
            }
        }
    }
}
