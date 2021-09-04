use super::flags::CpuFlag;

#[derive(Debug, Default)]
pub struct Registers {
    /// Accumulator
    pub a: u8,
    /// Flags
    pub f: CpuFlag,
    /// BC
    pub b: u8,
    pub c: u8,
    /// DE
    pub d: u8,
    pub e: u8,
    /// HL
    pub h: u8,
    pub l: u8,
}

impl Registers {
    #[inline(always)]
    pub const fn bc(&self) -> u16 {
        u16::from_be_bytes([self.b, self.c])
    }
    #[inline(always)]
    pub const fn de(&self) -> u16 {
        u16::from_be_bytes([self.d, self.e])
    }
    #[inline(always)]
    pub const fn hl(&self) -> u16 {
        u16::from_be_bytes([self.h, self.l])
    }
    #[inline(always)]
    pub const fn af(&self) -> u16 {
        u16::from_be_bytes([self.a, self.f.bits()])
    }

    #[inline(always)]
    pub fn set_bc(&mut self, value: u16) {
        [self.b, self.c] = value.to_be_bytes();
    }
    #[inline(always)]
    pub fn set_de(&mut self, value: u16) {
        [self.d, self.e] = value.to_be_bytes();
    }
    #[inline(always)]
    pub fn set_hl(&mut self, value: u16) {
        [self.h, self.l] = value.to_be_bytes();
    }
    #[inline(always)]
    pub fn set_af(&mut self, value: u16) {
        let bytes = value.to_be_bytes();
        self.a = bytes[0];
        self.f = CpuFlag::from_bits(bytes[1]).unwrap()
    }

    pub fn hli(&mut self) -> u16 {
        let hl = self.hl();
        [self.h, self.l] = hl.wrapping_add(1).to_be_bytes();
        hl
    }
    pub fn hld(&mut self) -> u16 {
        let hl = self.hl();
        [self.h, self.l] = hl.wrapping_sub(1).to_be_bytes();
        hl
    }
}
