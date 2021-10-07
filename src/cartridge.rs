// use crate::memory::MemoryAccess;

pub use mbc::MBC;

mod mbc {
    use super::MBC_KIND_ADDR;
    use std::{
        fs,
        io::{Read, Write},
        path::PathBuf,
    };

    #[allow(unused)]
    pub trait MBC: Send {
        fn rom_read(&self, addr: u16) -> u8 {
            0
        }
        fn ram_read(&self, addr: u16) -> u8 {
            0
        }
        fn rom_write(&mut self, addr: u16, value: u8) {}
        fn ram_write(&mut self, addr: u16, value: u8) {}
    }

    pub struct MBC0 {
        rom: Vec<u8>,
    }
    pub struct MBC1 {
        rom: Vec<u8>,
        ram: Vec<u8>,
        save: Option<PathBuf>,

        rom_bank: u8,
        ram_bank: u8,
        banking_mode: u8,
        ram_enabled: bool,
    }
    pub struct MBC3 {
        rom: Vec<u8>,
        ram: Vec<u8>,
        save: Option<PathBuf>,

        rom_bank: u8,
        ram_bank: u8,
        ram_enabled: bool,
    }

    impl MBC0 {
        pub fn new(rom: Vec<u8>) -> Box<dyn MBC + 'static> {
            Box::new(MBC0 { rom })
        }
    }
    impl MBC1 {
        pub fn new(rom: Vec<u8>, ram_size: usize) -> Box<dyn MBC + 'static> {
            println!("MBC1 cartridge");

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

            Box::new(MBC1 {
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
    impl MBC3 {
        pub fn new(rom: Vec<u8>, ram_size: usize) -> Box<dyn MBC + 'static> {
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

            Box::new(MBC3 {
                rom,
                ram,
                save,

                rom_bank: 1,
                ram_bank: 0,
                ram_enabled: false,
            })
        }
    }

    impl MBC for MBC0 {
        fn rom_read(&self, addr: u16) -> u8 {
            self.rom[addr as usize]
        }
    }

    impl MBC for MBC1 {
        fn rom_read(&self, addr: u16) -> u8 {
            let addr = addr as usize;

            let addr = if addr < 0x4000 {
                addr
            } else {
                addr & 0x3FFF | self.rom_bank as usize * 0x4000
            };
            self.rom[addr]
        }
        fn ram_read(&self, addr: u16) -> u8 {
            if self.ram_enabled {
                let ram_bank = if self.banking_mode != 0 {
                    self.ram_bank
                } else {
                    0
                };
                let addr = addr & 0x1FFF | ram_bank as u16 * 0x2000;

                self.ram[addr as usize]
            } else {
                0
            }
        }
        fn rom_write(&mut self, addr: u16, value: u8) {
            match addr {
                0x0000..=0x1FFF => self.ram_enabled = value == 0x0A,
                0x2000..=0x3FFF => {
                    self.rom_bank = self.rom_bank & 0x60
                        | match value & 0x1F {
                            0 => 1,
                            b => b,
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
            if self.ram_enabled {
                self.ram_enabled = false;
                let ram_bank = if self.banking_mode != 0 {
                    self.ram_bank
                } else {
                    0
                };
                let addr = addr & 0x1FFF | ram_bank as u16 * 0x2000;

                self.ram[addr as usize] = value;
            }
        }
    }

    impl MBC for MBC3 {
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
                // self.rtc_ram[self.rambank - 0x8] = value;
                // self.calc_rtc_zero();
            }
        }
    }

    impl Drop for MBC1 {
        fn drop(&mut self) {
            match self.save {
                None => (),
                Some(ref path) => {
                    println!("Saving game to file {:?}", path);
                    fs::write(path, &self.ram).ok();
                }
            }
        }
    }
    impl Drop for MBC3 {
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
}

const MBC_KIND_ADDR: usize = 0x147;

pub struct Cartridge {
    mbc: Box<dyn mbc::MBC + 'static>,
    title: String,
}

impl Cartridge {
    pub fn new(rom: &[u8]) -> Cartridge {
        const TITLE_ADDR: usize = 0x134;
        const RAM_SIZE_ADDR: usize = 0x149;

        let ram_size = ram_size(rom[RAM_SIZE_ADDR]);

        let mbc = match rom[MBC_KIND_ADDR] {
            0x00 => mbc::MBC0::new(rom.to_owned()),
            0x01..=0x03 => mbc::MBC1::new(rom.to_owned(), ram_size),
            0x0F..=0x13 => mbc::MBC3::new(rom.to_owned(), ram_size),
            _ => panic!("Unsupported MBC"),
        };

        let title = (0..16)
            .filter_map(|i| {
                let byte = mbc.rom_read(TITLE_ADDR as u16 + i);
                if byte != 0 {
                    Some(byte as char)
                } else {
                    None
                }
            })
            .collect::<String>();

        Cartridge { mbc, title }
    }

    pub fn rom_name(&self) -> &str {
        &self.title
    }
}

impl MBC for Cartridge {
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
