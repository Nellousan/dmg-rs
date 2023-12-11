use std::{cell::RefCell, rc::Rc};

use crate::mmu::MemoryMapUnit;

#[allow(dead_code)]
#[derive(Copy, Clone)]
pub enum Register8 {
    A,
    F,
    B,
    C,
    D,
    E,
    H,
    L,
}

#[allow(dead_code)]
#[derive(Copy, Clone)]
pub enum Register16 {
    AF,
    BC,
    DE,
    HL,
    SP,
    PC,
}

#[derive(Default, Clone)]
pub struct Registers {
    af: u16,
    bc: u16,
    de: u16,
    hl: u16,
    sp: u16,
    pc: u16,
}

const ZFLAGBIT: u8 = 7u8;
const NFLAGBIT: u8 = 6u8;
const HFLAGBIT: u8 = 5u8;
const CFLAGBIT: u8 = 4u8;

impl Registers {
    pub fn set_8(&mut self, register: Register8, value: u8) {
        let value16 = value as u16;
        let set_higher = |reg: u16| -> u16 { (value16 << 8) | (reg & 0x00FF) };
        let set_lower = |reg: u16| -> u16 { value16 | (reg & 0xFF00) };

        match register {
            Register8::A => {
                self.af = set_higher(self.af);
            }
            Register8::F => {
                self.af = set_lower(self.af);
            }
            Register8::B => {
                self.bc = set_higher(self.bc);
            }
            Register8::C => {
                self.bc = set_lower(self.bc);
            }
            Register8::D => {
                self.de = set_higher(self.de);
            }
            Register8::E => {
                self.de = set_lower(self.de);
            }
            Register8::H => {
                self.hl = set_higher(self.hl);
            }
            Register8::L => {
                self.hl = set_lower(self.hl);
            }
        }
    }

    pub fn get_8(&self, register: Register8) -> u8 {
        let get_lower = |reg: u16| -> u8 { (reg & 0x00FF) as u8 };
        let get_higher = |reg: u16| -> u8 { (reg >> 8) as u8 };

        match register {
            Register8::A => get_higher(self.af),
            Register8::F => get_lower(self.af),
            Register8::B => get_higher(self.bc),
            Register8::C => get_lower(self.bc),
            Register8::D => get_higher(self.de),
            Register8::E => get_lower(self.de),
            Register8::H => get_higher(self.hl),
            Register8::L => get_lower(self.hl),
        }
    }

    pub fn set_16(&mut self, register: Register16, value: u16) {
        match register {
            Register16::AF => self.af = value,
            Register16::BC => self.bc = value,
            Register16::DE => self.de = value,
            Register16::HL => self.hl = value,
            Register16::SP => self.sp = value,
            Register16::PC => self.pc = value,
        }
    }
    pub fn get_16(&self, register: Register16) -> u16 {
        match register {
            Register16::AF => self.af,
            Register16::BC => self.bc,
            Register16::DE => self.de,
            Register16::HL => self.hl,
            Register16::SP => self.sp,
            Register16::PC => self.pc,
        }
    }
    pub fn inc_pc(&mut self, value: u16) {
        self.pc += value;
    }

    pub fn set_zero_flag(&mut self, value: bool) {
        let f = self.get_8(Register8::F);
        if value {
            self.set_8(Register8::F, f | (1u8 << ZFLAGBIT));
        } else {
            self.set_8(Register8::F, f & !(1u8 << ZFLAGBIT));
        }
    }

    pub fn get_zero_flag(&self) -> bool {
        ((self.get_8(Register8::F) >> ZFLAGBIT) & 1u8) != 0
    }

    pub fn set_n_flag(&mut self, value: bool) {
        let f = self.get_8(Register8::F);
        if value {
            self.set_8(Register8::F, f | (1u8 << NFLAGBIT));
        } else {
            self.set_8(Register8::F, f & !(1u8 << NFLAGBIT));
        }
    }

    pub fn get_n_flag(&self) -> bool {
        ((self.get_8(Register8::F) >> NFLAGBIT) & 1u8) != 0
    }

    pub fn set_h_flag(&mut self, value: bool) {
        let f = self.get_8(Register8::F);
        if value {
            self.set_8(Register8::F, f | (1u8 << HFLAGBIT));
        } else {
            self.set_8(Register8::F, f & !(1u8 << HFLAGBIT));
        }
    }
    pub fn get_h_flag(&self) -> bool {
        ((self.get_8(Register8::F) >> HFLAGBIT) & 1u8) != 0
    }

    pub fn set_carry_flag(&mut self, value: bool) {
        let f = self.get_8(Register8::F);
        if value {
            self.set_8(Register8::F, f | (1u8 << CFLAGBIT));
        } else {
            self.set_8(Register8::F, f & !(1u8 << CFLAGBIT));
        }
    }

