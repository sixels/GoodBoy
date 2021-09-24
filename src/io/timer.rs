use crate::memory::MemoryAccess;


#[derive(Clone, Copy, Debug)]
pub enum TimerFrequency {
    X1 = 0b00,
    X64 = 0b01,
    X16 = 0b10,
    X4 = 0b11,
}

#[derive(Default, Debug)]
pub struct Timer {
    is_enabled: bool,
    frequency: TimerFrequency,
    pub div: u8,
    pub counter: u8,
    pub modulo: u8,

    div_clocks: u8,
    clocks: u16,

    pub interrupt: bool,
}

impl Timer {
    pub fn sync(&mut self, clocks: u8) {
        // update the DIV register
        self.div_clocks = {
            let (div_clocks, did_overflow) = self.div_clocks.overflowing_add(clocks);
            if did_overflow {
                self.div = self.div.wrapping_add(1);
            }
            div_clocks
        };

        if self.is_enabled {
            self.clocks += clocks as u16;

            while self.clocks >= self.frequency.divided() {
                self.clocks -= self.frequency.divided();

                self.counter = self.counter.wrapping_add(1);

                if self.counter == 0 {
                    self.counter = self.modulo;
                    self.interrupt = true;
                }
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
            0xFF07 => ((self.is_enabled as u8) << 2) | self.frequency.bits(),
            _ => panic!("Invalid Timer address"),
        }
    }
    fn mem_write(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF04 => self.div = value,
            0xFF05 => self.counter = value,
            0xFF06 => self.modulo = value,
            0xFF07 => {
                // println!("{:05b}", value);
                self.is_enabled = (value >> 2) == 1;
                self.frequency = (value & 0x3).into();
            }
            _ => panic!("Invalid Timer address"),
        }
    }
}

impl TimerFrequency {
    /// get the CPU frequency divided by the Timer frequency
    const fn divided(&self) -> u16 {
        match self {
            Self::X1 => 1024,
            Self::X64 => 16,
            Self::X16 => 64,
            Self::X4 => 256,
        }
    }

    const fn bits(&self) -> u8 {
        match self {
            Self::X1 => 0b00,
            Self::X64 => 0b01,
            Self::X16 => 0b10,
            Self::X4 => 0b11,
        }
    }
}

impl Default for TimerFrequency {
    fn default() -> Self {
        Self::X1
    }
}

impl From<u8> for TimerFrequency {
    fn from(value: u8) -> TimerFrequency {
        match value {
            0b00 => TimerFrequency::X1,
            0b01 => TimerFrequency::X64,
            0b10 => TimerFrequency::X16,
            0b11 => TimerFrequency::X4,
            _ => panic!("Invalid frequency value"),
        }
    }
}
