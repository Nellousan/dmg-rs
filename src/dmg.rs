use std::{
    cell::RefCell,
    rc::Rc,
    sync::mpsc::{Receiver, Sender},
};

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
        })
    }

    fn handle_gui_messages(&mut self) -> anyhow::Result<()> {
        while let Ok(result) = self.rx.try_recv() {
            match result {
                GuiMessage::Close => return Err(anyhow::Error::msg("")),
                _ => (),
            }
        }
        Ok(())
    }

    pub fn start_game(&mut self) -> anyhow::Result<()> {
        self.cpu.registers.set_16(Register16::PC, 0x0100);

        loop {
            if let Err(_) = self.handle_gui_messages() {
                break;
            }
        }

        Ok(())
    }
}