    pub fn get_carry_flag(&self) -> bool {
        ((self.get_8(Register8::F) >> CFLAGBIT) & 1u8) != 0
    }

    pub fn set_flags(&mut self, z: bool, n: bool, h: bool, c: bool) {
        self.set_zero_flag(z);
        self.set_n_flag(n);
        self.set_h_flag(h);
        self.set_carry_flag(c);
    }
}

pub struct LR35902 {
    pub registers: Registers,
    mmu: Rc<RefCell<MemoryMapUnit>>,
    ime: bool,
    halted: bool,
}

impl LR35902 {
    pub fn new(mmu: Rc<RefCell<MemoryMapUnit>>) -> Self {
        LR35902 {
            mmu,
            registers: Default::default(),
            ime: false,
            halted: false,
        }
    }

    pub fn next_instruction(&mut self) -> usize {
        let opcode = self.pc_next_8();
        match opcode {
            // Opcodes 0x
            0x00 => 4,
            0x01 => self.load_16_immediate(Register16::BC),
            0x02 => self.load_8_at(Register16::BC, Register8::A),
            0x03 => self.inc_16(Register16::BC),
            0x04 => self.inc_8(Register8::B),
            0x05 => self.dec_8(Register8::B),
            0x06 => self.load_8_immediate(Register8::B),
            0x07 => unimplemented!(),
            0x08 => self.load_16_at_immediate(Register16::SP),
            0x09 => self.add_16(Register16::HL, Register16::BC),
            0x0A => self.load_8_from(Register8::A, Register16::BC),
            0x0B => self.dec_16(Register16::BC),
            0x0C => self.inc_8(Register8::C),
            0x0D => self.dec_8(Register8::C),
            0x0E => self.load_8_immediate(Register8::C),
            0x0F => unimplemented!(),

            // Opcodes 1x
            0x10 => unimplemented!(),
            0x11 => self.load_16_immediate(Register16::DE),
            0x12 => self.load_8_at(Register16::DE, Register8::A),
            0x13 => self.inc_16(Register16::DE),
            0x14 => self.inc_8(Register8::D),
            0x15 => self.dec_8(Register8::D),
            0x16 => self.load_8_immediate(Register8::D),
            0x17 => unimplemented!(),
            0x18 => self.jump_if_immediate_8(true),
            0x19 => self.add_16(Register16::HL, Register16::DE),
            0x1A => self.load_8_from(Register8::A, Register16::DE),
            0x1B => self.dec_16(Register16::DE),
            0x1C => self.inc_8(Register8::E),
            0x1D => self.dec_8(Register8::E),
            0x1E => self.load_8_immediate(Register8::E),
            0x1F => unimplemented!(),

            // Opcodes 2x
            0x20 => self.jump_if_immediate_8(!self.registers.get_zero_flag()),
            0x21 => self.load_16_immediate(Register16::HL),
            0x22 => self.load_8_at_increment(Register16::HL, Register8::A),
            0x23 => self.inc_16(Register16::HL),
            0x24 => self.inc_8(Register8::H),
            0x25 => self.dec_8(Register8::H),
            0x26 => self.load_8_immediate(Register8::H),
            0x27 => self.decimal_adjust(),
            0x28 => self.jump_if_immediate_8(self.registers.get_zero_flag()),
            0x29 => self.add_16(Register16::HL, Register16::HL),
            0x2A => self.load_8_from_increment(Register8::A, Register16::HL),
            0x2B => self.dec_16(Register16::HL),
            0x2C => self.inc_8(Register8::L),
            0x2D => self.dec_8(Register8::L),
            0x2E => self.load_8_immediate(Register8::L),
            0x2F => self.complement(),

            // Opcodes 3x
            0x30 => self.jump_if_immediate_8(!self.registers.get_carry_flag()),
            0x31 => self.load_16_immediate(Register16::SP),
            0x32 => self.load_8_at_decrement(Register16::HL, Register8::A),
            0x33 => self.inc_16(Register16::SP),
            0x34 => self.inc_8_at(Register16::HL),
            0x35 => self.dec_8_at(Register16::HL),
            0x36 => self.load_8_immediate_at(Register16::HL),
            0x37 => self.set_carry_flag(),
            0x38 => self.jump_if_immediate_8(self.registers.get_carry_flag()),
            0x39 => self.add_16(Register16::HL, Register16::SP),
            0x3A => self.load_8_from_decrement(Register8::A, Register16::HL),
            0x3B => self.dec_16(Register16::SP),
            0x3C => self.inc_8(Register8::A),
            0x3D => self.dec_8(Register8::A),
            0x3E => self.load_8_immediate(Register8::A),
            0x3F => self.complement_carry_flag(),

            // Opcodes 4x
            0x40 => self.load_8(Register8::B, Register8::B),
            0x41 => self.load_8(Register8::B, Register8::C),
            0x42 => self.load_8(Register8::B, Register8::D),
            0x43 => self.load_8(Register8::B, Register8::E),
            0x44 => self.load_8(Register8::B, Register8::H),
            0x45 => self.load_8(Register8::B, Register8::L),
            0x46 => self.load_8_from(Register8::B, Register16::HL),
            0x47 => self.load_8(Register8::B, Register8::A),
            0x48 => self.load_8(Register8::C, Register8::B),
            0x49 => self.load_8(Register8::C, Register8::C),
            0x4A => self.load_8(Register8::C, Register8::D),
            0x4B => self.load_8(Register8::C, Register8::E),
            0x4C => self.load_8(Register8::C, Register8::H),
            0x4D => self.load_8(Register8::C, Register8::L),
            0x4E => self.load_8_from(Register8::C, Register16::HL),
            0x4F => self.load_8(Register8::C, Register8::A),

            // Opcodes 5x
            0x50 => self.load_8(Register8::D, Register8::B),
            0x51 => self.load_8(Register8::D, Register8::C),
            0x52 => self.load_8(Register8::D, Register8::D),
            0x53 => self.load_8(Register8::D, Register8::E),
            0x54 => self.load_8(Register8::D, Register8::H),
            0x55 => self.load_8(Register8::D, Register8::L),
            0x56 => self.load_8_from(Register8::D, Register16::HL),
            0x57 => self.load_8(Register8::D, Register8::A),
            0x58 => self.load_8(Register8::E, Register8::B),
            0x59 => self.load_8(Register8::E, Register8::C),
            0x5A => self.load_8(Register8::E, Register8::D),
            0x5B => self.load_8(Register8::E, Register8::E),
            0x5C => self.load_8(Register8::E, Register8::H),
            0x5D => self.load_8(Register8::E, Register8::L),
            0x5E => self.load_8_from(Register8::E, Register16::HL),
            0x5F => self.load_8(Register8::E, Register8::A),

            // Opcodes 6x
            0x60 => self.load_8(Register8::H, Register8::B),
            0x61 => self.load_8(Register8::H, Register8::C),
            0x62 => self.load_8(Register8::H, Register8::D),
            0x63 => self.load_8(Register8::H, Register8::E),
            0x64 => self.load_8(Register8::H, Register8::H),
            0x65 => self.load_8(Register8::H, Register8::L),
            0x66 => self.load_8_from(Register8::H, Register16::HL),
            0x67 => self.load_8(Register8::H, Register8::A),
            0x68 => self.load_8(Register8::L, Register8::B),
            0x69 => self.load_8(Register8::L, Register8::C),
            0x6A => self.load_8(Register8::L, Register8::D),
            0x6B => self.load_8(Register8::L, Register8::E),
            0x6C => self.load_8(Register8::L, Register8::H),
            0x6D => self.load_8(Register8::L, Register8::L),
            0x6E => self.load_8_from(Register8::L, Register16::HL),
            0x6F => self.load_8(Register8::L, Register8::A),

            // Opcodes 7x
            0x70 => self.load_8_at(Register16::HL, Register8::B),
            0x71 => self.load_8_at(Register16::HL, Register8::C),
            0x72 => self.load_8_at(Register16::HL, Register8::D),
            0x73 => self.load_8_at(Register16::HL, Register8::E),
            0x74 => self.load_8_at(Register16::HL, Register8::H),
            0x75 => self.load_8_at(Register16::HL, Register8::L),
            0x76 => unimplemented!(),
            0x77 => self.load_8_at(Register16::HL, Register8::A),
            0x78 => self.load_8(Register8::A, Register8::B),
            0x79 => self.load_8(Register8::A, Register8::C),
            0x7A => self.load_8(Register8::A, Register8::D),
            0x7B => self.load_8(Register8::A, Register8::E),
            0x7C => self.load_8(Register8::A, Register8::H),
            0x7D => self.load_8(Register8::A, Register8::L),
            0x7E => self.load_8_from(Register8::A, Register16::HL),
            0x7F => self.load_8(Register8::A, Register8::A),

            // Opcodes 8x
            0x80 => self.add_8(Register8::B),
            0x81 => self.add_8(Register8::C),
            0x82 => self.add_8(Register8::D),
            0x83 => self.add_8(Register8::E),
            0x84 => self.add_8(Register8::H),
            0x85 => self.add_8(Register8::L),
            0x86 => self.add_8_from(Register16::HL),
            0x87 => self.add_8(Register8::A),
            0x88 => self.add_carry_8(Register8::B),
            0x89 => self.add_carry_8(Register8::C),
            0x8A => self.add_carry_8(Register8::D),
            0x8B => self.add_carry_8(Register8::E),
            0x8C => self.add_carry_8(Register8::H),
            0x8D => self.add_carry_8(Register8::L),
            0x8E => self.add_carry_8_from(Register16::HL),
            0x8F => self.add_carry_8(Register8::A),

            // Opcodes 9x
            0x90 => self.sub_8(Register8::B),
            0x91 => self.sub_8(Register8::C),
            0x92 => self.sub_8(Register8::D),
            0x93 => self.sub_8(Register8::E),
            0x94 => self.sub_8(Register8::H),
            0x95 => self.sub_8(Register8::L),
            0x96 => self.sub_8_from(Register16::HL),
            0x97 => self.sub_8(Register8::A),
            0x98 => self.sub_carry_8(Register8::B),
            0x99 => self.sub_carry_8(Register8::C),
            0x9A => self.sub_carry_8(Register8::D),
            0x9B => self.sub_carry_8(Register8::E),
            0x9C => self.sub_carry_8(Register8::H),
            0x9D => self.sub_carry_8(Register8::L),
            0x9E => self.sub_carry_8_from(Register16::HL),
            0x9F => self.sub_carry_8(Register8::A),

            // Opcodes Ax
            0xA0 => self.and_8(Register8::B),
            0xA1 => self.and_8(Register8::C),
            0xA2 => self.and_8(Register8::D),
            0xA3 => self.and_8(Register8::E),
            0xA4 => self.and_8(Register8::H),
            0xA5 => self.and_8(Register8::L),
            0xA6 => self.and_8_from(Register16::HL),
            0xA7 => self.and_8(Register8::A),
            0xA8 => self.xor_8(Register8::B),
            0xA9 => self.xor_8(Register8::C),
            0xAA => self.xor_8(Register8::D),
            0xAB => self.xor_8(Register8::E),
            0xAC => self.xor_8(Register8::H),
            0xAD => self.xor_8(Register8::L),
            0xAE => self.xor_8_from(Register16::HL),
            0xAF => self.xor_8(Register8::A),

            // Opcodes Bx
            0xB0 => self.or_8(Register8::B),
            0xB1 => self.or_8(Register8::C),
            0xB2 => self.or_8(Register8::D),
            0xB3 => self.or_8(Register8::E),
            0xB4 => self.or_8(Register8::H),
            0xB5 => self.or_8(Register8::L),
            0xB6 => self.or_8_from(Register16::HL),
            0xB7 => self.or_8(Register8::A),
            0xB8 => self.cp_8(Register8::B),
            0xB9 => self.cp_8(Register8::C),
            0xBA => self.cp_8(Register8::D),
            0xBB => self.cp_8(Register8::E),
            0xBC => self.cp_8(Register8::H),
            0xBD => self.cp_8(Register8::L),
            0xBE => self.cp_8_from(Register16::HL),
            0xBF => self.cp_8(Register8::A),
            _ => unimplemented!(),
        }
    }

