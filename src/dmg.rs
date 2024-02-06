use std::{
    cell::RefCell,
    rc::Rc,
    sync::mpsc::{Receiver, Sender},
};

use tracing::{error, trace_span};

use crate::{
    cartridge,
    clock::{Clock, TickCoordinator},
    lr35902::LR35902,
    mmu::MemoryMapUnit,
    ppu::PixelProcessingUnit,
    thread::{DmgMessage, GuiMessage},
};

pub struct DotMatrixGame {
    mmu: Rc<RefCell<MemoryMapUnit>>,
    cpu: LR35902,
    ppu: PixelProcessingUnit,
    tx: Sender<DmgMessage>,
    rx: Receiver<GuiMessage>,
    step_mode: bool,
    next_step: bool,
}

pub type ClockTicks = usize;

impl DotMatrixGame {
    pub fn new_with_rom_path(
        path: &str,
        tx: Sender<DmgMessage>,
        rx: Receiver<GuiMessage>,
    ) -> anyhow::Result<Self> {
        let cartridge = cartridge::from_file(path)?;
        let mmu = Rc::new(RefCell::new(MemoryMapUnit::new(cartridge)));
        let ppu = PixelProcessingUnit::new(mmu.clone(), tx.clone());
        Ok(Self {
            mmu: mmu.clone(),
            cpu: LR35902::new(mmu),
            ppu,
            tx,
            rx,
            step_mode: false,
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
        let mut cpu_ticks = TickCoordinator::new();
        let mut ppu_ticks = TickCoordinator::new();
        loop {
            // let _ = tick_span.enter();

            if let false = self.handle_gui_messages() {
                break;
            }

            std::thread::sleep(std::time::Duration::from_millis(16));

            if !self.step_mode {
                // Normal execution flow
                for _ in 0..69905 {
                    if cpu_ticks.tick() {
                        let ticks = self.cpu.step();
                        cpu_ticks.wait_for(ticks);
                    }

                    if ppu_ticks.tick() {
                        let ticks = self.ppu.step();
                        ppu_ticks.wait_for(ticks);
                    }
                }
            } else {
                // Step mode execution flow
                if !self.next_step {
                    continue;
                }

                let ct = cpu_ticks.tick_all();
                let ticks = self.cpu.step();
                cpu_ticks.wait_for(ticks);

                if ppu_ticks.ticks(ct) {
                    let ticks = self.ppu.step();
                    ppu_ticks.wait_for(ticks);
                }

                self.next_step = false;
            }
        }

        Ok(())
    }
}
