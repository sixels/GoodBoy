mod mbc0;
mod mbc1;
mod mbc3;
mod mbc5;

pub use mbc0::Mbc0;
pub use mbc1::Mbc1;
pub use mbc3::Mbc3;
pub use mbc5::Mbc5;

#[allow(unused)]
pub trait Mbc: Send {
    fn rom_read(&self, addr: u16) -> u8 {
        0
    }
    fn ram_read(&self, addr: u16) -> u8 {
        0
    }
    fn rom_write(&mut self, addr: u16, value: u8) {}
    fn ram_write(&mut self, addr: u16, value: u8) {}

    fn kind(&self) -> Option<MbcKind<'_>> {
        None
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MbcKind<'a> {
    MBC0,
    MBC1(&'a Vec<MbcCapability>),
    MBC3(&'a Vec<MbcCapability>),
    MBC5(&'a Vec<MbcCapability>),
}

#[derive(Debug, Clone, Copy)]
pub enum MbcCapability {
    Ram,
    Battery,
    Timer,
    Rumble,
}