    fn pc_next_8(&mut self) -> u8 {
        let result = self.mmu.borrow().read_8(self.registers.pc);
        self.registers.pc += 1;
        result
    }

    fn pc_next_16(&mut self) -> u16 {
        let result = self.mmu.borrow().read_16(self.registers.pc);
        self.registers.pc += 2;
        result
    }

    //////
    // CPU Instructions
    //////

    // Load instructions
    fn load_8(&mut self, destination: Register8, source: Register8) -> usize {
        self.registers
            .set_8(destination, self.registers.get_8(source));
        4
    }

    fn load_8_at(&mut self, destination: Register16, source: Register8) -> usize {
        let address = self.registers.get_16(destination);
        let value = self.registers.get_8(source);
        self.mmu.borrow_mut().write_8(address, value);
        8
    }

    fn load_8_at_increment(&mut self, destination: Register16, source: Register8) -> usize {
        self.load_8_at(destination, source);
        self.registers
            .set_16(destination, self.registers.get_16(destination) + 1);
        8
    }

    fn load_8_at_decrement(&mut self, destination: Register16, source: Register8) -> usize {
        self.load_8_at(destination, source);
        self.registers
            .set_16(destination, self.registers.get_16(destination) - 1);
        8
    }

    fn load_8_from(&mut self, destination: Register8, source: Register16) -> usize {
        let address = self.registers.get_16(source);
        let value = self.mmu.borrow().read_8(address);
        self.registers.set_8(destination, value);
        8
    }

