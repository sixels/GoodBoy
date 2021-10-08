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

    pub interrupt: bool,
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
                    self.interrupt = true;
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
                ((self.is_enabled as u8) << 2)
                    | match self.step {
                        16 => 1,
                        64 => 2,
                        256 => 3,
                        _ => 0,
                    }
            }
            _ => panic!("Invalid Timer address"),
        }
    }
    fn mem_write(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF04 => self.div = value,
            0xFF05 => self.counter = value,
            0xFF06 => self.modulo = value,
            0xFF07 => {
                self.is_enabled = (value >> 2) == 1;
                self.step = match value & 0x3 {
                    1 => 16,
                    2 => 64,
                    3 => 256,
                    _ => 1024,
                };
            }
            _ => panic!("Invalid Timer address"),
        }
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self {
            is_enabled: Default::default(),
            step: 256,
            div: Default::default(),
            counter: Default::default(),
            modulo: Default::default(),
            div_clocks: Default::default(),
            clocks: Default::default(),
            interrupt: Default::default(),
        }
    }
}
