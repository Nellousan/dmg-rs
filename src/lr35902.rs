use std::{cell::RefCell, rc::Rc};

use crate::{dmg::ClockTicks, mmu::MemoryMapUnit, tracer::Tracer};

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

#[derive(Default, Clone, Debug)]
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

#[derive(Debug)]
pub struct LR35902 {
    pub tracer: Option<Tracer>,
    pub registers: Registers,
    mmu: Rc<RefCell<MemoryMapUnit>>,
    ime: bool,
    halted: bool,
}

pub const VBLANKBIT: u8 = 1u8 << 0u8;
pub const LCDBIT: u8 = 1u8 << 1u8;
pub const TIMERBIT: u8 = 1u8 << 2u8;
pub const SERIALBIT: u8 = 1u8 << 3u8;
pub const JOYPADBIT: u8 = 1u8 << 4u8;

impl LR35902 {
    pub fn new(mmu: Rc<RefCell<MemoryMapUnit>>) -> Self {
        LR35902 {
            tracer: None,
            mmu,
            registers: Default::default(),
            ime: false,
            halted: false,
        }
    }

    fn check_for_interrupt(&mut self) -> Option<()> {
        let interrupt_flag = self.mmu.borrow().read_8(0xFF0F);
        let interrupt_enable = self.mmu.borrow().read_8(0xFFFF);

        if interrupt_enable & VBLANKBIT != 0 && interrupt_flag & VBLANKBIT != 0 {
            self.call_vec(0x0040);
            let interrupt_flag = interrupt_flag & !VBLANKBIT;
            self.mmu.borrow_mut().write_8(0xFF0F, interrupt_flag);
            return Some(());
        }

        if interrupt_enable & LCDBIT != 0 && interrupt_flag & LCDBIT != 0 {
            self.call_vec(0x0048);
            let interrupt_flag = interrupt_flag & !LCDBIT;
            self.mmu.borrow_mut().write_8(0xFF0F, interrupt_flag);
            return Some(());
        }

        if interrupt_enable & TIMERBIT != 0 && interrupt_flag & TIMERBIT != 0 {
            self.call_vec(0x0050);
            let interrupt_flag = interrupt_flag & !TIMERBIT;
            self.mmu.borrow_mut().write_8(0xFF0F, interrupt_flag);
            return Some(());
        }

        if interrupt_enable & SERIALBIT != 0 && interrupt_flag & SERIALBIT != 0 {
            self.call_vec(0x0058);
            let interrupt_flag = interrupt_flag & !SERIALBIT;
            self.mmu.borrow_mut().write_8(0xFF0F, interrupt_flag);
            return Some(());
        }

        if interrupt_enable & JOYPADBIT != 0 && interrupt_flag & JOYPADBIT != 0 {
            self.call_vec(0x0060);
            let interrupt_flag = interrupt_flag & !JOYPADBIT;
            self.mmu.borrow_mut().write_8(0xFF0F, interrupt_flag);
            return Some(());
        }

        None
    }

    pub fn step(&mut self) -> ClockTicks {
        if self.ime == true {
            if let Some(()) = self.check_for_interrupt() {
                self.ime = false;
                self.halted = false;
                return 20;
            }
        }

        if self.halted == true {
            return 0;
        }

        self.next_instruction()
    }

