use std::{fs, path::PathBuf};

use crate::mmu::cartridge::MBC_KIND_ADDR;

use super::Mbc;

pub struct Mbc1 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    save: Option<PathBuf>,

    rom_bank: u8,
    ram_bank: u8,
    banking_mode: u8,
    ram_enabled: bool,
}

impl Mbc1 {
    pub fn new(rom: Vec<u8>, ram_size: usize) -> Box<dyn Mbc + 'static> {
        println!("MBC1 cartridge detected");

        dbg!(ram_size);

        let (ram, save) = match rom[MBC_KIND_ADDR] {
            0x02 => (None, None),
            0x03 => {
                let path = PathBuf::from("save_file.gbsave");
                let ram = match fs::read(&path) {
                    Ok(buffer) => Some(buffer),
                    // Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
                    Err(_) => None,
                };

                (ram, Some(path))
            }
            _ => (Some(Vec::new()), None),
        };

        let ram = ram.unwrap_or(std::iter::repeat(0).take(ram_size).collect());

        Box::new(Mbc1 {
            rom,
            ram,
            save,

            rom_bank: 1,
            ram_bank: 0,
            // 00h simple ROM banking mode (default)
            // 01h RAM banking mode / advanced rom banking mode
            banking_mode: 0,
            // 00h disable RAM (default)
            // 0Ah enable RAM
            ram_enabled: false,
        })
    }
}

impl Mbc for Mbc1 {
    fn rom_read(&self, addr: u16) -> u8 {
        let addr = if addr <= 0x3FFF {
            addr as usize
        } else {
            self.rom_bank as usize * 0x4000 | ((addr as usize) & 0x3FFF)
        };

        self.rom[addr]
    }
    fn ram_read(&self, addr: u16) -> u8 {
        if !self.ram_enabled {
            return 0;
        }

        let ram_bank = if self.banking_mode != 0 {
            self.ram_bank as usize
        } else {
            0
        };
        let addr = (ram_bank * 0x2000) | ((addr & 0x1FFF) as usize);

        self.ram[addr]
    }
    fn rom_write(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x1FFF => self.ram_enabled = value == 0b1010,
            0x2000..=0x3FFF => {
                self.rom_bank = self.rom_bank & 0x60
                    | match value & 0x1F {
                        0 => 1,
                        value => value,
                    };
            }
            0x4000..=0x5FFF => {
                if self.banking_mode == 0 {
                    self.rom_bank = self.rom_bank & 0x1F | ((value & 0x03) << 5);
                } else {
                    self.ram_bank = value & 0x03;
                }
            }
            0x6000..=0x7FFF => self.banking_mode = value & 0x01,
            _ => panic!("Invalid MBC1 ROM addr: 0x{:04X}", addr),
        }
    }

    fn ram_write(&mut self, addr: u16, value: u8) {
        if !self.ram_enabled {
            return;
        }
        // self.ram_enabled = false;

        let ram_bank = if self.banking_mode != 0 {
            self.ram_bank
        } else {
            0
        };
        let addr = addr & 0x1FFF | ram_bank as u16 * 0x2000;

        self.ram[addr as usize] = value;
    }
}

impl Drop for Mbc1 {
    fn drop(&mut self) {
        match self.save {
            None => (),
            Some(ref path) => {
                println!("Saving game to file {:?}", path);
                fs::write(path, &self.ram)
                    .map(|_| println!("Saved Successfully"))
                    .ok();
            }
        }
    }
}
