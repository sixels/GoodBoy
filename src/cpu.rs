pub mod instruction;
pub mod register;

use std::{fmt::Debug, io::Write};

use crate::{cpu::instruction::Operand, memory::MemoryAccess, utils::UnsignedValue};
use register::{Flags, Registers};

use self::instruction::{Condition, Instruction, Opcode, CB_OPCODE_MAP, OPCODE_MAP};

pub struct Cpu {
    // CPU registers
    pub regs: Registers,

    // Special purpose registers:
    /// Program Counter
    pub pc: u16,
    /// Stack Pointer
    pub sp: u16,

    /// Memory buffer
    memory: Box<[u8; 0x10000]>,
}

impl Cpu {
    pub fn new(buffer: &[u8]) -> Self {
        let mut cpu = Self::default();
        cpu.load(&buffer[..=0x7FFF], 0x0000);

        cpu
    }

    /// Create a new CPU with the given boot rom
    pub fn with_bootstrap(buffer: &[u8]) -> Self {
        let mut cpu = Self {
            ..Default::default()
        };

        let start_offset = 0x00;
        cpu.load(&buffer[0x00..=0xFF], start_offset);

        cpu
    }

    pub fn decode(byte: u8, prefixed: bool) -> Option<&'static Opcode<'static>> {
        if prefixed {
            CB_OPCODE_MAP.get(&byte).copied()
        } else {
            OPCODE_MAP.get(&byte).copied()
        }
    }

    /// Load a slice into the ROM
    pub fn load(&mut self, slice: &[u8], start_offset: usize) {
        self.memory[start_offset..slice.len() + start_offset].copy_from_slice(slice);
    }

    pub fn run(&mut self) -> ! {
        self.run_callback(|cpu| {
            if cpu.mem_read(0xFF02) == 0x81 {
                let ch = char::from(cpu.mem_read(0xFF01));
                eprint!("{:}", if ch != '\n' { ch } else { ' ' });
                std::io::stderr().flush().unwrap();
                cpu.mem_write(0xFF02, 0x00);
            }
        })
    }

    pub fn run_callback<FN>(&mut self, callback: FN) -> !
    where
        FN: Fn(&mut Self),
    {
        loop {
            callback(self);
            self.tick();
        }
    }

    pub fn tick(&mut self) -> u8 {
        let opcode = self.fetch_and_decode();
        self.exec_instruction(opcode).unwrap();
        opcode.cycles
    }

    /// Get the next byte and increment the PC by 1.
    pub fn fetch_byte(&mut self) -> u8 {
        let byte = self.mem_read(self.pc);
        self.pc += 1;
        byte
    }

    /// Get the next word and increment the PC by 2.
    pub fn fetch_word(&mut self) -> u16 {
        let word = self.mem_read_word(self.pc);
        self.pc += 2;
        word
    }