    pub fn next_instruction(&mut self) -> usize {
        let opcode = self.pc_next_8();
        if let Some(ref mut tracer) = self.tracer {
            tracer.trace(opcode, self.registers.pc, self.mmu.borrow());
        }
        match opcode {
            // Opcodes 0x
            0x00 => 4,
            0x01 => self.load_16_immediate(Register16::BC),
            0x02 => self.load_8_at(Register16::BC, Register8::A),
            0x03 => self.inc_16(Register16::BC),
            0x04 => self.inc_8(Register8::B),
            0x05 => self.dec_8(Register8::B),
            0x06 => self.load_8_immediate(Register8::B),
            0x07 => self.rotate_left_accumulator(false),
            0x08 => self.load_16_at_immediate(Register16::SP),
            0x09 => self.add_16(Register16::HL, Register16::BC),
            0x0A => self.load_8_from(Register8::A, Register16::BC),
            0x0B => self.dec_16(Register16::BC),
            0x0C => self.inc_8(Register8::C),
            0x0D => self.dec_8(Register8::C),
            0x0E => self.load_8_immediate(Register8::C),
            0x0F => self.rotate_right_accumulator(false),

            // Opcodes 1x
            0x10 => self.stop(),
            0x11 => self.load_16_immediate(Register16::DE),
            0x12 => self.load_8_at(Register16::DE, Register8::A),
            0x13 => self.inc_16(Register16::DE),
            0x14 => self.inc_8(Register8::D),
            0x15 => self.dec_8(Register8::D),
            0x16 => self.load_8_immediate(Register8::D),
            0x17 => self.rotate_left_accumulator(true),
            0x18 => self.jump_if_immediate_8(true),
            0x19 => self.add_16(Register16::HL, Register16::DE),
            0x1A => self.load_8_from(Register8::A, Register16::DE),
            0x1B => self.dec_16(Register16::DE),
            0x1C => self.inc_8(Register8::E),
            0x1D => self.dec_8(Register8::E),
            0x1E => self.load_8_immediate(Register8::E),
            0x1F => self.rotate_right_accumulator(true),

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
            0x76 => self.halt(),
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

            // Opcodes Cx
            0xC0 => self.ret_if(!self.registers.get_zero_flag()),
            0xC1 => self.pop(Register16::BC),
            0xC2 => self.jump_if_immediate_16(!self.registers.get_zero_flag()),
            0xC3 => self.jump_if_immediate_16(true),
            0xC4 => self.call(!self.registers.get_zero_flag()),
            0xC5 => self.push(Register16::BC),
            0xC6 => self.add_8_immediate(),
            0xC7 => self.call_vec(0x00u16),
            0xC8 => self.ret_if(self.registers.get_zero_flag()),
            0xC9 => self.ret(),
            0xCA => self.jump_if_immediate_16(self.registers.get_zero_flag()),
            0xCB => self.prefix_cb(),
            0xCC => self.call(self.registers.get_zero_flag()),
            0xCD => self.call(true),
            0xCE => self.add_carry_8_immediate(),
            0xCF => self.call_vec(0x08u16),

            // Opcodes Dx
            0xD0 => self.ret_if(!self.registers.get_carry_flag()),
            0xD1 => self.pop(Register16::DE),
            0xD2 => self.jump_if_immediate_16(!self.registers.get_carry_flag()),
            0xD3 => unreachable!(),
            0xD4 => self.call(!self.registers.get_carry_flag()),
            0xD5 => self.push(Register16::DE),
            0xD6 => self.sub_8_immediate(),
            0xD7 => self.call_vec(0x10u16),
            0xD8 => self.ret_if(self.registers.get_carry_flag()),
            0xD9 => self.ret_interrupt(),
            0xDA => self.jump_if_immediate_16(self.registers.get_carry_flag()),
            0xDB => unreachable!(),
            0xDC => self.call(self.registers.get_carry_flag()),
            0xDD => unreachable!(),
            0xDE => self.sub_carry_8_immediate(),
            0xDF => self.call_vec(0x18u16),

            // Opcodes Ex
            0xE0 => self.load_8_at_io_immediate(Register8::A),
            0xE1 => self.pop(Register16::HL),
            0xE2 => self.load_8_at_io(Register8::C, Register8::A),
            0xE3 => unreachable!(),
            0xE4 => unreachable!(),
            0xE5 => self.push(Register16::HL),
            0xE6 => self.and_8_immediate(),
            0xE7 => self.call_vec(0x20u16),
            0xE8 => self.add_16_immediate(Register16::SP),
            0xE9 => self.jump(Register16::HL),
            0xEA => self.load_8_at_immediate(Register8::A),
            0xEB => unreachable!(),
            0xEC => unreachable!(),
            0xED => unreachable!(),
            0xEE => self.xor_8_immediate(),
            0xEF => self.call_vec(0x28u16),

            // Opcodes Fx
            0xF0 => self.load_8_from_io_immediate(Register8::A),
            0xF1 => self.pop(Register16::AF),
            0xF2 => self.load_8_from_io(Register8::C, Register8::A),
            0xF3 => self.disable_interrupts(),
            0xF4 => unreachable!(),
            0xF5 => self.push(Register16::AF),
            0xF6 => self.or_8_immediate(),
            0xF7 => self.call_vec(0x30u16),
            0xF8 => self.load_16_add_immediate(Register16::HL, Register16::SP),
            0xF9 => self.load_16(Register16::SP, Register16::HL),
            0xFA => self.load_8_from_immediate(Register8::A),
            0xFB => self.enable_interrupts(),
            0xFC => unreachable!(),
            0xFD => unreachable!(),
            0xFE => self.cp_8_immediate(),
            0xFF => self.call_vec(0x38u16),
        }
    }

