use std::{
    fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

use crate::mmu::{cartridge::MBC_KIND_ADDR, mbc::MbcCapability};

use super::{Mbc, MbcDescription};

pub struct Mbc5 {
    capabilities: Vec<MbcCapability>,

    rom: Vec<u8>,
    ram: Vec<u8>,
    save_path: Option<PathBuf>,

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
        let capabilities = Mbc5::get_capabilities(rom[MBC_KIND_ADDR]);

        let mut ram = if capabilities.contains(&MbcCapability::Ram) {
            std::iter::repeat(0).take(ram_size + 8).collect()
        } else {
            Vec::new()
        };

        let save_path = if capabilities.contains(&MbcCapability::Battery) {
            Some(save_path.as_ref().to_path_buf())
        } else {
            None
        };

        save_path.as_ref().map(|path| {
            let mut buf = vec![];
            fs::File::open(path)
                .and_then(|mut save_file| save_file.read_to_end(&mut buf))
                .map_or_else(
                    |e| {
                        if e.kind() != io::ErrorKind::NotFound {
                            panic!("Error reading file \"{path:?}\": {e:?}")
                        }
                    },
                    |_| ram = buf,
                )
        });

        Box::new(Mbc5 {
            capabilities,

            rom,
            ram,
            save_path,

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

    pub fn get_capabilities(cartridge_kind: u8) -> Vec<MbcCapability> {
        match cartridge_kind {
            // mbc5
            0x19 => vec![],
            // mbc5+ram
            0x1A => vec![MbcCapability::Ram],
            0x1B => vec![MbcCapability::Ram, MbcCapability::Battery],
            0x1C => vec![MbcCapability::Rumble],
            0x1D => vec![MbcCapability::Rumble, MbcCapability::Ram],
            0x1E => vec![
                MbcCapability::Rumble,
                MbcCapability::Ram,
                MbcCapability::Battery,
            ],
            _ => panic!("Invalid MBC5 cartridge"),
        }
    }
}

impl Mbc for Mbc5 {
    fn description(&self) -> Option<super::MbcDescription<'_>> {
        Some(MbcDescription::MBC5(&self.capabilities))
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
        let addr = (self.ram_bank * 0x2000) | ((addr as usize) & 0x1FFF);
        self.ram[addr]
    }
    fn rom_write(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x1FFF => self.ram_enabled = value == 0x0A,
            0x2000..=0x2FFF => self.rom_bank = (self.rom_bank & 0x100) | (value as usize),
            0x3000..=0x3FFF => {
                self.rom_bank = (self.rom_bank & 0xFF) | (((value as usize) & 0x1) << 8)
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
        self.ram[(self.ram_bank * 0x2000) + ((addr as usize) & 0x1FFF)] = value;
    }
}

impl Drop for Mbc5 {
    fn drop(&mut self) {
        match self.save_path {
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