    pub fn fetch_and_decode(&mut self) -> &'static Opcode<'static> {
        let byte = self.fetch_byte();

        let opcode = Cpu::decode(byte, false);

        let opcode = match opcode {
            Some(Opcode { code: 0xCB, .. }) => Cpu::decode(self.fetch_byte(), true),
            _ => opcode,
        };

        opcode.unwrap_or_else(|| panic!("Unknown opcode: 0x{:02X}", byte))
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

    // TODO: rename to exec_opcode
    pub fn exec_instruction<'a>(&mut self, opcode: &Opcode<'a>) -> Result<(), String> {
        match opcode.instruction {
            Instruction::NOP => (),

            Instruction::LDIM16(target) => {
                let immediate = self.fetch_word();
                self.set_operand_value(&target, immediate);
            }
            Instruction::LDIM8(target) => {
                let immediate = self.fetch_byte();
                self.set_operand_value(&target, immediate);
            }
            Instruction::LDMEM(target) => self.set_operand_value(&target, self.regs.a),

            Instruction::LD16A => {
                let immediate = self.fetch_word();
                self.mem_write(immediate, self.regs.a);
            }
            Instruction::LDA16 => {
                let immediate = self.fetch_word();
                self.regs.a = self.mem_read(immediate);
            }

            Instruction::LDFF8A => {
                let immediate = self.fetch_byte();
                self.mem_write(0xFF00 | (immediate as u16), self.regs.a);
            }
            Instruction::LDAFF8 => {
                let immediate = self.fetch_byte();
                self.regs.a = self.mem_read(0xFF00 | (immediate as u16));
            }
            Instruction::LDFFCA => {
                self.mem_write(0xFF00 | (self.regs.c as u16), self.regs.a);
            }
            Instruction::LDAFFC => {
                self.regs.a = self.mem_read(0xFF00 | (self.regs.c as u16));
            }

            Instruction::LD16SP => {
                let addr = self.fetch_word();
                self.mem_write_word(addr, self.sp);
            }

            Instruction::INC16(target) => {
                let value = self
                    .get_operand_value(&target, false)
                    .unwrap()
                    .unwrap_u16()
                    .wrapping_add(1);
                self.set_operand_value(&target, value);
            }
            Instruction::DEC16(target) => {
                let value = self
                    .get_operand_value(&target, false)
                    .unwrap()
                    .unwrap_u16()
                    .wrapping_sub(1);
                self.set_operand_value(&target, value);
            }

            Instruction::INC8(target) => self.alu_inc(&target),
            Instruction::DEC8(target) => self.alu_dec(&target),

            Instruction::RLCA => {
                self.alu_rlc(&Operand::A);
                self.regs.f.remove(Flags::Z);
            }
            Instruction::RLA => {
                self.alu_rl(&Operand::A);
                self.regs.f.remove(Flags::Z);
            }
            Instruction::RRCA => {
                self.alu_rrc(&Operand::A);
                self.regs.f.remove(Flags::Z);
            }
            Instruction::RRA => {
                self.alu_rr(&Operand::A);
                self.regs.f.remove(Flags::Z);
            }
            Instruction::RLC(target) => self.alu_rlc(&target),
            Instruction::RL(target) => self.alu_rl(&target),
            Instruction::RRC(target) => self.alu_rrc(&target),
            Instruction::RR(target) => self.alu_rr(&target),

            Instruction::ADDHL(source) => {
                let value = self.get_operand_value(&source, false).unwrap().unwrap_u16();
                self.alu16_add(value);
            }

            Instruction::LDRR(target, Operand::HLI) => {
                let addr = self.regs.hli();
                let value = self.mem_read(addr);
                self.set_operand_value(&target, value)
            }
            Instruction::LDRR(target, Operand::HLD) => {
                let addr = self.regs.hld();
                let value = self.mem_read(addr);
                self.set_operand_value(&target, value)
            }
            Instruction::LDRR(target, source) => {
                let value = self.get_operand_value(&source, true).unwrap().unwrap_u8();
                self.set_operand_value(&target, value)
            }

            Instruction::DAA => {
                let mut a = self.regs.a;
                let mut adjust = if self.regs.f.contains(Flags::C) {
                    0x60
                } else {
                    0x00
                };
                if self.regs.f.contains(Flags::H) {
                    adjust |= 0x06;
                };
                if !self.regs.f.contains(Flags::N) {
                    if a & 0x0F > 0x09 {
                        adjust |= 0x06;
                    };
                    if a > 0x99 {
                        adjust |= 0x60;
                    };
                    a = a.wrapping_add(adjust);
                } else {
                    a = a.wrapping_sub(adjust);
                }

                self.regs.f.set(Flags::C, adjust >= 0x60);
                self.regs.f.remove(Flags::H);
                self.regs.f.set(Flags::Z, a == 0);

                self.regs.a = a;
            }

            Instruction::SCF => {
                self.regs.f.remove(Flags::N | Flags::H);
                self.regs.f.insert(Flags::C);
            }
            Instruction::CCF => {
                self.regs.f.remove(Flags::N | Flags::H);
                self.regs.f.toggle(Flags::C);
            }
            Instruction::CPL => {
                self.regs.a = !self.regs.a;
                self.regs.f.insert(Flags::N | Flags::H);
            }

            Instruction::ADD(source) => {
                let value = if source == Operand::IM8 {
                    self.fetch_byte()
                } else {
                    self.get_operand_value(&source, true).unwrap().unwrap_u8()
                };
                self.alu_add(value, false);
            }

            Instruction::ADC(source) => {
                let value = if source == Operand::IM8 {
                    self.fetch_byte()
                } else {
                    self.get_operand_value(&source, true).unwrap().unwrap_u8()
                };
                self.alu_add(value, true);
            }
            Instruction::SUB(source) => {
                let value = if source == Operand::IM8 {
                    self.fetch_byte()
                } else {
                    self.get_operand_value(&source, true).unwrap().unwrap_u8()
                };
                self.alu_sub(value, false);
            }
            Instruction::SBC(source) => {
                let value = if source == Operand::IM8 {
                    self.fetch_byte()
                } else {
                    self.get_operand_value(&source, true).unwrap().unwrap_u8()
                };
                self.alu_sub(value, true);
            }
            Instruction::AND(source) => {
                let value = if source == Operand::IM8 {
                    self.fetch_byte()
                } else {
                    self.get_operand_value(&source, true).unwrap().unwrap_u8()
                };
                self.alu_and(value);
            }
            Instruction::XOR(source) => {
                let value = if source == Operand::IM8 {
                    self.fetch_byte()
                } else {
                    self.get_operand_value(&source, true).unwrap().unwrap_u8()
                };
                self.alu_xor(value);
            }
            Instruction::OR(source) => {
                let value = if source == Operand::IM8 {
                    self.fetch_byte()
                } else {
                    self.get_operand_value(&source, true).unwrap().unwrap_u8()
                };
                self.alu_or(value);
            }
            Instruction::CP(source) => {
                let value = if source == Operand::IM8 {
                    self.fetch_byte()
                } else {
                    self.get_operand_value(&source, true).unwrap().unwrap_u8()
                };
                self.alu_cp(value);
            }

            Instruction::JR(None) => {
                self.branch_jr(true);
            }
            Instruction::JR(Some(condition)) => {
                let cc = self.get_condition_value(&condition);
                self.branch_jr(cc);
            }
            Instruction::RET(None) => {
                self.branch_ret(true);
            }
            Instruction::RET(Some(condition)) => {
                let cc = self.get_condition_value(&condition);
                self.branch_ret(cc);
            }
            Instruction::JP(None) => {
                self.branch_jp(true);
            }
            Instruction::JP(Some(condition)) => {
                let cc = self.get_condition_value(&condition);
                self.branch_jp(cc);
            }
            Instruction::CALL(None) => {
                self.branch_call(true);
            }
            Instruction::CALL(Some(condition)) => {
                let cc = self.get_condition_value(&condition);
                self.branch_call(cc);
            }

            Instruction::JPHL => self.pc = self.regs.hl(),

            Instruction::POP(target) => {
                let value = self.pop_stack();
                self.set_operand_value(&target, value);
            }
            Instruction::PUSH(target) => {
                let value = self.get_operand_value(&target, false).unwrap().unwrap_u16();
                self.push_stack(value);
            }

            Instruction::DI => (),
            Instruction::EI => (),

            Instruction::ADDSP => self.alu16_add_imm(&Operand::SP, self.sp),
            Instruction::ADDHLSP => self.alu16_add_imm(&Operand::HL, self.sp),
            Instruction::LDSPHL => self.sp = self.regs.hl(),

            Instruction::SLA(target) => self.alu_sla(&target),
            Instruction::SRA(target) => self.alu_sra(&target),

            Instruction::SWAP(target) => self.alu_swap(&target),
            Instruction::SRL(target) => self.alu_srl(&target),

            Instruction::BIT(b, r) => {
                let value = self.get_operand_value(&r, true).unwrap().unwrap_u8();
                self.alu_bit(b, value);
            }
            Instruction::RES(b, target) => {
                let value = self.get_operand_value(&target, true).unwrap().unwrap_u8();
                self.set_operand_value(&target, value & !(1 << b))
            }
            Instruction::SET(b, target) => {
                let value = self.get_operand_value(&target, true).unwrap().unwrap_u8();
                self.set_operand_value(&target, value | (1 << b))
            }

            _ => {
                return Err(format!(
                    "Unimplemented opcode at 0x{:04X}: {:02x?}.",
                    self.pc.saturating_sub(opcode.length as u16),
                    opcode
                ))
            }
        };

        Ok(())
    }

