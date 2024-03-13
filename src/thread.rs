use std::sync::Arc;

use crate::{lr35902::Registers, ppu};

pub enum DmgMessage {
    RegistersStatus(Registers),
    MemoryState(Arc<[u8; 0x10000]>),
    Render(Arc<ppu::PixelBuffer>),
}

#[derive(Debug)]
pub enum DmgButton {
    Up,
    Down,
    Left,
    Right,
    A,
    B,
    Start,
    Select,
}

pub enum GuiMessage {
    ButtonPressed(DmgButton),
    ButtonReleased(DmgButton),
    NextInstruction(usize),
    RequestState,
    Close,
    StepMode(bool),
}
