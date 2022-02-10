use crate::mmu::MemoryAccess;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DmaMode {
    Gdma,
    Hdma,
}

#[derive(Default)]
pub struct Dma {
    // hdma1..hdma4
    pub regs: [u8; 4],

    // hdma5
    // transfer length (0x00..=0x7F)
    pub dma_length: u8,
    // 0 -> general purpose dma
    // 1 -> hblank dma
    pub dma_mode: Option<DmaMode>,

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
                if let Some(DmaMode::Hdma) = self.dma_mode {
                    if value & 0x80 == 0 {
                        self.dma_mode = None;
                    }
                    return;
                }

                self.src = ((self.regs[0] as u16) << 8) | (self.regs[1] as u16);
                assert!(self.src <= 0x7FF0 || (self.src >= 0xA000 && self.src <= 0xDFF0));
                self.dst = ((self.regs[2] as u16) << 8) | (self.regs[3] as u16) | 0x8000;

                self.dma_mode = Some(match value & 0x80 {
                    0x80 => DmaMode::Hdma,
                    _ => DmaMode::Gdma,
                });
                self.dma_length = value & 0x7F;
            }
            _ => unreachable!(),
        };
    }

    fn mem_read(&self, addr: u16) -> u8 {
        match addr {
            0xff51..=0xff54 => self.regs[(addr - 0xFF51) as usize],
            0xff55 => {
                self.dma_length
                    | if let Some(DmaMode::Hdma) = self.dma_mode {
                        0x80
                    } else {
                        0x00
                    }
            }
            _ => unreachable!(),
        }
    }
}
