use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use crate::mmu::{cartridge::MBC_KIND_ADDR, mbc::MbcCapability};

use super::{Mbc, MbcDescription};

#[derive(Default)]
struct Rtc {
    pub sec: u8,
    pub min: u8,
    pub hour: u8,
    pub dayl: u8,
    pub dayh: u8,

    latched: bool,
    start: u64,
}

impl Rtc {
    pub fn as_slice<'a>(&self) -> [&u8; 5] {
        [&self.sec, &self.min, &self.hour, &self.dayl, &self.dayh]
    }
    pub fn as_mut_slice<'a>(&mut self) -> [&mut u8; 5] {
        [
            &mut self.sec,
            &mut self.min,
            &mut self.hour,
            &mut self.dayl,
            &mut self.dayh,
        ]
    }

    pub fn toggle(&mut self) -> bool {
        let latched = self.latched;
        self.latched ^= true;
        latched
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn update(&mut self) {
        let tstart = std::time::UNIX_EPOCH + std::time::Duration::from_secs(self.start);

        if self.dayh & 0x40 == 0x40 {
            return;
        }

        let dt = match std::time::SystemTime::now().duration_since(tstart) {
            Ok(t) => t.as_secs(),
            _ => 0,
        };

        self.sec = (dt % 60) as u8;
        self.min = ((dt / 60) % 60) as u8;
        self.hour = ((dt / 3600) % 24) as u8;
        let days = dt / (3600 * 24);
        self.dayl = days as u8;
        self.dayh = (self.dayh & 0xFE) | (((days >> 8) & 0x01) as u8);

        if days >= 512 {
            self.dayh |= 0x80;
            self.update_start();
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn update_start(&mut self) {
        let tstart = {
            let dt = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();

            let [sec, min, hour, dayl, dayh] = self.as_slice().map(|a| *a as u64);

            let secs = sec;
            let mins = min * 60;
            let hours = hour * 3600;
            let days = (((dayh & 0x1) << 8) | dayl) * 3600 * 24;

            dt - secs - mins - hours - days
        };

        self.start = tstart;
    }

    #[cfg(target_arch = "wasm32")]
    pub fn update(&mut self) {}
    #[cfg(target_arch = "wasm32")]
    pub fn update_start(&mut self) {}
}

pub struct Mbc3 {
    capabilities: Vec<MbcCapability>,

    rom: Vec<u8>,
    ram: Vec<u8>,
    save: Option<PathBuf>,

    rom_bank: u8,
    ram_bank: u8,
    ram_enabled: bool,

    rtc: Option<Rtc>,
    rtc_last_byte: u8,
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
                    0x10 => vec![MbcCapability::Timer, MbcCapability::Ram],
                    0x13 | _ => vec![MbcCapability::Ram],
                };
                capabilities.push(MbcCapability::Battery);

                // try to retrieve the save file
                let ram = match (cfg!(target_arch = "wasm32"), fs::File::open(&save_path)) {
                    (true, Ok(mut f)) => {
                        let mut ram: Vec<u8> = std::iter::repeat(0).take(ram_size).collect();
                        f.read_to_end(&mut ram).map(|_| ram).ok()
                    }
                    (_, Err(_)) | (false, _) => None,
                };

                (ram, Some(save_path.as_ref().to_path_buf()), capabilities)
            }
            0x12 => (None, None, vec![MbcCapability::Ram]),
            0x11 | _ => (Some(Vec::new()), None, vec![]),
        };
        let ram = ram.unwrap_or_else(|| std::iter::repeat(0).take(ram_size).collect());

        let rtc = if capabilities.contains(&MbcCapability::Timer) {
            Some(Rtc::default())
        } else {
            None
        };

        Box::new(Mbc3 {
            capabilities,

            rom,
            ram,
            save,

            rom_bank: 1,
            ram_bank: 0,
            ram_enabled: false,

            rtc,
            rtc_last_byte: 0xff,
        })
    }
}

impl Mbc for Mbc3 {
    fn description(&self) -> Option<super::MbcDescription<'_>> {
        Some(MbcDescription::MBC3(&self.capabilities))
    }

    fn rom_read(&self, addr: u16) -> u8 {
        let addr = if addr < 0x4000 {
            addr as usize
        } else {
            (addr as usize & 0x3FFF) | (self.rom_bank as usize * 0x4000)
        };
        self.rom.get(addr).copied().unwrap_or(0)
    }

    fn ram_read(&self, addr: u16) -> u8 {
        if !self.ram_enabled {
            0
        } else if self.ram_bank <= 3 {
            let addr = addr as usize & 0x1FFF | (self.ram_bank as usize * 0x2000);
            self.ram[addr]
        } else {
            self.rtc
                .as_ref()
                .map_or(0x00, |rtc| *rtc.as_slice()[(self.ram_bank - 0x08) as usize])
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
            0x6000..=0x7FFF => {
                self.rtc.as_mut().map(|rtc| {
                    let latched = if value == 0x01 && self.rtc_last_byte == 0x00 {
                        rtc.toggle()
                    } else {
                        false
                    };
                    self.rtc_last_byte = value;

                    if !latched {
                        rtc.update()
                    }
                });
            }
            _ => panic!("Could not write to {:04X} (MBC3)", addr),
        }
    }

    fn ram_write(&mut self, addr: u16, value: u8) {
        if !self.ram_enabled {
            return;
        }
        if self.ram_bank <= 3 {
            self.ram[(self.ram_bank as usize * 0x2000) | ((addr as usize) & 0x1FFF)] = value;
        } else {
            self.rtc.as_mut().map(|rtc| {
                *rtc.as_mut_slice()[(self.ram_bank - 0x08) as usize] = value;
                rtc.update_start();
            });
        }
    }
}

impl Drop for Mbc3 {
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
