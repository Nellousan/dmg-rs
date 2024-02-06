#[derive(Default)]
#[allow(dead_code)]
pub struct Instruction {
    pub address: u16,
    pub opcode: u8,
    pub mnemonic: String,
    pub length: u32,
    immediate_8: Option<u8>,
    immediate_16: Option<u16>,
    // cycles: u32,
    // cycles2: Option<u32>,
}

impl Instruction {
    pub fn new(address: u16, opcode: u8, mnemonic: &str) -> Self {
        Self {
            address,
            opcode,
            mnemonic: mnemonic.to_owned(),
            length: 1,
            ..Default::default()
        }
    }

    pub fn new_8(pc: &mut u16, rom: &[u8], opcode: u8, mnemonic: &'static str) -> Self {
        let immediate = next_8(*pc + 1, rom);
        let address = *pc;
        *pc += 1;
        Self {
            address,
            opcode,
            mnemonic: mnemonic.replace("{}", format!("{:02X}", immediate).as_str()),
            length: 2,
            immediate_8: Some(immediate),
            ..Default::default()
        }
    }

    pub fn new_16(pc: &mut u16, rom: &[u8], opcode: u8, mnemonic: &'static str) -> Self {
        let immediate = next_16(*pc + 1, rom);
        let address = *pc;
        *pc += 2;
        Self {
            address,
            opcode,
            mnemonic: mnemonic.replace("{}", format!("{:04X}", immediate).as_str()),
            length: 3,
            immediate_16: Some(immediate),
            ..Default::default()
        }
    }
}

fn next_8(pc: u16, rom: &[u8]) -> u8 {
    let res = rom[pc as usize];
    res
}

fn next_16(pc: u16, rom: &[u8]) -> u16 {
    let n1 = rom[pc as usize];
    let n2 = rom[(pc + 1) as usize];
    u16::from_le_bytes([n1, n2])
}

