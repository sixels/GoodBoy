use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Operand {
    A,
    F,
    B,
    C,
    D,
    E,
    H,
    L,
    AF,
    BC,
    DE,
    HL,
    HLI,
    HLD,
    SP,
    IM8,
}

#[derive(Debug, Clone, Copy)]
pub enum Condition {
    Z,
    C,
    NZ,
    NC,
}

#[derive(Debug, Clone, Copy)]
pub enum Instruction {
    Unused,
    ADC(Operand),
    ADD(Operand),
    ADDHL(Operand),
    ADDHLSP,
    ADDSP,
    AND(Operand),
    BIT(u8, Operand),
    CALL(Option<Condition>),
    CB,
    CCF,
    CP(Operand),
    CPL,
    DAA,
    DEC16(Operand),
    DEC8(Operand),
    DI,
    EI,
    HALT,
    INC16(Operand),
    INC8(Operand),
    JP(Option<Condition>),
    JPHL,
    JR(Option<Condition>),
    LD16A,
    LD16SP,
    LDA16,
    LDAFF8,
    LDAFFC,
    LDFF8A,
    LDFFCA,
    LDIM16(Operand),
    LDIM8(Operand),
    LDMEM(Operand),
    LDRR(Operand, Operand),
    LDSPHL,
    NOP,
    OR(Operand),
    POP(Operand),
    PUSH(Operand),
    RES(u8, Operand),
    RET(Option<Condition>),
    RETI,
    RL(Operand),
    RLA,
    RLC(Operand),
    RLCA,
    RR(Operand),
    RRA,
    RRC(Operand),
    RRCA,
    RST(u16),
    SBC(Operand),
    SCF,
    SET(u8, Operand),
    SLA(Operand),
    SRA(Operand),
    SRL(Operand),
    STOP,
    SUB(Operand),
    SWAP(Operand),
    XOR(Operand),
}

#[derive(Debug)]
pub struct Opcode<'a> {
    pub code: u8,
    pub instruction: Instruction,
    pub length: usize,
    pub cycles: u32,
    pub name: &'a str,
    pub prefixed: bool,
}

impl<'a> Opcode<'a> {
    /// Creates a new instruction
    pub fn new(
        code: u8,
        len: usize,
        cycles: u32,
        instruction: Instruction,
        name: &'a str,
    ) -> Opcode {
        Opcode {
            code,
            instruction,
            cycles,
            length: len,
            name,
            prefixed: false,
        }
    }

    /// Creates a new prefixed instruction
    pub fn cb(
        code: u8,
        len: usize,
        cycles: u32,
        instruction: Instruction,
        name: &'a str,
    ) -> Opcode {
        Opcode {
            code,
            instruction,
            cycles,
            length: len,
            name,
            prefixed: true,
        }
    }
}

impl<'a> PartialEq for Opcode<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.code == other.code
    }
}

impl<'a> Eq for Opcode<'a> {}

impl<'a> PartialOrd for Opcode<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.code.cmp(&other.code))
    }
}

impl<'a> Ord for Opcode<'a> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.code.cmp(&other.code)
    }
}

macro_rules! unused_instruction {
    ($opcode:expr) => {
        Opcode::new($opcode, 1, 0, Instruction::Unused, "UNUSED")
    };
}