    fn get_operand_value(&self, operand: &Operand, is_u8: bool) -> Option<UnsignedValue> {
        match operand {
            // r
            Operand::A if is_u8 => Some(self.regs.a.into()),
            Operand::B if is_u8 => Some(self.regs.b.into()),
            Operand::C if is_u8 => Some(self.regs.c.into()),
            Operand::D if is_u8 => Some(self.regs.d.into()),
            Operand::E if is_u8 => Some(self.regs.e.into()),
            Operand::H if is_u8 => Some(self.regs.h.into()),
            Operand::L if is_u8 => Some(self.regs.l.into()),
            Operand::F if is_u8 => Some(self.regs.f.bits().into()),

            // (rr)
            Operand::AF if is_u8 => Some(self.mem_read(self.regs.af()).into()),
            Operand::BC if is_u8 => Some(self.mem_read(self.regs.bc()).into()),
            Operand::DE if is_u8 => Some(self.mem_read(self.regs.de()).into()),
            Operand::HL if is_u8 => Some(self.mem_read(self.regs.hl()).into()),
            Operand::SP if is_u8 => Some(self.mem_read(self.sp).into()),

            // rr
            Operand::AF if !is_u8 => Some(self.regs.af().into()),
            Operand::BC if !is_u8 => Some(self.regs.bc().into()),
            Operand::DE if !is_u8 => Some(self.regs.de().into()),
            Operand::HL if !is_u8 => Some(self.regs.hl().into()),
            Operand::SP if !is_u8 => Some(self.sp.into()),
            _ => None,
        }
    }

