#[derive(Default)]
pub struct Instruction {
    address: u16,
    opcode: u8,
    mnemonic: String,
    length: u32,
    immediate_8: Option<u8>,
    immediate_16: Option<u16>,
    // cycles: u32,
    // cycles2: Option<u32>,
}

impl Instruction {
    pub fn new(address: u16, opcode: u8, mnemonic: String, length: u32) -> Self {
        Self {
            address,
            opcode,
            mnemonic,
            length,
            ..Default::default()
        }
    }

    pub fn immediate_8(mut self, value: u8) -> Self {
        self.immediate_8 = Some(value);
        self
    }

    pub fn immediate_16(mut self, value: u16) -> Self {
        self.immediate_16 = Some(value);
        self
    }
}

fn next_8(pc: &mut u16, rom: &[u8]) -> u8 {
    let res = rom[*pc as usize];
    *pc += 1;
    res
}

fn next_16(pc: &mut u16, rom: &[u8]) -> u16 {
    let n1 = rom[*pc as usize];
    let n2 = rom[(*pc + 1) as usize];
    *pc += 2;
    u16::from_le_bytes([n1, n2])
}

pub fn disassemble(pc: u16, rom: &[u8], count: usize) -> Vec<Instruction> {
    unimplemented!()
}
