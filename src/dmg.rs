use std::sync::mpsc::{Receiver, Sender};

use crate::{
    cartridge,
    lr35902::LR35902,
    mmu::MemoryMapUnit,
    thread::{DmgMessage, GuiMessage},
};

pub struct DotMatrixGame {
    mmu: MemoryMapUnit,
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
    ) -> Result<Self, anyhow::Error> {
        let cartridge = cartridge::from_file(path)?;
        Ok(Self {
            mmu: MemoryMapUnit::new(cartridge),
            cpu: LR35902::new(),
            clock_ticks: 0,
            tx,
            rx,
        })
    }

    pub fn start_game(&mut self) -> Result<(), anyhow::Error> {
        loop {
            if let Ok(result) = self.rx.try_recv() {
                match result {
                    GuiMessage::Close => break,
                    _ => (),
                }
            }
        }

        Ok(())
    }
}
