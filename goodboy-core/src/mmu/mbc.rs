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
    /// Read a byte from ROM
    /// `addr` must be within (0x0000, 0x8000]
    fn rom_read(&self, addr: u16) -> u8 {
        0
    }
    /// Read a byte from RAM
    /// `addr` must be within (0xA000, 0xC000]
    fn ram_read(&self, addr: u16) -> u8 {
        0
    }

    /// Write a byte to ROM
    /// `addr` must be within (0x0000, 0x8000]
    fn rom_write(&mut self, addr: u16, value: u8) {}
    /// Write a byte from RAM
    /// `addr` must be within (0xA000, 0xC000]
    fn ram_write(&mut self, addr: u16, value: u8) {}

    /// Return the MBC type description
    fn description(&self) -> Option<MbcDescription<'_>> {
        None
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MbcDescription<'a> {
    MBC0,
    MBC1(&'a Vec<MbcCapability>),
    MBC3(&'a Vec<MbcCapability>),
    MBC5(&'a Vec<MbcCapability>),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MbcCapability {
    Ram,
    Battery,
    Timer,
    Rumble,
}
