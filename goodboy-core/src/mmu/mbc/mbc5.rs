use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};

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
    pub fn new(
        rom: Vec<u8>,
        ram_size: usize,
        save_path: impl AsRef<Path>,
    ) -> Box<dyn Mbc + 'static> {
        let capb = rom[MBC_KIND_ADDR];
        let (ram, save, capabilities) = match capb {
            // mbc5
            0x19 => (
                Some(std::iter::repeat(0).take(ram_size).collect()),
                None,
                vec![],
            ),
            // mbc5+ram
            0x1A => (None, None, vec![MbcCapability::Ram]),
            // mbc5+ram+battery | mbc5+rumble+ram+battery
            0x1B | 0x1E => {
                let mut capabilities = if capb == 0x1E {
                    vec![MbcCapability::Rumble]
                } else {
                    vec![]
                };
                capabilities.extend(vec![MbcCapability::Ram, MbcCapability::Battery]);

                let ram = match fs::File::open(&save_path) {
                    Ok(mut f) => {
                        let mut ram: Vec<u8> = std::iter::repeat(0).take(ram_size).collect();
                        match f.read_to_end(&mut ram).map(|_| ram) {
                            Ok(buffer) => {
                                log::info!("Loaded save file: {:?}", save_path.as_ref());
                                Some(buffer)
                            }
                            Err(e) => {
                                log::warn!("Could not read {:?}: {:?}", save_path.as_ref(), e);
                                None
                            }
                        }
                    }
                    Err(_) => None,
                };

                (ram, Some(save_path.as_ref().to_path_buf()), capabilities)
            }
            // mbc5+rumble | mbc5+rumble+ram
            0x1C | 0x1D => {
                let mut ram = None;
                let mut capabilities = vec![MbcCapability::Rumble];
                if capb == 0x1C {
                    ram = Some(std::iter::repeat(0).take(ram_size).collect())
                } else {
                    capabilities.push(MbcCapability::Ram);
                }
                (ram, None, capabilities)
            }
            _ => panic!("Invalid MBC5 cartridge"),
        };

        let ram = ram.unwrap_or_else(|| std::iter::repeat(0).take(ram_size).collect());

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
    fn kind(&self) -> Option<super::MbcKind<'_>> {
        Some(MbcKind::MBC5(&self.capabilities))
    }

    fn rom_read(&self, addr: u16) -> u8 {
        let addr = if addr >= 0x4000 {
            (self.rom_bank * 0x4000) | ((addr as usize) & 0x3FFF)
        } else {
            addr as usize
        };
        self.rom[addr]
    }
    fn ram_read(&self, addr: u16) -> u8 {
        if !self.ram_enabled {
            return 0;
        }
        let addr = (self.ram_bank * 0x2000) | ((addr as usize) & 0x1FFF);
        self.ram[addr]
    }
    fn rom_write(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x1FFF => self.ram_enabled = value == 0x0A,
            0x2000..=0x2FFF => self.rom_bank = (self.rom_bank & 0x100) | (value as usize),
            0x3000..=0x3FFF => {
                self.rom_bank = (self.rom_bank & 0x0FF) | (((value as usize) & 0x1) << 8)
            }
            0x4000..=0x5FFF => self.ram_bank = (value & 0x0F) as usize,
            0x6000..=0x7FFF => {}
            _ => panic!("Invalid MBC5 ROM addr: 0x{:04X}", addr),
        }
    }
    fn ram_write(&mut self, addr: u16, value: u8) {
        if !self.ram_enabled {
            return;
        }
        self.ram[(self.ram_bank * 0x2000) | ((addr as usize) & 0x1FFF)] = value;
    }
}

impl Drop for Mbc5 {
    fn drop(&mut self) {
        match self.save {
            None => (),
            Some(ref path) => {
                log::info!("Saving game to file {:?}", path);
                if let Ok(mut f) = fs::File::create(path) {
                    f.write_all(&self.ram).ok();
                }
            }
        }
    }
}
