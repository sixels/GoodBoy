// use crate::memory::MemoryAccess;

pub use mbc::MBC;

mod mbc {
    use super::MBC_KIND_ADDR;
    use std::{fs, path::PathBuf};

    pub trait MBC: Send {
        fn rom_read(&self, _addr: u16) -> u8 {
            0
        }
        fn ram_read(&self, _addr: u16) -> u8 {
            0
        }
        fn rom_write(&mut self, _addr: u16, _b: u8) {}
        fn ram_write(&mut self, _addr: u16, _b: u8) {}
    }

    pub struct MBC0 {
        rom: Vec<u8>,
    }
    impl MBC0 {
        pub fn new(rom: Vec<u8>) -> MBC0 {
            MBC0 { rom }
        }
    }
    impl MBC for MBC0 {
        fn rom_read(&self, addr: u16) -> u8 {
            self.rom[addr as usize]
        }
    }

    pub struct MBC1 {
        rom: Vec<u8>,
        ram: Vec<u8>,
        save: Option<PathBuf>,

        rom_bank: usize,

        ram_bank: usize,
        ram_mode: bool,
        ram_enabled: bool,
    }
    impl MBC1 {
        pub fn new(rom: Vec<u8>, ram_size: usize) -> MBC1 {
            let (save, ram_size) = match rom[MBC_KIND_ADDR] {
                0x02 => (None, ram_size),
                0x03 => (Some(PathBuf::from("save_file.gbsave")), ram_size),
                _ => (None, 0),
            };

            let ram = match save {
                None => std::iter::repeat(0).take(ram_size).collect(),
                Some(ref path) => match fs::read(path) {
                    Ok(buffer) => buffer,
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                        std::iter::repeat(0).take(ram_size).collect()
                    }
                    Err(_) => panic!("Could not read the save file"),
                },
            };

            MBC1 {
                rom,
                ram,
                save,

                rom_bank: 1,

                ram_bank: 0,
                ram_mode: false,
                ram_enabled: false,
            }
        }
    }
    impl MBC for MBC1 {
        fn rom_read(&self, addr: u16) -> u8 {
            let addr = addr as usize;
            let addr = if addr < 0x4000 {
                addr
            } else {
                self.rom_bank * 0x4000 | addr & 0x3FFF
            };
            self.rom[addr]
        }
        fn ram_read(&self, addr: u16) -> u8 {
            if self.ram_enabled {
                let ram_bank = if self.ram_mode { self.ram_bank } else { 0 };
                self.ram[ram_bank * 0x2000 | (addr as usize) & 0x1FFF]
            } else {
                0
            }
        }
        fn rom_write(&mut self, addr: u16, b: u8) {
            match addr {
                0x0000..=0x1FFF => self.ram_enabled = b == 0x0A,
                0x2000..=0x3FFF => {
                    self.rom_bank = (self.rom_bank & 0x60)
                        | match b & 0x1F {
                            0 => 1,
                            b => b as usize,
                        }
                }
                0x4000..=0x5FFF => {
                    if !self.ram_mode {
                        self.rom_bank = self.rom_bank & 0x1F | ((b as usize & 0x03) << 5)
                    } else {
                        self.ram_bank = (b as usize) & 0x03;
                    }
                }
                0x6000..=0x7FFF => self.ram_mode = (b & 0x01) == 0x01,
                _ => panic!("Invalid MBC1 ROM addr: 0x{:04X}", addr),
            }
        }

        fn ram_write(&mut self, addr: u16, b: u8) {
            if self.ram_enabled {
                let ram_bank = if self.ram_mode { self.ram_bank } else { 0 };
                let addr = addr as usize;
                self.ram[(ram_bank * 0x2000) | addr & 0x1FFF] = b
            }
        }
    }
    impl Drop for MBC1 {
        fn drop(&mut self) {
            match self.save {
                None => (),
                Some(ref path) => {
                    fs::write(path, &self.ram).ok();
                }
            }
        }
    }

    // pub struct MBC3 {}
    // impl MBC for MBC3 {}
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
            0x00 => Box::new(mbc::MBC0::new(rom.to_owned())) as Box<dyn MBC + 'static>,
            0x01..=0x03 => {
                Box::new(mbc::MBC1::new(rom.to_owned(), ram_size)) as Box<dyn MBC + 'static>
            }
            // 0x0F..=0x13 => {
            //     // mbc 3
            // }
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
    fn ram_write(&mut self, addr: u16, b: u8) {
        self.mbc.ram_write(addr, b)
    }
    fn rom_write(&mut self, addr: u16, b: u8) {
        self.mbc.rom_write(addr, b)
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
