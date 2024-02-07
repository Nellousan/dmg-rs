use std::{
    cell::RefCell,
    rc::Rc,
    sync::{mpsc::Sender, Arc},
};

use eframe::epaint::Color32;
use tracing::error;

use crate::{dmg::ClockTicks, graphics, lr35902, mmu::MemoryMapUnit, thread::DmgMessage};

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

const LCDC_HBLANK: u8 = 1u8 << 3;
const LCDC_VBLANK: u8 = 1u8 << 4;
const LCDC_OAM: u8 = 1u8 << 5;
const LCDC_LYC: u8 = 1u8 << 6;

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

    fn trigger_interrupts(&self) {
        let lcdc = self.mmu.borrow().read_8(0xFF41);
        let mut int_flag = self.mmu.borrow().read_8(0xFF0F);
        let int_enable = self.mmu.borrow().read_8(0xFFFF);
        match self.mode {
            Mode::HBlank => {
                if lcdc & LCDC_HBLANK != 0 && int_enable & lr35902::LCDBIT != 0 {
                    int_flag |= lr35902::LCDBIT;
                }
            }
            Mode::VBlank => {
                if lcdc & LCDC_VBLANK != 0 && int_enable & lr35902::LCDBIT != 0 {
                    int_flag |= lr35902::LCDBIT;
                }
                if int_enable & lr35902::VBLANKBIT != 0 {
                    int_flag |= lr35902::VBLANKBIT;
                }
            }
            Mode::OAMSearch => {
                if lcdc & LCDC_OAM != 0 && int_enable & lr35902::LCDBIT != 0 {
                    int_flag |= lr35902::LCDBIT;
                }
            }
            _ => (),
        };
        self.mmu.borrow_mut().write_8(0xFF0F, int_flag);
    }

    fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
        let stat = self.mmu.borrow().read_8(0xFF41);
        let stat = (stat & 0xFC) | mode as u8;
        self.mmu.borrow_mut().write_8(0xFF41, stat);
        self.trigger_interrupts();
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
        self.line_to_draw += 1;
        if self.line_to_draw > 153 {
            self.line_to_draw = 0;
            self.set_mode(Mode::OAMSearch);
        }
        self.mmu
            .borrow_mut()
            .write_8(0xFF44, self.line_to_draw as u8);
        456
    }

    // #[tracing::instrument]
    fn draw_line(&mut self) -> ClockTicks {
        // TODO: Suboptimal hack used to facilitate image drawing
        // Potential bottleneck

        // Step 1: Draw the background
        let vram = self.mmu.borrow().vram();
        self.draw_bg_line(&vram);

        // Step 2: Draw the sprites

        // Step 3: Draw the window

        self.line_to_draw += 1;
        self.mmu
            .borrow_mut()
            .write_8(0xFF44, self.line_to_draw as u8);

        // Checks for LYC == LY to trigger interrupts
        let mut int_flag = self.mmu.borrow().read_8(0xFF0F);
        let lcdc = self.mmu.borrow().read_8(0xFF41);
        let int_enable = self.mmu.borrow().read_8(0xFFFF);
        let ly = self.mmu.borrow().read_8(0xFF44);
        let lyc = self.mmu.borrow().read_8(0xFF45);
        if ly == lyc && lcdc & LCDC_LYC != 0 && int_enable & lr35902::LCDBIT != 0 {
            int_flag |= lr35902::LCDBIT;
        }
        self.mmu.borrow_mut().write_8(0xFF0F, int_flag);
        172
    }

    fn draw_bg_line(&mut self, vram: &[u8]) {
        let lcdc = self.mmu.borrow().read_8(0xFF40);
        let tile_data;
        if lcdc & 0b00010000 != 0 {
            tile_data = &vram[0x0000..=0x0FFF];
        } else {
            tile_data = &vram[0x0800..=0x17FF];
            // tile_data = &vram[0x0000..=0x17FF];
        }

        let bg_data = &vram[0x1800..=0x1BFF];
        let palette = graphics::ColorPalette::from_dmg_palette(self.mmu.borrow().read_8(0xFF47));
        let scy = self.mmu.borrow().read_8(0xFF42) as usize;
        let scx = self.mmu.borrow().read_8(0xFF43) as usize;
        let line_y = scy + self.line_to_draw;
        let pixel_line =
            &mut self.pixel_buffer[(self.line_to_draw * 160)..((self.line_to_draw + 1) * 160)];
        for i in 0..160 {
            let bg_tile = bg_data[(line_y / 8) * 32 + ((scx + i) / 8)] as usize;

            let tile_array = &tile_data[(bg_tile * 16)..(bg_tile * 16 + 16)];
            let tile_px_y = line_y % 8;
            let tile_px_x = (scx + i) % 8;

            let px_byte_a = tile_array[tile_px_y * 2];
            let px_byte_b = tile_array[tile_px_y * 2 + 1];

            let bit_a = (px_byte_a.wrapping_shr(7 - tile_px_x as u32)) & 0x01;
            let bit_b = (px_byte_b.wrapping_shr(7 - tile_px_x as u32)) & 0x01;
            let color = (bit_b << 1) | bit_a;

            pixel_line[i] = palette[color as usize];
        }
        // Determine the tile byte given the coordinate of pixel
    }
}