    fn load_8_from_increment(&mut self, destination: Register8, source: Register16) -> usize {
        self.load_8_from(destination, source);
        self.registers
            .set_16(source, self.registers.get_16(source).wrapping_add(1));
        8
    }

    fn load_8_from_decrement(&mut self, destination: Register8, source: Register16) -> usize {
        self.load_8_from(destination, source);
        self.registers
            .set_16(source, self.registers.get_16(source).wrapping_sub(1));
        8
    }

    fn load_8_immediate(&mut self, destination: Register8) -> usize {
        let value = self.pc_next_8();
        self.registers.set_8(destination, value);
        8
    }

    fn load_8_immediate_at(&mut self, destination: Register16) -> usize {
        let address = self.registers.get_16(destination);
        let value = self.pc_next_8();
        self.mmu.borrow_mut().write_8(address, value);
        12
    }

    fn load_8_from_immediate(&mut self, destination: Register8) -> usize {
        let address = self.pc_next_16();
        let value = self.mmu.borrow().read_8(address);
        self.registers.set_8(destination, value);
        16
    }

    fn load_8_at_immediate(&mut self, source: Register8) -> usize {
        let address = self.pc_next_16();
        let value = self.registers.get_8(source);
        self.mmu.borrow_mut().write_8(address, value);
        16
    }

