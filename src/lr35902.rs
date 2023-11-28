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
}

pub struct LR35902 {
    pub registers: Registers,
    mmu: Rc<RefCell<MemoryMapUnit>>,
}

impl LR35902 {
    pub fn new(mmu: Rc<RefCell<MemoryMapUnit>>) -> Self {
        LR35902 {
            mmu,
            registers: Default::default(),
        }
    }

    pub fn next_instruction(&mut self) -> usize {
        unimplemented!()
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

    // CPU Instruction helpers
    fn load_8(&mut self, destination: Register8, source: Register8) {
        self.registers
            .set_8(destination, self.registers.get_8(source));
    }

    fn load_8_at(&mut self, destination: Register16, source: Register8) {
        let address = self.registers.get_16(destination);
        let value = self.registers.get_8(source);
        self.mmu.borrow_mut().write_8(address, value);
    }

    fn load_8_at_increment(&mut self, destination: Register16, source: Register8) {
        self.load_8_at(destination, source);
        self.registers
            .set_16(destination, self.registers.get_16(destination) + 1);
    }

    fn load_8_at_decrement(&mut self, destination: Register16, source: Register8) {
        self.load_8_at(destination, source);
        self.registers
            .set_16(destination, self.registers.get_16(destination) - 1);
    }

    fn load_8_from(&mut self, destination: Register8, source: Register16) {
        let address = self.registers.get_16(source);
        let value = self.mmu.borrow().read_8(address);
        self.registers.set_8(destination, value);
    }

    fn load_8_from_increment(&mut self, destination: Register8, source: Register16) {
        self.load_8_from(destination, source);
        self.registers
            .set_16(source, self.registers.get_16(source) + 1);
    }

    fn load_8_from_decrement(&mut self, destination: Register8, source: Register16) {
        self.load_8_from(destination, source);
        self.registers
            .set_16(source, self.registers.get_16(source) - 1);
    }

    fn load_8_immediate(&mut self, destination: Register8) {
        let value = self.pc_next_8();
        self.registers.set_8(destination, value);
    }

    fn load_8_immediate_at(&mut self, destination: Register16) {
        let address = self.registers.get_16(destination);
        let value = self.pc_next_8();
        self.mmu.borrow_mut().write_8(address, value);
    }

    fn load_8_from_immediate(&mut self, destination: Register8) {
        let address = self.pc_next_16();
        let value = self.mmu.borrow().read_8(address);
        self.registers.set_8(destination, value);
    }

    fn load_8_at_immediate(&mut self, source: Register8) {
        let address = self.pc_next_16();
        let value = self.registers.get_8(source);
        self.mmu.borrow_mut().write_8(address, value);
    }
}
