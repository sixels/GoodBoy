use crate::mmu::MemoryAccess;

#[derive(Default)]
pub struct Dma {
    // hdma1..hdma4
    regs: [u8; 4],

    // hdma5
    // transfer length (0x00..=0x7F)
    dma_length: u8,
    // 0 -> general purpose dma
    // 1 -> hblank dma
    dma_mode: u8,
    // if the dma should start
    dma_start: bool,
}

impl MemoryAccess for Dma {
    fn mem_write(&mut self, addr: u16, value: u8) {
        let Self {
            regs,
            dma_start,
            dma_mode,
            dma_length,
        } = self;

        match addr {
            0xff51 => regs[0] = value & 0xFF,
            0xff52 => regs[1] = value & 0xF0,
            0xff53 => regs[2] = value & 0x1F,
            0xff54 => regs[3] = value & 0xF0,
            0xff55 => {
                *dma_start = true;
                *dma_mode = value & 0x80;
                *dma_length = value & 0x7F;
            }
            _ => unimplemented!(),
        };
    }

    fn mem_read(&self, addr: u16) -> u8 {
        match addr {
            0xff51..=0xff54 => self.regs[(addr & 0xF) as usize - 1],
            0xff55 => ((self.dma_mode as u8) << 7) | self.dma_length,
            _ => unimplemented!(),
        }
    }
}
