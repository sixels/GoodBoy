use std::{fs, io::{Read, Write}, path::PathBuf};

use crate::mmu::cartridge::MBC_KIND_ADDR;

use super::Mbc;

pub struct Mbc3 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    save: Option<PathBuf>,

    rom_bank: u8,
    ram_bank: u8,
    ram_enabled: bool,
}

impl Mbc3 {
    pub fn new(rom: Vec<u8>, ram_size: usize) -> Box<dyn Mbc + 'static> {
        println!("MBC3 cartridge");

        let (ram, save) = match rom[MBC_KIND_ADDR] {
            0x0F | 0x10 | 0x13 => {
                let path = PathBuf::from("save_file.gbsave");

                // try to retrieve the save file
                let ram = match fs::File::open(&path) {
                    Ok(mut f) => {
                        let mut ram: Vec<u8> = Vec::with_capacity(ram_size);
                        f.read_to_end(&mut ram).and_then(|_| Ok(ram)).ok()
                    }
                    Err(_) => None,
                };

                (ram, Some(path))
            }
            0x12 => (None, None),
            _ => (Some(Vec::new()), None),
        };
        let ram = ram.unwrap_or(std::iter::repeat(0).take(ram_size).collect());

        Box::new(Mbc3 {
            rom,
            ram,
            save,

            rom_bank: 1,
            ram_bank: 0,
            ram_enabled: false,
        })
    }
}

impl Mbc for Mbc3 {
    fn rom_read(&self, addr: u16) -> u8 {
        let addr = if addr < 0x4000 {
            addr as usize
        } else {
            (addr as usize & 0x3FFF) | self.rom_bank as usize * 0x4000
        };
        self.rom.get(addr).copied().unwrap_or(0)
    }

    fn ram_read(&self, addr: u16) -> u8 {
        if !self.ram_enabled {
            0
        } else {
            if self.ram_bank <= 3 {
                let addr = addr as usize & 0x1FFF | self.ram_bank as usize * 0x2000;
                self.ram[addr]
            } else {
                panic!("RTC not implemented")
            }
        }
    }

    fn rom_write(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x1FFF => self.ram_enabled = value == 0x0A,
            0x2000..=0x3FFF => {
                self.rom_bank = match value & 0x7F {
                    0 => 1,
                    n => n,
                }
            }
            0x4000..=0x5FFF => self.ram_bank = value,
            0x6000..=0x7FFF => match value {
                0 => {
                    panic!("RTC not implemented");
                }
                1 => {
                    panic!("RTC not implemented")
                }
                _ => {}
            },
            _ => panic!("Could not write to {:04X} (MBC3)", addr),
        }
    }

    fn ram_write(&mut self, addr: u16, value: u8) {
        if !self.ram_enabled {
            return;
        }
        if self.ram_bank <= 3 {
            self.ram[self.ram_bank as usize * 0x2000 | ((addr as usize) & 0x1FFF)] = value;
        } else {
            panic!("Timer not implemented");
        }
    }
}

impl Drop for Mbc3 {
    fn drop(&mut self) {
        match self.save {
            None => (),
            Some(ref path) => {
                println!("Saving game to file {:?}", path);
                match fs::File::create(path) {
                    Ok(mut f) => {
                        f.write_all(&self.ram).ok();
                    }
                    Err(_) => (),
                }
            }
        }
    }
}