lazy_static::lazy_static! {
    pub static ref OPCODE_MAP: HashMap<u8, &'static Opcode<'static>> = {
        let opcodes = OPCODE_VEC.iter()
            .filter_map(|opcode| (!opcode.prefixed).then(|| (opcode.code, opcode)))
            .collect::<HashMap<_, _>>();
        opcodes
    };
    pub static ref CB_OPCODE_MAP: HashMap<u8, &'static Opcode<'static>> = {
        let opcodes = OPCODE_VEC.iter()
            .filter_map(|opcode| opcode.prefixed.then(|| (opcode.code, opcode)))
            .collect::<HashMap<_, _>>();
        opcodes
    };

    static ref OPCODE_VEC: Vec<Opcode<'static>> = vec![
        Opcode::new(0x00, 1, 4, Instruction::NOP, "NOP"),

        unused_instruction!(0xD3),
        unused_instruction!(0xDB),
        unused_instruction!(0xDD),
        unused_instruction!(0xE3),
        unused_instruction!(0xE4),
        unused_instruction!(0xEB),
        unused_instruction!(0xEC),
        unused_instruction!(0xED),
        unused_instruction!(0xF4),
        unused_instruction!(0xFC),
        unused_instruction!(0xFD),


        Opcode::new(0x10, 1, 4, Instruction::STOP, "STOP"),
        Opcode::new(0x76, 1, 4, Instruction::HALT, "HALT"),

        Opcode::new(0x01, 3, 12, Instruction::LDIM16(Operand::BC), "LD BC,u16"),
        Opcode::new(0x11, 3, 12, Instruction::LDIM16(Operand::DE), "LD DE,u16"),
        Opcode::new(0x21, 3, 12, Instruction::LDIM16(Operand::HL), "LD HL,u16"),
        Opcode::new(0x31, 3, 12, Instruction::LDIM16(Operand::SP), "LD SP,u16"),

        Opcode::new(0x02, 1, 8, Instruction::LDMEM(Operand::BC), "LD (BC),A"),
        Opcode::new(0x12, 1, 8, Instruction::LDMEM(Operand::DE), "LD (DE),A"),
        Opcode::new(0x22, 1, 8, Instruction::LDMEM(Operand::HLI), "LD (HL+),A"),
        Opcode::new(0x32, 1, 8, Instruction::LDMEM(Operand::HLD), "LD (HL-),A"),

        Opcode::new(0x03, 1, 8, Instruction::INC16(Operand::BC), "INC BC"),
        Opcode::new(0x13, 1, 8, Instruction::INC16(Operand::DE), "INC DE"),
        Opcode::new(0x23, 1, 8, Instruction::INC16(Operand::HL), "INC HL"),
        Opcode::new(0x33, 1, 8, Instruction::INC16(Operand::SP), "INC SP"),

        Opcode::new(0x0B, 1, 8, Instruction::DEC16(Operand::BC), "DEC BC"),
        Opcode::new(0x1B, 1, 8, Instruction::DEC16(Operand::DE), "DEC DE"),
        Opcode::new(0x2B, 1, 8, Instruction::DEC16(Operand::HL), "DEC HL"),
        Opcode::new(0x3B, 1, 8, Instruction::DEC16(Operand::SP), "DEC SP"),

        Opcode::new(0x04, 1, 4, Instruction::INC8(Operand::B), "INC B"),
        Opcode::new(0x14, 1, 4, Instruction::INC8(Operand::D), "INC D"),
        Opcode::new(0x24, 1, 4, Instruction::INC8(Operand::H), "INC H"),
        Opcode::new(0x34, 1, 12, Instruction::INC8(Operand::HL), "INC (HL)"),
        Opcode::new(0x0C, 1, 4, Instruction::INC8(Operand::C), "INC C"),
        Opcode::new(0x1C, 1, 4, Instruction::INC8(Operand::E), "INC E"),
        Opcode::new(0x2C, 1, 4, Instruction::INC8(Operand::L), "INC L"),
        Opcode::new(0x3C, 1, 4, Instruction::INC8(Operand::A), "INC A"),

        Opcode::new(0x05, 1, 4, Instruction::DEC8(Operand::B), "DEC B"),
        Opcode::new(0x15, 1, 4, Instruction::DEC8(Operand::D), "DEC D"),
        Opcode::new(0x25, 1, 4, Instruction::DEC8(Operand::H), "DEC H"),
        Opcode::new(0x35, 1, 12, Instruction::DEC8(Operand::HL), "DEC (HL)"),
        Opcode::new(0x0D, 1, 4, Instruction::DEC8(Operand::C), "DEC C"),
        Opcode::new(0x1D, 1, 4, Instruction::DEC8(Operand::E), "DEC E"),
        Opcode::new(0x2D, 1, 4, Instruction::DEC8(Operand::L), "DEC L"),
        Opcode::new(0x3D, 1, 4, Instruction::DEC8(Operand::A), "DEC A"),

        Opcode::new(0x06, 2, 8, Instruction::LDIM8(Operand::B), "LD B,u8"),
        Opcode::new(0x16, 2, 8, Instruction::LDIM8(Operand::D), "LD D,u8"),
        Opcode::new(0x26, 2, 8, Instruction::LDIM8(Operand::H), "LD H,u8"),
        Opcode::new(0x36, 2, 12, Instruction::LDIM8(Operand::HL), "LD (HL),u8"),
        Opcode::new(0x0E, 2, 8, Instruction::LDIM8(Operand::C), "LD C,u8"),
        Opcode::new(0x1E, 2, 8, Instruction::LDIM8(Operand::E), "LD E,u8"),
        Opcode::new(0x2E, 2, 8, Instruction::LDIM8(Operand::L), "LD L,u8"),
        Opcode::new(0x3E, 2, 8, Instruction::LDIM8(Operand::A), "LD A,u8"),

        Opcode::new(0x07, 1, 4, Instruction::RLCA, "RLCA"),
        Opcode::new(0x17, 1, 4, Instruction::RLA, "RLA"),
        Opcode::new(0x0F, 1, 4, Instruction::RRCA, "RRCA"),
        Opcode::new(0x1F, 1, 4, Instruction::RRA, "RRA"),

        Opcode::new(0x08, 3, 20, Instruction::LD16SP, "LD (u16),SP"),

        Opcode::new(0x09, 1, 8, Instruction::ADDHL(Operand::BC), "ADD HL,BC"),
        Opcode::new(0x19, 1, 8, Instruction::ADDHL(Operand::DE), "ADD HL,DE"),
        Opcode::new(0x29, 1, 8, Instruction::ADDHL(Operand::HL), "ADD HL,HL"),
        Opcode::new(0x39, 1, 8, Instruction::ADDHL(Operand::SP), "ADD HL,SP"),

        Opcode::new(0x0A, 1, 8, Instruction::LDRR(Operand::A, Operand::BC), "LD A,(BC)"),
        Opcode::new(0x1A, 1, 8, Instruction::LDRR(Operand::A, Operand::DE), "LD A,(DE)"),
        Opcode::new(0x2A, 1, 8, Instruction::LDRR(Operand::A, Operand::HLI), "LD A,(HL+)"),
        Opcode::new(0x3A, 1, 8, Instruction::LDRR(Operand::A, Operand::HLD), "LD A,(HL-)"),

        Opcode::new(0x18, 2, 12, Instruction::JR(None), "JR i8"),
        Opcode::new(0x20, 2, 12, Instruction::JR(Some(Condition::NZ)), "JR NZ,i8"),
        Opcode::new(0x30, 2, 12, Instruction::JR(Some(Condition::NC)), "JR NC,i8"),
        Opcode::new(0x28, 2, 12, Instruction::JR(Some(Condition::Z)), "JR Z,i8"),
        Opcode::new(0x38, 2, 12, Instruction::JR(Some(Condition::C)), "JR C,i8"),

        Opcode::new(0x27, 1, 4, Instruction::DAA, "DAA"),
        Opcode::new(0x37, 1, 4, Instruction::SCF, "SCF"),
        Opcode::new(0x2F, 1, 4, Instruction::CPL, "CPL"),
        Opcode::new(0x3F, 1, 4, Instruction::CCF, "CCF"),

        Opcode::new(0x40, 1, 4, Instruction::LDRR(Operand::B, Operand::B),  "LD B,B"),
        Opcode::new(0x41, 1, 4, Instruction::LDRR(Operand::B, Operand::C),  "LD B,C"),
        Opcode::new(0x42, 1, 4, Instruction::LDRR(Operand::B, Operand::D),  "LD B,D"),
        Opcode::new(0x43, 1, 4, Instruction::LDRR(Operand::B, Operand::E),  "LD B,E"),
        Opcode::new(0x44, 1, 4, Instruction::LDRR(Operand::B, Operand::H),  "LD B,H"),
        Opcode::new(0x45, 1, 4, Instruction::LDRR(Operand::B, Operand::L),  "LD B,L"),
        Opcode::new(0x46, 1, 8, Instruction::LDRR(Operand::B, Operand::HL), "LD B,(HL)"),
        Opcode::new(0x47, 1, 4, Instruction::LDRR(Operand::B, Operand::A),  "LD B,A"),
        Opcode::new(0x48, 1, 4, Instruction::LDRR(Operand::C, Operand::B),  "LD C,B"),
        Opcode::new(0x49, 1, 4, Instruction::LDRR(Operand::C, Operand::C),  "LD C,C"),
        Opcode::new(0x4A, 1, 4, Instruction::LDRR(Operand::C, Operand::D),  "LD C,D"),
        Opcode::new(0x4B, 1, 4, Instruction::LDRR(Operand::C, Operand::E),  "LD C,E"),
        Opcode::new(0x4C, 1, 4, Instruction::LDRR(Operand::C, Operand::H),  "LD C,H"),
        Opcode::new(0x4D, 1, 4, Instruction::LDRR(Operand::C, Operand::L),  "LD C,L"),
        Opcode::new(0x4E, 1, 8, Instruction::LDRR(Operand::C, Operand::HL), "LD C,(HL)"),
        Opcode::new(0x4F, 1, 4, Instruction::LDRR(Operand::C, Operand::A),  "LD C,A"),

        Opcode::new(0x50, 1, 4, Instruction::LDRR(Operand::D, Operand::B),  "LD D,B"),
        Opcode::new(0x51, 1, 4, Instruction::LDRR(Operand::D, Operand::C),  "LD D,C"),
        Opcode::new(0x52, 1, 4, Instruction::LDRR(Operand::D, Operand::D),  "LD D,D"),
        Opcode::new(0x53, 1, 4, Instruction::LDRR(Operand::D, Operand::E),  "LD D,E"),
        Opcode::new(0x54, 1, 4, Instruction::LDRR(Operand::D, Operand::H),  "LD D,H"),
        Opcode::new(0x55, 1, 4, Instruction::LDRR(Operand::D, Operand::L),  "LD D,L"),
        Opcode::new(0x56, 1, 8, Instruction::LDRR(Operand::D, Operand::HL), "LD D,(HL)"),
        Opcode::new(0x57, 1, 4, Instruction::LDRR(Operand::D, Operand::A),  "LD D,A"),
        Opcode::new(0x58, 1, 4, Instruction::LDRR(Operand::E, Operand::B),  "LD E,B"),
        Opcode::new(0x59, 1, 4, Instruction::LDRR(Operand::E, Operand::C),  "LD E,C"),
        Opcode::new(0x5A, 1, 4, Instruction::LDRR(Operand::E, Operand::D),  "LD E,D"),
        Opcode::new(0x5B, 1, 4, Instruction::LDRR(Operand::E, Operand::E),  "LD E,E"),
        Opcode::new(0x5C, 1, 4, Instruction::LDRR(Operand::E, Operand::H),  "LD E,H"),
        Opcode::new(0x5D, 1, 4, Instruction::LDRR(Operand::E, Operand::L),  "LD E,L"),
        Opcode::new(0x5E, 1, 8, Instruction::LDRR(Operand::E, Operand::HL), "LD E,(HL)"),
        Opcode::new(0x5F, 1, 4, Instruction::LDRR(Operand::E, Operand::A),  "LD E,A"),

        Opcode::new(0x60, 1, 4, Instruction::LDRR(Operand::H, Operand::B),  "LD H,B"),
        Opcode::new(0x61, 1, 4, Instruction::LDRR(Operand::H, Operand::C),  "LD H,C"),
        Opcode::new(0x62, 1, 4, Instruction::LDRR(Operand::H, Operand::D),  "LD H,D"),
        Opcode::new(0x63, 1, 4, Instruction::LDRR(Operand::H, Operand::E),  "LD H,E"),
        Opcode::new(0x64, 1, 4, Instruction::LDRR(Operand::H, Operand::H),  "LD H,H"),
        Opcode::new(0x65, 1, 4, Instruction::LDRR(Operand::H, Operand::L),  "LD H,L"),
        Opcode::new(0x66, 1, 8, Instruction::LDRR(Operand::H, Operand::HL), "LD H,(HL)"),
        Opcode::new(0x67, 1, 4, Instruction::LDRR(Operand::H, Operand::A),  "LD H,A"),
        Opcode::new(0x68, 1, 4, Instruction::LDRR(Operand::L, Operand::B),  "LD L,B"),
        Opcode::new(0x69, 1, 4, Instruction::LDRR(Operand::L, Operand::C),  "LD L,C"),
        Opcode::new(0x6A, 1, 4, Instruction::LDRR(Operand::L, Operand::D),  "LD L,D"),
        Opcode::new(0x6B, 1, 4, Instruction::LDRR(Operand::L, Operand::E),  "LD L,E"),
        Opcode::new(0x6C, 1, 4, Instruction::LDRR(Operand::L, Operand::H),  "LD L,H"),
        Opcode::new(0x6D, 1, 4, Instruction::LDRR(Operand::L, Operand::L),  "LD L,L"),
        Opcode::new(0x6E, 1, 8, Instruction::LDRR(Operand::L, Operand::HL), "LD L,(HL)"),
        Opcode::new(0x6F, 1, 4, Instruction::LDRR(Operand::L, Operand::A),  "LD L,A"),

        Opcode::new(0x70, 1, 8, Instruction::LDRR(Operand::HL, Operand::B), "LD (HL),B"),
        Opcode::new(0x71, 1, 8, Instruction::LDRR(Operand::HL, Operand::C), "LD (HL),C"),
        Opcode::new(0x72, 1, 8, Instruction::LDRR(Operand::HL, Operand::D), "LD (HL),D"),
        Opcode::new(0x73, 1, 8, Instruction::LDRR(Operand::HL, Operand::E), "LD (HL),E"),
        Opcode::new(0x74, 1, 8, Instruction::LDRR(Operand::HL, Operand::H), "LD (HL),H"),
        Opcode::new(0x75, 1, 8, Instruction::LDRR(Operand::HL, Operand::L), "LD (HL),L"),
        Opcode::new(0x77, 1, 8, Instruction::LDRR(Operand::HL, Operand::A), "LD (HL),A"),
        Opcode::new(0x78, 1, 4, Instruction::LDRR(Operand::A, Operand::B),  "LD A,B"),
        Opcode::new(0x79, 1, 4, Instruction::LDRR(Operand::A, Operand::C),  "LD A,C"),
        Opcode::new(0x7A, 1, 4, Instruction::LDRR(Operand::A, Operand::D),  "LD A,D"),
        Opcode::new(0x7B, 1, 4, Instruction::LDRR(Operand::A, Operand::E),  "LD A,E"),
        Opcode::new(0x7C, 1, 4, Instruction::LDRR(Operand::A, Operand::H),  "LD A,H"),
        Opcode::new(0x7D, 1, 4, Instruction::LDRR(Operand::A, Operand::L),  "LD A,L"),
        Opcode::new(0x7E, 1, 8, Instruction::LDRR(Operand::A, Operand::HL), "LD A,(HL)"),
        Opcode::new(0x7F, 1, 4, Instruction::LDRR(Operand::A, Operand::A),  "LD A,A"),

        Opcode::new(0x80, 1, 4, Instruction::ADD(Operand::B),  "ADD A,B"),
        Opcode::new(0x81, 1, 4, Instruction::ADD(Operand::C),  "ADD A,C"),
        Opcode::new(0x82, 1, 4, Instruction::ADD(Operand::D),  "ADD A,D"),
        Opcode::new(0x83, 1, 4, Instruction::ADD(Operand::E),  "ADD A,E"),
        Opcode::new(0x84, 1, 4, Instruction::ADD(Operand::H),  "ADD A,H"),
        Opcode::new(0x85, 1, 4, Instruction::ADD(Operand::L),  "ADD A,L"),
        Opcode::new(0x86, 1, 8, Instruction::ADD(Operand::HL), "ADD A,(HL)"),
        Opcode::new(0x87, 1, 4, Instruction::ADD(Operand::A),  "ADD A,A"),
        Opcode::new(0x88, 1, 4, Instruction::ADC(Operand::B),  "ADC A,B"),
        Opcode::new(0x89, 1, 4, Instruction::ADC(Operand::C),  "ADC A,C"),
        Opcode::new(0x8A, 1, 4, Instruction::ADC(Operand::D),  "ADC A,D"),
        Opcode::new(0x8B, 1, 4, Instruction::ADC(Operand::E),  "ADC A,E"),
        Opcode::new(0x8C, 1, 4, Instruction::ADC(Operand::H),  "ADC A,H"),
        Opcode::new(0x8D, 1, 4, Instruction::ADC(Operand::L),  "ADC A,L"),
        Opcode::new(0x8E, 1, 8, Instruction::ADC(Operand::HL), "ADC A,(HL)"),
        Opcode::new(0x8F, 1, 4, Instruction::ADC(Operand::A),  "ADC A,A"),

        Opcode::new(0x90, 1, 4, Instruction::SUB(Operand::B),  "SUB A,B"),
        Opcode::new(0x91, 1, 4, Instruction::SUB(Operand::C),  "SUB A,C"),
        Opcode::new(0x92, 1, 4, Instruction::SUB(Operand::D),  "SUB A,D"),
        Opcode::new(0x93, 1, 4, Instruction::SUB(Operand::E),  "SUB A,E"),
        Opcode::new(0x94, 1, 4, Instruction::SUB(Operand::H),  "SUB A,H"),
        Opcode::new(0x95, 1, 4, Instruction::SUB(Operand::L),  "SUB A,L"),
        Opcode::new(0x96, 1, 8, Instruction::SUB(Operand::HL), "SUB A,(HL)"),
        Opcode::new(0x97, 1, 4, Instruction::SUB(Operand::A),  "SUB A,A"),
        Opcode::new(0x98, 1, 4, Instruction::SBC(Operand::B),  "SBC A,B"),
        Opcode::new(0x99, 1, 4, Instruction::SBC(Operand::C),  "SBC A,C"),
        Opcode::new(0x9A, 1, 4, Instruction::SBC(Operand::D),  "SBC A,D"),
        Opcode::new(0x9B, 1, 4, Instruction::SBC(Operand::E),  "SBC A,E"),
        Opcode::new(0x9C, 1, 4, Instruction::SBC(Operand::H),  "SBC A,H"),
        Opcode::new(0x9D, 1, 4, Instruction::SBC(Operand::L),  "SBC A,L"),
        Opcode::new(0x9E, 1, 8, Instruction::SBC(Operand::HL), "SBC A,(HL)"),
        Opcode::new(0x9F, 1, 4, Instruction::SBC(Operand::A),  "SBC A,A"),

        Opcode::new(0xA0, 1, 4, Instruction::AND(Operand::B),  "AND A,B"),
        Opcode::new(0xA1, 1, 4, Instruction::AND(Operand::C),  "AND A,C"),
        Opcode::new(0xA2, 1, 4, Instruction::AND(Operand::D),  "AND A,D"),
        Opcode::new(0xA3, 1, 4, Instruction::AND(Operand::E),  "AND A,E"),
        Opcode::new(0xA4, 1, 4, Instruction::AND(Operand::H),  "AND A,H"),
        Opcode::new(0xA5, 1, 4, Instruction::AND(Operand::L),  "AND A,L"),
        Opcode::new(0xA6, 1, 8, Instruction::AND(Operand::HL), "AND A,(HL)"),
        Opcode::new(0xA7, 1, 4, Instruction::AND(Operand::A),  "AND A,A"),
        Opcode::new(0xA8, 1, 4, Instruction::XOR(Operand::B),  "XOR A,B"),
        Opcode::new(0xA9, 1, 4, Instruction::XOR(Operand::C),  "XOR A,C"),
        Opcode::new(0xAA, 1, 4, Instruction::XOR(Operand::D),  "XOR A,D"),
        Opcode::new(0xAB, 1, 4, Instruction::XOR(Operand::E),  "XOR A,E"),
        Opcode::new(0xAC, 1, 4, Instruction::XOR(Operand::H),  "XOR A,H"),
        Opcode::new(0xAD, 1, 4, Instruction::XOR(Operand::L),  "XOR A,L"),
        Opcode::new(0xAE, 1, 8, Instruction::XOR(Operand::HL), "XOR A,(HL)"),
        Opcode::new(0xAF, 1, 4, Instruction::XOR(Operand::A),  "XOR A,A"),

        Opcode::new(0xB0, 1, 4, Instruction::OR(Operand::B),  "OR A,B"),
        Opcode::new(0xB1, 1, 4, Instruction::OR(Operand::C),  "OR A,C"),
        Opcode::new(0xB2, 1, 4, Instruction::OR(Operand::D),  "OR A,D"),
        Opcode::new(0xB3, 1, 4, Instruction::OR(Operand::E),  "OR A,E"),
        Opcode::new(0xB4, 1, 4, Instruction::OR(Operand::H),  "OR A,H"),
        Opcode::new(0xB5, 1, 4, Instruction::OR(Operand::L),  "OR A,L"),
        Opcode::new(0xB6, 1, 8, Instruction::OR(Operand::HL), "OR A,(HL)"),
        Opcode::new(0xB7, 1, 4, Instruction::OR(Operand::A),  "OR A,A"),
        Opcode::new(0xB8, 1, 4, Instruction::CP(Operand::B),  "CP A,B"),
        Opcode::new(0xB9, 1, 4, Instruction::CP(Operand::C),  "CP A,C"),
        Opcode::new(0xBA, 1, 4, Instruction::CP(Operand::D),  "CP A,D"),
        Opcode::new(0xBB, 1, 4, Instruction::CP(Operand::E),  "CP A,E"),
        Opcode::new(0xBC, 1, 4, Instruction::CP(Operand::H),  "CP A,H"),
        Opcode::new(0xBD, 1, 4, Instruction::CP(Operand::L),  "CP A,L"),
        Opcode::new(0xBE, 1, 8, Instruction::CP(Operand::HL), "CP A,(HL)"),
        Opcode::new(0xBF, 1, 4, Instruction::CP(Operand::A),  "CP A,A"),

        Opcode::new(0xC9, 1, 16, Instruction::RET(None), "RET"),
        Opcode::new(0xC0, 1, 20, Instruction::RET(Some(Condition::NZ)), "RET NZ"),
        Opcode::new(0xD0, 1, 20, Instruction::RET(Some(Condition::NC)), "RET NC"),
        Opcode::new(0xC8, 1, 20, Instruction::RET(Some(Condition::Z)), "RET Z"),
        Opcode::new(0xD8, 1, 20, Instruction::RET(Some(Condition::C)), "RET C"),

        Opcode::new(0xD9, 1, 16, Instruction::RETI, "RETI"),

        Opcode::new(0xC3, 3, 16, Instruction::JP(None), "JP"),
        Opcode::new(0xC2, 3, 16, Instruction::JP(Some(Condition::NZ)), "JP NZ"),
        Opcode::new(0xD2, 3, 16, Instruction::JP(Some(Condition::NC)), "JP NC"),
        Opcode::new(0xCA, 3, 16, Instruction::JP(Some(Condition::Z)), "JP Z"),
        Opcode::new(0xDA, 3, 16, Instruction::JP(Some(Condition::C)), "JP C"),
        Opcode::new(0xE9, 1, 4, Instruction::JPHL, "JP (HL)"),

        Opcode::new(0xCD, 3, 24, Instruction::CALL(None), "CALL"),
        Opcode::new(0xC4, 3, 24, Instruction::CALL(Some(Condition::NZ)), "CALL NZ"),
        Opcode::new(0xD4, 3, 24, Instruction::CALL(Some(Condition::NC)), "CALL NC"),
        Opcode::new(0xCC, 3, 24, Instruction::CALL(Some(Condition::Z)), "CALL Z"),
        Opcode::new(0xDC, 3, 24, Instruction::CALL(Some(Condition::C)), "CALL C"),

        Opcode::new(0xC1, 1, 12, Instruction::POP(Operand::BC), "POP BC"),
        Opcode::new(0xD1, 1, 12, Instruction::POP(Operand::DE), "POP DE"),
        Opcode::new(0xE1, 1, 12, Instruction::POP(Operand::HL), "POP HL"),
        Opcode::new(0xF1, 1, 12, Instruction::POP(Operand::AF), "POP AF"),

        Opcode::new(0xC5, 1, 16, Instruction::PUSH(Operand::BC), "PUSH BC"),
        Opcode::new(0xD5, 1, 16, Instruction::PUSH(Operand::DE), "PUSH DE"),
        Opcode::new(0xE5, 1, 16, Instruction::PUSH(Operand::HL), "PUSH HL"),
        Opcode::new(0xF5, 1, 16, Instruction::PUSH(Operand::AF), "PUSH AF"),

        Opcode::new(0xC6, 2, 8, Instruction::ADD(Operand::IM8), "ADD A,u8"),
        Opcode::new(0xD6, 2, 8, Instruction::SUB(Operand::IM8), "SUB A,u8"),
        Opcode::new(0xE6, 2, 8, Instruction::AND(Operand::IM8), "AND A,u8"),
        Opcode::new(0xF6, 2, 8, Instruction::OR(Operand::IM8) , "OR A,u8"),
        Opcode::new(0xCE, 2, 8, Instruction::ADC(Operand::IM8), "ADC A,u8"),
        Opcode::new(0xDE, 2, 8, Instruction::SBC(Operand::IM8), "SBC A,u8"),
        Opcode::new(0xEE, 2, 8, Instruction::XOR(Operand::IM8), "XOR A,u8"),
        Opcode::new(0xFE, 2, 8, Instruction::CP(Operand::IM8) , "CP A,u8"),

        Opcode::new(0xE0, 2, 12, Instruction::LDFF8A, "LD (FF00+u8),A"),
        Opcode::new(0xE2, 1, 8, Instruction::LDFFCA, "LD (FF00+C),A"),
        Opcode::new(0xF0, 2, 12, Instruction::LDAFF8, "LD A,(FF00+u8)"),
        Opcode::new(0xF2, 1, 8, Instruction::LDAFFC, "LD A,(FF00+C)"),

        Opcode::new(0xEA, 3, 16, Instruction::LD16A, "LD (u16),A"),
        Opcode::new(0xFA, 3, 16, Instruction::LDA16, "LD A,(u16)"),

        Opcode::new(0xF3, 1, 4, Instruction::DI, "DI"),
        Opcode::new(0xFB, 1, 4, Instruction::EI, "EI"),

        Opcode::new(0xE8, 2, 16, Instruction::ADDSP, "ADD SP,i8"),
        Opcode::new(0xF8, 2, 12, Instruction::ADDHLSP, "ADD HL,SP+i8"),

        Opcode::new(0xF9, 1, 8, Instruction::LDSPHL, "LD SP,HL"),

        Opcode::new(0xC7, 1, 16, Instruction::RST(0x00), "RST 0x00"),
        Opcode::new(0xD7, 1, 16, Instruction::RST(0x10), "RST 0x10"),
        Opcode::new(0xE7, 1, 16, Instruction::RST(0x20), "RST 0x20"),
        Opcode::new(0xF7, 1, 16, Instruction::RST(0x30), "RST 0x30"),
        Opcode::new(0xCF, 1, 16, Instruction::RST(0x08), "RST 0x08"),
        Opcode::new(0xDF, 1, 16, Instruction::RST(0x18), "RST 0x18"),
        Opcode::new(0xEF, 1, 16, Instruction::RST(0x28), "RST 0x28"),
        Opcode::new(0xFF, 1, 16, Instruction::RST(0x38), "RST 0x38"),


        // PREFIXED
        Opcode::new(0xCB, 1, 0, Instruction::CB, "0xCB"),

        Opcode::cb(0x00, 1, 8,  Instruction::RLC(Operand::B),  "RLC B"),
        Opcode::cb(0x01, 1, 8,  Instruction::RLC(Operand::C),  "RLC C"),
        Opcode::cb(0x02, 1, 8,  Instruction::RLC(Operand::D),  "RLC D"),
        Opcode::cb(0x03, 1, 8,  Instruction::RLC(Operand::E),  "RLC E"),
        Opcode::cb(0x04, 1, 8,  Instruction::RLC(Operand::H),  "RLC H"),
        Opcode::cb(0x05, 1, 8,  Instruction::RLC(Operand::L),  "RLC L"),
        Opcode::cb(0x06, 1, 16, Instruction::RLC(Operand::HL), "RLC (HL)"),
        Opcode::cb(0x07, 1, 8,  Instruction::RLC(Operand::A),  "RLC A"),
        Opcode::cb(0x08, 1, 8,  Instruction::RRC(Operand::B),  "RRC B"),
        Opcode::cb(0x09, 1, 8,  Instruction::RRC(Operand::C),  "RRC C"),
        Opcode::cb(0x0A, 1, 8,  Instruction::RRC(Operand::D),  "RRC D"),
        Opcode::cb(0x0B, 1, 8,  Instruction::RRC(Operand::E),  "RRC E"),
        Opcode::cb(0x0C, 1, 8,  Instruction::RRC(Operand::H),  "RRC H"),
        Opcode::cb(0x0D, 1, 8,  Instruction::RRC(Operand::L),  "RRC L"),
        Opcode::cb(0x0E, 1, 16, Instruction::RRC(Operand::HL), "RRC (HL)"),
        Opcode::cb(0x0F, 1, 8,  Instruction::RRC(Operand::A),  "RRC A"),

        Opcode::cb(0x10, 1, 8,  Instruction::RL(Operand::B),  "RL B"),
        Opcode::cb(0x11, 1, 8,  Instruction::RL(Operand::C),  "RL C"),
        Opcode::cb(0x12, 1, 8,  Instruction::RL(Operand::D),  "RL D"),
        Opcode::cb(0x13, 1, 8,  Instruction::RL(Operand::E),  "RL E"),
        Opcode::cb(0x14, 1, 8,  Instruction::RL(Operand::H),  "RL H"),
        Opcode::cb(0x15, 1, 8,  Instruction::RL(Operand::L),  "RL L"),
        Opcode::cb(0x16, 1, 16, Instruction::RL(Operand::HL), "RL (HL)"),
        Opcode::cb(0x17, 1, 8,  Instruction::RL(Operand::A),  "RL A"),
        Opcode::cb(0x18, 1, 8,  Instruction::RR(Operand::B),  "RR B"),
        Opcode::cb(0x19, 1, 8,  Instruction::RR(Operand::C),  "RR C"),
        Opcode::cb(0x1A, 1, 8,  Instruction::RR(Operand::D),  "RR D"),
        Opcode::cb(0x1B, 1, 8,  Instruction::RR(Operand::E),  "RR E"),
        Opcode::cb(0x1C, 1, 8,  Instruction::RR(Operand::H),  "RR H"),
        Opcode::cb(0x1D, 1, 8,  Instruction::RR(Operand::L),  "RR L"),
        Opcode::cb(0x1E, 1, 16, Instruction::RR(Operand::HL), "RR (HL)"),
        Opcode::cb(0x1F, 1, 8,  Instruction::RR(Operand::A),  "RR A"),

        Opcode::cb(0x20, 1, 8,  Instruction::SLA(Operand::B),  "SLA B"),
        Opcode::cb(0x21, 1, 8,  Instruction::SLA(Operand::C),  "SLA C"),
        Opcode::cb(0x22, 1, 8,  Instruction::SLA(Operand::D),  "SLA D"),
        Opcode::cb(0x23, 1, 8,  Instruction::SLA(Operand::E),  "SLA E"),
        Opcode::cb(0x24, 1, 8,  Instruction::SLA(Operand::H),  "SLA H"),
        Opcode::cb(0x25, 1, 8,  Instruction::SLA(Operand::L),  "SLA L"),
        Opcode::cb(0x26, 1, 16, Instruction::SLA(Operand::HL), "SLA (HL)"),
        Opcode::cb(0x27, 1, 8,  Instruction::SLA(Operand::A),  "SLA A"),
        Opcode::cb(0x28, 1, 8,  Instruction::SRA(Operand::B),  "SRA B"),
        Opcode::cb(0x29, 1, 8,  Instruction::SRA(Operand::C),  "SRA C"),
        Opcode::cb(0x2A, 1, 8,  Instruction::SRA(Operand::D),  "SRA D"),
        Opcode::cb(0x2B, 1, 8,  Instruction::SRA(Operand::E),  "SRA E"),
        Opcode::cb(0x2C, 1, 8,  Instruction::SRA(Operand::H),  "SRA H"),
        Opcode::cb(0x2D, 1, 8,  Instruction::SRA(Operand::L),  "SRA L"),
        Opcode::cb(0x2E, 1, 16, Instruction::SRA(Operand::HL), "SRA (HL)"),
        Opcode::cb(0x2F, 1, 8,  Instruction::SRA(Operand::A),  "SRA A"),

        Opcode::cb(0x30, 1, 8,  Instruction::SWAP(Operand::B),  "SWAP B"),
        Opcode::cb(0x31, 1, 8,  Instruction::SWAP(Operand::C),  "SWAP C"),
        Opcode::cb(0x32, 1, 8,  Instruction::SWAP(Operand::D),  "SWAP D"),
        Opcode::cb(0x33, 1, 8,  Instruction::SWAP(Operand::E),  "SWAP E"),
        Opcode::cb(0x34, 1, 8,  Instruction::SWAP(Operand::H),  "SWAP H"),
        Opcode::cb(0x35, 1, 8,  Instruction::SWAP(Operand::L),  "SWAP L"),
        Opcode::cb(0x36, 1, 16, Instruction::SWAP(Operand::HL), "SWAP (HL)"),
        Opcode::cb(0x37, 1, 8,  Instruction::SWAP(Operand::A),  "SWAP A"),
        Opcode::cb(0x38, 1, 8,  Instruction::SRL(Operand::B),  "SRL B"),
        Opcode::cb(0x39, 1, 8,  Instruction::SRL(Operand::C),  "SRL C"),
        Opcode::cb(0x3A, 1, 8,  Instruction::SRL(Operand::D),  "SRL D"),
        Opcode::cb(0x3B, 1, 8,  Instruction::SRL(Operand::E),  "SRL E"),
        Opcode::cb(0x3C, 1, 8,  Instruction::SRL(Operand::H),  "SRL H"),
        Opcode::cb(0x3D, 1, 8,  Instruction::SRL(Operand::L),  "SRL L"),
        Opcode::cb(0x3E, 1, 16, Instruction::SRL(Operand::HL), "SRL (HL)"),
        Opcode::cb(0x3F, 1, 8,  Instruction::SRL(Operand::A),  "SRL A"),

        Opcode::cb(0x40, 1, 8,  Instruction::BIT(0, Operand::B),  "BIT 0,B"),
        Opcode::cb(0x41, 1, 8,  Instruction::BIT(0, Operand::C),  "BIT 0,C"),
        Opcode::cb(0x42, 1, 8,  Instruction::BIT(0, Operand::D),  "BIT 0,D"),
        Opcode::cb(0x43, 1, 8,  Instruction::BIT(0, Operand::E),  "BIT 0,E"),
        Opcode::cb(0x44, 1, 8,  Instruction::BIT(0, Operand::H),  "BIT 0,H"),
        Opcode::cb(0x45, 1, 8,  Instruction::BIT(0, Operand::L),  "BIT 0,L"),
        Opcode::cb(0x46, 1, 12, Instruction::BIT(0, Operand::HL), "BIT 0,(HL)"),
        Opcode::cb(0x47, 1, 8,  Instruction::BIT(0, Operand::A),  "BIT 0,A"),
        Opcode::cb(0x48, 1, 8,  Instruction::BIT(1, Operand::B),  "BIT 1,B"),
        Opcode::cb(0x49, 1, 8,  Instruction::BIT(1, Operand::C),  "BIT 1,C"),
        Opcode::cb(0x4A, 1, 8,  Instruction::BIT(1, Operand::D),  "BIT 1,D"),
        Opcode::cb(0x4B, 1, 8,  Instruction::BIT(1, Operand::E),  "BIT 1,E"),
        Opcode::cb(0x4C, 1, 8,  Instruction::BIT(1, Operand::H),  "BIT 1,H"),
        Opcode::cb(0x4D, 1, 8,  Instruction::BIT(1, Operand::L),  "BIT 1,L"),
        Opcode::cb(0x4E, 1, 12, Instruction::BIT(1, Operand::HL), "BIT 1,(HL)"),
        Opcode::cb(0x4F, 1, 8,  Instruction::BIT(1, Operand::A),  "BIT 1,A"),
        Opcode::cb(0x50, 1, 8,  Instruction::BIT(2, Operand::B),  "BIT 2,B"),
        Opcode::cb(0x51, 1, 8,  Instruction::BIT(2, Operand::C),  "BIT 2,C"),
        Opcode::cb(0x52, 1, 8,  Instruction::BIT(2, Operand::D),  "BIT 2,D"),
        Opcode::cb(0x53, 1, 8,  Instruction::BIT(2, Operand::E),  "BIT 2,E"),
        Opcode::cb(0x54, 1, 8,  Instruction::BIT(2, Operand::H),  "BIT 2,H"),
        Opcode::cb(0x55, 1, 8,  Instruction::BIT(2, Operand::L),  "BIT 2,L"),
        Opcode::cb(0x56, 1, 12, Instruction::BIT(2, Operand::HL), "BIT 2,(HL)"),
        Opcode::cb(0x57, 1, 8,  Instruction::BIT(2, Operand::A),  "BIT 2,A"),
        Opcode::cb(0x58, 1, 8,  Instruction::BIT(3, Operand::B),  "BIT 3,B"),
        Opcode::cb(0x59, 1, 8,  Instruction::BIT(3, Operand::C),  "BIT 3,C"),
        Opcode::cb(0x5A, 1, 8,  Instruction::BIT(3, Operand::D),  "BIT 3,D"),
        Opcode::cb(0x5B, 1, 8,  Instruction::BIT(3, Operand::E),  "BIT 3,E"),
        Opcode::cb(0x5C, 1, 8,  Instruction::BIT(3, Operand::H),  "BIT 3,H"),
        Opcode::cb(0x5D, 1, 8,  Instruction::BIT(3, Operand::L),  "BIT 3,L"),
        Opcode::cb(0x5E, 1, 12, Instruction::BIT(3, Operand::HL), "BIT 3,(HL)"),
        Opcode::cb(0x5F, 1, 8,  Instruction::BIT(3, Operand::A),  "BIT 3,A"),
        Opcode::cb(0x60, 1, 8,  Instruction::BIT(4, Operand::B),  "BIT 4,B"),
        Opcode::cb(0x61, 1, 8,  Instruction::BIT(4, Operand::C),  "BIT 4,C"),
        Opcode::cb(0x62, 1, 8,  Instruction::BIT(4, Operand::D),  "BIT 4,D"),
        Opcode::cb(0x63, 1, 8,  Instruction::BIT(4, Operand::E),  "BIT 4,E"),
        Opcode::cb(0x64, 1, 8,  Instruction::BIT(4, Operand::H),  "BIT 4,H"),
        Opcode::cb(0x65, 1, 8,  Instruction::BIT(4, Operand::L),  "BIT 4,L"),
        Opcode::cb(0x66, 1, 12, Instruction::BIT(4, Operand::HL), "BIT 4,(HL)"),
        Opcode::cb(0x67, 1, 8,  Instruction::BIT(4, Operand::A),  "BIT 4,A"),
        Opcode::cb(0x68, 1, 8,  Instruction::BIT(5, Operand::B),  "BIT 5,B"),
        Opcode::cb(0x69, 1, 8,  Instruction::BIT(5, Operand::C),  "BIT 5,C"),
        Opcode::cb(0x6A, 1, 8,  Instruction::BIT(5, Operand::D),  "BIT 5,D"),
        Opcode::cb(0x6B, 1, 8,  Instruction::BIT(5, Operand::E),  "BIT 5,E"),
        Opcode::cb(0x6C, 1, 8,  Instruction::BIT(5, Operand::H),  "BIT 5,H"),
        Opcode::cb(0x6D, 1, 8,  Instruction::BIT(5, Operand::L),  "BIT 5,L"),
        Opcode::cb(0x6E, 1, 12, Instruction::BIT(5, Operand::HL), "BIT 5,(HL)"),
        Opcode::cb(0x6F, 1, 8,  Instruction::BIT(5, Operand::A),  "BIT 5,A"),
        Opcode::cb(0x70, 1, 8,  Instruction::BIT(6, Operand::B),  "BIT 6,B"),
        Opcode::cb(0x71, 1, 8,  Instruction::BIT(6, Operand::C),  "BIT 6,C"),
        Opcode::cb(0x72, 1, 8,  Instruction::BIT(6, Operand::D),  "BIT 6,D"),
        Opcode::cb(0x73, 1, 8,  Instruction::BIT(6, Operand::E),  "BIT 6,E"),
        Opcode::cb(0x74, 1, 8,  Instruction::BIT(6, Operand::H),  "BIT 6,H"),
        Opcode::cb(0x75, 1, 8,  Instruction::BIT(6, Operand::L),  "BIT 6,L"),
        Opcode::cb(0x76, 1, 12, Instruction::BIT(6, Operand::HL), "BIT 6,(HL)"),
        Opcode::cb(0x77, 1, 8,  Instruction::BIT(6, Operand::A),  "BIT 6,A"),
        Opcode::cb(0x78, 1, 8,  Instruction::BIT(7, Operand::B),  "BIT 7,B"),
        Opcode::cb(0x79, 1, 8,  Instruction::BIT(7, Operand::C),  "BIT 7,C"),
        Opcode::cb(0x7A, 1, 8,  Instruction::BIT(7, Operand::D),  "BIT 7,D"),
        Opcode::cb(0x7B, 1, 8,  Instruction::BIT(7, Operand::E),  "BIT 7,E"),
        Opcode::cb(0x7C, 1, 8,  Instruction::BIT(7, Operand::H),  "BIT 7,H"),
        Opcode::cb(0x7D, 1, 8,  Instruction::BIT(7, Operand::L),  "BIT 7,L"),
        Opcode::cb(0x7E, 1, 12, Instruction::BIT(7, Operand::HL), "BIT 7,(HL)"),
        Opcode::cb(0x7F, 1, 8,  Instruction::BIT(7, Operand::A),  "BIT 7,A"),

        Opcode::cb(0x80, 1, 8,  Instruction::RES(0, Operand::B),  "RES 0,B"),
        Opcode::cb(0x81, 1, 8,  Instruction::RES(0, Operand::C),  "RES 0,C"),
        Opcode::cb(0x82, 1, 8,  Instruction::RES(0, Operand::D),  "RES 0,D"),
        Opcode::cb(0x83, 1, 8,  Instruction::RES(0, Operand::E),  "RES 0,E"),
        Opcode::cb(0x84, 1, 8,  Instruction::RES(0, Operand::H),  "RES 0,H"),
        Opcode::cb(0x85, 1, 8,  Instruction::RES(0, Operand::L),  "RES 0,L"),
        Opcode::cb(0x86, 1, 16, Instruction::RES(0, Operand::HL), "RES 0,(HL)"),
        Opcode::cb(0x87, 1, 8,  Instruction::RES(0, Operand::A),  "RES 0,A"),
        Opcode::cb(0x88, 1, 8,  Instruction::RES(1, Operand::B),  "RES 1,B"),
        Opcode::cb(0x89, 1, 8,  Instruction::RES(1, Operand::C),  "RES 1,C"),
        Opcode::cb(0x8A, 1, 8,  Instruction::RES(1, Operand::D),  "RES 1,D"),
        Opcode::cb(0x8B, 1, 8,  Instruction::RES(1, Operand::E),  "RES 1,E"),
        Opcode::cb(0x8C, 1, 8,  Instruction::RES(1, Operand::H),  "RES 1,H"),
        Opcode::cb(0x8D, 1, 8,  Instruction::RES(1, Operand::L),  "RES 1,L"),
        Opcode::cb(0x8E, 1, 16, Instruction::RES(1, Operand::HL), "RES 1,(HL)"),
        Opcode::cb(0x8F, 1, 8,  Instruction::RES(1, Operand::A),  "RES 1,A"),
        Opcode::cb(0x90, 1, 8,  Instruction::RES(2, Operand::B),  "RES 2,B"),
        Opcode::cb(0x91, 1, 8,  Instruction::RES(2, Operand::C),  "RES 2,C"),
        Opcode::cb(0x92, 1, 8,  Instruction::RES(2, Operand::D),  "RES 2,D"),
        Opcode::cb(0x93, 1, 8,  Instruction::RES(2, Operand::E),  "RES 2,E"),
        Opcode::cb(0x94, 1, 8,  Instruction::RES(2, Operand::H),  "RES 2,H"),
        Opcode::cb(0x95, 1, 8,  Instruction::RES(2, Operand::L),  "RES 2,L"),
        Opcode::cb(0x96, 1, 16, Instruction::RES(2, Operand::HL), "RES 2,(HL)"),
        Opcode::cb(0x97, 1, 8,  Instruction::RES(2, Operand::A),  "RES 2,A"),
        Opcode::cb(0x98, 1, 8,  Instruction::RES(3, Operand::B),  "RES 3,B"),
        Opcode::cb(0x99, 1, 8,  Instruction::RES(3, Operand::C),  "RES 3,C"),
        Opcode::cb(0x9A, 1, 8,  Instruction::RES(3, Operand::D),  "RES 3,D"),
        Opcode::cb(0x9B, 1, 8,  Instruction::RES(3, Operand::E),  "RES 3,E"),
        Opcode::cb(0x9C, 1, 8,  Instruction::RES(3, Operand::H),  "RES 3,H"),
        Opcode::cb(0x9D, 1, 8,  Instruction::RES(3, Operand::L),  "RES 3,L"),
        Opcode::cb(0x9E, 1, 16, Instruction::RES(3, Operand::HL), "RES 3,(HL)"),
        Opcode::cb(0x9F, 1, 8,  Instruction::RES(3, Operand::A),  "RES 3,A"),
        Opcode::cb(0xA0, 1, 8,  Instruction::RES(4, Operand::B),  "RES 4,B"),
        Opcode::cb(0xA1, 1, 8,  Instruction::RES(4, Operand::C),  "RES 4,C"),
        Opcode::cb(0xA2, 1, 8,  Instruction::RES(4, Operand::D),  "RES 4,D"),
        Opcode::cb(0xA3, 1, 8,  Instruction::RES(4, Operand::E),  "RES 4,E"),
        Opcode::cb(0xA4, 1, 8,  Instruction::RES(4, Operand::H),  "RES 4,H"),
        Opcode::cb(0xA5, 1, 8,  Instruction::RES(4, Operand::L),  "RES 4,L"),
        Opcode::cb(0xA6, 1, 16, Instruction::RES(4, Operand::HL), "RES 4,(HL)"),
        Opcode::cb(0xA7, 1, 8,  Instruction::RES(4, Operand::A),  "RES 4,A"),
        Opcode::cb(0xA8, 1, 8,  Instruction::RES(5, Operand::B),  "RES 5,B"),
        Opcode::cb(0xA9, 1, 8,  Instruction::RES(5, Operand::C),  "RES 5,C"),
        Opcode::cb(0xAA, 1, 8,  Instruction::RES(5, Operand::D),  "RES 5,D"),
        Opcode::cb(0xAB, 1, 8,  Instruction::RES(5, Operand::E),  "RES 5,E"),
        Opcode::cb(0xAC, 1, 8,  Instruction::RES(5, Operand::H),  "RES 5,H"),
        Opcode::cb(0xAD, 1, 8,  Instruction::RES(5, Operand::L),  "RES 5,L"),
        Opcode::cb(0xAE, 1, 16, Instruction::RES(5, Operand::HL), "RES 5,(HL)"),
        Opcode::cb(0xAF, 1, 8,  Instruction::RES(5, Operand::A),  "RES 5,A"),
        Opcode::cb(0xB0, 1, 8,  Instruction::RES(6, Operand::B),  "RES 6,B"),
        Opcode::cb(0xB1, 1, 8,  Instruction::RES(6, Operand::C),  "RES 6,C"),
        Opcode::cb(0xB2, 1, 8,  Instruction::RES(6, Operand::D),  "RES 6,D"),
        Opcode::cb(0xB3, 1, 8,  Instruction::RES(6, Operand::E),  "RES 6,E"),
        Opcode::cb(0xB4, 1, 8,  Instruction::RES(6, Operand::H),  "RES 6,H"),
        Opcode::cb(0xB5, 1, 8,  Instruction::RES(6, Operand::L),  "RES 6,L"),
        Opcode::cb(0xB6, 1, 16, Instruction::RES(6, Operand::HL), "RES 6,(HL)"),
        Opcode::cb(0xB7, 1, 8,  Instruction::RES(6, Operand::A),  "RES 6,A"),
        Opcode::cb(0xB8, 1, 8,  Instruction::RES(7, Operand::B),  "RES 7,B"),
        Opcode::cb(0xB9, 1, 8,  Instruction::RES(7, Operand::C),  "RES 7,C"),
        Opcode::cb(0xBA, 1, 8,  Instruction::RES(7, Operand::D),  "RES 7,D"),
        Opcode::cb(0xBB, 1, 8,  Instruction::RES(7, Operand::E),  "RES 7,E"),
        Opcode::cb(0xBC, 1, 8,  Instruction::RES(7, Operand::H),  "RES 7,H"),
        Opcode::cb(0xBD, 1, 8,  Instruction::RES(7, Operand::L),  "RES 7,L"),
        Opcode::cb(0xBE, 1, 16, Instruction::RES(7, Operand::HL), "RES 7,(HL)"),
        Opcode::cb(0xBF, 1, 8,  Instruction::RES(7, Operand::A),  "RES 7,A"),

        Opcode::cb(0xC0, 1, 8,  Instruction::SET(0, Operand::B),  "SET 0,B"),
        Opcode::cb(0xC1, 1, 8,  Instruction::SET(0, Operand::C),  "SET 0,C"),
        Opcode::cb(0xC2, 1, 8,  Instruction::SET(0, Operand::D),  "SET 0,D"),
        Opcode::cb(0xC3, 1, 8,  Instruction::SET(0, Operand::E),  "SET 0,E"),
        Opcode::cb(0xC4, 1, 8,  Instruction::SET(0, Operand::H),  "SET 0,H"),
        Opcode::cb(0xC5, 1, 8,  Instruction::SET(0, Operand::L),  "SET 0,L"),
        Opcode::cb(0xC6, 1, 16, Instruction::SET(0, Operand::HL), "SET 0,(HL)"),
        Opcode::cb(0xC7, 1, 8,  Instruction::SET(0, Operand::A),  "SET 0,A"),
        Opcode::cb(0xC8, 1, 8,  Instruction::SET(1, Operand::B),  "SET 1,B"),
        Opcode::cb(0xC9, 1, 8,  Instruction::SET(1, Operand::C),  "SET 1,C"),
        Opcode::cb(0xCA, 1, 8,  Instruction::SET(1, Operand::D),  "SET 1,D"),
        Opcode::cb(0xCB, 1, 8,  Instruction::SET(1, Operand::E),  "SET 1,E"),
        Opcode::cb(0xCC, 1, 8,  Instruction::SET(1, Operand::H),  "SET 1,H"),
        Opcode::cb(0xCD, 1, 8,  Instruction::SET(1, Operand::L),  "SET 1,L"),
        Opcode::cb(0xCE, 1, 16, Instruction::SET(1, Operand::HL), "SET 1,(HL)"),
        Opcode::cb(0xCF, 1, 8,  Instruction::SET(1, Operand::A),  "SET 1,A"),
        Opcode::cb(0xD0, 1, 8,  Instruction::SET(2, Operand::B),  "SET 2,B"),
        Opcode::cb(0xD1, 1, 8,  Instruction::SET(2, Operand::C),  "SET 2,C"),
        Opcode::cb(0xD2, 1, 8,  Instruction::SET(2, Operand::D),  "SET 2,D"),
        Opcode::cb(0xD3, 1, 8,  Instruction::SET(2, Operand::E),  "SET 2,E"),
        Opcode::cb(0xD4, 1, 8,  Instruction::SET(2, Operand::H),  "SET 2,H"),
        Opcode::cb(0xD5, 1, 8,  Instruction::SET(2, Operand::L),  "SET 2,L"),
        Opcode::cb(0xD6, 1, 16, Instruction::SET(2, Operand::HL), "SET 2,(HL)"),
        Opcode::cb(0xD7, 1, 8,  Instruction::SET(2, Operand::A),  "SET 2,A"),
        Opcode::cb(0xD8, 1, 8,  Instruction::SET(3, Operand::B),  "SET 3,B"),
        Opcode::cb(0xD9, 1, 8,  Instruction::SET(3, Operand::C),  "SET 3,C"),
        Opcode::cb(0xDA, 1, 8,  Instruction::SET(3, Operand::D),  "SET 3,D"),
        Opcode::cb(0xDB, 1, 8,  Instruction::SET(3, Operand::E),  "SET 3,E"),
        Opcode::cb(0xDC, 1, 8,  Instruction::SET(3, Operand::H),  "SET 3,H"),
        Opcode::cb(0xDD, 1, 8,  Instruction::SET(3, Operand::L),  "SET 3,L"),
        Opcode::cb(0xDE, 1, 16, Instruction::SET(3, Operand::HL), "SET 3,(HL)"),
        Opcode::cb(0xDF, 1, 8,  Instruction::SET(3, Operand::A),  "SET 3,A"),
        Opcode::cb(0xE0, 1, 8,  Instruction::SET(4, Operand::B),  "SET 4,B"),
        Opcode::cb(0xE1, 1, 8,  Instruction::SET(4, Operand::C),  "SET 4,C"),
        Opcode::cb(0xE2, 1, 8,  Instruction::SET(4, Operand::D),  "SET 4,D"),
        Opcode::cb(0xE3, 1, 8,  Instruction::SET(4, Operand::E),  "SET 4,E"),
        Opcode::cb(0xE4, 1, 8,  Instruction::SET(4, Operand::H),  "SET 4,H"),
        Opcode::cb(0xE5, 1, 8,  Instruction::SET(4, Operand::L),  "SET 4,L"),
        Opcode::cb(0xE6, 1, 16, Instruction::SET(4, Operand::HL), "SET 4,(HL)"),
        Opcode::cb(0xE7, 1, 8,  Instruction::SET(4, Operand::A),  "SET 4,A"),
        Opcode::cb(0xE8, 1, 8,  Instruction::SET(5, Operand::B),  "SET 5,B"),
        Opcode::cb(0xE9, 1, 8,  Instruction::SET(5, Operand::C),  "SET 5,C"),
        Opcode::cb(0xEA, 1, 8,  Instruction::SET(5, Operand::D),  "SET 5,D"),
        Opcode::cb(0xEB, 1, 8,  Instruction::SET(5, Operand::E),  "SET 5,E"),
        Opcode::cb(0xEC, 1, 8,  Instruction::SET(5, Operand::H),  "SET 5,H"),
        Opcode::cb(0xED, 1, 8,  Instruction::SET(5, Operand::L),  "SET 5,L"),
        Opcode::cb(0xEE, 1, 16, Instruction::SET(5, Operand::HL), "SET 5,(HL)"),
        Opcode::cb(0xEF, 1, 8,  Instruction::SET(5, Operand::A),  "SET 5,A"),
        Opcode::cb(0xF0, 1, 8,  Instruction::SET(6, Operand::B),  "SET 6,B"),
        Opcode::cb(0xF1, 1, 8,  Instruction::SET(6, Operand::C),  "SET 6,C"),
        Opcode::cb(0xF2, 1, 8,  Instruction::SET(6, Operand::D),  "SET 6,D"),
        Opcode::cb(0xF3, 1, 8,  Instruction::SET(6, Operand::E),  "SET 6,E"),
        Opcode::cb(0xF4, 1, 8,  Instruction::SET(6, Operand::H),  "SET 6,H"),
        Opcode::cb(0xF5, 1, 8,  Instruction::SET(6, Operand::L),  "SET 6,L"),
        Opcode::cb(0xF6, 1, 16, Instruction::SET(6, Operand::HL), "SET 6,(HL)"),
        Opcode::cb(0xF7, 1, 8,  Instruction::SET(6, Operand::A),  "SET 6,A"),
        Opcode::cb(0xF8, 1, 8,  Instruction::SET(7, Operand::B),  "SET 7,B"),
        Opcode::cb(0xF9, 1, 8,  Instruction::SET(7, Operand::C),  "SET 7,C"),
        Opcode::cb(0xFA, 1, 8,  Instruction::SET(7, Operand::D),  "SET 7,D"),
        Opcode::cb(0xFB, 1, 8,  Instruction::SET(7, Operand::E),  "SET 7,E"),
        Opcode::cb(0xFC, 1, 8,  Instruction::SET(7, Operand::H),  "SET 7,H"),
        Opcode::cb(0xFD, 1, 8,  Instruction::SET(7, Operand::L),  "SET 7,L"),
        Opcode::cb(0xFE, 1, 16, Instruction::SET(7, Operand::HL), "SET 7,(HL)"),
        Opcode::cb(0xFF, 1, 8,  Instruction::SET(7, Operand::A),  "SET 7,A"),
    ];
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn check_opcode_duplicates() {
        env_logger::init();

        // unprefixed
        let has_duplicates = has_duplicated_opcodes(OPCODE_MAP.values().collect(), false);
        assert!(!has_duplicates);
        // prefixed
        let cb_has_duplicates = has_duplicated_opcodes(CB_OPCODE_MAP.values().collect(), true);
        assert!(!cb_has_duplicates);
    }

    fn has_duplicated_opcodes(source: Vec<&&Opcode>, prefixed: bool) -> bool {
        let mut unique = source.clone();

        let mut set = HashSet::new();
        unique.retain(|op| {
            let unique = set.insert(op.code);
            if !unique {
                log::error!("Duplicated: 0x{:02x}", op.code);
            }
            unique
        });

        let opcode_vec = OPCODE_VEC
            .iter()
            .filter(|opcode| opcode.prefixed == prefixed)
            .collect::<Vec<_>>();

        unique.len() != opcode_vec.len()
    }
}
