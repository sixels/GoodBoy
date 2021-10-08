mod mbc0;
mod mbc1;
mod mbc3;

pub use mbc0::Mbc0;
pub use mbc1::Mbc1;
pub use mbc3::Mbc3;

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
}