    fn prefix_cb(&mut self) -> usize {
        let opcode = self.pc_next_8();

        match opcode {
            // Opcodes 0x
            0x00 => self.rotate_left(Register8::B),
            0x01 => self.rotate_left(Register8::C),
            0x02 => self.rotate_left(Register8::D),
            0x03 => self.rotate_left(Register8::E),
            0x04 => self.rotate_left(Register8::H),
            0x05 => self.rotate_left(Register8::L),
            0x06 => self.rotate_left_at(Register16::HL),
            0x07 => self.rotate_left(Register8::A),
            0x08 => self.rotate_right(Register8::B),
            0x09 => self.rotate_right(Register8::C),
            0x0A => self.rotate_right(Register8::D),
            0x0B => self.rotate_right(Register8::E),
            0x0C => self.rotate_right(Register8::H),
            0x0D => self.rotate_right(Register8::L),
            0x0E => self.rotate_right_at(Register16::HL),
            0x0F => self.rotate_right(Register8::A),

            // Opcodes 1x
            0x10 => self.rotate_left_carry(Register8::B),
            0x11 => self.rotate_left_carry(Register8::C),
            0x12 => self.rotate_left_carry(Register8::D),
            0x13 => self.rotate_left_carry(Register8::E),
            0x14 => self.rotate_left_carry(Register8::H),
            0x15 => self.rotate_left_carry(Register8::L),
            0x16 => self.rotate_left_carry_at(Register16::HL),
            0x17 => self.rotate_left_carry(Register8::A),
            0x18 => self.rotate_right_carry(Register8::B),
            0x19 => self.rotate_right_carry(Register8::C),
            0x1A => self.rotate_right_carry(Register8::D),
            0x1B => self.rotate_right_carry(Register8::E),
            0x1C => self.rotate_right_carry(Register8::H),
            0x1D => self.rotate_right_carry(Register8::L),
            0x1E => self.rotate_right_carry_at(Register16::HL),
            0x1F => self.rotate_right_carry(Register8::A),

            // Opcodes 2x
            0x20 => self.shift_left(Register8::B),
            0x21 => self.shift_left(Register8::C),
            0x22 => self.shift_left(Register8::D),
            0x23 => self.shift_left(Register8::E),
            0x24 => self.shift_left(Register8::H),
            0x25 => self.shift_left(Register8::L),
            0x26 => self.shift_left_at(Register16::HL),
            0x27 => self.shift_left(Register8::A),
            0x28 => self.shift_right(Register8::B),
            0x29 => self.shift_right(Register8::C),
            0x2A => self.shift_right(Register8::D),
            0x2B => self.shift_right(Register8::E),
            0x2C => self.shift_right(Register8::H),
            0x2D => self.shift_right(Register8::L),
            0x2E => self.shift_right_at(Register16::HL),
            0x2F => self.shift_right(Register8::A),

            // Opcodes 3x
            0x30 => self.swap(Register8::B),
            0x31 => self.swap(Register8::C),
            0x32 => self.swap(Register8::D),
            0x33 => self.swap(Register8::E),
            0x34 => self.swap(Register8::H),
            0x35 => self.swap(Register8::L),
            0x36 => self.swap_at(Register16::HL),
            0x37 => self.swap(Register8::A),
            0x38 => self.shift_right_logic(Register8::B),
            0x39 => self.shift_right_logic(Register8::C),
            0x3A => self.shift_right_logic(Register8::D),
            0x3B => self.shift_right_logic(Register8::E),
            0x3C => self.shift_right_logic(Register8::H),
            0x3D => self.shift_right_logic(Register8::L),
            0x3E => self.shift_right_logic_at(Register16::HL),
            0x3F => self.shift_right_logic(Register8::A),

            // Opcodes 4x
            0x40 => self.bit(0, Register8::B),
            0x41 => self.bit(0, Register8::C),
            0x42 => self.bit(0, Register8::D),
            0x43 => self.bit(0, Register8::E),
            0x44 => self.bit(0, Register8::H),
            0x45 => self.bit(0, Register8::L),
            0x46 => self.bit_at(0, Register16::HL),
            0x47 => self.bit(0, Register8::A),
            0x48 => self.bit(1, Register8::B),
            0x49 => self.bit(1, Register8::C),
            0x4A => self.bit(1, Register8::D),
            0x4B => self.bit(1, Register8::E),
            0x4C => self.bit(1, Register8::H),
            0x4D => self.bit(1, Register8::L),
            0x4E => self.bit_at(1, Register16::HL),
            0x4F => self.bit(1, Register8::A),

            // Opcodes 5x
            0x50 => self.bit(2, Register8::B),
            0x51 => self.bit(2, Register8::C),
            0x52 => self.bit(2, Register8::D),
            0x53 => self.bit(2, Register8::E),
            0x54 => self.bit(2, Register8::H),
            0x55 => self.bit(2, Register8::L),
            0x56 => self.bit_at(2, Register16::HL),
            0x57 => self.bit(2, Register8::A),
            0x58 => self.bit(3, Register8::B),
            0x59 => self.bit(3, Register8::C),
            0x5A => self.bit(3, Register8::D),
            0x5B => self.bit(3, Register8::E),
            0x5C => self.bit(3, Register8::H),
            0x5D => self.bit(3, Register8::L),
            0x5E => self.bit_at(3, Register16::HL),
            0x5F => self.bit(3, Register8::A),

            // Opcodes 6x
            0x60 => self.bit(4, Register8::B),
            0x61 => self.bit(4, Register8::C),
            0x62 => self.bit(4, Register8::D),
            0x63 => self.bit(4, Register8::E),
            0x64 => self.bit(4, Register8::H),
            0x65 => self.bit(4, Register8::L),
            0x66 => self.bit_at(4, Register16::HL),
            0x67 => self.bit(4, Register8::A),
            0x68 => self.bit(5, Register8::B),
            0x69 => self.bit(5, Register8::C),
            0x6A => self.bit(5, Register8::D),
            0x6B => self.bit(5, Register8::E),
            0x6C => self.bit(5, Register8::H),
            0x6D => self.bit(5, Register8::L),
            0x6E => self.bit_at(5, Register16::HL),
            0x6F => self.bit(5, Register8::A),

            // Opcodes 7x
            0x70 => self.bit(6, Register8::B),
            0x71 => self.bit(6, Register8::C),
            0x72 => self.bit(6, Register8::D),
            0x73 => self.bit(6, Register8::E),
            0x74 => self.bit(6, Register8::H),
            0x75 => self.bit(6, Register8::L),
            0x76 => self.bit_at(6, Register16::HL),
            0x77 => self.bit(6, Register8::A),
            0x78 => self.bit(7, Register8::B),
            0x79 => self.bit(7, Register8::C),
            0x7A => self.bit(7, Register8::D),
            0x7B => self.bit(7, Register8::E),
            0x7C => self.bit(7, Register8::H),
            0x7D => self.bit(7, Register8::L),
            0x7E => self.bit_at(7, Register16::HL),
            0x7F => self.bit(7, Register8::A),

            // Opcodes 8x
            0x80 => self.reset_bit(0, Register8::B),
            0x81 => self.reset_bit(0, Register8::C),
            0x82 => self.reset_bit(0, Register8::D),
            0x83 => self.reset_bit(0, Register8::E),
            0x84 => self.reset_bit(0, Register8::H),
            0x85 => self.reset_bit(0, Register8::L),
            0x86 => self.reset_bit_at(0, Register16::HL),
            0x87 => self.reset_bit(0, Register8::A),
            0x88 => self.reset_bit(1, Register8::B),
            0x89 => self.reset_bit(1, Register8::C),
            0x8A => self.reset_bit(1, Register8::D),
            0x8B => self.reset_bit(1, Register8::E),
            0x8C => self.reset_bit(1, Register8::H),
            0x8D => self.reset_bit(1, Register8::L),
            0x8E => self.reset_bit_at(1, Register16::HL),
            0x8F => self.reset_bit(1, Register8::A),

            // Opcodes 9x
            0x90 => self.reset_bit(2, Register8::B),
            0x91 => self.reset_bit(2, Register8::C),
            0x92 => self.reset_bit(2, Register8::D),
            0x93 => self.reset_bit(2, Register8::E),
            0x94 => self.reset_bit(2, Register8::H),
            0x95 => self.reset_bit(2, Register8::L),
            0x96 => self.reset_bit_at(2, Register16::HL),
            0x97 => self.reset_bit(2, Register8::A),
            0x98 => self.reset_bit(3, Register8::B),
            0x99 => self.reset_bit(3, Register8::C),
            0x9A => self.reset_bit(3, Register8::D),
            0x9B => self.reset_bit(3, Register8::E),
            0x9C => self.reset_bit(3, Register8::H),
            0x9D => self.reset_bit(3, Register8::L),
            0x9E => self.reset_bit_at(3, Register16::HL),
            0x9F => self.reset_bit(3, Register8::A),

            // Opcodes Ax
            0xA0 => self.reset_bit(4, Register8::B),
            0xA1 => self.reset_bit(4, Register8::C),
            0xA2 => self.reset_bit(4, Register8::D),
            0xA3 => self.reset_bit(4, Register8::E),
            0xA4 => self.reset_bit(4, Register8::H),
            0xA5 => self.reset_bit(4, Register8::L),
            0xA6 => self.reset_bit_at(4, Register16::HL),
            0xA7 => self.reset_bit(4, Register8::A),
            0xA8 => self.reset_bit(5, Register8::B),
            0xA9 => self.reset_bit(5, Register8::C),
            0xAA => self.reset_bit(5, Register8::D),
            0xAB => self.reset_bit(5, Register8::E),
            0xAC => self.reset_bit(5, Register8::H),
            0xAD => self.reset_bit(5, Register8::L),
            0xAE => self.reset_bit_at(5, Register16::HL),
            0xAF => self.reset_bit(5, Register8::A),

            // Opcodes Bx
            0xB0 => self.reset_bit(6, Register8::B),
            0xB1 => self.reset_bit(6, Register8::C),
            0xB2 => self.reset_bit(6, Register8::D),
            0xB3 => self.reset_bit(6, Register8::E),
            0xB4 => self.reset_bit(6, Register8::H),
            0xB5 => self.reset_bit(6, Register8::L),
            0xB6 => self.reset_bit_at(6, Register16::HL),
            0xB7 => self.reset_bit(6, Register8::A),
            0xB8 => self.reset_bit(7, Register8::B),
            0xB9 => self.reset_bit(7, Register8::C),
            0xBA => self.reset_bit(7, Register8::D),
            0xBB => self.reset_bit(7, Register8::E),
            0xBC => self.reset_bit(7, Register8::H),
            0xBD => self.reset_bit(7, Register8::L),
            0xBE => self.reset_bit_at(7, Register16::HL),
            0xBF => self.reset_bit(7, Register8::A),

            // Opcodes Cx
            0xC0 => self.set_bit(0, Register8::B),
            0xC1 => self.set_bit(0, Register8::C),
            0xC2 => self.set_bit(0, Register8::D),
            0xC3 => self.set_bit(0, Register8::E),
            0xC4 => self.set_bit(0, Register8::H),
            0xC5 => self.set_bit(0, Register8::L),
            0xC6 => self.set_bit_at(0, Register16::HL),
            0xC7 => self.set_bit(0, Register8::A),
            0xC8 => self.set_bit(1, Register8::B),
            0xC9 => self.set_bit(1, Register8::C),
            0xCA => self.set_bit(1, Register8::D),
            0xCB => self.set_bit(1, Register8::E),
            0xCC => self.set_bit(1, Register8::H),
            0xCD => self.set_bit(1, Register8::L),
            0xCE => self.set_bit_at(1, Register16::HL),
            0xCF => self.set_bit(1, Register8::A),

            // Opcodes Dx
            0xD0 => self.set_bit(2, Register8::B),
            0xD1 => self.set_bit(2, Register8::C),
            0xD2 => self.set_bit(2, Register8::D),
            0xD3 => self.set_bit(2, Register8::E),
            0xD4 => self.set_bit(2, Register8::H),
            0xD5 => self.set_bit(2, Register8::L),
            0xD6 => self.set_bit_at(2, Register16::HL),
            0xD7 => self.set_bit(2, Register8::A),
            0xD8 => self.set_bit(3, Register8::B),
            0xD9 => self.set_bit(3, Register8::C),
            0xDA => self.set_bit(3, Register8::D),
            0xDB => self.set_bit(3, Register8::E),
            0xDC => self.set_bit(3, Register8::H),
            0xDD => self.set_bit(3, Register8::L),
            0xDE => self.set_bit_at(3, Register16::HL),
            0xDF => self.set_bit(3, Register8::A),

            // Opcodes Ex
            0xE0 => self.set_bit(4, Register8::B),
            0xE1 => self.set_bit(4, Register8::C),
            0xE2 => self.set_bit(4, Register8::D),
            0xE3 => self.set_bit(4, Register8::E),
            0xE4 => self.set_bit(4, Register8::H),
            0xE5 => self.set_bit(4, Register8::L),
            0xE6 => self.set_bit_at(4, Register16::HL),
            0xE7 => self.set_bit(4, Register8::A),
            0xE8 => self.set_bit(5, Register8::B),
            0xE9 => self.set_bit(5, Register8::C),
            0xEA => self.set_bit(5, Register8::D),
            0xEB => self.set_bit(5, Register8::E),
            0xEC => self.set_bit(5, Register8::H),
            0xED => self.set_bit(5, Register8::L),
            0xEE => self.set_bit_at(5, Register16::HL),
            0xEF => self.set_bit(5, Register8::A),

            // Opcodes Fx
            0xF0 => self.set_bit(6, Register8::B),
            0xF1 => self.set_bit(6, Register8::C),
            0xF2 => self.set_bit(6, Register8::D),
            0xF3 => self.set_bit(6, Register8::E),
            0xF4 => self.set_bit(6, Register8::H),
            0xF5 => self.set_bit(6, Register8::L),
            0xF6 => self.set_bit_at(6, Register16::HL),
            0xF7 => self.set_bit(6, Register8::A),
            0xF8 => self.set_bit(7, Register8::B),
            0xF9 => self.set_bit(7, Register8::C),
            0xFA => self.set_bit(7, Register8::D),
            0xFB => self.set_bit(7, Register8::E),
            0xFC => self.set_bit(7, Register8::H),
            0xFD => self.set_bit(7, Register8::L),
            0xFE => self.set_bit_at(7, Register16::HL),
            0xFF => self.set_bit(7, Register8::A),
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

    // Bit shift instructions

    fn rotate_left_accumulator(&mut self, with_carry: bool) -> usize {
        let value = self.registers.get_8(Register8::A);
        let carry = self.registers.get_carry_flag();
        let r_carry = value & 0x80 != 0;
        let mut value = value.rotate_left(1);

        if with_carry {
            let bit: u8 = if carry { 1u8 } else { 0u8 };
            value = value & !(1u8 << 0) | (bit << 0);
        }
        self.registers.set_8(Register8::A, value);
        self.registers.set_flags(false, false, false, r_carry);
        4
    }

    fn rotate_right_accumulator(&mut self, with_carry: bool) -> usize {
        let value = self.registers.get_8(Register8::A);
        let carry = self.registers.get_carry_flag();
        let r_carry = value & 0x01 != 0;
        let mut value = value.rotate_right(1);

        if with_carry {
            let bit: u8 = if carry { 1u8 } else { 0u8 };
            value = value & !(1u8 << 7) | (bit << 7);
        }
        self.registers.set_8(Register8::A, value);
        self.registers.set_flags(false, false, false, r_carry);
        4
    }

    // Prefix operations

    fn _rotate_left_inner(&mut self, destination: u8, with_carry: bool) -> u8 {
        let carry = self.registers.get_carry_flag();
        let r_carry = destination & 0x80 != 0;
        let mut value = destination.rotate_left(1);

        if with_carry {
            let bit: u8 = if carry { 1u8 } else { 0u8 };
            value = value & !(1u8 << 0) | (bit << 0);
        }
        self.registers.set_flags(value == 0, false, false, r_carry);
        value
    }

    fn rotate_left(&mut self, destination: Register8) -> usize {
        let value = self.registers.get_8(destination);
        let result = self._rotate_left_inner(value, false);
        self.registers.set_8(destination, result);

        8
    }

    fn rotate_left_at(&mut self, destination: Register16) -> usize {
        let address = self.registers.get_16(destination);
        let value = self.mmu.borrow().read_8(address);
        let result = self._rotate_left_inner(value, false);
        self.mmu.borrow_mut().write_8(address, result);

        16
    }

    fn rotate_left_carry(&mut self, destination: Register8) -> usize {
        let value = self.registers.get_8(destination);
        let result = self._rotate_left_inner(value, true);
        self.registers.set_8(destination, result);

        8
    }

    fn rotate_left_carry_at(&mut self, destination: Register16) -> usize {
        let address = self.registers.get_16(destination);
        let value = self.mmu.borrow().read_8(address);
        let result = self._rotate_left_inner(value, true);
        self.mmu.borrow_mut().write_8(address, result);

        16
    }

    fn _rotate_right_inner(&mut self, destination: u8, with_carry: bool) -> u8 {
        let carry = self.registers.get_carry_flag();
        let r_carry = destination & 0x01 != 0;
        let mut value = destination.rotate_right(1);

        if with_carry {
            let bit: u8 = if carry { 1u8 } else { 0u8 };
            value = value & !(1u8 << 7) | (bit << 7);
        }
        self.registers.set_flags(value == 0, false, false, r_carry);
        value
    }

    fn rotate_right(&mut self, destination: Register8) -> usize {
        let value = self.registers.get_8(destination);
        let result = self._rotate_right_inner(value, false);
        self.registers.set_8(destination, result);

        8
    }

    fn rotate_right_at(&mut self, destination: Register16) -> usize {
        let address = self.registers.get_16(destination);
        let value = self.mmu.borrow().read_8(address);
        let result = self._rotate_right_inner(value, false);
        self.mmu.borrow_mut().write_8(address, result);

        16
    }

    fn rotate_right_carry(&mut self, destination: Register8) -> usize {
        let value = self.registers.get_8(destination);
        let result = self._rotate_right_inner(value, true);
        self.registers.set_8(destination, result);

        8
    }

    fn rotate_right_carry_at(&mut self, destination: Register16) -> usize {
        let address = self.registers.get_16(destination);
        let value = self.mmu.borrow().read_8(address);
        let result = self._rotate_right_inner(value, true);
        self.mmu.borrow_mut().write_8(address, result);

        16
    }

    fn shift_left(&mut self, destination: Register8) -> usize {
        let value = self.registers.get_8(destination);
        let carry = value & 0x80 != 0;
        let result = value << 1;

        self.registers.set_8(destination, result);
        self.registers.set_flags(result == 0, false, false, carry);

        8
    }

    fn shift_left_at(&mut self, destination: Register16) -> usize {
        let address = self.registers.get_16(destination);
        let value = self.mmu.borrow().read_8(address);
        let carry = value & 0x80 != 0;
        let result = value << 1;

        self.mmu.borrow_mut().write_8(address, result);
        self.registers.set_flags(result == 0, false, false, carry);

        16
    }

    fn shift_right(&mut self, destination: Register8) -> usize {
        let value = self.registers.get_8(destination);
        let carry = value & 0x01 != 0;
        let result = (value >> 1) & !(1u8 << 7) | (value & 0x80);

        self.registers.set_8(destination, result);
        self.registers.set_flags(result == 0, false, false, carry);

        8
    }

    fn shift_right_at(&mut self, destination: Register16) -> usize {
        let address = self.registers.get_16(destination);
        let value = self.mmu.borrow().read_8(address);
        let carry = value & 0x01 != 0;
        let result = (value >> 1) & !(1u8 << 7) | (value & 0x80);

        self.mmu.borrow_mut().write_8(address, result);
        self.registers.set_flags(result == 0, false, false, carry);

        16
    }

    fn swap(&mut self, destination: Register8) -> usize {
        let value = self.registers.get_8(destination);
        let result: u8 = (value << 4) | (value >> 4);

        self.registers.set_8(destination, result);
        self.registers.set_flags(result == 0, false, false, false);

        8
    }

    fn swap_at(&mut self, destination: Register16) -> usize {
        let address = self.registers.get_16(destination);
        let value = self.mmu.borrow().read_8(address);
        let result: u8 = (value << 4) | (value >> 4);

        self.mmu.borrow_mut().write_8(address, result);
        self.registers.set_flags(result == 0, false, false, false);

        16
    }

    fn shift_right_logic(&mut self, destination: Register8) -> usize {
        let value = self.registers.get_8(destination);
        let carry = value & 0x01 != 0;
        let result = value >> 1;

        self.registers.set_8(destination, result);
        self.registers.set_flags(result == 0, false, false, carry);

        8
    }

    fn shift_right_logic_at(&mut self, destination: Register16) -> usize {
        let address = self.registers.get_16(destination);
        let value = self.mmu.borrow().read_8(address);
        let carry = value & 0x01 != 0;
        let result = value >> 1;

        self.mmu.borrow_mut().write_8(address, result);
        self.registers.set_flags(result == 0, false, false, carry);

        16
    }

    fn bit(&mut self, n: u8, source: Register8) -> usize {
        let value = self.registers.get_8(source);
        let result = (value >> n) & 0x01 != 0;

        self.registers.set_zero_flag(result);
        self.registers.set_n_flag(false);
        self.registers.set_h_flag(true);

        8
    }

    fn bit_at(&mut self, n: u8, source: Register16) -> usize {
        let address = self.registers.get_16(source);
        let value = self.mmu.borrow().read_8(address);
        let result = (value >> n) & 0x01 != 0;

        self.registers.set_zero_flag(result);
        self.registers.set_n_flag(false);
        self.registers.set_h_flag(true);

        12
    }

    fn reset_bit(&mut self, n: u8, source: Register8) -> usize {
        let value = self.registers.get_8(source);
        let result = value & !(1 << n);

        self.registers.set_8(source, result);
        8
    }

    fn reset_bit_at(&mut self, n: u8, source: Register16) -> usize {
        let address = self.registers.get_16(source);
        let value = self.mmu.borrow().read_8(address);
        let result = value & !(1 << n);

        self.mmu.borrow_mut().write_8(address, result);
        16
    }

    fn set_bit(&mut self, n: u8, source: Register8) -> usize {
        let value = self.registers.get_8(source);
        let result = value | (1 << n);

        self.registers.set_8(source, result);
        8
    }

    fn set_bit_at(&mut self, n: u8, source: Register16) -> usize {
        let address = self.registers.get_16(source);
        let value = self.mmu.borrow().read_8(address);
        let result = value | (1 << n);

        self.mmu.borrow_mut().write_8(address, result);
        16
    }
}
