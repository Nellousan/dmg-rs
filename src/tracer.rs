#![allow(dead_code)]
use std::{cell::Ref, collections::HashMap};

use crate::{
    disassembler::{disassemble_one, Instruction},
    mmu::MemoryMapUnit,
};

const _JUMP_INSTRUCTIONS: &'static [u8] = &[
    0x18, 0x20, 0x28, 0x30, 0x38, 0xC0, 0xC2, 0xC3, 0xC4, 0xC7, 0xC8, 0xC9, 0xCA, 0xCC, 0xCD, 0xCF,
    0xD0, 0xD2, 0xD4, 0xD8, 0xD9, 0xDA, 0xDC, 0xDF, 0xE7, 0xE9, 0xEF, 0xF7, 0xFF,
];

const _CALL_INSTRUCTIONS_FULL: &'static [u8] = &[
    0xC4, 0xC7, 0xCC, 0xCD, 0xCF, 0xD4, 0xD7, 0xDC, 0xDF, 0xE7, 0xEF, 0xF7, 0xFF,
];

const CALL_INSTRUCTIONS: &'static [u8] = &[0xC4, 0xCC, 0xCD, 0xD4, 0xDC, 0xEF, 0xFF];

const _RET_INSTUCTIONS_FULL: &'static [u8] = &[0xC0, 0xC8, 0xC9, 0xD0, 0xD8, 0xD9];

const RET_INSTUCTIONS: &'static [u8] = &[0xC0, 0xC8, 0xC9, 0xD0, 0xD8];

#[derive(Debug)]
enum InstructionRole {
    Call,
    Return,
    Other,
}

#[derive(Debug)]
struct Trace {
    depth: u32,
    instruction: Instruction,
}

#[derive(Debug)]
pub struct Tracer {
    to_trace: HashMap<u8, InstructionRole>,
    pub pc_to_trace: HashMap<u16, ()>,
    traces: Vec<Trace>,
    current_depth: u32,
}

impl Tracer {
    pub fn new_call_tracer() -> Self {
        let mut to_trace: HashMap<u8, InstructionRole> = HashMap::new();
        for opcode in CALL_INSTRUCTIONS {
            let _ = to_trace.insert(*opcode, InstructionRole::Call);
        }

        for opcode in RET_INSTUCTIONS {
            let _ = to_trace.insert(*opcode, InstructionRole::Return);
        }

        Self {
            to_trace,
            pc_to_trace: HashMap::new(),
            traces: Vec::new(),
            current_depth: 0,
        }
    }

    pub fn trace(&mut self, opcode: u8, pc: u16, memory: Ref<'_, MemoryMapUnit>) {
        let tmp = pc - 1; // Needed bcz pc_next_8 is called before trace so the pc is offset
        if let Some(_) = self.to_trace.get(&opcode) {
            self.trace_opcode(opcode, pc - 1, memory);
        } else if let Some(_) = self.pc_to_trace.get(&tmp) {
            self.trace_address(opcode, pc - 1, memory);
        }
    }

    fn trace_opcode(&mut self, opcode: u8, pc: u16, memory: Ref<'_, MemoryMapUnit>) {
        let mut pc = pc;
        let instruction = disassemble_one(opcode, &mut pc, memory.borrow_rom());

        if let InstructionRole::Return = self.to_trace[&opcode] {
            self.current_depth -= 1;
        }

        let trace = Trace {
            depth: self.current_depth,
            instruction,
        };

        self.traces.push(trace);

        if let InstructionRole::Call = self.to_trace[&opcode] {
            self.current_depth += 1;
        }
    }

    fn trace_address(&mut self, opcode: u8, pc: u16, memory: Ref<'_, MemoryMapUnit>) {
        let mut pc = pc;
        let instruction = disassemble_one(opcode, &mut pc, memory.borrow_rom());

        self.traces.push(Trace {
            depth: self.current_depth,
            instruction,
        })
    }
}

impl ToString for Tracer {
    fn to_string(&self) -> String {
        let mut res = String::new();
        res.reserve(self.traces.len() * 20); // 20 is arbitrary number for the average length of the output per instruction
        for trace in &self.traces {
            res.push_str(&format!("{:#06X}| ", trace.instruction.address));
            for _ in 0..trace.depth {
                res.push_str(". ");
            }
            res.push_str(&trace.instruction.mnemonic);
            res.push_str("\n");
        }
        res
    }
}