    fn load_8_from_io(&mut self, destination: Register8, source: Register8) -> usize {
        let address = 0xFF00 + self.registers.get_8(source) as u16;
        let value = self.mmu.borrow().read_8(address);
        self.registers.set_8(destination, value);
        8
    }

    fn load_8_from_io_immediate(&mut self, destination: Register8) -> usize {
        let address = 0xFF00 + self.pc_next_8() as u16;
        let value = self.mmu.borrow().read_8(address);
        self.registers.set_8(destination, value);
        12
    }

    fn load_8_at_io(&mut self, destination: Register8, source: Register8) -> usize {
        let address = 0xFF00 + self.registers.get_8(destination) as u16;
        let value = self.registers.get_8(source);
        self.mmu.borrow_mut().write_8(address, value);
        8
    }

    fn load_8_at_io_immediate(&mut self, source: Register8) -> usize {
        let address = 0xFF00 + self.pc_next_8() as u16;
        let value = self.registers.get_8(source);
        self.mmu.borrow_mut().write_8(address, value);
        12
    }

    fn load_16(&mut self, destination: Register16, source: Register16) -> usize {
        let value = self.registers.get_16(source);
        self.registers.set_16(destination, value);
        8
    }

    fn load_16_at_immediate(&mut self, source: Register16) -> usize {
        let address = self.pc_next_16();
        let value = self.registers.get_16(source);
        self.mmu.borrow_mut().write_16(address, value);
        20
    }

    fn load_16_immediate(&mut self, destination: Register16) -> usize {
        let value = self.pc_next_16();
        self.registers.set_16(destination, value);
        12
    }

    // TODO: maybe bugged h_flag
    fn load_16_add_immediate(&mut self, destination: Register16, source: Register16) -> usize {
        let immediate = self.pc_next_8();
        let value = self.registers.get_16(source);

        let res = (((value & 0x000F) as i8) + ((immediate & 0x0F) as i8)) as u8;
        let h_flag = res > 0x0F;
        let c_flag = value.checked_add(immediate as u16) == None;

        let value = value.wrapping_add_signed(immediate as i8 as i16);
        self.registers.set_16(destination, value);

        self.registers.set_flags(false, false, h_flag, c_flag);
        12
    }

    fn push(&mut self, source: Register16) -> usize {
        let value = self.registers.get_16(source);
        let address = self.registers.get_16(Register16::SP);
        self.registers.set_16(Register16::SP, address - 2);
        self.mmu.borrow_mut().write_16(address - 2, value);
        16
    }

    fn pop(&mut self, destination: Register16) -> usize {
        let address = self.registers.get_16(Register16::SP);
        let value = self.mmu.borrow().read_16(address);
        self.registers.set_16(destination, value);
        self.registers.set_16(Register16::SP, address + 2);
        12
    }

    // Arithmetic instructions

    // Helper functions for ADD and ADC to avoid code duplication
    fn _add_8_inner(&mut self, destination: u8, source: u8, carry: u8) -> u8 {
        let h_flag = (destination & 0x0F) + (source & 0x0F) + carry > 0x0F;
        let mut c_flag = destination.checked_add(source) == None;
        let mut res = destination.wrapping_add(source);
        if let None = res.checked_add(carry) {
            c_flag = true;
        }
        res = res.wrapping_add(carry);
        let z_flag = res == 0;
        self.registers.set_flags(z_flag, false, h_flag, c_flag);
        res
    }

    fn add_8(&mut self, source: Register8) -> usize {
        let value = self.registers.get_8(source);
        let a_value = self.registers.get_8(Register8::A);

        let res = self._add_8_inner(a_value, value, 0);
        self.registers.set_8(Register8::A, res);
        4
    }

    fn add_8_from(&mut self, source: Register16) -> usize {
        let address = self.registers.get_16(source);
        let value = self.mmu.borrow().read_8(address);
        let a_value = self.registers.get_8(Register8::A);

        let res = self._add_8_inner(a_value, value, 0);
        self.registers.set_8(Register8::A, res);
        8
    }

    fn add_8_immediate(&mut self) -> usize {
        let value = self.pc_next_8();
        let a_value = self.registers.get_8(Register8::A);

        let res = self._add_8_inner(a_value, value, 0);
        self.registers.set_8(Register8::A, res);
        8
    }

