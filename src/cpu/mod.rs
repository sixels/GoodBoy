mod flags;
pub mod registers;

use crate::memory::MemoryAccess;
use flags::CpuFlag;
use registers::Registers;

#[derive(Debug)]
pub struct Cpu {
    // CPU registers
    pub reg: Registers,

    /// Special purpose registers:
    /// Program Counter
    pub pc: u16,
    /// Stack Pointer
    pub sp: u16,

    memory: Box<[u8; 0xFFFF]>,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            reg: Registers::default(),
            pc: 0,
            sp: 0,
            memory: Box::new([0; 0xFFFF]),
        }
    }

    /// Create a new CPU with the given bootstrap rom buffer
    pub fn with_bootstrap(buffer: &[u8]) -> Self {
        let mut cpu = Self::new();

        let start_offset = 0x00;
        cpu.load(&buffer[0x00..=0xFF], start_offset);

        cpu
    }

    /// Load a slice into the ROM
    pub fn load(&mut self, slice: &[u8], start_offset: usize) {
        self.memory[start_offset..slice.len()].copy_from_slice(slice);
    }

    pub fn run(&mut self) -> ! {
        loop {
            self.tick();
        }
    }

    pub fn tick(&mut self) -> u8 {
        let opcode = self.fetch_byte();
        self.exec_opcode(opcode)
    }

    /// Get the next byte and increment the PC by 1.
    pub fn fetch_byte(&mut self) -> u8 {
        let pc = self.pc;
        let byte = self.mem_read(pc);
        self.pc += 1;
        byte
    }

    /// Get the next word and increment the PC by 2.
    pub fn fetch_word(&mut self) -> u16 {
        let word = self.mem_read_word(self.pc);
        self.pc += 2;
        word
    }

    fn push_stack(&mut self, value: u16) {
        self.sp -= 2;
        self.mem_write_word(self.sp, value)
    }

    fn pop_stack(&mut self) -> u16 {
        let w = self.mem_read_word(self.sp);
        self.sp += 2;
        w
    }

    #[rustfmt::skip]
    // Execute the next instruction returning its number of cycles
    fn exec_opcode(&mut self, opcode: u8) -> u8 {
        match opcode {
            // nop
            0x00 => 4,

            // --- LD INSTRUCTIONS ---

            // ld rr,u16
            0x01 => { let w = self.fetch_word(); self.reg.set_bc(w); 12 }
            0x11 => { let w = self.fetch_word(); self.reg.set_de(w); 12 }
            0x21 => { let w = self.fetch_word(); self.reg.set_hl(w); 12 }
            0x31 => { self.sp = self.fetch_word(); 12 }

            // ld (rr),a
            0x02 => { self.mem_write(self.reg.bc(), self.reg.a); 8 }
            0x12 => { self.mem_write(self.reg.de(), self.reg.a); 8 }
            0x22 => { let hl = self.reg.hli(); self.mem_write(hl, self.reg.a); 8 }
            0x32 => { let hl = self.reg.hld(); self.mem_write(hl, self.reg.a); 8 }

            // ld r,u8
            0x06 => { self.reg.b = self.fetch_byte(); 8 }
            0x16 => { self.reg.d = self.fetch_byte(); 8 }
            0x26 => { self.reg.h = self.fetch_byte(); 8 }
            0x36 => { let b = self.fetch_byte(); let hl = self.reg.hl(); self.mem_write(hl, b); 12 }
            0x0E => { self.reg.c = self.fetch_byte(); 8 }
            0x1E => { self.reg.e = self.fetch_byte(); 8 }
            0x2E => { self.reg.l = self.fetch_byte(); 8 }
            0x3E => { self.reg.a = self.fetch_byte(); 8 }

            // ld (u16),sp
            0x08 => { let w = self.fetch_word(); self.mem_write_word(w, self.sp); 20 }

            // ld a,(rr)
            0x0A => { self.reg.a = self.mem_read(self.reg.bc()); 8 }
            0x1A => { self.reg.a = self.mem_read(self.reg.de()); 8 }
            0x2A => { let hl = self.reg.hli(); self.reg.a = self.mem_read(hl); 8 }
            0x3A => { let hl = self.reg.hld(); self.reg.a = self.mem_read(hl); 8 }

            // ld b,r
            0x40 => 4,
            0x41 => { self.reg.b = self.reg.c; 4 }
            0x42 => { self.reg.b = self.reg.d; 4 }
            0x43 => { self.reg.b = self.reg.e; 4 }
            0x44 => { self.reg.b = self.reg.h; 4 }
            0x45 => { self.reg.b = self.reg.l; 4 }
            0x46 => { self.reg.b = self.mem_read(self.reg.hl()); 8 }
            0x47 => { self.reg.b = self.reg.a; 4 }
            // ld c,r
            0x48 => { self.reg.c = self.reg.b; 4 }
            0x49 => 4,
            0x4A => { self.reg.c = self.reg.d; 4 }
            0x4B => { self.reg.c = self.reg.e; 4 }
            0x4C => { self.reg.c = self.reg.h; 4 }
            0x4D => { self.reg.c = self.reg.l; 4 }
            0x4E => { self.reg.c = self.mem_read(self.reg.hl()); 8 }
            0x4F => { self.reg.c = self.reg.a; 4 }

            // ld d,r
            0x50 => { self.reg.d = self.reg.b; 4 }
            0x51 => { self.reg.d = self.reg.c; 4 }
            0x52 => 4,
            0x53 => { self.reg.d = self.reg.e; 4 }
            0x54 => { self.reg.d = self.reg.h; 4 }
            0x55 => { self.reg.d = self.reg.l; 4 }
            0x56 => { self.reg.d = self.mem_read(self.reg.hl()); 8 }
            0x57 => { self.reg.d = self.reg.a; 4 }
            // ld e,r
            0x58 => { self.reg.e = self.reg.b; 4 }
            0x59 => { self.reg.e = self.reg.c; 4 }
            0x5A => { self.reg.e = self.reg.d; 4 }
            0x5B => 4,
            0x5C => { self.reg.e = self.reg.h; 4 }
            0x5D => { self.reg.e = self.reg.l; 4 }
            0x5E => { self.reg.e = self.mem_read(self.reg.hl()); 8 }
            0x5F => { self.reg.e = self.reg.a; 4 }

            // ld h,r
            0x60 => { self.reg.h = self.reg.b; 4 }
            0x61 => { self.reg.h = self.reg.c; 4 }
            0x62 => { self.reg.h = self.reg.d; 4 }
            0x63 => { self.reg.h = self.reg.e; 4 }
            0x64 => 4,
            0x65 => { self.reg.h = self.reg.l; 4 }
            0x66 => { self.reg.h = self.mem_read(self.reg.hl()); 8 }
            0x67 => { self.reg.h = self.reg.a; 4 }
            // ld l,r
            0x68 => { self.reg.l = self.reg.b; 4 }
            0x69 => { self.reg.l = self.reg.c; 4 }
            0x6A => { self.reg.l = self.reg.d; 4 }
            0x6B => { self.reg.l = self.reg.e; 4 }
            0x6C => { self.reg.l = self.reg.h; 4 }
            0x6D => 4,
            0x6E => { self.reg.l = self.mem_read(self.reg.hl()); 8 }
            0x6F => { self.reg.l = self.reg.a; 4 }

            // ld (hl),r
            0x70 => { self.mem_write(self.reg.hl(), self.reg.a); 8 },
            0x71 => { self.mem_write(self.reg.hl(), self.reg.c); 8 },
            0x72 => { self.mem_write(self.reg.hl(), self.reg.d); 8 },
            0x73 => { self.mem_write(self.reg.hl(), self.reg.e); 8 },
            0x74 => { self.mem_write(self.reg.hl(), self.reg.h); 8 },
            0x75 => { self.mem_write(self.reg.hl(), self.reg.l); 8 },
            0x77 => { self.mem_write(self.reg.hl(), self.reg.a); 8 },

            // ld a,r
            0x78 => { self.reg.a = self.reg.b; 4 }
            0x79 => { self.reg.a = self.reg.c; 4 }
            0x7A => { self.reg.a = self.reg.d; 4 }
            0x7B => { self.reg.a = self.reg.e; 4 }
            0x7C => { self.reg.a = self.reg.h; 4 }
            0x7D => { self.reg.a = self.reg.l; 4 }
            0x7E => { self.reg.a = self.mem_read(self.reg.hl()); 8 }
            0x7F => 4,

            // ld (ff00+u8),a
            0xE0 => { let b = self.fetch_byte(); self.mem_write(0xFF00 | b as u16, self.reg.a); 12 }
            // ld a,(ff00+u8)
            0xF0 => { let b = self.fetch_byte(); self.mem_write(self.reg.a as u16, self.mem_read(0xFF00 | b as u16)); 12 }
            // ld (ff00+c),a
            0xE2 => { self.mem_write(0xFF00 | self.reg.c as u16, self.reg.a); 8 }
            // ld a,(ff00+c)
            0xF2 => { self.mem_write(self.reg.a as u16, self.mem_read(0xFF00 | self.reg.c as u16)); 8 }

            // ld (u16),a
            0xEA => { let w = self.fetch_word(); self.mem_write(w, self.reg.a); 16 }
            // ld a,(u16)
            0xFA => { let w = self.fetch_word(); self.reg.a = self.mem_read(w); 16 }

            // --- BRANCH INSTRUCTIONS

            // jr i8
            0x18 => self.branch_jr(true),
            // jr z,i8
            0x28 => self.branch_jr(self.reg.f.contains(CpuFlag::Z)),
            // jr c,i8
            0x38 => self.branch_jr(self.reg.f.contains(CpuFlag::C)),
            // jr nz,i8
            0x20 => self.branch_jr(!self.reg.f.contains(CpuFlag::Z)),
            // jr nc,i8
            0x30 => self.branch_jr(!self.reg.f.contains(CpuFlag::C)),

            //jp u16
            0xC3 => self.branch_jp(true),
            // jp z,u16
            0xCA => self.branch_jp(self.reg.f.contains(CpuFlag::Z)),
            // jp c,u16
            0xDA => self.branch_jp(self.reg.f.contains(CpuFlag::C)),
            // jp nz,u16
            0xC2 => self.branch_jp(!self.reg.f.contains(CpuFlag::Z)),
            // jp nc,u16
            0xD2 => self.branch_jp(!self.reg.f.contains(CpuFlag::C)),

            // call u16
            0xCD => self.branch_call(true),
            0xCC => self.branch_call(self.reg.f.contains(CpuFlag::Z)),
            0xDC => self.branch_call(self.reg.f.contains(CpuFlag::C)),
            0xC4 => self.branch_call(!self.reg.f.contains(CpuFlag::Z)),
            0xD4 => self.branch_call(!self.reg.f.contains(CpuFlag::Z)),

            0xC9 => { self.pc = self.pop_stack(); 4 }

            // --- STORE INSTRUCTIONS ---

            // push rr
            0xC5 => { self.push_stack(self.reg.bc()); 16 }
            0xD5 => { self.push_stack(self.reg.de()); 16 }
            0xE5 => { self.push_stack(self.reg.hl()); 16 }
            0xF5 => { self.push_stack(self.reg.af()); 16 }

            0xC1 => { let w = self.pop_stack(); self.reg.set_bc(w); 12 }
            0xD1 => { let w = self.pop_stack(); self.reg.set_de(w); 12 }
            0xE1 => { let w = self.pop_stack(); self.reg.set_hl(w); 12 }
            0xF1 => { let w = self.pop_stack(); self.reg.set_af(w); 12 }

            // --- ALU INSTRUCTIONS ---

            // inc r
            0x04 => { let cyc; (self.reg.b, cyc) = self.alu_inc(self.reg.b); cyc }
            0x14 => { let cyc; (self.reg.d, cyc) = self.alu_inc(self.reg.d); cyc }
            0x24 => { let cyc; (self.reg.h, cyc) = self.alu_inc(self.reg.h); cyc }
            0x34 => { let hl = self.reg.hl(); let (v, cyc) = self.alu_inc(self.mem_read(hl));
                self.mem_write(hl, v); cyc + 8 }
            0x0C => { let cyc; (self.reg.c, cyc) = self.alu_inc(self.reg.c); cyc }
            0x1C => { let cyc; (self.reg.e, cyc) = self.alu_inc(self.reg.e); cyc }
            0x2C => { let cyc; (self.reg.l, cyc) = self.alu_inc(self.reg.l); cyc }
            0x3C => { let cyc; (self.reg.a, cyc) = self.alu_inc(self.reg.a); cyc }

            // inc rr
            0x03 => { let w = self.reg.bc().wrapping_add(1); self.reg.set_bc(w); 8 }
            0x13 => { let w = self.reg.de().wrapping_add(1); self.reg.set_de(w); 8 }
            0x23 => { let w = self.reg.hl().wrapping_add(1); self.reg.set_hl(w); 8 }
            0x33 => { self.sp = self.sp.wrapping_add(1); 8 }

            // dec r
            0x05 => { let cyc; (self.reg.b, cyc) = self.alu_dec(self.reg.b); cyc }
            0x15 => { let cyc; (self.reg.d, cyc) = self.alu_dec(self.reg.d); cyc }
            0x25 => { let cyc; (self.reg.h, cyc) = self.alu_dec(self.reg.h); cyc }
            0x35 => { let hl = self.reg.hl(); let (v, cyc) = self.alu_dec(self.mem_read(hl));
                self.mem_write(hl, v); cyc + 8 }
            0x0D => { let cyc; (self.reg.c, cyc) = self.alu_dec(self.reg.c); cyc }
            0x1D => { let cyc; (self.reg.e, cyc) = self.alu_dec(self.reg.e); cyc }
            0x2D => { let cyc; (self.reg.l, cyc) = self.alu_dec(self.reg.l); cyc }
            0x3D => { let cyc; (self.reg.a, cyc) = self.alu_dec(self.reg.a); cyc }

            // rla
            0x07 => { self.reg.a = self.alu_rl(self.reg.a).0; 4 }
            // rlca
            0x17 => { self.reg.a = self.alu_rlc(self.reg.a).0; 4 }
            // rra
            0x0F => { self.reg.a = self.alu_rr(self.reg.a).0; 4 }
            // rrca
            0x1F => { self.reg.a = self.alu_rrc(self.reg.a).0; 4 }
         
            // add a,u8
            0x80 => self.alu_add(self.reg.b, false),
            0x81 => self.alu_add(self.reg.c, false),
            0x82 => self.alu_add(self.reg.d, false),
            0x83 => self.alu_add(self.reg.e, false),
            0x84 => self.alu_add(self.reg.h, false),
            0x85 => self.alu_add(self.reg.l, false),
            0x86 => self.alu_add(self.mem_read(self.reg.hl()), false) + 4,
            0x87 => self.alu_add(self.reg.a, false),
            0xC6 => { let b = self.fetch_byte(); self.alu_add(b, false) + 4 }

            // adc a,u8
            0x88 => self.alu_add(self.reg.b, true),
            0x89 => self.alu_add(self.reg.c, true),
            0x8A => self.alu_add(self.reg.d, true),
            0x8B => self.alu_add(self.reg.e, true),
            0x8C => self.alu_add(self.reg.h, true),
            0x8D => self.alu_add(self.reg.l, true),
            0x8E => self.alu_add(self.mem_read(self.reg.hl()), true) + 4,
            0x8F => self.alu_add(self.reg.a, true),
            0xCE => { let b = self.fetch_byte(); self.alu_add(b, true) + 4 }

            // sub a,u8
            0x90 => self.alu_sub(self.reg.b, false),
            0x91 => self.alu_sub(self.reg.c, false),
            0x92 => self.alu_sub(self.reg.d, false),
            0x93 => self.alu_sub(self.reg.e, false),
            0x94 => self.alu_sub(self.reg.h, false),
            0x95 => self.alu_sub(self.reg.l, false),
            0x96 => self.alu_sub(self.mem_read(self.reg.hl()), false) + 4,
            0x97 => self.alu_sub(self.reg.a, false),
            0xD6 => { let b = self.fetch_byte(); self.alu_sub(b, false) + 4 }

            // sbc a,u8
            0x98 => self.alu_sub(self.reg.b, true),
            0x99 => self.alu_sub(self.reg.c, true),
            0x9A => self.alu_sub(self.reg.d, true),
            0x9B => self.alu_sub(self.reg.e, true),
            0x9C => self.alu_sub(self.reg.h, true),
            0x9D => self.alu_sub(self.reg.l, true),
            0x9E => self.alu_sub(self.mem_read(self.reg.hl()), true) + 4,
            0x9F => self.alu_sub(self.reg.a, true),
            0xDE => { let b = self.fetch_byte(); self.alu_sub(b, true) + 4 }

            // and a,u8
            0xA0 => self.alu_and(self.reg.b),
            0xA1 => self.alu_and(self.reg.c),
            0xA2 => self.alu_and(self.reg.d),
            0xA3 => self.alu_and(self.reg.e),
            0xA4 => self.alu_and(self.reg.h),
            0xA5 => self.alu_and(self.reg.l),
            0xA6 => self.alu_and(self.mem_read(self.reg.hl())) + 4,
            0xA7 => self.alu_and(self.reg.a),
            0xE6 => { let b = self.fetch_byte(); self.alu_and(b) + 4 }

            // xor a,u8
            0xA8 => self.alu_xor(self.reg.b),
            0xA9 => self.alu_xor(self.reg.c),
            0xAA => self.alu_xor(self.reg.d),
            0xAB => self.alu_xor(self.reg.e),
            0xAC => self.alu_xor(self.reg.h),
            0xAD => self.alu_xor(self.reg.l),
            0xAE => self.alu_xor(self.mem_read(self.reg.hl())) + 4,
            0xAF => self.alu_xor(self.reg.a),
            0xEE => { let b = self.fetch_byte(); self.alu_xor(b) + 4 }

            // or a,u8
            0xB0 => self.alu_or(self.reg.b),
            0xB1 => self.alu_or(self.reg.c),
            0xB2 => self.alu_or(self.reg.d),
            0xB3 => self.alu_or(self.reg.e),
            0xB4 => self.alu_or(self.reg.h),
            0xB5 => self.alu_or(self.reg.l),
            0xB6 => self.alu_or(self.mem_read(self.reg.hl())) + 4,
            0xB7 => self.alu_or(self.reg.a),
            0xF6 => { let b = self.fetch_byte(); self.alu_or(b) + 4 }

            // cp a,u8
            0xB8 => self.alu_cp(self.reg.b),
            0xB9 => self.alu_cp(self.reg.c),
            0xBA => self.alu_cp(self.reg.d),
            0xBB => self.alu_cp(self.reg.e),
            0xBC => self.alu_cp(self.reg.h),
            0xBD => self.alu_cp(self.reg.l),
            0xBE => self.alu_cp(self.mem_read(self.reg.hl())) + 4,
            0xBF => self.alu_cp(self.reg.a),
            0xFE => { let b = self.fetch_byte(); self.alu_cp(b) + 4 }

            // --- PREFIXED INSTRUCTIONS ---

            0xCB => { let op = self.fetch_byte(); self.exec_cb_opcode(op) },

            opcode => unimplemented!(
                "Unknown opcode at \x1B[1m0x{:04X}\x1B[0m: \x1B[31;1m0x{:02X}\x1B[0m.",
                self.pc.saturating_sub(1),
                opcode
            ),
        }
    }

    #[rustfmt::skip]
    fn exec_cb_opcode(&mut self, opcode: u8) -> u8 {
        match opcode {
            // rlc r
            0x00 => { let cyc; (self.reg.b, cyc) = self.alu_rlc(self.reg.b); cyc }
            0x01 => { let cyc; (self.reg.c, cyc) = self.alu_rlc(self.reg.c); cyc }
            0x02 => { let cyc; (self.reg.d, cyc) = self.alu_rlc(self.reg.d); cyc }
            0x03 => { let cyc; (self.reg.e, cyc) = self.alu_rlc(self.reg.e); cyc }
            0x04 => { let cyc; (self.reg.h, cyc) = self.alu_rlc(self.reg.h); cyc }
            0x05 => { let cyc; (self.reg.l, cyc) = self.alu_rlc(self.reg.l); cyc }
            0x06 => { let hl = self.reg.hl(); let (v, cyc) = self.alu_rlc(self.mem_read(hl));
                self.mem_write(hl, v); cyc + 8 }
            0x07 => { let cyc; (self.reg.a, cyc) = self.alu_rlc(self.reg.a); cyc }
            // rrc r
            0x08 => { let cyc; (self.reg.b, cyc) = self.alu_rrc(self.reg.b); cyc }
            0x09 => { let cyc; (self.reg.c, cyc) = self.alu_rrc(self.reg.c); cyc }
            0x0A => { let cyc; (self.reg.d, cyc) = self.alu_rrc(self.reg.d); cyc }
            0x0B => { let cyc; (self.reg.e, cyc) = self.alu_rrc(self.reg.e); cyc }
            0x0C => { let cyc; (self.reg.h, cyc) = self.alu_rrc(self.reg.h); cyc }
            0x0D => { let cyc; (self.reg.l, cyc) = self.alu_rrc(self.reg.l); cyc }
            0x0E => { let hl = self.reg.hl(); let (v, cyc) = self.alu_rrc(self.mem_read(hl));
                self.mem_write(hl, v); cyc + 8 }
            0x0F => { let cyc; (self.reg.a, cyc) = self.alu_rrc(self.reg.a); cyc }

            // rl r
            0x10 => { let cyc; (self.reg.b, cyc) = self.alu_rl(self.reg.b); cyc }
            0x11 => { let cyc; (self.reg.c, cyc) = self.alu_rl(self.reg.c); cyc }
            0x12 => { let cyc; (self.reg.d, cyc) = self.alu_rl(self.reg.d); cyc }
            0x13 => { let cyc; (self.reg.e, cyc) = self.alu_rl(self.reg.e); cyc }
            0x14 => { let cyc; (self.reg.h, cyc) = self.alu_rl(self.reg.h); cyc }
            0x15 => { let cyc; (self.reg.l, cyc) = self.alu_rl(self.reg.l); cyc }
            0x16 => { let hl = self.reg.hl(); let (v, cyc) = self.alu_rl(self.mem_read(hl));
                self.mem_write(hl, v); cyc + 8 }
            0x17 => { let cyc; (self.reg.a, cyc) = self.alu_rl(self.reg.a); cyc }
            // rr r
            0x18 => { let cyc; (self.reg.b, cyc) = self.alu_rr(self.reg.b); cyc }
            0x19 => { let cyc; (self.reg.c, cyc) = self.alu_rr(self.reg.c); cyc }
            0x1A => { let cyc; (self.reg.d, cyc) = self.alu_rr(self.reg.d); cyc }
            0x1B => { let cyc; (self.reg.e, cyc) = self.alu_rr(self.reg.e); cyc }
            0x1C => { let cyc; (self.reg.h, cyc) = self.alu_rr(self.reg.h); cyc }
            0x1D => { let cyc; (self.reg.l, cyc) = self.alu_rr(self.reg.l); cyc }
            0x1E => { let hl = self.reg.hl(); let (v, cyc) = self.alu_rr(self.mem_read(hl));
                self.mem_write(hl, v); cyc + 8 }
            0x1F => { let cyc; (self.reg.a, cyc) = self.alu_rr(self.reg.a); cyc }

            // bit 0,r
            0x40 => self.alu_bit(0, self.reg.b),
            0x41 => self.alu_bit(0, self.reg.c),
            0x42 => self.alu_bit(0, self.reg.d),
            0x43 => self.alu_bit(0, self.reg.e),
            0x44 => self.alu_bit(0, self.reg.h),
            0x45 => self.alu_bit(0, self.reg.l),
            0x46 => self.alu_bit(0, self.mem_read(self.reg.hl())) + 4,
            0x47 => self.alu_bit(0, self.reg.a),
            // bit 1,r
            0x48 => self.alu_bit(1, self.reg.b),
            0x49 => self.alu_bit(1, self.reg.c),
            0x4A => self.alu_bit(1, self.reg.d),
            0x4B => self.alu_bit(1, self.reg.e),
            0x4C => self.alu_bit(1, self.reg.h),
            0x4D => self.alu_bit(1, self.reg.l),
            0x4E => self.alu_bit(1, self.mem_read(self.reg.hl())) + 4,
            0x4F => self.alu_bit(1, self.reg.a),
            // bit 2,r
            0x50 => self.alu_bit(2, self.reg.b),
            0x51 => self.alu_bit(2, self.reg.c),
            0x52 => self.alu_bit(2, self.reg.d),
            0x53 => self.alu_bit(2, self.reg.e),
            0x54 => self.alu_bit(2, self.reg.h),
            0x55 => self.alu_bit(2, self.reg.l),
            0x56 => self.alu_bit(2, self.mem_read(self.reg.hl())) + 4,
            0x57 => self.alu_bit(2, self.reg.a),
            // bit 3,r
            0x58 => self.alu_bit(3, self.reg.b),
            0x59 => self.alu_bit(3, self.reg.c),
            0x5A => self.alu_bit(3, self.reg.d),
            0x5B => self.alu_bit(3, self.reg.e),
            0x5C => self.alu_bit(3, self.reg.h),
            0x5D => self.alu_bit(3, self.reg.l),
            0x5E => self.alu_bit(3, self.mem_read(self.reg.hl())) + 4,
            0x5F => self.alu_bit(3, self.reg.a),
            // bit 4,r
            0x60 => self.alu_bit(4, self.reg.b),
            0x61 => self.alu_bit(4, self.reg.c),
            0x62 => self.alu_bit(4, self.reg.d),
            0x63 => self.alu_bit(4, self.reg.e),
            0x64 => self.alu_bit(4, self.reg.h),
            0x65 => self.alu_bit(4, self.reg.l),
            0x66 => self.alu_bit(4, self.mem_read(self.reg.hl())) + 4,
            0x67 => self.alu_bit(4, self.reg.a),
            //bit 5,r
            0x68 => self.alu_bit(5, self.reg.b),
            0x69 => self.alu_bit(5, self.reg.c),
            0x6A => self.alu_bit(5, self.reg.d),
            0x6B => self.alu_bit(5, self.reg.e),
            0x6C => self.alu_bit(5, self.reg.h),
            0x6D => self.alu_bit(5, self.reg.l),
            0x6E => self.alu_bit(5, self.mem_read(self.reg.hl())) + 4,
            0x6F => self.alu_bit(5, self.reg.a),
            // bit 6,r
            0x70 => self.alu_bit(6, self.reg.b),
            0x71 => self.alu_bit(6, self.reg.c),
            0x72 => self.alu_bit(6, self.reg.d),
            0x73 => self.alu_bit(6, self.reg.e),
            0x74 => self.alu_bit(6, self.reg.h),
            0x75 => self.alu_bit(6, self.reg.l),
            0x76 => self.alu_bit(6, self.mem_read(self.reg.hl())) + 4,
            0x77 => self.alu_bit(6, self.reg.a),
            // bit 7,r
            0x78 => self.alu_bit(7, self.reg.b),
            0x79 => self.alu_bit(7, self.reg.c),
            0x7A => self.alu_bit(7, self.reg.d),
            0x7B => self.alu_bit(7, self.reg.e),
            0x7C => self.alu_bit(7, self.reg.h),
            0x7D => self.alu_bit(7, self.reg.l),
            0x7E => self.alu_bit(7, self.mem_read(self.reg.hl())) + 4,
            0x7F => self.alu_bit(7, self.reg.a),

            // res 0,r
            0x80 => { self.reg.b = self.reg.b & !(1<<0); 8 }
            0x81 => { self.reg.c = self.reg.c & !(1<<0); 8 }
            0x82 => { self.reg.d = self.reg.d & !(1<<0); 8 }
            0x83 => { self.reg.e = self.reg.e & !(1<<0); 8 }
            0x84 => { self.reg.h = self.reg.h & !(1<<0); 8 }
            0x85 => { self.reg.l = self.reg.l & !(1<<0); 8 }
            0x86 => { let hl = self.reg.hl(); let v = self.mem_read(hl) & !(1<<0); self.mem_write(hl, v); 16 }
            0x87 => { self.reg.a = self.reg.a & !(1<<0); 8 }
            // res 1,r
            0x88 => { self.reg.b = self.reg.b & !(1<<1); 8 }
            0x89 => { self.reg.c = self.reg.c & !(1<<1); 8 }
            0x8A => { self.reg.d = self.reg.d & !(1<<1); 8 }
            0x8B => { self.reg.e = self.reg.e & !(1<<1); 8 }
            0x8C => { self.reg.h = self.reg.h & !(1<<1); 8 }
            0x8D => { self.reg.l = self.reg.l & !(1<<1); 8 }
            0x8E => { let hl = self.reg.hl(); let v = self.mem_read(hl) & !(1<<1); self.mem_write(hl, v); 16 }
            0x8F => { self.reg.a = self.reg.a & !(1<<1); 8 }
            // res 2,r
            0x90 => { self.reg.b = self.reg.b & !(1<<2); 8 }
            0x91 => { self.reg.c = self.reg.c & !(1<<2); 8 }
            0x92 => { self.reg.d = self.reg.d & !(1<<2); 8 }
            0x93 => { self.reg.e = self.reg.e & !(1<<2); 8 }
            0x94 => { self.reg.h = self.reg.h & !(1<<2); 8 }
            0x95 => { self.reg.l = self.reg.l & !(1<<2); 8 }
            0x96 => { let hl = self.reg.hl(); let v = self.mem_read(hl) & !(1<<2); self.mem_write(hl, v); 16 }
            0x97 => { self.reg.a = self.reg.a & !(1<<2); 8 }
            // res 3,r
            0x98 => { self.reg.b = self.reg.b & !(1<<3); 8 }
            0x99 => { self.reg.c = self.reg.c & !(1<<3); 8 }
            0x9A => { self.reg.d = self.reg.d & !(1<<3); 8 }
            0x9B => { self.reg.e = self.reg.e & !(1<<3); 8 }
            0x9C => { self.reg.h = self.reg.h & !(1<<3); 8 }
            0x9D => { self.reg.l = self.reg.l & !(1<<3); 8 }
            0x9E => { let hl = self.reg.hl(); let v = self.mem_read(hl) & !(1<<3); self.mem_write(hl, v); 16 }
            0x9F => { self.reg.a = self.reg.a & !(1<<3); 8 }
            // res 4,r
            0xA0 => { self.reg.b = self.reg.b & !(1<<4); 8 }
            0xA1 => { self.reg.c = self.reg.c & !(1<<4); 8 }
            0xA2 => { self.reg.d = self.reg.d & !(1<<4); 8 }
            0xA3 => { self.reg.e = self.reg.e & !(1<<4); 8 }
            0xA4 => { self.reg.h = self.reg.h & !(1<<4); 8 }
            0xA5 => { self.reg.l = self.reg.l & !(1<<4); 8 }
            0xA6 => { let hl = self.reg.hl(); let v = self.mem_read(hl) & !(1<<4); self.mem_write(hl, v); 16 }
            0xA7 => { self.reg.a = self.reg.a & !(1<<4); 8 }
            // res 5,r
            0xA8 => { self.reg.b = self.reg.b & !(1<<5); 8 }
            0xA9 => { self.reg.c = self.reg.c & !(1<<5); 8 }
            0xAA => { self.reg.d = self.reg.d & !(1<<5); 8 }
            0xAB => { self.reg.e = self.reg.e & !(1<<5); 8 }
            0xAC => { self.reg.h = self.reg.h & !(1<<5); 8 }
            0xAD => { self.reg.l = self.reg.l & !(1<<5); 8 }
            0xAE => { let hl = self.reg.hl(); let v = self.mem_read(hl) & !(1<<5); self.mem_write(hl, v); 16 }
            0xAF => { self.reg.a = self.reg.a & !(1<<5); 8 }
            // res 6,r
            0xB0 => { self.reg.b = self.reg.b & !(1<<6); 8 }
            0xB1 => { self.reg.c = self.reg.c & !(1<<6); 8 }
            0xB2 => { self.reg.d = self.reg.d & !(1<<6); 8 }
            0xB3 => { self.reg.e = self.reg.e & !(1<<6); 8 }
            0xB4 => { self.reg.h = self.reg.h & !(1<<6); 8 }
            0xB5 => { self.reg.l = self.reg.l & !(1<<6); 8 }
            0xB6 => { let hl = self.reg.hl(); let v = self.mem_read(hl) & !(1<<6); self.mem_write(hl, v); 16 }
            0xB7 => { self.reg.a = self.reg.a & !(1<<6); 8 }
            // res 7,r
            0xB8 => { self.reg.b = self.reg.b & !(1<<7); 8 }
            0xB9 => { self.reg.c = self.reg.c & !(1<<7); 8 }
            0xBA => { self.reg.d = self.reg.d & !(1<<7); 8 }
            0xBB => { self.reg.e = self.reg.e & !(1<<7); 8 }
            0xBC => { self.reg.h = self.reg.h & !(1<<7); 8 }
            0xBD => { self.reg.l = self.reg.l & !(1<<7); 8 }
            0xBE => { let hl = self.reg.hl(); let v = self.mem_read(hl) & !(1<<7); self.mem_write(hl, v); 16 }
            0xBF => { self.reg.a = self.reg.a & !(1<<7); 8 }

            // set 0,r
            0xC0 => { self.reg.b = self.reg.b | (1<<0); 8 }
            0xC1 => { self.reg.c = self.reg.c | (1<<0); 8 }
            0xC2 => { self.reg.d = self.reg.d | (1<<0); 8 }
            0xC3 => { self.reg.e = self.reg.e | (1<<0); 8 }
            0xC4 => { self.reg.h = self.reg.h | (1<<0); 8 }
            0xC5 => { self.reg.l = self.reg.l | (1<<0); 8 }
            0xC6 => { let hl = self.reg.hl(); let v = self.mem_read(hl) | (1<<0); self.mem_write(hl, v); 16 }
            0xC7 => { self.reg.a = self.reg.a | (1<<0); 8 }
            // set 1,r
            0xC8 => { self.reg.b = self.reg.b | (1<<1); 8 }
            0xC9 => { self.reg.c = self.reg.c | (1<<1); 8 }
            0xCA => { self.reg.d = self.reg.d | (1<<1); 8 }
            0xCB => { self.reg.e = self.reg.e | (1<<1); 8 }
            0xCC => { self.reg.h = self.reg.h | (1<<1); 8 }
            0xCD => { self.reg.l = self.reg.l | (1<<1); 8 }
            0xCE => { let hl = self.reg.hl(); let v = self.mem_read(hl) | (1<<1); self.mem_write(hl, v); 16 }
            0xCF => { self.reg.a = self.reg.a | (1<<1); 8 }
            // set 2,r
            0xD0 => { self.reg.b = self.reg.b | (1<<2); 8 }
            0xD1 => { self.reg.c = self.reg.c | (1<<2); 8 }
            0xD2 => { self.reg.d = self.reg.d | (1<<2); 8 }
            0xD3 => { self.reg.e = self.reg.e | (1<<2); 8 }
            0xD4 => { self.reg.h = self.reg.h | (1<<2); 8 }
            0xD5 => { self.reg.l = self.reg.l | (1<<2); 8 }
            0xD6 => { let hl = self.reg.hl(); let v = self.mem_read(hl) | (1<<2); self.mem_write(hl, v); 16 }
            0xD7 => { self.reg.a = self.reg.a | (1<<2); 8 }
            // set 3,r
            0xD8 => { self.reg.b = self.reg.b | (1<<3); 8 }
            0xD9 => { self.reg.c = self.reg.c | (1<<3); 8 }
            0xDA => { self.reg.d = self.reg.d | (1<<3); 8 }
            0xDB => { self.reg.e = self.reg.e | (1<<3); 8 }
            0xDC => { self.reg.h = self.reg.h | (1<<3); 8 }
            0xDD => { self.reg.l = self.reg.l | (1<<3); 8 }
            0xDE => { let hl = self.reg.hl(); let v = self.mem_read(hl) | (1<<3); self.mem_write(hl, v); 16 }
            0xDF => { self.reg.a = self.reg.a | (1<<3); 8 }
            // set 4,r
            0xE0 => { self.reg.b = self.reg.b | (1<<4); 8 }
            0xE1 => { self.reg.c = self.reg.c | (1<<4); 8 }
            0xE2 => { self.reg.d = self.reg.d | (1<<4); 8 }
            0xE3 => { self.reg.e = self.reg.e | (1<<4); 8 }
            0xE4 => { self.reg.h = self.reg.h | (1<<4); 8 }
            0xE5 => { self.reg.l = self.reg.l | (1<<4); 8 }
            0xE6 => { let hl = self.reg.hl(); let v = self.mem_read(hl) | (1<<4); self.mem_write(hl, v); 16 }
            0xE7 => { self.reg.a = self.reg.a | (1<<4); 8 }
            // set 5,r
            0xE8 => { self.reg.b = self.reg.b | (1<<5); 8 }
            0xE9 => { self.reg.c = self.reg.c | (1<<5); 8 }
            0xEA => { self.reg.d = self.reg.d | (1<<5); 8 }
            0xEB => { self.reg.e = self.reg.e | (1<<5); 8 }
            0xEC => { self.reg.h = self.reg.h | (1<<5); 8 }
            0xED => { self.reg.l = self.reg.l | (1<<5); 8 }
            0xEE => { let hl = self.reg.hl(); let v = self.mem_read(hl) | (1<<5); self.mem_write(hl, v); 16 }
            0xEF => { self.reg.a = self.reg.a | (1<<5); 8 }
            // set 6,r
            0xF0 => { self.reg.b = self.reg.b | (1<<6); 8 }
            0xF1 => { self.reg.c = self.reg.c | (1<<6); 8 }
            0xF2 => { self.reg.d = self.reg.d | (1<<6); 8 }
            0xF3 => { self.reg.e = self.reg.e | (1<<6); 8 }
            0xF4 => { self.reg.h = self.reg.h | (1<<6); 8 }
            0xF5 => { self.reg.l = self.reg.l | (1<<6); 8 }
            0xF6 => { let hl = self.reg.hl(); let v = self.mem_read(hl) | (1<<6); self.mem_write(hl, v); 16 }
            0xF7 => { self.reg.a = self.reg.a | (1<<6); 8 }
            // set 7,r
            0xF8 => { self.reg.b = self.reg.b | (1<<7); 8 }
            0xF9 => { self.reg.c = self.reg.c | (1<<7); 8 }
            0xFA => { self.reg.d = self.reg.d | (1<<7); 8 }
            0xFB => { self.reg.e = self.reg.e | (1<<7); 8 }
            0xFC => { self.reg.h = self.reg.h | (1<<7); 8 }
            0xFD => { self.reg.l = self.reg.l | (1<<7); 8 }
            0xFE => { let hl = self.reg.hl(); let v = self.mem_read(hl) | (1<<7); self.mem_write(hl, v); 16 }
            0xFF => { self.reg.a = self.reg.a | (1<<7); 8 }

            opcode => unimplemented!("Unknown \x1B[4mprefixed\x1B[0m opcode at \x1B[1m0x{:04X}\x1B[0m: \x1B[31;1m0xCB{:02X}\x1B[0m.", self.pc.saturating_sub(2), opcode)
        }
    }

    // --- Branch ---

    /// If `condition` is true, adds the next signed byte to PC (PC = PC + i8),
    /// otherwise, do nothing. \
    /// Returns the instruction cycles.
    fn branch_jr(&mut self, condition: bool) -> u8 {
        let offset = self.fetch_byte() as i8;
        condition
            .then(|| {
                self.pc = self.pc.wrapping_add(offset as u16);
                12
            })
            .unwrap_or(8)
    }

    /// If `condition` is true, jump to the offset denoted by the next word (PC = u16),
    /// otherwise, do nothing. \
    /// Returns the instruction cycles.
    fn branch_jp(&mut self, condition: bool) -> u8 {
        let offset = self.fetch_word();
        condition
            .then(|| {
                self.pc = offset;
                16
            })
            .unwrap_or(12)
    }

    /// If `condition` is true, save the address of the next instruction onto the stack,
    /// then jump to the address denoted by the next word, otherwise, do nothing. \
    /// Returns the instruction cycles.
    fn branch_call(&mut self, condition: bool) -> u8 {
        condition
            .then(|| {
                self.push_stack(self.pc);
                self.branch_jp(true);
                24
            })
            .unwrap_or_else(|| {
                // skip the next word
                self.fetch_word();
                12
            })
    }

    // --- ALU ---

    /// Increment `r` returning its new value and the instruction cycles.
    ///
    /// # Flags affected
    ///
    /// Z: Set if result is 0 \
    /// N: 0 \
    /// H: Set if carry from bit 3
    fn alu_inc(&mut self, r: u8) -> (u8, u8) {
        let result = r.wrapping_add(1);

        self.reg.f.set(CpuFlag::Z, result == 0);
        self.reg.f.remove(CpuFlag::N);
        self.reg.f.set(CpuFlag::H, (r & 0xF) + 1 > 0xF);

        (result, 4)
    }

    /// Decrement `r` returning its new value and the instruction cycles.
    ///
    /// # Flags affected
    ///
    /// Z: Set if result is 0 \
    /// N: 1 \
    /// H: Set if no borrow from bit 4
    fn alu_dec(&mut self, r: u8) -> (u8, u8) {
        let result = r.wrapping_add(1);

        self.reg.f.set(CpuFlag::Z, result == 0);
        self.reg.f.insert(CpuFlag::N);
        self.reg.f.set(CpuFlag::H, (r & 0xF) == 0);

        (result, 4)
    }

    /// Add `n` + `adc` to A. \
    /// Returns the instruction cycles.
    ///
    /// # Flags affected
    ///
    /// Z: Set if result is 0 \
    /// N: 0 \
    /// H: Set if overflow from bit 3 \
    /// C: Set if overflow from bit 7
    fn alu_add(&mut self, n: u8, adc: bool) -> u8 {
        let a = self.reg.a;
        let c = adc as u8;
        let result = a.wrapping_add(n.wrapping_add(c));

        self.reg.f.set(CpuFlag::Z, result == 0);
        self.reg.f.remove(CpuFlag::N);
        self.reg.f.set(CpuFlag::H, (a & 0xF) + (n & 0xF) + c > 0xF);
        self.reg
            .f
            .set(CpuFlag::C, (a as u16) + (n as u16) + (c as u16) > 0xFF);

        self.reg.a = result;

        4
    }

    /// Subtract `n` + `adc` from A. \
    /// Returns the instruction cycles.
    ///
    /// # Flags affected
    ///
    /// Z: Set if result is 0 \
    /// N: 1 \
    /// H: Set if no borrow from bit 4 \
    /// C: Set if no borrow
    fn alu_sub(&mut self, n: u8, sbc: bool) -> u8 {
        let c = sbc as u8;
        let a = self.reg.a;
        let result = a.wrapping_sub(n.wrapping_add(c));

        self.reg.f.set(CpuFlag::Z, result == 0);
        self.reg.f.insert(CpuFlag::N);
        self.reg.f.set(CpuFlag::H, (a & 0xF) < (n & 0xF) + c);
        self.reg
            .f
            .set(CpuFlag::C, (a as u16) < (n as u16) + c as u16);

        self.reg.a = result;

        4
    }

    /// Logically AND `n` with A, result in A. \
    /// Returns the instruction cycles.
    ///
    /// # Flags affected
    ///
    /// Z: Set if result is 0 \
    /// N: 0 \
    /// H: 1 \
    /// C: 0
    fn alu_and(&mut self, n: u8) -> u8 {
        let result = self.reg.a & n;

        self.reg.f.set(CpuFlag::Z, result == 0);
        self.reg.f.remove(CpuFlag::N);
        self.reg.f.insert(CpuFlag::H);
        self.reg.f.remove(CpuFlag::C);

        self.reg.a = result;

        4
    }

    /// Logically XOR `n` with A, result in A. \
    /// Returns the instruction cycles.
    ///
    /// # Flags affected
    ///
    /// Z: Set if result is 0 \
    /// N: 0 \
    /// H: 0 \
    /// C: 0
    fn alu_xor(&mut self, n: u8) -> u8 {
        let result = self.reg.a ^ n;

        self.reg.f.set(CpuFlag::Z, result == 0);
        self.reg.f.remove(CpuFlag::N);
        self.reg.f.remove(CpuFlag::H);
        self.reg.f.remove(CpuFlag::C);

        self.reg.a = result;

        4
    }

    /// Logically OR `n` with register A, result in A. \
    /// Returns the instruction cycles.
    ///
    /// # Flags affected
    ///
    /// Z: Set if result is 0 \
    /// N: 0 \
    /// H: 0 \
    /// C: 0
    fn alu_or(&mut self, n: u8) -> u8 {
        let result = self.reg.a | n;

        self.reg.f.set(CpuFlag::Z, result == 0);
        self.reg.f.remove(CpuFlag::N);
        self.reg.f.remove(CpuFlag::H);
        self.reg.f.remove(CpuFlag::C);

        self.reg.a = result;

        4
    }

    /// Compare A with `n`. \
    /// Returns the instruction cycles.
    ///
    /// # Flags affected
    ///
    /// Z: Set if result is 0. (A == `n`) \
    /// N: 1 \
    /// H: Set if no borrow from bit 4 \
    /// C: Set for no borrow. (A < `n`)
    fn alu_cp(&mut self, n: u8) -> u8 {
        let a = self.reg.a;
        let result = a.wrapping_sub(n);

        self.reg.f.set(CpuFlag::Z, result == 0);
        self.reg.f.insert(CpuFlag::N);
        self.reg.f.set(CpuFlag::H, (a & 0xF) < (n & 0xF));
        self.reg.f.set(CpuFlag::C, a < n);

        4
    }

    /// Rotate `r` bits to the left. \
    /// Returns the new `r` value and the instruction cycles.
    /// ```not_rust
    ///  carry     r             carry    r << 1
    ///             ------->-------
    ///            |               |      
    ///   -      0b1010_1010       1     0b0101_0101
    ///                            |               |
    ///                             ------->-------
    /// ```
    ///
    /// # Flags affected
    ///
    /// Z: Set if result is 0
    /// N: 0
    /// H: 0
    /// C: Old `r` value's bit 7
    fn alu_rlc(&mut self, r: u8) -> (u8, u8) {
        let c = r & 0x80 == 0x80;

        let result = (r << 1) | c as u8;

        self.reg.f.set(CpuFlag::Z, result == 0);
        self.reg.f.remove(CpuFlag::N);
        self.reg.f.remove(CpuFlag::H);
        self.reg.f.set(CpuFlag::C, c);

        (result, 8)
    }

    /// Rotate `r` bits to the right. \
    /// Returns the new `r` value and the instruction cycles.
    /// ```not_rust
    ///  carry     r             carry    r >> 1
    ///                     --->---
    ///                    |       |      
    ///   -      0b1010_1010       0     0b0101_0100
    ///                            |               |
    ///                             ------->-------
    /// ```
    ///
    /// # Flags affected
    ///
    /// Z: Set if result is 0
    /// N: 0
    /// H: 0
    /// C: Old `r` value's bit 0
    fn alu_rrc(&mut self, r: u8) -> (u8, u8) {
        let c = r & 1;

        let result = (c << 7) | (r >> 1);

        self.reg.f.set(CpuFlag::Z, result == 0);
        self.reg.f.remove(CpuFlag::N);
        self.reg.f.remove(CpuFlag::H);
        self.reg.f.set(CpuFlag::C, c == 1);

        (result, 8)
    }

    /// Rotate `r` bits to the left through Carry. \
    /// Returns the new `r` value and the instruction cycles.
    /// ```not_rust
    ///  carry     r             carry     r << 1
    ///             ------->-------
    ///            |               |
    ///   1      0b1010_1010       1     0b0101_0101
    ///   |                                        |
    ///    ---------------------->-----------------
    /// ```
    ///
    /// # Flags affected
    ///
    /// Z: Set if result is 0
    /// N: 0
    /// H: 0
    /// C: Old `r` value's bit 7
    fn alu_rl(&mut self, r: u8) -> (u8, u8) {
        let c = r & 0x80 == 0x80;

        let result = (r << 1) | self.reg.f.contains(CpuFlag::C) as u8;

        self.reg.f.set(CpuFlag::Z, result == 0);
        self.reg.f.remove(CpuFlag::N);
        self.reg.f.remove(CpuFlag::H);
        self.reg.f.set(CpuFlag::C, c);

        (result, 8)
    }

    /// Rotate `r` bits to the right through Carry. \
    /// Returns the new `r` value and the instruction cycles.
    /// ```not_rust
    ///  carry     r             carry     r >> 1
    ///                     ---->---
    ///                    |        |
    ///   0      0b1010_1010        0     0b0101_0101
    ///   |                                 |
    ///    ----------------------->---------
    /// ```
    ///
    /// # Flags affected
    ///
    /// Z: Set if result is 0
    /// N: 0
    /// H: 0
    /// C: Old `r` value's bit 0
    fn alu_rr(&mut self, r: u8) -> (u8, u8) {
        let c = r & 1 == 1;

        let result = ((self.reg.f.contains(CpuFlag::C) as u8) << 7) | (r >> 1);

        self.reg.f.set(CpuFlag::Z, result == 0);
        self.reg.f.remove(CpuFlag::N);
        self.reg.f.remove(CpuFlag::H);
        self.reg.f.set(CpuFlag::C, c);

        (result, 8)
    }

    /// Test bit `b` in register `r`. \
    /// Returns the instruction cycles.
    ///
    /// # Flags affected
    ///
    /// Z: Set if bit `b` of register `r` is 0 \
    /// N: 0 \
    /// H: 1 \
    fn alu_bit(&mut self, b: u8, r: u8) -> u8 {
        let result = r & (1 << b);

        self.reg.f.set(CpuFlag::Z, result == 0);
        self.reg.f.remove(CpuFlag::N);
        self.reg.f.insert(CpuFlag::H);

        8
    }
}

impl Default for Cpu {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryAccess for Cpu {
    fn mem_read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }
    fn mem_write(&mut self, addr: u16, value: u8) {
        self.memory[addr as usize] = value
    }
}
