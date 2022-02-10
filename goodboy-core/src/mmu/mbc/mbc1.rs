use std::{fs, path::PathBuf};

use crate::mmu::cartridge::MBC_KIND_ADDR;

use super::{Mbc, MbcCapability, MbcDescription};

pub struct Mbc1 {
    capabilities: Vec<MbcCapability>,

    rom: Vec<u8>,
    ram: Vec<u8>,
    save: Option<PathBuf>,

    rom_bank: usize,
    ram_bank: usize,
    ram_mode: bool,
    ram_enabled: bool,
}

impl Mbc1 {
    pub fn new(rom: Vec<u8>, ram_size: usize) -> Box<dyn Mbc + 'static> {
        let (ram, save, capabilities) = match rom[MBC_KIND_ADDR] {
            0x02 => (None, None, vec![MbcCapability::Ram]),
            0x03 => {
                let path = PathBuf::from("save_file.gbsave");
                let ram = match fs::read(&path) {
                    Ok(buffer) => Some(buffer),
                    // Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
                    Err(_) => None,
                };

                (
                    ram,
                    Some(path),
                    vec![MbcCapability::Ram, MbcCapability::Battery],
                )
            }
            0x01 | _ => (Some(Vec::new()), None, vec![]),
        };

        let ram = ram.unwrap_or_else(|| std::iter::repeat(0).take(ram_size).collect());

        Box::new(Mbc1 {
            capabilities,

            rom,
            ram,
            save,

            rom_bank: 1,
            ram_bank: 0,
            // 00h simple ROM banking mode (default)
            // 01h RAM banking mode / advanced rom banking mode
            ram_mode: false,
            // 00h disable RAM (default)
            // 0Ah enable RAM
            ram_enabled: false,
        })
    }
}

impl Mbc for Mbc1 {
    fn description(&self) -> Option<super::MbcDescription<'_>> {
        Some(MbcDescription::MBC1(&self.capabilities))
    }

    fn rom_read(&self, addr: u16) -> u8 {
        let addr = if addr >= 0x4000 {
            ((self.rom_bank * 0x4000) | ((addr as usize) & 0x3FFF)) % self.rom.len()
        } else {
            addr as usize
        };
        self.rom[addr]
    }
    fn ram_read(&self, addr: u16) -> u8 {
        if !self.ram_enabled {
            return 0;
        }

        let ram_bank = if self.ram_mode { self.ram_bank } else { 0 };
        let addr = (ram_bank * 0x2000) | ((addr & 0x1FFF) as usize);

        self.ram[addr]
    }
    fn rom_write(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x1FFF => self.ram_enabled = value == 0x0A,
            0x2000..=0x3FFF => {
                self.rom_bank = (self.rom_bank & 0x60)
                    | match (value as usize) & 0x1F {
                        0 => 1,
                        n => n,
                    };
            }
            0x4000..=0x5FFF => {
                if self.ram_mode {
                    self.ram_bank = (value as usize) & 0x03;
                } else {
                    let value = value as usize;
                    self.rom_bank = match (self.rom_bank & 0x1F) | ((value & 0x03) << 5) {
                        n @ (0x00 | 0x20 | 0x40 | 0x60) => n + 1,
                        n => n,
                    };
                }
            }
            0x6000..=0x7FFF => self.ram_mode = value & 0x01 == 0x01,
            _ => panic!("Invalid MBC1 ROM addr: 0x{:04X}", addr),
        }
    }

    fn ram_write(&mut self, addr: u16, value: u8) {
        if !self.ram_enabled {
            return;
        }

        let ram_bank = if self.ram_mode { self.ram_bank } else { 0 };
        let addr = ((addr as usize) & 0x1FFF) + (ram_bank * 0x2000);

        self.ram[addr as usize] = value;
    }
}

impl Drop for Mbc1 {
    fn drop(&mut self) {
        match self.save {
            None => (),
            Some(ref path) => {
                log::info!("Saving game to file {:?}", path);
                fs::write(path, &self.ram)
                    .map(|_| log::info!("Saved Successfully"))
                    .ok();
            }
        }
    }
}