    fn add_carry_8(&mut self, source: Register8) -> usize {
        let value = self.registers.get_8(source);
        let a_value = self.registers.get_8(Register8::A);
        let carry = if self.registers.get_carry_flag() {
            1u8
        } else {
            0u8
        };

        let res = self._add_8_inner(a_value, value, carry);
        self.registers.set_8(Register8::A, res);
        4
    }

    fn add_carry_8_from(&mut self, source: Register16) -> usize {
        let address = self.registers.get_16(source);
        let value = self.mmu.borrow().read_8(address);
        let a_value = self.registers.get_8(Register8::A);
        let carry = if self.registers.get_carry_flag() {
            1u8
        } else {
            0u8
        };

        let res = self._add_8_inner(a_value, value, carry);
        self.registers.set_8(Register8::A, res);
        8
    }

    fn add_carry_8_immediate(&mut self) -> usize {
        let value = self.pc_next_8();
        let a_value = self.registers.get_8(Register8::A);
        let carry = if self.registers.get_carry_flag() {
            1u8
        } else {
            0u8
        };

        let res = self._add_8_inner(a_value, value, carry);
        self.registers.set_8(Register8::A, res);
        8
    }

    fn inc_8(&mut self, destination: Register8) -> usize {
        let value = self.registers.get_8(destination);

        let h_flag = (value & 0x0F) + 1 > 0x0F;
        let res = value.wrapping_add(1);
        let z_flag = res == 0;

        self.registers.set_zero_flag(z_flag);
        self.registers.set_n_flag(false);
        self.registers.set_h_flag(h_flag);
        self.registers.set_8(destination, res);
        4
    }

    fn inc_8_at(&mut self, destination: Register16) -> usize {
        let address = self.registers.get_16(destination);
        let value = self.mmu.borrow().read_8(address);

        let h_flag = (value & 0x0F) + 1 > 0x0F;
        let res = value.wrapping_add(1);
        let z_flag = res == 0;

        self.registers.set_zero_flag(z_flag);
        self.registers.set_n_flag(false);
        self.registers.set_h_flag(h_flag);
        self.mmu.borrow_mut().write_8(address, res);
        12
    }

    fn add_16(&mut self, destination: Register16, source: Register16) -> usize {
        let d_value = self.registers.get_16(destination);
        let s_value = self.registers.get_16(source);

        let h_flag = (d_value & 0x000F) + (s_value + 0x000F) > 0x0F;
        let mut c_flag = false;
        if let None = d_value.checked_add(s_value) {
            c_flag = true;
        }
        let res = d_value.wrapping_add(s_value);

        self.registers.set_n_flag(false);
        self.registers.set_h_flag(h_flag);
        self.registers.set_carry_flag(c_flag);
        self.registers.set_16(destination, res);
        8
    }

    // TODO: Maybe bugged h_flag
    fn add_16_immediate(&mut self, destination: Register16) -> usize {
        let d_value = self.registers.get_16(destination);
        let value = self.pc_next_8() as i8;

        let h_flag = (d_value & 0x000F).wrapping_add_signed((value & 0x0F) as i16) > 0x0F;
        let mut c_flag = false;
        if let None = d_value.checked_add_signed(value as i16) {
            c_flag = true;
        }
        let res = d_value.wrapping_add_signed(value as i16);

        self.registers.set_flags(false, false, h_flag, c_flag);
        self.registers.set_16(destination, res);
        16
    }

    fn inc_16(&mut self, destination: Register16) -> usize {
        let value = self.registers.get_16(destination);
        let res = value.wrapping_add(1u16);

        self.registers.set_16(destination, res);
        8
    }

    // SUB & SBC helper function to avoid code duplication
    fn _sub_8_inner(&mut self, destination: u8, source: u8, carry: u8) -> u8 {
        let mut h_flag = (destination & 0x0F).checked_sub(source & 0x0F) == None;
        let h_res = (destination & 0x0F).wrapping_sub(source & 0x0F);
        if let None = h_res.checked_sub(carry) {
            h_flag = true;
        }

        let mut c_flag = destination.checked_sub(source) == None;
        let mut res = destination.wrapping_sub(source);
        if let None = res.checked_sub(carry) {
            c_flag = true;
        }
        res = res.wrapping_sub(carry);

        let z_flag = res == 0;
        self.registers.set_flags(z_flag, true, h_flag, c_flag);
        res
    }

    fn sub_8(&mut self, source: Register8) -> usize {
        let value = self.registers.get_8(source);
        let a_value = self.registers.get_8(Register8::A);

        let res = self._sub_8_inner(a_value, value, 0);
        self.registers.set_8(Register8::A, res);
        4
    }

    fn sub_8_from(&mut self, source: Register16) -> usize {
        let address = self.registers.get_16(source);
        let value = self.mmu.borrow().read_8(address);
        let a_value = self.registers.get_8(Register8::A);

        let res = self._sub_8_inner(a_value, value, 0);
        self.registers.set_8(Register8::A, res);
        8
    }

