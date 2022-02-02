use crate::mmu::MemoryAccess;

#[derive(Default)]
pub struct Dma {
    // hdma1..hdma4
    pub regs: [u8; 4],

    // hdma5
    // transfer length (0x00..=0x7F)
    pub dma_length: u8,
    // 0 -> general purpose dma
    // 1 -> hblank dma
    pub dma_mode: u8,
    // if the dma should start
    pub dma_start: bool,

    pub src: u16,
    pub dst: u16,
}

impl MemoryAccess for Dma {
    fn mem_write(&mut self, addr: u16, value: u8) {
        match addr {
            0xff51 => self.regs[0] = value & 0xFF,
            0xff52 => self.regs[1] = value & 0xF0,
            0xff53 => self.regs[2] = value & 0x1F,
            0xff54 => self.regs[3] = value & 0xF0,
            0xff55 => {
                if self.dma_start && self.dma_mode == 1 {
                    if value & 0x80 == 0 {
                        self.dma_start = false;
                    }
                    return;
                }
                self.dma_start = true;

                self.src = self.mem_read_word(0xFF51);
                assert!(self.src <= 0x7FF0 || (self.src >= 0xA000 && self.src <= 0xDFF0));
                self.dst = self.mem_read_word(0xFF53) | 0x8000;

                self.dma_mode = value & 0x80;
                self.dma_length = value & 0x7F;
            }
            _ => unreachable!(),
        };
    }

    fn mem_read(&self, addr: u16) -> u8 {
        match addr {
            0xff51..=0xff54 => self.regs[(addr & 0xF) as usize - 1],
            0xff55 => self.dma_length | (self.dma_mode << 7),
            _ => unreachable!(),
        }
    }
}
