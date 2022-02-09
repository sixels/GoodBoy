use std::hint::unreachable_unchecked;

use crate::mmu::MemoryAccess;

#[derive(Debug)]
pub struct Timer {
    is_enabled: bool,
    step: u32,
    div: u8,
    counter: u8,
    modulo: u8,

    div_clocks: u32,
    clocks: u32,

    pub interrupt: u8,
}

impl Timer {
    pub fn sync(&mut self, clocks: u32) {
        self.div_clocks += clocks;
        while self.div_clocks >= 256 {
            self.div = self.div.wrapping_add(1);
            self.div_clocks -= 256;
        }

        if self.is_enabled {
            self.clocks += clocks;

            while self.clocks >= self.step {
                self.counter = self.counter.wrapping_add(1);

                if self.counter == 0 {
                    self.counter = self.modulo;
                    self.interrupt |= 0x04;
                }
                self.clocks -= self.step;
            }
        }
    }
}

impl MemoryAccess for Timer {
    fn mem_read(&self, addr: u16) -> u8 {
        match addr {
            0xFF04 => self.div,
            0xFF05 => self.counter,
            0xFF06 => self.modulo,
            0xFF07 => {
                (if self.is_enabled { 0x04 } else { 0x00 })
                    | (match self.step {
                        16 => 1,
                        64 => 2,
                        256 => 3,
                        1024 => 0,
                        _ => {
                            // SAFETY:
                            // `self.step` is guaranteed to always be either one of the following
                            // values 16, 64, 256, or 1024, because it is already initialized as 256
                            // and its default value is only writen in `self.mem_write()` function,
                            // with a strict possibility of values defined by a `match` block.
                            unsafe { unreachable_unchecked() }
                        }
                    })
            }
            _ => panic!("Reading an invalid Timer address: {addr}"),
        }
    }
    fn mem_write(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF04 => self.div = 0,
            0xFF05 => self.counter = value,
            0xFF06 => self.modulo = value,
            0xFF07 => {
                self.is_enabled = value & 0x04 != 0;
                self.step = match value & 0x03 {
                    1 => 16,
                    2 => 64,
                    3 => 256,
                    0 => 1024,
                    _ => {
                        // SAFETY:
                        // `value` will always be a number within 0 and 3 because of the bitwise-and
                        // operation on bits 0 and 1.
                        unsafe { unreachable_unchecked() }
                    }
                };
            }
            _ => panic!("Writing to invalid Timer address: {addr}"),
        }
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self {
            is_enabled: false,
            step: 256,
            div: 0,
            counter: 0,
            modulo: 0,
            div_clocks: 0,
            clocks: 0,
            interrupt: 0,
        }
    }
}