fn disassemble_one(opcode: u8, pc: &mut u16, rom: &[u8]) -> Instruction {
    match opcode {
        // Opcode 0x
        0x00 => Instruction::new(*pc, opcode, "NOP"),
        0x01 => Instruction::new_16(pc, rom, opcode, "LD  BC, {}"),
        0x02 => Instruction::new(*pc, opcode, "LD  [BC], A"),
        0x03 => Instruction::new(*pc, opcode, "INC BC"),
        0x04 => Instruction::new(*pc, opcode, "INC B"),
        0x05 => Instruction::new(*pc, opcode, "DEC B"),
        0x06 => Instruction::new_8(pc, rom, opcode, "LD  B, {}"),
        0x07 => Instruction::new(*pc, opcode, "RLCA"),
        0x08 => Instruction::new_16(pc, rom, opcode, "LD  [{}], SP"),
        0x09 => Instruction::new(*pc, opcode, "ADD HL, BC"),
        0x0A => Instruction::new(*pc, opcode, "LD  A, [BC]"),
        0x0B => Instruction::new(*pc, opcode, "DEC BC"),
        0x0C => Instruction::new(*pc, opcode, "INC C"),
        0x0D => Instruction::new(*pc, opcode, "DEC C"),
        0x0E => Instruction::new_8(pc, rom, opcode, "LD  C, {}"),
        0x0F => Instruction::new(*pc, opcode, "RRCA"),

        // Opcode 1x
        0x10 => Instruction::new_8(pc, rom, opcode, "STOP {}"),
        0x11 => Instruction::new_16(pc, rom, opcode, "LD  DE, {}"),
        0x12 => Instruction::new(*pc, opcode, "LD  [DE], A"),
        0x13 => Instruction::new(*pc, opcode, "INC DE"),
        0x14 => Instruction::new(*pc, opcode, "INC D"),
        0x15 => Instruction::new(*pc, opcode, "DEC D"),
        0x16 => Instruction::new_8(pc, rom, opcode, "LD  D, {}"),
        0x17 => Instruction::new(*pc, opcode, "RLA"),
        0x18 => Instruction::new_8(pc, rom, opcode, "JR {}"),
        0x19 => Instruction::new(*pc, opcode, "ADD HL, DE"),
        0x1A => Instruction::new(*pc, opcode, "LD  A, [DE]"),
        0x1B => Instruction::new(*pc, opcode, "DEC DE"),
        0x1C => Instruction::new(*pc, opcode, "INC E"),
        0x1D => Instruction::new(*pc, opcode, "DEC E"),
        0x1E => Instruction::new_8(pc, rom, opcode, "LD  E, {}"),
        0x1F => Instruction::new(*pc, opcode, "RRA"),

        // Opcode 2x
        0x20 => Instruction::new_8(pc, rom, opcode, "JR NZ, {}"),
        0x21 => Instruction::new_16(pc, rom, opcode, "LD  HL, {}"),
        0x22 => Instruction::new(*pc, opcode, "LD  [HL+], A"),
        0x23 => Instruction::new(*pc, opcode, "INC HL"),
        0x24 => Instruction::new(*pc, opcode, "INC H"),
        0x25 => Instruction::new(*pc, opcode, "DEC H"),
        0x26 => Instruction::new_8(pc, rom, opcode, "LD  H, {}"),
        0x27 => Instruction::new(*pc, opcode, "DAA"),
        0x28 => Instruction::new_8(pc, rom, opcode, "JR Z, {}"),
        0x29 => Instruction::new(*pc, opcode, "ADD HL, HL"),
        0x2A => Instruction::new(*pc, opcode, "LD  A, [HL+]"),
        0x2B => Instruction::new(*pc, opcode, "DEC HL"),
        0x2C => Instruction::new(*pc, opcode, "INC L"),
        0x2D => Instruction::new(*pc, opcode, "DEC L"),
        0x2E => Instruction::new_8(pc, rom, opcode, "LD  L, {}"),
        0x2F => Instruction::new(*pc, opcode, "CPL"),

        // Opcode 3x
        0x30 => Instruction::new_8(pc, rom, opcode, "JR NC, {}"),
        0x31 => Instruction::new_16(pc, rom, opcode, "LD  SP, {}"),
        0x32 => Instruction::new(*pc, opcode, "LD  [HL-], A"),
        0x33 => Instruction::new(*pc, opcode, "INC SP"),
        0x34 => Instruction::new(*pc, opcode, "INC [HL]"),
        0x35 => Instruction::new(*pc, opcode, "DEC [HL]"),
        0x36 => Instruction::new_8(pc, rom, opcode, "LD  [HL], {}"),
        0x37 => Instruction::new(*pc, opcode, "SCF"),
        0x38 => Instruction::new_8(pc, rom, opcode, "JR C, {}"),
        0x39 => Instruction::new(*pc, opcode, "ADD HL, SP"),
        0x3A => Instruction::new(*pc, opcode, "LD  A, [HL-]"),
        0x3B => Instruction::new(*pc, opcode, "DEC SP"),
        0x3C => Instruction::new(*pc, opcode, "INC A"),
        0x3D => Instruction::new(*pc, opcode, "DEC A"),
        0x3E => Instruction::new_8(pc, rom, opcode, "LD  A, {}"),
        0x3F => Instruction::new(*pc, opcode, "CCF"),

        // Opcode 4x
        0x40 => Instruction::new(*pc, opcode, "LD  B, B"),
        0x41 => Instruction::new(*pc, opcode, "LD  B, C"),
        0x42 => Instruction::new(*pc, opcode, "LD  B, D"),
        0x43 => Instruction::new(*pc, opcode, "LD  B, E"),
        0x44 => Instruction::new(*pc, opcode, "LD  B, H"),
        0x45 => Instruction::new(*pc, opcode, "LD  B, L"),
        0x46 => Instruction::new(*pc, opcode, "LD  B, [HL]"),
        0x47 => Instruction::new(*pc, opcode, "LD  B, A"),
        0x48 => Instruction::new(*pc, opcode, "LD  C, B"),
        0x49 => Instruction::new(*pc, opcode, "LD  C, C"),
        0x4A => Instruction::new(*pc, opcode, "LD  C, D"),
        0x4B => Instruction::new(*pc, opcode, "LD  C, E"),
        0x4C => Instruction::new(*pc, opcode, "LD  C, H"),
        0x4D => Instruction::new(*pc, opcode, "LD  C, L"),
        0x4E => Instruction::new(*pc, opcode, "LD  C, [HL]"),
        0x4F => Instruction::new(*pc, opcode, "LD  C, A"),

        // Opcode 5x
        0x50 => Instruction::new(*pc, opcode, "LD  D, B"),
        0x51 => Instruction::new(*pc, opcode, "LD  D, C"),
        0x52 => Instruction::new(*pc, opcode, "LD  D, D"),
        0x53 => Instruction::new(*pc, opcode, "LD  D, E"),
        0x54 => Instruction::new(*pc, opcode, "LD  D, H"),
        0x55 => Instruction::new(*pc, opcode, "LD  D, L"),
        0x56 => Instruction::new(*pc, opcode, "LD  D, [HL]"),
        0x57 => Instruction::new(*pc, opcode, "LD  D, A"),
        0x58 => Instruction::new(*pc, opcode, "LD  E, B"),
        0x59 => Instruction::new(*pc, opcode, "LD  E, C"),
        0x5A => Instruction::new(*pc, opcode, "LD  E, D"),
        0x5B => Instruction::new(*pc, opcode, "LD  E, E"),
        0x5C => Instruction::new(*pc, opcode, "LD  E, H"),
        0x5D => Instruction::new(*pc, opcode, "LD  E, L"),
        0x5E => Instruction::new(*pc, opcode, "LD  E, [HL]"),
        0x5F => Instruction::new(*pc, opcode, "LD  E, A"),

        // Opcode 6x
        0x60 => Instruction::new(*pc, opcode, "LD  H, B"),
        0x61 => Instruction::new(*pc, opcode, "LD  H, C"),
        0x62 => Instruction::new(*pc, opcode, "LD  H, D"),
        0x63 => Instruction::new(*pc, opcode, "LD  H, E"),
        0x64 => Instruction::new(*pc, opcode, "LD  H, H"),
        0x65 => Instruction::new(*pc, opcode, "LD  H, L"),
        0x66 => Instruction::new(*pc, opcode, "LD  H, [HL]"),
        0x67 => Instruction::new(*pc, opcode, "LD  H, A"),
        0x68 => Instruction::new(*pc, opcode, "LD  L, B"),
        0x69 => Instruction::new(*pc, opcode, "LD  L, C"),
        0x6A => Instruction::new(*pc, opcode, "LD  L, D"),
        0x6B => Instruction::new(*pc, opcode, "LD  L, E"),
        0x6C => Instruction::new(*pc, opcode, "LD  L, H"),
        0x6D => Instruction::new(*pc, opcode, "LD  L, L"),
        0x6E => Instruction::new(*pc, opcode, "LD  L, [HL]"),
        0x6F => Instruction::new(*pc, opcode, "LD  L, A"),

        // Opcode 7x
        0x70 => Instruction::new(*pc, opcode, "LD  [HL], B"),
        0x71 => Instruction::new(*pc, opcode, "LD  [HL], C"),
        0x72 => Instruction::new(*pc, opcode, "LD  [HL], D"),
        0x73 => Instruction::new(*pc, opcode, "LD  [HL], E"),
        0x74 => Instruction::new(*pc, opcode, "LD  [HL], H"),
        0x75 => Instruction::new(*pc, opcode, "LD  [HL], L"),
        0x76 => Instruction::new(*pc, opcode, "HALT"),
        0x77 => Instruction::new(*pc, opcode, "LD  [HL], A"),
        0x78 => Instruction::new(*pc, opcode, "LD  A, B"),
        0x79 => Instruction::new(*pc, opcode, "LD  A, C"),
        0x7A => Instruction::new(*pc, opcode, "LD  A, D"),
        0x7B => Instruction::new(*pc, opcode, "LD  A, E"),
        0x7C => Instruction::new(*pc, opcode, "LD  A, H"),
        0x7D => Instruction::new(*pc, opcode, "LD  A, L"),
        0x7E => Instruction::new(*pc, opcode, "LD  A, [HL]"),
        0x7F => Instruction::new(*pc, opcode, "LD  A, A"),

        // Opcode 8x
        0x80 => Instruction::new(*pc, opcode, "ADD A, B"),
        0x81 => Instruction::new(*pc, opcode, "ADD A, C"),
        0x82 => Instruction::new(*pc, opcode, "ADD A, D"),
        0x83 => Instruction::new(*pc, opcode, "ADD A, E"),
        0x84 => Instruction::new(*pc, opcode, "ADD A, H"),
        0x85 => Instruction::new(*pc, opcode, "ADD A, L"),
        0x86 => Instruction::new(*pc, opcode, "ADD A, [HL]"),
        0x87 => Instruction::new(*pc, opcode, "ADD A, A"),
        0x88 => Instruction::new(*pc, opcode, "ADC A, B"),
        0x89 => Instruction::new(*pc, opcode, "ADC A, C"),
        0x8A => Instruction::new(*pc, opcode, "ADC A, D"),
        0x8B => Instruction::new(*pc, opcode, "ADC A, E"),
        0x8C => Instruction::new(*pc, opcode, "ADC A, H"),
        0x8D => Instruction::new(*pc, opcode, "ADC A, L"),
        0x8E => Instruction::new(*pc, opcode, "ADC A, [HL]"),
        0x8F => Instruction::new(*pc, opcode, "ADC A, A"),

        // Opcode 9x
        0x90 => Instruction::new(*pc, opcode, "SUB B"),
        0x91 => Instruction::new(*pc, opcode, "SUB C"),
        0x92 => Instruction::new(*pc, opcode, "SUB D"),
        0x93 => Instruction::new(*pc, opcode, "SUB E"),
        0x94 => Instruction::new(*pc, opcode, "SUB H"),
        0x95 => Instruction::new(*pc, opcode, "SUB L"),
        0x96 => Instruction::new(*pc, opcode, "SUB [HL]"),
        0x97 => Instruction::new(*pc, opcode, "SUB A"),
        0x98 => Instruction::new(*pc, opcode, "SBC A, B"),
        0x99 => Instruction::new(*pc, opcode, "SBC A, C"),
        0x9A => Instruction::new(*pc, opcode, "SBC A, D"),
        0x9B => Instruction::new(*pc, opcode, "SBC A, E"),
        0x9C => Instruction::new(*pc, opcode, "SBC A, H"),
        0x9D => Instruction::new(*pc, opcode, "SBC A, L"),
        0x9E => Instruction::new(*pc, opcode, "SBC A, [HL]"),
        0x9F => Instruction::new(*pc, opcode, "SBC A, A"),

        // Opcode Ax
        0xA0 => Instruction::new(*pc, opcode, "AND B"),
        0xA1 => Instruction::new(*pc, opcode, "AND C"),
        0xA2 => Instruction::new(*pc, opcode, "AND D"),
        0xA3 => Instruction::new(*pc, opcode, "AND E"),
        0xA4 => Instruction::new(*pc, opcode, "AND H"),
        0xA5 => Instruction::new(*pc, opcode, "AND L"),
        0xA6 => Instruction::new(*pc, opcode, "AND [HL]"),
        0xA7 => Instruction::new(*pc, opcode, "AND A"),
        0xA8 => Instruction::new(*pc, opcode, "XOR B"),
        0xA9 => Instruction::new(*pc, opcode, "XOR C"),
        0xAA => Instruction::new(*pc, opcode, "XOR D"),
        0xAB => Instruction::new(*pc, opcode, "XOR E"),
        0xAC => Instruction::new(*pc, opcode, "XOR H"),
        0xAD => Instruction::new(*pc, opcode, "XOR L"),
        0xAE => Instruction::new(*pc, opcode, "XOR [HL]"),
        0xAF => Instruction::new(*pc, opcode, "XOR A"),

        // Opcode Bx
        0xB0 => Instruction::new(*pc, opcode, "OR  B"),
        0xB1 => Instruction::new(*pc, opcode, "OR  C"),
        0xB2 => Instruction::new(*pc, opcode, "OR  D"),
        0xB3 => Instruction::new(*pc, opcode, "OR  E"),
        0xB4 => Instruction::new(*pc, opcode, "OR  H"),
        0xB5 => Instruction::new(*pc, opcode, "OR  L"),
        0xB6 => Instruction::new(*pc, opcode, "OR  [HL]"),
        0xB7 => Instruction::new(*pc, opcode, "OR  A"),
        0xB8 => Instruction::new(*pc, opcode, "CP  B"),
        0xB9 => Instruction::new(*pc, opcode, "CP  C"),
        0xBA => Instruction::new(*pc, opcode, "CP  D"),
        0xBB => Instruction::new(*pc, opcode, "CP  E"),
        0xBC => Instruction::new(*pc, opcode, "CP  H"),
        0xBD => Instruction::new(*pc, opcode, "CP  L"),
        0xBE => Instruction::new(*pc, opcode, "CP  [HL]"),
        0xBF => Instruction::new(*pc, opcode, "CP  A"),

        // Opcode Cx
        0xC0 => Instruction::new_8(pc, rom, opcode, "RET NZ, {}"),
        0xC1 => Instruction::new(*pc, opcode, "POP BC"),
        0xC2 => Instruction::new_16(pc, rom, opcode, "JP  NZ, {}"),
        0xC3 => Instruction::new_16(pc, rom, opcode, "JP  {}"),
        0xC4 => Instruction::new_16(pc, rom, opcode, "CALL NZ, {}"),
        0xC5 => Instruction::new(*pc, opcode, "PUSH BC"),
        0xC6 => Instruction::new_8(pc, rom, opcode, "ADD A, {}"),
        0xC7 => Instruction::new(*pc, opcode, "RST 00H"),
        0xC8 => Instruction::new_8(pc, rom, opcode, "RET Z, {}"),
        0xC9 => Instruction::new(*pc, opcode, "RET"),
        0xCA => Instruction::new_16(pc, rom, opcode, "JP  Z, {}"),
        0xCB => Instruction::new_8(pc, rom, opcode, "CB {}"),
        0xCC => Instruction::new_16(pc, rom, opcode, "CALL Z, {}"),
        0xCD => Instruction::new_16(pc, rom, opcode, "CALL {}"),
        0xCE => Instruction::new_8(pc, rom, opcode, "ADC A, {}"),
        0xCF => Instruction::new(*pc, opcode, "RST 08H"),

        // Opcode Dx
        0xD0 => Instruction::new_8(pc, rom, opcode, "RET NC, {}"),
        0xD1 => Instruction::new(*pc, opcode, "POP DE"),
        0xD2 => Instruction::new_16(pc, rom, opcode, "JP  NC, {}"),
        0xD4 => Instruction::new_16(pc, rom, opcode, "CALL NC, {}"),
        0xD5 => Instruction::new(*pc, opcode, "PUSH DE"),
        0xD6 => Instruction::new_8(pc, rom, opcode, "SUB {}"),
        0xD7 => Instruction::new(*pc, opcode, "RST 10H"),
        0xD8 => Instruction::new_8(pc, rom, opcode, "RET C, {}"),
        0xD9 => Instruction::new(*pc, opcode, "RETI"),
        0xDA => Instruction::new_16(pc, rom, opcode, "JP  C, {}"),
        0xDC => Instruction::new_16(pc, rom, opcode, "CALL C, {}"),
        0xDE => Instruction::new_8(pc, rom, opcode, "SBC A, {}"),
        0xDF => Instruction::new(*pc, opcode, "RST 18H"),

        // Opcode Ex
        0xE0 => Instruction::new_8(pc, rom, opcode, "LDH [FF00+{}], A"),
        0xE1 => Instruction::new(*pc, opcode, "POP HL"),
        0xE2 => Instruction::new(*pc, opcode, "LD  [C], A"),
        0xE5 => Instruction::new(*pc, opcode, "PUSH HL"),
        0xE6 => Instruction::new_8(pc, rom, opcode, "AND {}"),
        0xE7 => Instruction::new(*pc, opcode, "RST 20H"),
        0xE8 => Instruction::new_8(pc, rom, opcode, "ADD SP, {}"),
        0xE9 => Instruction::new(*pc, opcode, "JP  [HL]"),
        0xEA => Instruction::new_16(pc, rom, opcode, "LD  [{}], A"),
        0xEE => Instruction::new_8(pc, rom, opcode, "XOR {}"),
        0xEF => Instruction::new(*pc, opcode, "RST 28H"),

        // Opcode Fx
        0xF0 => Instruction::new_8(pc, rom, opcode, "LDH A, [FF00+{}]"),
        0xF1 => Instruction::new(*pc, opcode, "POP AF"),
        0xF2 => Instruction::new(*pc, opcode, "LD  A, [C]"),
        0xF3 => Instruction::new(*pc, opcode, "DI"),
        0xF5 => Instruction::new(*pc, opcode, "PUSH AF"),
        0xF6 => Instruction::new_8(pc, rom, opcode, "OR {}"),
        0xF7 => Instruction::new(*pc, opcode, "RST 30H"),
        0xF8 => Instruction::new_8(pc, rom, opcode, "LDHL SP, {}"),
        0xF9 => Instruction::new(*pc, opcode, "LD  SP, HL"),
        0xFA => Instruction::new_16(pc, rom, opcode, "LD  A, [{}]"),
        0xFB => Instruction::new(*pc, opcode, "EI"),
        0xFE => Instruction::new_8(pc, rom, opcode, "CP {}"),
        0xFF => Instruction::new(*pc, opcode, "RST 38H"),

        _ => Instruction::new(*pc, opcode, "???"),
    }
}

pub fn disassemble(pc: u16, rom: &[u8], count: usize) -> Vec<Instruction> {
    let count = (pc as usize + count) % 0xFFFF - pc as usize;
    let mut pc = pc;
    let mut res = Vec::new();
    for _ in 0..count {
        res.push(disassemble_one(rom[pc as usize], &mut pc, rom));
        pc += 1;
    }

    res
}
