use super::{Mbc, MbcKind};

pub struct Mbc0 {
    rom: Vec<u8>,
}

impl Mbc0 {
    pub fn new(rom: Vec<u8>) -> Box<dyn Mbc + 'static> {
        Box::new(Mbc0 { rom })
    }
}

impl Mbc for Mbc0 {
    fn rom_read(&self, addr: u16) -> u8 {
        self.rom[addr as usize]
    }
    fn kind(&self) -> Option<super::MbcKind<'_>> {
        Some(MbcKind::MBC0)
    }
}
