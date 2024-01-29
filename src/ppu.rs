use std::{cell::RefCell, rc::Rc};

use eframe::epaint::Color32;

use crate::{dmg::ClockTicks, mmu::MemoryMapUnit};

pub type PixelBuffer = [Color32; 160 * 144];

pub enum Mode {
    OAMSearch,
    PixelTransfer,
    HBlank,
    VBlank,
}

pub struct PixelProcessingUnit {
    mmu: Rc<RefCell<MemoryMapUnit>>,
    mode: Mode,
    pixel_buffer: PixelBuffer,
    line_to_draw: usize,
}

impl PixelProcessingUnit {
    fn new(mmu: Rc<RefCell<MemoryMapUnit>>) -> Self {
        Self {
            mmu,
            mode: Mode::OAMSearch,
            pixel_buffer: [Color32::WHITE; 160 * 144],
            line_to_draw: 0,
        }
    }

    fn step(&mut self) -> ClockTicks {
        match self.mode {
            Mode::OAMSearch => self.step_oam_search(),
            Mode::PixelTransfer => self.step_pixel_transfer(),
            Mode::HBlank => self.step_h_blank(),
            Mode::VBlank => self.step_v_blank(),
        }
    }

    fn step_oam_search(&mut self) -> ClockTicks {
        self.mode = Mode::PixelTransfer;
        20
    }

    fn step_pixel_transfer(&mut self) -> ClockTicks {
        self.draw_line()
    }

    fn step_h_blank(&mut self) -> ClockTicks {
        unimplemented!()
    }

    fn step_v_blank(&mut self) -> ClockTicks {
        1140
    }

    fn draw_line(&mut self) -> ClockTicks {
        unimplemented!()
    }
}