    fn set_operand_value<T: Into<UnsignedValue>>(&mut self, operand: &Operand, value: T) {
        let value = value.into();
        match (operand, value) {
            // r
            (Operand::A, UnsignedValue::U8(value)) => self.regs.a = value,
            (Operand::B, UnsignedValue::U8(value)) => self.regs.b = value,
            (Operand::C, UnsignedValue::U8(value)) => self.regs.c = value,
            (Operand::D, UnsignedValue::U8(value)) => self.regs.d = value,
            (Operand::E, UnsignedValue::U8(value)) => self.regs.e = value,
            (Operand::H, UnsignedValue::U8(value)) => self.regs.h = value,
            (Operand::L, UnsignedValue::U8(value)) => self.regs.l = value,
            (Operand::F, UnsignedValue::U8(value)) => {
                self.regs.f = Flags::from_bits_truncate(value)
            }

            // (rr)
            (Operand::AF, UnsignedValue::U8(value)) => self.mem_write(self.regs.af(), value),
            (Operand::BC, UnsignedValue::U8(value)) => self.mem_write(self.regs.bc(), value),
            (Operand::DE, UnsignedValue::U8(value)) => self.mem_write(self.regs.de(), value),
            (Operand::HL, UnsignedValue::U8(value)) => self.mem_write(self.regs.hl(), value),
            (Operand::SP, UnsignedValue::U8(value)) => self.mem_write(self.sp, value),

            // rr
            (Operand::AF, UnsignedValue::U16(value)) => self.regs.set_af(value),
            (Operand::BC, UnsignedValue::U16(value)) => self.regs.set_bc(value),
            (Operand::DE, UnsignedValue::U16(value)) => self.regs.set_de(value),
            (Operand::HL, UnsignedValue::U16(value)) => self.regs.set_hl(value),
            (Operand::SP, UnsignedValue::U16(value)) => self.sp = value,

            // hl+, hl-
            (Operand::HLI, UnsignedValue::U8(value)) => {
                let hl = self.regs.hli();
                self.mem_write(hl, value);
            }
            (Operand::HLD, UnsignedValue::U8(value)) => {
                let hl = self.regs.hld();
                self.mem_write(hl, value);
            }

            _ => panic!("Invalid operand"),
        }
    }

