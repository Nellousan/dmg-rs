use std::{
    cell::RefCell,
    rc::Rc,
    sync::{mpsc::Sender, Arc},
};

use eframe::epaint::Color32;
use tracing::{error, info};

use crate::{dmg::ClockTicks, graphics, mmu::MemoryMapUnit, thread::DmgMessage};

pub type PixelBuffer = [Color32; 160 * 144];

#[derive(Debug, Clone, Copy)]
pub enum Mode {
    HBlank,
    VBlank,
    OAMSearch,
    PixelTransfer,
}

#[derive(Debug)]
pub struct PixelProcessingUnit {
    mmu: Rc<RefCell<MemoryMapUnit>>,
    mode: Mode,
    pixel_buffer: PixelBuffer,
    line_to_draw: usize,
    tx: Sender<DmgMessage>,
}

impl PixelProcessingUnit {
    pub fn new(mmu: Rc<RefCell<MemoryMapUnit>>, tx: Sender<DmgMessage>) -> Self {
        Self {
            mmu,
            mode: Mode::OAMSearch,
            pixel_buffer: [Color32::WHITE; 160 * 144],
            line_to_draw: 0,
            tx,
        }
    }

    pub fn step(&mut self) -> ClockTicks {
        match self.mode {
            Mode::OAMSearch => self.step_oam_search(),
            Mode::PixelTransfer => self.step_pixel_transfer(),
            Mode::HBlank => self.step_h_blank(),
            Mode::VBlank => self.step_v_blank(),
        }
    }

    fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
        let stat = self.mmu.borrow().read_8(0xFF41);
        let stat = (stat & 0xFC) | mode as u8;
        self.mmu.borrow_mut().write_8(0xFF41, stat);
    }

    fn step_oam_search(&mut self) -> ClockTicks {
        self.set_mode(Mode::PixelTransfer);
        80
    }

    fn step_pixel_transfer(&mut self) -> ClockTicks {
        let ticks = self.draw_line();
        if self.line_to_draw >= 144 {
            if let Err(err) = self
                .tx
                .send(DmgMessage::Render(Arc::new(self.pixel_buffer.clone())))
            {
                error!("Could not send Pixel buffer to GUI: {:?}", err);
            }
        }
        self.set_mode(Mode::HBlank);
        ticks
    }

    fn step_h_blank(&mut self) -> ClockTicks {
        if self.line_to_draw >= 144 {
            self.set_mode(Mode::VBlank);
        } else {
            self.set_mode(Mode::OAMSearch);
        }
        204
    }

    fn step_v_blank(&mut self) -> ClockTicks {
        self.line_to_draw = 0;
        self.mmu
            .borrow_mut()
            .write_8(0xFF44, self.line_to_draw as u8);
        self.set_mode(Mode::OAMSearch);
        4560
    }

    // #[tracing::instrument]
    fn draw_line(&mut self) -> ClockTicks {
        // TODO: Suboptimal hack used to facilitate image drawing
        // Potential bottleneck

        // Step 1: Draw the background
        let memory = self.mmu.borrow().get_memory_dump();
        let tile_data = graphics::draw_tile_data(&memory[0x8000..=0x97FF], memory[0xFF47]);
        let bg_map = graphics::draw_bg_map(&memory[0x9800..=0x9BFF], &tile_data);
        let scy = memory[0xFF42];
        let scx = memory[0xFF43];
        let pixel_line =
            &mut self.pixel_buffer[(self.line_to_draw * 160)..((self.line_to_draw + 1) * 160)];
        for i in 0..160 {
            pixel_line[i] = bg_map[(scy as usize, scx as usize + i)];
        }
        // Step 2: Draw the sprites

        // Step 3: Draw the window

        self.line_to_draw += 1;
        self.mmu
            .borrow_mut()
            .write_8(0xFF44, self.line_to_draw as u8);
        172
    }
}
