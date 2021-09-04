mod flags;
pub mod registers;

use std::{hint::unreachable_unchecked, ops::Range};

use crate::memory::MemoryAccess;
use flags::CpuFlags;
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

        let rom_range = 0x00..0x100;
        cpu.load(buffer, rom_range);

        cpu
    }

    /// Load a slice into the ROM
    pub fn load(&mut self, slice: &[u8], range: Range<usize>) {
        self.memory[range].copy_from_slice(slice);
    }

    pub fn run(&mut self) -> ! {
        loop {
            self.tick();
        }
    }

    pub fn tick(&mut self) -> u8 {
        let opcode = self.next_byte();
        self.exec_opcode(opcode)
    }

    /// Get the next byte and increment the PC by 1.
    pub fn next_byte(&mut self) -> u8 {
        let pc = self.pc;
        let byte = self.mem_read(pc);
        self.pc += 1;
        byte
    }

    /// Get the next word and increment the PC by 2.
    pub fn next_word(&mut self) -> u16 {
        let word = self.mem_read_word(self.pc);
        self.pc += 2;
        word
    }

    // Execute the next instruction returning its number of cycles
    fn exec_opcode(&mut self, opcode: u8) -> u8 {
        match opcode {
            // nop
            0x00 => 4,

            // --- LD INSTRUCTIONS ---
            ld_rr_u16 @ (0x01 | 0x11 | 0x21 | 0x31) => {
                // Get the imediate word
                let im_word = self.next_word();

                match ld_rr_u16 {
                    // ld bc,u16
                    0x01 => self.reg.set_bc(im_word),
                    // ld de,u16
                    0x11 => self.reg.set_de(im_word),
                    // ld hl,u16
                    0x21 => self.reg.set_hl(im_word),
                    // ld sp,u16
                    0x31 => self.sp = im_word,
                    _ => unsafe { unreachable_unchecked() },
                };

                12
            }

            ld_mem_a @ (0x02 | 0x12 | 0x22 | 0x32) => {
                let addr = match ld_mem_a {
                    // ld (bc),a
                    0x02 => self.reg.bc(),
                    // ld (de),a
                    0x12 => self.reg.de(),
                    // ld (hl+),a
                    0x22 => self.reg.hli(),
                    // ld (hl-),a
                    0x32 => self.reg.hld(),
                    _ => unsafe { unreachable_unchecked() },
                };

                self.mem_write(addr, self.reg.a);

                8
            }

            ld_r_u8 @ (0x06 | 0x16 | 0x26 | 0x36) | ld_r_u8 @ (0x0E | 0x1E | 0x2E | 0x3E) => {
                let im_byte = self.next_byte();
                let mut cycles = 8;

                match ld_r_u8 {
                    // ld b,u8
                    0x06 => self.reg.b = im_byte,
                    // ld d,u8
                    0x16 => self.reg.d = im_byte,
                    // ld h,u8
                    0x26 => self.reg.h = im_byte,
                    // ld (hl),u8
                    0x36 => {
                        let hl = self.reg.hl();
                        self.mem_write(hl, im_byte);
                        cycles = 12;
                    }

                    // ld c,u8
                    0x0E => self.reg.c = im_byte,
                    // ld e,u8
                    0x1E => self.reg.e = im_byte,
                    // ld l,u8
                    0x2E => self.reg.l = im_byte,
                    // ld a,u8
                    0x3E => self.reg.a = im_byte,

                    _ => unsafe { unreachable_unchecked() },
                };

                cycles
            }

            // ld (u16),sp
            0x08 => {
                let im_word = self.next_word();
                self.mem_write_word(im_word, self.sp);

                20
            }

            ld_a_mem_rr @ (0x0A | 0x1A | 0x2A | 0x3A) => {
                let offset = match ld_a_mem_rr {
                    // ld a,(bc)
                    0x0A => self.reg.bc(),
                    // ld a,(de)
                    0x1A => self.reg.de(),
                    // ld a,(hl+)
                    0x2A => self.reg.hli(),
                    // ld a,(hl-)
                    0x3A => self.reg.hld(),

                    _ => unsafe { unreachable_unchecked() }
                };

                self.reg.a = self.mem_read(offset);

                8
            }

            // ld b,r
            ld_r_r @ (0x40..=0x47)
            // ld c,r
            | ld_r_r @ (0x48..=0x4F)
            // ld d,r
            | ld_r_r @ (0x50..=0x57)
            // ld e,r
            | ld_r_r @ (0x58..=0x5F)
            // ld h,r
            | ld_r_r @ (0x60..=0x67)
            // ld l,r
            | ld_r_r @ (0x68..=0x6F)
            // ld (hl),r
            | ld_r_r @ (0x70..=0x75 | 0x77)
            // ld a,r
            | ld_r_r @ (0x78..=0x7F) => {
                let mut cycles = 4;

                match ld_r_r {
                    // ld b,b
                    0x40 => (),
                    // ld b,c
                    0x41 => self.reg.b = self.reg.c,
                    // ld b,d
                    0x42 => self.reg.b = self.reg.d,
                    // ld b,e
                    0x43 => self.reg.b = self.reg.e,
                    // ld b,h
                    0x44 => self.reg.b = self.reg.h,
                    // ld b,l
                    0x45 => self.reg.b = self.reg.l,
                    // ld b,(hl)
                    0x46 => {
                        let hl = self.reg.hl();
                        self.reg.b = self.mem_read(hl);
                        cycles = 8
                    }
                    // ld b,a
                    0x47 => self.reg.b = self.reg.a,

                    // ld c,b
                    0x48 => self.reg.c = self.reg.b,
                    // ld c,c
                    0x49 => (),
                    // ld c,d
                    0x4A => self.reg.c = self.reg.d,
                    // ld c,e
                    0x4B => self.reg.c = self.reg.e,
                    // ld c,h
                    0x4C => self.reg.c = self.reg.h,
                    // ld c,l
                    0x4D => self.reg.c = self.reg.l,
                    // ld c,(hl)
                    0x4E => {
                        let hl = self.reg.hl();
                        self.reg.c = self.mem_read(hl);
                        cycles = 8
                    }
                    // ld c,a
                    0x4F => self.reg.c = self.reg.a,

                    // ld d,b
                    0x50 => self.reg.d = self.reg.b,
                    // ld d,c
                    0x51 => self.reg.d = self.reg.c,
                    // ld d,d
                    0x52 => (),
                    // ld d,e
                    0x53 => self.reg.d = self.reg.e,
                    // ld d,h
                    0x54 => self.reg.d = self.reg.h,
                    // ld d,l
                    0x55 => self.reg.d = self.reg.l,
                    // ld d,(hl)
                    0x56 => {
                        let hl = self.reg.hl();
                        self.reg.d = self.mem_read(hl);
                        cycles = 8
                    }
                    // ld d,a
                    0x57 => self.reg.d = self.reg.a,

                    // ld e,b
                    0x58 => self.reg.e = self.reg.b,
                    // ld e,c
                    0x59 => self.reg.e = self.reg.c,
                    // ld e,d
                    0x5A => self.reg.e = self.reg.d,
                    // ld e,e
                    0x5B => (),
                    // ld e,h
                    0x5C => self.reg.e = self.reg.h,
                    // ld e,l
                    0x5D => self.reg.e = self.reg.l,
                    // ld e,(hl)
                    0x5E => {
                        let hl = self.reg.hl();
                        self.reg.e = self.mem_read(hl);
                        cycles = 8
                    }
                    // ld e,a
                    0x5F => self.reg.e = self.reg.a,

                    // ld h,b
                    0x60 => self.reg.h = self.reg.a,
                    // ld h,c
                    0x61 => self.reg.h = self.reg.c,
                    // ld h,d
                    0x62 => self.reg.h = self.reg.d,
                    // ld h,e
                    0x63 => self.reg.h = self.reg.e,
                    // ld h,h
                    0x64 => (),
                    // ld h,l
                    0x65 => self.reg.h = self.reg.l,
                    // ld h,(hl)
                    0x66 => {
                        let hl = self.reg.hl();
                        self.reg.h = self.mem_read(hl);
                        cycles = 8
                    }
                    // ld h,a
                    0x67 => self.reg.h = self.reg.a,

                    // ld l,b
                    0x68 => self.reg.l = self.reg.b,
                    // ld l,c
                    0x69 => self.reg.l = self.reg.c,
                    // ld l,d
                    0x6A => self.reg.l = self.reg.d,
                    // ld l,e
                    0x6B => self.reg.l = self.reg.e,
                    // ld l,h
                    0x6C => self.reg.l = self.reg.h,
                    // ld l,l
                    0x6D => (),
                    // ld l,(hl)
                    0x6E => {
                        let hl = self.reg.hl();
                        self.reg.l = self.mem_read(hl);
                        cycles = 8
                    }
                    // ld l,a
                    0x6F => self.reg.l = self.reg.a,


                    ld_hl_r @ (0x70..=0x75 | 0x77) => {
                        let hl = self.reg.hl();

                        let r = match ld_hl_r {
                            // ld (hl),b
                            0x70 => self.reg.a,
                            // ld (hl),c
                            0x71 => self.reg.c,
                            // ld (hl),d
                            0x72 => self.reg.d,
                            // ld (hl),e
                            0x73 => self.reg.e,
                            // ld (hl),h
                            0x74 => self.reg.h,
                            // ld (hl),l
                            0x75 => self.reg.l,
                            // ld (hl),a
                            0x77 => self.reg.a,
                            _ =>  unsafe { unreachable_unchecked() }
                        };

                        self.mem_write(hl, r);
                        cycles = 8
                    }

                    // ld a,b
                    0x78 => self.reg.a = self.reg.b,
                    // ld a,c
                    0x79 => self.reg.a = self.reg.c,
                    // ld a,d
                    0x7A => self.reg.a = self.reg.d,
                    // ld a,e
                    0x7B => self.reg.a = self.reg.e,
                    // ld a,h
                    0x7C => self.reg.a = self.reg.h,
                    // ld a,l
                    0x7D => self.reg.a = self.reg.l,
                    // ld a,(hl)
                    0x7E => {
                      let hl = self.reg.hl();
                        self.reg.l = self.mem_read(hl);
                        cycles = 8
                    }
                    // ld a,a
                    0x7F => (),

                    _ => unsafe { unreachable_unchecked() },
                }

                cycles
            }
            // ---

            // --- ALU INSTRUCTIONS ---
            // add a,u8
            0x80 => self.alu_add(self.reg.b, false),
            0x81 => self.alu_add(self.reg.c, false),
            0x82 => self.alu_add(self.reg.d, false),
            0x83 => self.alu_add(self.reg.e, false),
            0x84 => self.alu_add(self.reg.h, false),
            0x85 => self.alu_add(self.reg.l, false),
            0x86 => self.alu_add(self.mem_read(self.reg.hl()), false) + 4,
            0x87 => self.alu_add(self.reg.a, false),

            // adc a,u8
            0x88 => self.alu_add(self.reg.b, true),
            0x89 => self.alu_add(self.reg.c, true),
            0x8A => self.alu_add(self.reg.d, true),
            0x8B => self.alu_add(self.reg.e, true),
            0x8C => self.alu_add(self.reg.h, true),
            0x8D => self.alu_add(self.reg.l, true),
            0x8E => self.alu_add(self.mem_read(self.reg.hl()), true) + 4,
            0x8F => self.alu_add(self.reg.a, true),

            // sub a,u8
            0x90 => self.alu_sub(self.reg.b, false),
            0x91 => self.alu_sub(self.reg.c, false),
            0x92 => self.alu_sub(self.reg.d, false),
            0x93 => self.alu_sub(self.reg.e, false),
            0x94 => self.alu_sub(self.reg.h, false),
            0x95 => self.alu_sub(self.reg.l, false),
            0x96 => self.alu_sub(self.mem_read(self.reg.hl()), false) + 4,
            0x97 => self.alu_sub(self.reg.a, false),

            // sbc a,u8
            0x98 => self.alu_sub(self.reg.b, true),
            0x99 => self.alu_sub(self.reg.c, true),
            0x9A => self.alu_sub(self.reg.d, true),
            0x9B => self.alu_sub(self.reg.e, true),
            0x9C => self.alu_sub(self.reg.h, true),
            0x9D => self.alu_sub(self.reg.l, true),
            0x9E => self.alu_sub(self.mem_read(self.reg.hl()), true) + 4,
            0x9F => self.alu_sub(self.reg.a, true),

            // and a,u8
            0xA0 => self.alu_and(self.reg.b),
            0xA1 => self.alu_and(self.reg.c),
            0xA2 => self.alu_and(self.reg.d),
            0xA3 => self.alu_and(self.reg.e),
            0xA4 => self.alu_and(self.reg.h),
            0xA5 => self.alu_and(self.reg.l),
            0xA6 => self.alu_and(self.mem_read(self.reg.hl())) + 4,
            0xA7 => self.alu_and(self.reg.a),

            // xor a,u8
            0xA8 => self.alu_xor(self.reg.b),
            0xA9 => self.alu_xor(self.reg.c),
            0xAA => self.alu_xor(self.reg.d),
            0xAB => self.alu_xor(self.reg.e),
            0xAC => self.alu_xor(self.reg.h),
            0xAD => self.alu_xor(self.reg.l),
            0xAE => self.alu_xor(self.mem_read(self.reg.hl())) + 4,
            0xAF => self.alu_xor(self.reg.a),

            // or a,u8
            0xB0 => self.alu_or(self.reg.b),
            0xB1 => self.alu_or(self.reg.c),
            0xB2 => self.alu_or(self.reg.d),
            0xB3 => self.alu_or(self.reg.e),
            0xB4 => self.alu_or(self.reg.h),
            0xB5 => self.alu_or(self.reg.l),
            0xB6 => self.alu_or(self.mem_read(self.reg.hl())) + 4,
            0xB7 => self.alu_or(self.reg.a),

            // cp a,u8
            0xB8 => self.alu_cp(self.reg.b),
            0xB9 => self.alu_cp(self.reg.c),
            0xBA => self.alu_cp(self.reg.d),
            0xBB => self.alu_cp(self.reg.e),
            0xBC => self.alu_cp(self.reg.h),
            0xBD => self.alu_cp(self.reg.l),
            0xBE => self.alu_cp(self.mem_read(self.reg.hl())) + 4,
            0xBF => self.alu_cp(self.reg.a),

            // --- PREFIXED INSTRUCTIONS ---
            0xCB => self.exec_cb_opcode(),

            opcode => unimplemented!(
                "Unknown opcode at \x1B[1m0x{:04X}\x1B[0m: \x1B[31;1m0x{:02X}\x1B[0m.",
                self.pc.saturating_sub(1), opcode
            ),
        }
    }

    fn exec_cb_opcode(&mut self) -> u8 {
        let cb_opcode = self.next_byte();

        match cb_opcode {
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
            0x80 => { self.reg.b = self.reg.b & !(1<<0); 8 },
            0x81 => { self.reg.c = self.reg.c & !(1<<0); 8 },
            0x82 => { self.reg.d = self.reg.d & !(1<<0); 8 },
            0x83 => { self.reg.e = self.reg.e & !(1<<0); 8 },
            0x84 => { self.reg.h = self.reg.h & !(1<<0); 8 },
            0x85 => { self.reg.l = self.reg.l & !(1<<0); 8 },
            0x86 => { let hl = self.reg.hl(); let v = self.mem_read(hl) & !(1<<0); self.mem_write(hl, v); 16 }
            0x87 => { self.reg.a = self.reg.a & !(1<<0); 8 },
            // res 1,r
            0x88 => { self.reg.b = self.reg.b & !(1<<1); 8 },
            0x89 => { self.reg.c = self.reg.c & !(1<<1); 8 },
            0x8A => { self.reg.d = self.reg.d & !(1<<1); 8 },
            0x8B => { self.reg.e = self.reg.e & !(1<<1); 8 },
            0x8C => { self.reg.h = self.reg.h & !(1<<1); 8 },
            0x8D => { self.reg.l = self.reg.l & !(1<<1); 8 },
            0x8E => { let hl = self.reg.hl(); let v = self.mem_read(hl) & !(1<<1); self.mem_write(hl, v); 16 }
            0x8F => { self.reg.a = self.reg.a & !(1<<1); 8 },
            // res 2,r
            0x90 => { self.reg.b = self.reg.b & !(1<<2); 8 },
            0x91 => { self.reg.c = self.reg.c & !(1<<2); 8 },
            0x92 => { self.reg.d = self.reg.d & !(1<<2); 8 },
            0x93 => { self.reg.e = self.reg.e & !(1<<2); 8 },
            0x94 => { self.reg.h = self.reg.h & !(1<<2); 8 },
            0x95 => { self.reg.l = self.reg.l & !(1<<2); 8 },
            0x96 => { let hl = self.reg.hl(); let v = self.mem_read(hl) & !(1<<2); self.mem_write(hl, v); 16 }
            0x97 => { self.reg.a = self.reg.a & !(1<<2); 8 },
            // res 3,r
            0x98 => { self.reg.b = self.reg.b & !(1<<3); 8 },
            0x99 => { self.reg.c = self.reg.c & !(1<<3); 8 },
            0x9A => { self.reg.d = self.reg.d & !(1<<3); 8 },
            0x9B => { self.reg.e = self.reg.e & !(1<<3); 8 },
            0x9C => { self.reg.h = self.reg.h & !(1<<3); 8 },
            0x9D => { self.reg.l = self.reg.l & !(1<<3); 8 },
            0x9E => { let hl = self.reg.hl(); let v = self.mem_read(hl) & !(1<<3); self.mem_write(hl, v); 16 }
            0x9F => { self.reg.a = self.reg.a & !(1<<3); 8 },
            // res 4,r
            0xA0 => { self.reg.b = self.reg.b & !(1<<4); 8 },
            0xA1 => { self.reg.c = self.reg.c & !(1<<4); 8 },
            0xA2 => { self.reg.d = self.reg.d & !(1<<4); 8 },
            0xA3 => { self.reg.e = self.reg.e & !(1<<4); 8 },
            0xA4 => { self.reg.h = self.reg.h & !(1<<4); 8 },
            0xA5 => { self.reg.l = self.reg.l & !(1<<4); 8 },
            0xA6 => { let hl = self.reg.hl(); let v = self.mem_read(hl) & !(1<<4); self.mem_write(hl, v); 16 }
            0xA7 => { self.reg.a = self.reg.a & !(1<<4); 8 },
            // res 5,r
            0xA8 => { self.reg.b = self.reg.b & !(1<<5); 8 },
            0xA9 => { self.reg.c = self.reg.c & !(1<<5); 8 },
            0xAA => { self.reg.d = self.reg.d & !(1<<5); 8 },
            0xAB => { self.reg.e = self.reg.e & !(1<<5); 8 },
            0xAC => { self.reg.h = self.reg.h & !(1<<5); 8 },
            0xAD => { self.reg.l = self.reg.l & !(1<<5); 8 },
            0xAE => { let hl = self.reg.hl(); let v = self.mem_read(hl) & !(1<<5); self.mem_write(hl, v); 16 }
            0xAF => { self.reg.a = self.reg.a & !(1<<5); 8 },
            // res 6,r
            0xB0 => { self.reg.b = self.reg.b & !(1<<6); 8 },
            0xB1 => { self.reg.c = self.reg.c & !(1<<6); 8 },
            0xB2 => { self.reg.d = self.reg.d & !(1<<6); 8 },
            0xB3 => { self.reg.e = self.reg.e & !(1<<6); 8 },
            0xB4 => { self.reg.h = self.reg.h & !(1<<6); 8 },
            0xB5 => { self.reg.l = self.reg.l & !(1<<6); 8 },
            0xB6 => { let hl = self.reg.hl(); let v = self.mem_read(hl) & !(1<<6); self.mem_write(hl, v); 16 }
            0xB7 => { self.reg.a = self.reg.a & !(1<<6); 8 },
            // res 7,r
            0xB8 => { self.reg.b = self.reg.b & !(1<<7); 8 },
            0xB9 => { self.reg.c = self.reg.c & !(1<<7); 8 },
            0xBA => { self.reg.d = self.reg.d & !(1<<7); 8 },
            0xBB => { self.reg.e = self.reg.e & !(1<<7); 8 },
            0xBC => { self.reg.h = self.reg.h & !(1<<7); 8 },
            0xBD => { self.reg.l = self.reg.l & !(1<<7); 8 },
            0xBE => { let hl = self.reg.hl(); let v = self.mem_read(hl) & !(1<<7); self.mem_write(hl, v); 16 }
            0xBF => { self.reg.a = self.reg.a & !(1<<7); 8 },

            // set 0,r
            0xC0 => { self.reg.b = self.reg.b | (1<<0); 8 },
            0xC1 => { self.reg.c = self.reg.c | (1<<0); 8 },
            0xC2 => { self.reg.d = self.reg.d | (1<<0); 8 },
            0xC3 => { self.reg.e = self.reg.e | (1<<0); 8 },
            0xC4 => { self.reg.h = self.reg.h | (1<<0); 8 },
            0xC5 => { self.reg.l = self.reg.l | (1<<0); 8 },
            0xC6 => { let hl = self.reg.hl(); let v = self.mem_read(hl) | (1<<0); self.mem_write(hl, v); 16 }
            0xC7 => { self.reg.a = self.reg.a | (1<<0); 8 },
            // set 1,r
            0xC8 => { self.reg.b = self.reg.b | (1<<1); 8 },
            0xC9 => { self.reg.c = self.reg.c | (1<<1); 8 },
            0xCA => { self.reg.d = self.reg.d | (1<<1); 8 },
            0xCB => { self.reg.e = self.reg.e | (1<<1); 8 },
            0xCC => { self.reg.h = self.reg.h | (1<<1); 8 },
            0xCD => { self.reg.l = self.reg.l | (1<<1); 8 },
            0xCE => { let hl = self.reg.hl(); let v = self.mem_read(hl) | (1<<1); self.mem_write(hl, v); 16 }
            0xCF => { self.reg.a = self.reg.a | (1<<1); 8 },
            // set 2,r
            0xD0 => { self.reg.b = self.reg.b | (1<<2); 8 },
            0xD1 => { self.reg.c = self.reg.c | (1<<2); 8 },
            0xD2 => { self.reg.d = self.reg.d | (1<<2); 8 },
            0xD3 => { self.reg.e = self.reg.e | (1<<2); 8 },
            0xD4 => { self.reg.h = self.reg.h | (1<<2); 8 },
            0xD5 => { self.reg.l = self.reg.l | (1<<2); 8 },
            0xD6 => { let hl = self.reg.hl(); let v = self.mem_read(hl) | (1<<2); self.mem_write(hl, v); 16 }
            0xD7 => { self.reg.a = self.reg.a | (1<<2); 8 },
            // set 3,r
            0xD8 => { self.reg.b = self.reg.b | (1<<3); 8 },
            0xD9 => { self.reg.c = self.reg.c | (1<<3); 8 },
            0xDA => { self.reg.d = self.reg.d | (1<<3); 8 },
            0xDB => { self.reg.e = self.reg.e | (1<<3); 8 },
            0xDC => { self.reg.h = self.reg.h | (1<<3); 8 },
            0xDD => { self.reg.l = self.reg.l | (1<<3); 8 },
            0xDE => { let hl = self.reg.hl(); let v = self.mem_read(hl) | (1<<3); self.mem_write(hl, v); 16 }
            0xDF => { self.reg.a = self.reg.a | (1<<3); 8 },
            // set 4,r
            0xE0 => { self.reg.b = self.reg.b | (1<<4); 8 },
            0xE1 => { self.reg.c = self.reg.c | (1<<4); 8 },
            0xE2 => { self.reg.d = self.reg.d | (1<<4); 8 },
            0xE3 => { self.reg.e = self.reg.e | (1<<4); 8 },
            0xE4 => { self.reg.h = self.reg.h | (1<<4); 8 },
            0xE5 => { self.reg.l = self.reg.l | (1<<4); 8 },
            0xE6 => { let hl = self.reg.hl(); let v = self.mem_read(hl) | (1<<4); self.mem_write(hl, v); 16 }
            0xE7 => { self.reg.a = self.reg.a | (1<<4); 8 },
            // set 5,r
            0xE8 => { self.reg.b = self.reg.b | (1<<5); 8 },
            0xE9 => { self.reg.c = self.reg.c | (1<<5); 8 },
            0xEA => { self.reg.d = self.reg.d | (1<<5); 8 },
            0xEB => { self.reg.e = self.reg.e | (1<<5); 8 },
            0xEC => { self.reg.h = self.reg.h | (1<<5); 8 },
            0xED => { self.reg.l = self.reg.l | (1<<5); 8 },
            0xEE => { let hl = self.reg.hl(); let v = self.mem_read(hl) | (1<<5); self.mem_write(hl, v); 16 }
            0xEF => { self.reg.a = self.reg.a | (1<<5); 8 },
            // set 6,r
            0xF0 => { self.reg.b = self.reg.b | (1<<6); 8 },
            0xF1 => { self.reg.c = self.reg.c | (1<<6); 8 },
            0xF2 => { self.reg.d = self.reg.d | (1<<6); 8 },
            0xF3 => { self.reg.e = self.reg.e | (1<<6); 8 },
            0xF4 => { self.reg.h = self.reg.h | (1<<6); 8 },
            0xF5 => { self.reg.l = self.reg.l | (1<<6); 8 },
            0xF6 => { let hl = self.reg.hl(); let v = self.mem_read(hl) | (1<<6); self.mem_write(hl, v); 16 }
            0xF7 => { self.reg.a = self.reg.a | (1<<6); 8 },
            // set 7,r
            0xF8 => { self.reg.b = self.reg.b | (1<<7); 8 },
            0xF9 => { self.reg.c = self.reg.c | (1<<7); 8 },
            0xFA => { self.reg.d = self.reg.d | (1<<7); 8 },
            0xFB => { self.reg.e = self.reg.e | (1<<7); 8 },
            0xFC => { self.reg.h = self.reg.h | (1<<7); 8 },
            0xFD => { self.reg.l = self.reg.l | (1<<7); 8 },
            0xFE => { let hl = self.reg.hl(); let v = self.mem_read(hl) | (1<<7); self.mem_write(hl, v); 16 }
            0xFF => { self.reg.a = self.reg.a | (1<<7); 8 },

            opcode => unimplemented!("Unknown \x1B[4mprefixed\x1B[0m opcode at \x1B[1m0x{:04X}\x1B[0m: \x1B[31;1m0xCB{:02X}\x1B[0m.", self.pc.saturating_sub(2), opcode)
        }
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

        self.reg.f.set(CpuFlags::Z, result == 0);
        self.reg.f.remove(CpuFlags::N);
        self.reg.f.set(CpuFlags::H, (a & 0xF) + (n & 0xF) + c > 0xF);
        self.reg
            .f
            .set(CpuFlags::C, (a as u16) + (n as u16) + (c as u16) > 0xFF);

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

        self.reg.f.set(CpuFlags::Z, result == 0);
        self.reg.f.insert(CpuFlags::N);
        self.reg.f.set(CpuFlags::H, (a & 0xF) < (n & 0xF) + c);
        self.reg
            .f
            .set(CpuFlags::C, (a as u16) < (n as u16) + c as u16);

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

        self.reg.f.set(CpuFlags::Z, result == 0);
        self.reg.f.remove(CpuFlags::N);
        self.reg.f.insert(CpuFlags::H);
        self.reg.f.remove(CpuFlags::C);

        self.reg.a = result;

        4
    }

    /// Logically XOR `n` with A, result in A. \
    /// Returns the instruction cycles.
    ///
    /// # Flags affected
    ///
    /// Z: Set if result is zero \
    /// N: 0 \
    /// H: 0 \
    /// C: 0
    fn alu_xor(&mut self, n: u8) -> u8 {
        let result = self.reg.a ^ n;

        self.reg.f.set(CpuFlags::Z, result == 0);
        self.reg.f.remove(CpuFlags::N);
        self.reg.f.remove(CpuFlags::H);
        self.reg.f.remove(CpuFlags::C);

        self.reg.a = result;

        4
    }

    /// Logically OR `n` with register A, result in A. \
    /// Returns the instruction cycles.
    ///
    /// # Flags affected
    ///
    /// Z: Set if result is zero \
    /// N: 0 \
    /// H: 0 \
    /// C: 0
    fn alu_or(&mut self, n: u8) -> u8 {
        let result = self.reg.a | n;

        self.reg.f.set(CpuFlags::Z, result == 0);
        self.reg.f.remove(CpuFlags::N);
        self.reg.f.remove(CpuFlags::H);
        self.reg.f.remove(CpuFlags::C);

        self.reg.a = result;

        4
    }

    /// Compare A with `n`. \
    /// Returns the instruction cycles.
    ///
    /// # Flags affected
    ///
    /// Z: Set if result is zero. (A == `n`) \
    /// N: 1 \
    /// H: Set if no borrow from bit 4 \
    /// C: Set for no borrow. (A < `n`)
    fn alu_cp(&mut self, n: u8) -> u8 {
        let a = self.reg.a;
        let result = a.wrapping_sub(n);

        self.reg.f.set(CpuFlags::Z, result == 0);
        self.reg.f.insert(CpuFlags::N);
        self.reg.f.set(CpuFlags::H, (a & 0xF) < (n & 0xF));
        self.reg.f.set(CpuFlags::C, a < n);

        4
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

        self.reg.f.set(CpuFlags::Z, result == 0);
        self.reg.f.remove(CpuFlags::N);
        self.reg.f.insert(CpuFlags::H);

        8
    }

    fn jr(&mut self, condition: bool, offset: i8) -> u8 {
        if condition {
            self.pc = self.pc.wrapping_add(offset as u16);
            0
        } else {
            4
        }
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