    fn sub_8_immediate(&mut self) -> usize {
        let value = self.pc_next_8();
        let a_value = self.registers.get_8(Register8::A);

        let res = self._sub_8_inner(a_value, value, 0);
        self.registers.set_8(Register8::A, res);
        8
    }

    fn sub_carry_8(&mut self, source: Register8) -> usize {
        let value = self.registers.get_8(source);
        let a_value = self.registers.get_8(Register8::A);
        let carry = if self.registers.get_carry_flag() {
            1u8
        } else {
            0u8
        };

        let res = self._add_8_inner(a_value, value, carry);
        self.registers.set_8(Register8::A, res);
        4
    }

    fn sub_carry_8_from(&mut self, source: Register16) -> usize {
        let address = self.registers.get_16(source);
        let value = self.mmu.borrow().read_8(address);
        let a_value = self.registers.get_8(Register8::A);
        let carry = if self.registers.get_carry_flag() {
            1u8
        } else {
            0u8
        };

        let res = self._add_8_inner(a_value, value, carry);
        self.registers.set_8(Register8::A, res);
        8
    }

    fn sub_carry_8_immediate(&mut self) -> usize {
        let value = self.pc_next_8();
        let a_value = self.registers.get_8(Register8::A);
        let carry = if self.registers.get_carry_flag() {
            1u8
        } else {
            0u8
        };

        let res = self._add_8_inner(a_value, value, carry);
        self.registers.set_8(Register8::A, res);
        8
    }

    fn dec_8(&mut self, destination: Register8) -> usize {
        let value = self.registers.get_8(destination);

        let h_flag = (value & 0x0F).checked_sub(1) == None;
        let res = value.wrapping_sub(1);
        let z_flag = res == 0;

        self.registers.set_zero_flag(z_flag);
        self.registers.set_n_flag(false);
        self.registers.set_h_flag(h_flag);
        self.registers.set_8(destination, res);
        4
    }

    fn dec_8_at(&mut self, destination: Register16) -> usize {
        let address = self.registers.get_16(destination);
        let value = self.mmu.borrow().read_8(address);

        let h_flag = (value & 0x0F).checked_sub(1) == None;
        let res = value.wrapping_sub(1);
        let z_flag = res == 0;

        self.registers.set_zero_flag(z_flag);
        self.registers.set_n_flag(false);
        self.registers.set_h_flag(h_flag);
        self.mmu.borrow_mut().write_8(address, res);
        8
    }

    fn dec_16(&mut self, destination: Register16) -> usize {
        let value = self.registers.get_16(destination);
        let res = value.wrapping_sub(1);
        self.registers.set_16(destination, res);
        8
    }

    // AND helper to avoid code duplication
    fn _and_8_inner(&mut self, destination: u8, source: u8) {
        let res = destination & source;
        let z_flag = res == 0;

        self.registers.set_flags(z_flag, false, true, false);
        self.registers.set_8(Register8::A, res);
    }

    fn and_8(&mut self, source: Register8) -> usize {
        let value = self.registers.get_8(source);
        let a_value = self.registers.get_8(Register8::A);

        self._and_8_inner(a_value, value);
        4
    }

    fn and_8_from(&mut self, source: Register16) -> usize {
        let address = self.registers.get_16(source);
        let value = self.mmu.borrow().read_8(address);
        let a_value = self.registers.get_8(Register8::A);

        self._and_8_inner(a_value, value);
        8
    }

    fn and_8_immediate(&mut self) -> usize {
        let value = self.pc_next_8();
        let a_value = self.registers.get_8(Register8::A);

        self._and_8_inner(a_value, value);
        8
    }

    fn _xor_8_inner(&mut self, destination: u8, source: u8) {
        let res = destination ^ source;
        let z_flag = res == 0;

        self.registers.set_flags(z_flag, false, false, false);
        self.registers.set_8(Register8::A, res);
    }

    fn xor_8(&mut self, source: Register8) -> usize {
        let value = self.registers.get_8(source);
        let a_value = self.registers.get_8(Register8::A);

        self._xor_8_inner(a_value, value);
        4
    }

    fn xor_8_from(&mut self, source: Register16) -> usize {
        let address = self.registers.get_16(source);
        let value = self.mmu.borrow().read_8(address);
        let a_value = self.registers.get_8(Register8::A);

        self._xor_8_inner(a_value, value);
        8
    }

    fn xor_8_immediate(&mut self) -> usize {
        let value = self.pc_next_8();
        let a_value = self.registers.get_8(Register8::A);

        self._xor_8_inner(a_value, value);
        8
    }

    fn _or_8_inner(&mut self, destination: u8, source: u8) {
        let res = destination | source;
        let z_flag = res == 0;

        self.registers.set_flags(z_flag, false, false, false);
        self.registers.set_8(Register8::A, res);
    }

