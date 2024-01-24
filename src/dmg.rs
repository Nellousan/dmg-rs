use std::{
    cell::RefCell,
    rc::Rc,
    sync::mpsc::{Receiver, Sender},
};

use tracing::{debug, error};

use crate::{
    cartridge,
    lr35902::{Register16, LR35902},
    mmu::MemoryMapUnit,
    thread::{DmgMessage, GuiMessage},
};

pub struct DotMatrixGame {
    mmu: Rc<RefCell<MemoryMapUnit>>,
    cpu: LR35902,
    clock_ticks: usize,
    tx: Sender<DmgMessage>,
    rx: Receiver<GuiMessage>,
    step_mode: bool,
    next_step: bool,
}

impl DotMatrixGame {
    pub fn new_with_rom_path(
        path: &str,
        tx: Sender<DmgMessage>,
        rx: Receiver<GuiMessage>,
    ) -> anyhow::Result<Self> {
        let cartridge = cartridge::from_file(path)?;
        let mmu = Rc::new(RefCell::new(MemoryMapUnit::new(cartridge)));
        Ok(Self {
            mmu: mmu.clone(),
            cpu: LR35902::new(mmu),
            clock_ticks: 0,
            tx,
            rx,
            step_mode: false,
            next_step: false,
        })
    }

    pub fn new_with_test_rom(
        path: &str,
        tx: Sender<DmgMessage>,
        rx: Receiver<GuiMessage>,
    ) -> anyhow::Result<Self> {
        let cartridge = cartridge::test_rom_from_file(path)?;
        let mmu = Rc::new(RefCell::new(MemoryMapUnit::new(cartridge)));
        Ok(Self {
            mmu: mmu.clone(),
            cpu: LR35902::new(mmu),
            clock_ticks: 0,
            tx,
            rx,
            step_mode: true,
            next_step: false,
        })
    }

    fn handle_gui_messages(&mut self) -> bool {
        while let Ok(message) = self.rx.try_recv() {
            match message {
                GuiMessage::Close => return false,
                GuiMessage::NextInstruction => self.next_step = true,
                GuiMessage::RequestState => self.send_state_messages(),
                GuiMessage::StepMode(mode) => self.step_mode = mode,
                _ => (),
            };
        }
        true
    }

    fn send_state_messages(&mut self) {
        let registers_copy = self.cpu.registers.clone();
        if let Err(_) = self.tx.send(DmgMessage::RegistersStatus(registers_copy)) {
            error!("Could not send Registers Message !");
        }

        let memory = self.mmu.borrow().get_memory_dump();
        if let Err(_) = self.tx.send(DmgMessage::MemoryState(memory)) {
            error!("Could not send Memory Message !");
        }
    }

    pub fn start_game(&mut self) -> anyhow::Result<()> {
        loop {
            if let false = self.handle_gui_messages() {
                break;
            }

            if self.step_mode && !self.next_step {
                continue;
            }

            let _ticks = self.cpu.step();

            if self.step_mode {
                self.next_step = false;
            }
        }

        Ok(())
    }
}
