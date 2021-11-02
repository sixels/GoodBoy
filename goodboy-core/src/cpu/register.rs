use crate::gb_mode::GbMode;

bitflags::bitflags! {
    #[derive(Default)]
    pub struct Flags: u8 {
        // Zero flag
        const Z = 1 << 7;
        // Add/Subtract flag
        const N = 1 << 6;
        // Half Carry flag
        const H = 1 << 5;
        // Carry flag
        const C = 1 << 4;
    }
}

impl From<u8> for Flags {
    fn from(byte: u8) -> Self {
        Flags::from_bits_truncate(byte)
    }
}

pub struct Registers {
    /// Accumulator
    pub a: u8,
    /// BC
    pub b: u8,
    pub c: u8,
    /// DE
    pub d: u8,
    pub e: u8,
    /// HL
    pub h: u8,
    pub l: u8,
    /// Flags
    pub f: Flags,
}

impl Registers {
    pub(crate) fn initialized(gb_mode: GbMode) -> Registers {
        let regs = Self {
            a: 0x01,
            b: 0x00,
            c: 0x13,
            d: 0x00,
            e: 0xD8,
            h: 0x01,
            l: 0x4D,
            f: Flags::from(0xB0),
        };

        if gb_mode == GbMode::CGB {
            return Self { a: 0x11, ..regs };
        }

        regs
    }

    pub fn bc(&self) -> u16 {
        u16::from_be_bytes([self.b, self.c])
    }
    pub fn de(&self) -> u16 {
        u16::from_be_bytes([self.d, self.e])
    }
    pub fn hl(&self) -> u16 {
        u16::from_be_bytes([self.h, self.l])
    }
    pub fn af(&self) -> u16 {
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
        self.f = Flags::from(bytes[1])
    }

    pub fn hli(&mut self) -> u16 {
        let hl = self.hl();
        self.set_hl(hl.wrapping_add(1));
        hl
    }
    pub fn hld(&mut self) -> u16 {
        let hl = self.hl();
        self.set_hl(hl.wrapping_sub(1));
        hl
    }
}

impl Default for Registers {
    fn default() -> Self {
        Self {
            a: 0x01,
            b: 0x00,
            c: 0x13,
            d: 0x00,
            e: 0xD8,
            h: 0x01,
            l: 0x4D,
            f: Flags::from(0xB0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_register() {
        let registers = Registers::default();

        assert_eq!(registers.af(), 0x01B0);
        assert_eq!(registers.bc(), 0x0013);
        assert_eq!(registers.de(), 0x00D8);
        assert_eq!(registers.hl(), 0x014D);
    }

    #[test]
    fn assign_register_pair() {
        let mut registers = Registers::default();

        registers.set_af(0x4200);
        assert_eq!(registers.a, 0x42);
        assert_eq!(registers.f.bits(), 0x00);

        registers.set_bc(0x2021);
        assert_eq!(registers.b, 0x20);
        assert_eq!(registers.c, 0x21);

        registers.set_de(0x6968);
        assert_eq!(registers.d, 0x69);
        assert_eq!(registers.e, 0x68);

        registers.set_hl(0x1234);
        assert_eq!(registers.h, 0x12);
        assert_eq!(registers.l, 0x34);
    }

    #[test]
    fn inc_dec_hl() {
        let mut registers = Registers::default();

        registers.set_hl(0xFFFF);
        assert_eq!(registers.hli(), 0xFFFF);
        assert_eq!(registers.hl(), 0x0000);

        assert_eq!(registers.hld(), 0x0000);
        assert_eq!(registers.hl(), 0xFFFF);

        registers.hli();
        registers.hli();
        assert_eq!(registers.hl(), 0x0001);
    }

    #[test]
    fn remove_insert_flags() {
        let mut f = Flags::default();

        f.insert(Flags::N | Flags::Z | Flags::H);

        assert!(f.contains(Flags::H));
        assert!(f.contains(Flags::H));
        assert!(f.contains(Flags::Z));

        f.remove(Flags::H | Flags::N);

        assert!(f.contains(Flags::Z));
        assert!(!f.contains(Flags::H));
        assert!(!f.contains(Flags::H));

        f.set(Flags::C, true);

        assert!(f.contains(Flags::C));
    }
}

impl std::fmt::Debug for Registers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("REGS")
            .field("AF", &self.af())
            .field("BC", &self.bc())
            .field("DE", &self.de())
            .field("HL", &self.hl())
            .finish()
    }
}