    fn or_8(&mut self, source: Register8) -> usize {
        let value = self.registers.get_8(source);
        let a_value = self.registers.get_8(Register8::A);

        self._or_8_inner(a_value, value);
        4
    }

    fn or_8_from(&mut self, source: Register16) -> usize {
        let address = self.registers.get_16(source);
        let value = self.mmu.borrow().read_8(address);
        let a_value = self.registers.get_8(Register8::A);

        self._or_8_inner(a_value, value);
        8
    }

    fn or_8_immediate(&mut self) -> usize {
        let value = self.pc_next_8();
        let a_value = self.registers.get_8(Register8::A);

        self._or_8_inner(a_value, value);
        8
    }

    fn cp_8(&mut self, source: Register8) -> usize {
        let value = self.registers.get_8(source);
        let a_value = self.registers.get_8(Register8::A);

        let _res = self._sub_8_inner(a_value, value, 0);
        4
    }

    fn cp_8_from(&mut self, source: Register16) -> usize {
        let address = self.registers.get_16(source);
        let value = self.mmu.borrow().read_8(address);
        let a_value = self.registers.get_8(Register8::A);

        let _res = self._sub_8_inner(a_value, value, 0);
        8
    }

    fn cp_8_immediate(&mut self) -> usize {
        let value = self.pc_next_8();
        let a_value = self.registers.get_8(Register8::A);

        let _res = self._sub_8_inner(a_value, value, 0);
        8
    }

    fn set_carry_flag(&mut self) -> usize {
        self.registers.set_n_flag(false);
        self.registers.set_h_flag(false);
        self.registers.set_carry_flag(true);
        4
    }

    fn complement_carry_flag(&mut self) -> usize {
        self.registers.set_n_flag(false);
        self.registers.set_h_flag(false);
        let carry = self.registers.get_carry_flag();
        self.registers.set_carry_flag(!carry);
        4
    }

    fn complement(&mut self) -> usize {
        let value = self.registers.get_8(Register8::A);
        self.registers.set_8(Register8::A, !value);
        self.registers.set_n_flag(true);
        self.registers.set_h_flag(true);
        4
    }

    fn decimal_adjust(&mut self) -> usize {
        let mut offset = 0u8;
        let mut carry = false;

        let value = self.registers.get_8(Register8::A);
        let h_flag = self.registers.get_h_flag();
        let c_flag = self.registers.get_carry_flag();
        let n_flag = self.registers.get_n_flag();

        if (!n_flag && value & 0x0F > 0x09) || h_flag {
            offset |= 0x06;
        }

        if (!n_flag && value > 0x99) || c_flag {
            offset |= 0x60;
            carry = true;
        }

        let result = if n_flag {
            value.wrapping_sub(offset)
        } else {
            value.wrapping_add(offset)
        };

        self.registers.set_zero_flag(result == 0);
        self.registers.set_h_flag(false);
        self.registers.set_carry_flag(carry);
        self.registers.set_8(Register8::A, result);

        4
    }

    // Control fow

    fn jump(&mut self, source: Register16) -> usize {
        let value = self.registers.get_16(source);
        self.registers.set_16(Register16::PC, value);

        4
    }

    fn jump_if_immediate_16(&mut self, condition: bool) -> usize {
        let value = self.pc_next_16();
        if !condition {
            return 12;
        }

        self.registers.set_16(Register16::PC, value);
        16
    }

    fn jump_if_immediate_8(&mut self, condition: bool) -> usize {
        let value = self.pc_next_8() as i8 as i16;
        if !condition {
            return 8;
        }

        let pc = self.registers.get_16(Register16::PC);
        let pc = pc.wrapping_add_signed(value);
        self.registers.set_16(Register16::PC, pc);
        12
    }

    fn call(&mut self, condition: bool) -> usize {
        let address = self.pc_next_16();
        if !condition {
            return 12;
        }

        self.push(Register16::PC);
        self.registers.set_16(Register16::PC, address);
        24
    }

    fn call_vec(&mut self, address: u16) -> usize {
        self.push(Register16::PC);
        self.registers.set_16(Register16::PC, address);
        16
    }

    fn ret(&mut self) -> usize {
        self.pop(Register16::PC);
        16
    }

    fn ret_if(&mut self, condition: bool) -> usize {
        if !condition {
            return 8;
        }

        self.pop(Register16::PC);
        20
    }

    fn ret_interrupt(&mut self) -> usize {
        self.ime = true;
        self.pop(Register16::PC);
        16
    }

    // Miscellaneous instructions

    fn stop(&mut self) -> usize {
        self.pc_next_8();
        4
    }

    fn disable_interrupts(&mut self) -> usize {
        self.ime = false;
        4
    }

    // TODO: flag is supposed to be set *after* the next instruction
    fn enable_interrupts(&mut self) -> usize {
        self.ime = true;
        4
    }

    fn halt(&mut self) -> usize {
        self.halted = true;
        4
    }
}