    fn get_condition_value(&self, condition: &Condition) -> bool {
        match condition {
            Condition::Z => self.regs.f.contains(Flags::Z),
            Condition::C => self.regs.f.contains(Flags::C),
            Condition::NZ => !self.regs.f.contains(Flags::Z),
            Condition::NC => !self.regs.f.contains(Flags::C),
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
                self.push_stack(self.pc + 2);
                self.branch_jp(true);
                24
            })
            .unwrap_or_else(|| {
                // skip the next word
                self.pc += 2;
                12
            })
    }

    /// If `condition` is true, set the PC to the last address on the stack, otherwise, do nothing. \
    /// Returns the instruction cycles.
    fn branch_ret(&mut self, condition: bool) -> u8 {
        condition
            .then(|| {
                self.pc = self.pop_stack();
                16
            })
            .unwrap_or(8)
    }

    // --- ALU ---

    /// Increment `target`.
    ///
    /// # Flags affected
    ///
    /// Z: Set if result is 0 \
    /// N: 0 \
    /// H: Set if carry from bit 3
    fn alu_inc(&mut self, target: &Operand) {
        let r = self.get_operand_value(target, true).unwrap().unwrap_u8();

        let result = r.wrapping_add(1);

        self.regs.f.set(Flags::Z, result == 0);
        self.regs.f.remove(Flags::N);
        self.regs.f.set(Flags::H, ((r & 0xF) + 1) > 0xF);

        self.set_operand_value(target, result);
    }

    /// Decrement `target`.
    ///
    /// # Flags affected
    ///
    /// Z: Set if result is 0 \
    /// N: 0 \
    /// H: Set if carry from bit 3
    fn alu_dec(&mut self, target: &Operand) {
        let r = self.get_operand_value(target, true).unwrap().unwrap_u8();

        let result = r.wrapping_sub(1);

        self.regs.f.set(Flags::Z, result == 0);
        self.regs.f.insert(Flags::N);
        self.regs.f.set(Flags::H, (r & 0xF) == 0);

        self.set_operand_value(target, result);
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
    fn alu_add(&mut self, n: u8, adc: bool) {
        let a = self.regs.a;
        let c = (adc & self.regs.f.contains(Flags::C)) as u8;
        let result = a.wrapping_add(n.wrapping_add(c));

        self.regs.f.set(Flags::Z, result == 0);
        self.regs.f.remove(Flags::N);
        self.regs.f.set(Flags::H, ((a & 0xF) + (n & 0xF) + c) > 0xF);
        self.regs
            .f
            .set(Flags::C, ((a as u16) + (n as u16) + (c as u16)) > 0xFF);

        self.regs.a = result;
    }

    fn alu16_add(&mut self, n: u16) {
        let hl = self.regs.hl();
        let result = hl.wrapping_add(n);

        self.regs
            .f
            .set(Flags::H, (hl & 0x7FF) + (n & 0x7FF) > 0x7FF);
        self.regs.f.remove(Flags::N);
        self.regs.f.set(Flags::C, hl > 0xFFFF - n);

        self.regs.set_hl(result);
    }
    fn alu16_add_imm(&mut self, target: &Operand, rr: u16) {
        let rr = rr;
        let n = self.fetch_byte() as i8 as i16 as u16;

        let result = rr.wrapping_add(n);

        self.regs.f.remove(Flags::Z | Flags::N);
        self.regs
            .f
            .set(Flags::H, (rr & 0x000F) + (n & 0x000F) > 0x000F);
        self.regs
            .f
            .set(Flags::C, (rr & 0x00FF) + (n & 0x00FF) > 0x00FF);

        self.set_operand_value(target, result)
    }

    /// Subtract `n` + `sbc` from A. \
    /// Returns the instruction cycles.
    ///
    /// # Flags affected
    ///
    /// Z: Set if result is 0 \
    /// N: 1 \
    /// H: Set if no borrow from bit 4 \
    /// C: Set if no borrow
    fn alu_sub(&mut self, n: u8, sbc: bool) -> u8 {
        let a = self.regs.a;
        let c = (sbc & self.regs.f.contains(Flags::C)) as u8;

        let result = a.wrapping_sub(n.wrapping_add(c));

        self.regs.f.set(Flags::Z, result == 0);
        self.regs.f.insert(Flags::N);
        self.regs.f.set(Flags::H, (a & 0xF) < (n & 0xF) + c);
        self.regs
            .f
            .set(Flags::C, (a as u16) < (n as u16) + c as u16);

        self.regs.a = result;

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
        let result = self.regs.a & n;

        self.regs.f.set(Flags::Z, result == 0);
        self.regs.f.remove(Flags::N);
        self.regs.f.insert(Flags::H);
        self.regs.f.remove(Flags::C);

        self.regs.a = result;

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
        let result = self.regs.a ^ n;

        self.regs.f.set(Flags::Z, result == 0);
        self.regs.f.remove(Flags::N | Flags::H | Flags::C);

        self.regs.a = result;

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
        let result = self.regs.a | n;

        self.regs.f.set(Flags::Z, result == 0);
        self.regs.f.remove(Flags::N | Flags::H | Flags::C);

        self.regs.a = result;

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
        let a = self.regs.a;
        let result = a.wrapping_sub(n);

        self.regs.f.set(Flags::Z, result == 0);
        self.regs.f.insert(Flags::N);
        self.regs.f.set(Flags::H, (a & 0xF) < (n & 0xF));
        self.regs.f.set(Flags::C, a < n);

        4
    }

    /// Rotate `target` bits to the left. \
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
    fn alu_rlc(&mut self, target: &Operand) {
        let r = self.get_operand_value(target, true).unwrap().unwrap_u8();
        let c = r >> 7;

        let result = (r << 1) | c;

        self.regs.f.set(Flags::Z, result == 0);
        self.regs.f.remove(Flags::N | Flags::H);
        self.regs.f.set(Flags::C, c == 1);

        self.set_operand_value(target, result);
    }

    /// Rotate `target` bits to the right. \
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
    fn alu_rrc(&mut self, target: &Operand) {
        let r = self.get_operand_value(target, true).unwrap().unwrap_u8();
        let c = r & 1;

        let result = (c << 7) | (r >> 1);

        self.regs.f.set(Flags::Z, result == 0);
        self.regs.f.remove(Flags::N | Flags::H);
        self.regs.f.set(Flags::C, c == 1);

        self.set_operand_value(target, result);
    }

    /// Rotate `target` bits to the left through Carry. \
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
    fn alu_rl(&mut self, target: &Operand) {
        let r = self.get_operand_value(target, true).unwrap().unwrap_u8();
        let c = r & 0x80 == 0x80;

        let result = (r << 1) | self.regs.f.contains(Flags::C) as u8;

        self.regs.f.set(Flags::Z, result == 0);
        self.regs.f.remove(Flags::N | Flags::H);
        self.regs.f.set(Flags::C, c);

        self.set_operand_value(target, result);
    }

    /// Rotate `target` bits to the right through Carry. \
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
    fn alu_rr(&mut self, target: &Operand) {
        let r = self.get_operand_value(target, true).unwrap().unwrap_u8();
        let c = r & 1 == 1;

        let result = ((self.regs.f.contains(Flags::C) as u8) << 7) | (r >> 1);

        self.regs.f.set(Flags::Z, result == 0);
        self.regs.f.remove(Flags::N | Flags::H);
        self.regs.f.set(Flags::C, c);

        self.set_operand_value(target, result);
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

        self.regs.f.set(Flags::Z, result == 0);
        self.regs.f.remove(Flags::N);
        self.regs.f.insert(Flags::H);

        8
    }

    fn alu_sla(&mut self, target: &Operand) {
        let value = self.get_operand_value(target, true).unwrap().unwrap_u8();

        let result = value << 1;
        self.regs.f.set(Flags::Z, result == 0);
        self.regs.f.remove(Flags::N | Flags::H);
        self.regs.f.set(Flags::C, (value & 0x80) == 0x80);

        self.set_operand_value(target, result)
    }
    fn alu_sra(&mut self, target: &Operand) {
        let value = self.get_operand_value(target, true).unwrap().unwrap_u8();

        let result = value >> 1 | (value & 0x80);
        self.regs.f.set(Flags::Z, result == 0);
        self.regs.f.remove(Flags::N | Flags::H);
        self.regs.f.set(Flags::C, (value & 1) == 1);

        self.set_operand_value(target, result)
    }

    fn alu_srl(&mut self, target: &Operand) {
        let value = self.get_operand_value(target, true).unwrap().unwrap_u8();

        let result = value >> 1;
        self.regs.f.set(Flags::Z, result == 0);
        self.regs.f.remove(Flags::N | Flags::H);
        self.regs.f.set(Flags::C, (value & 1) == 1);

        self.set_operand_value(target, result)
    }

    fn alu_swap(&mut self, target: &Operand) {
        let value = self.get_operand_value(target, true).unwrap().unwrap_u8();

        let result = (value >> 4) | (value << 4);
        self.regs.f.set(Flags::Z, result == 0);
        self.regs.f.remove(Flags::N | Flags::H | Flags::C);

        self.set_operand_value(target, result)
    }
}

