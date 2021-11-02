use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use crate::mmu::{cartridge::MBC_KIND_ADDR, mbc::MbcCapability};

use super::{Mbc, MbcKind};

pub struct Mbc3 {
    capabilities: Vec<MbcCapability>,

    rom: Vec<u8>,
    ram: Vec<u8>,
    save: Option<PathBuf>,

    rom_bank: u8,
    ram_bank: u8,
    ram_enabled: bool,
}

impl Mbc3 {
    pub fn new(
        rom: Vec<u8>,
        ram_size: usize,
        save_path: impl AsRef<Path>,
    ) -> Box<dyn Mbc + 'static> {
        let (ram, save, capabilities) = match rom[MBC_KIND_ADDR] {
            b @ 0x0F | b @ 0x10 | b @ 0x13 => {
                let mut capabilities = match b {
                    0x0F => vec![MbcCapability::Timer],
                    0x10 => vec![MbcCapability::Timer, MbcCapability::RAM],
                    0x13 | _ => vec![MbcCapability::RAM],
                };
                capabilities.push(MbcCapability::Battery);

                // try to retrieve the save file
                let ram = match fs::File::open(&save_path) {
                    Ok(mut f) => {
                        let mut ram: Vec<u8> = std::iter::repeat(0).take(ram_size).collect();
                        f.read_to_end(&mut ram).and_then(|_| Ok(ram)).ok()
                    }
                    Err(_) => None,
                };

                (ram, Some(save_path.as_ref().to_path_buf()), capabilities)
            }
            0x12 => (None, None, vec![MbcCapability::RAM]),
            0x11 | _ => (Some(Vec::new()), None, vec![]),
        };
        let ram = ram.unwrap_or(std::iter::repeat(0).take(ram_size).collect());

        Box::new(Mbc3 {
            capabilities,

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
    fn kind<'a>(&'a self) -> Option<super::MbcKind<'a>> {
        Some(MbcKind::MBC3(&self.capabilities))
    }

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
                    ()
                    // panic!("RTC not implemented");
                }
                1 => {
                    ()
                    // panic!("RTC not implemented");
                }
                _ => (),
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
                log::info!("Saving game to file {:?}", path);
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