impl Default for Cpu {
    fn default() -> Self {
        let regs = Registers::default();

        let mut memory = Box::new([0; 0x10000]);

        memory[0xFF05] = 0x00; // TIMA
        memory[0xFF06] = 0x00; // TMA
        memory[0xFF07] = 0x00; // TAC
        memory[0xFF10] = 0x80; // NR10
        memory[0xFF11] = 0xBF; // NR11
        memory[0xFF12] = 0xF3; // NR12
        memory[0xFF14] = 0xBF; // NR14
        memory[0xFF16] = 0x3F; // NR21
        memory[0xFF17] = 0x00; // NR22
        memory[0xFF19] = 0xBF; // NR24
        memory[0xFF1A] = 0x7F; // NR30
        memory[0xFF1B] = 0xFF; // NR31
        memory[0xFF1C] = 0x9F; // NR32
        memory[0xFF1E] = 0xBF; // NR33
        memory[0xFF20] = 0xFF; // NR41
        memory[0xFF21] = 0x00; // NR42
        memory[0xFF22] = 0x00; // NR43
        memory[0xFF23] = 0xBF; // NR44
        memory[0xFF24] = 0x77; // NR50
        memory[0xFF25] = 0xF3; // NR51
        memory[0xFF26] = 0xF1; // NR52
        memory[0xFF40] = 0x91; // LCDC
        memory[0xFF42] = 0x00; // SCY
        memory[0xFF43] = 0x00; // SCX
        memory[0xFF45] = 0x00; // LYC
        memory[0xFF47] = 0xFC; // BGP
        memory[0xFF48] = 0xFF; // OBP0
        memory[0xFF49] = 0xFF; // OBP1
        memory[0xFF4A] = 0x00; // WY
        memory[0xFF4B] = 0x00; // WX
        memory[0xFFFF] = 0x00; // IE

        Self {
            regs,
            sp: 0xFFFE,
            pc: 0x0100,
            memory,
        }
    }
}

impl Debug for Cpu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = format!(
            "PC:${:04x?} SP:${:04x?} CUR:0x{:02x} {:04x?}",
            self.pc,
            self.sp,
            self.mem_read(self.pc),
            self.regs
        );

        write!(f, "{}", output)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fetch_opcodes() {
        let mut cpu = Cpu::default();

        // 0x0000: 0x01 0x02 0x03 0x04 0x05 ...
        let buffer = (1..10).collect::<Vec<u8>>();

        cpu.load(&buffer, 0);

        assert_eq!(cpu.fetch_byte(), 0x01);
        assert_eq!(cpu.fetch_byte(), 0x02);

        assert_eq!(cpu.fetch_word(), 0x0403);
        assert_eq!(cpu.fetch_word(), 0x0605);
    }
}
