use std::ops::Index;

use eframe::epaint::{Color32, ColorImage};
use tracing::{debug, info};

static PALETTE: ColorPalette = DEFAULT_PALETTE;

pub const DEFAULT_PALETTE: ColorPalette = ColorPalette(
    Color32::from_rgb(0xE0, 0xF8, 0xD0),
    Color32::from_rgb(0x88, 0xC0, 0x70),
    Color32::from_rgb(0x34, 0x68, 0x56),
    Color32::from_rgb(0x08, 0x18, 0x20),
);

pub type DmgPalette = u8;

pub struct ColorPalette(Color32, Color32, Color32, Color32);

impl ColorPalette {
    pub fn from_colors(r: Color32, g: Color32, b: Color32, a: Color32) -> Self {
        ColorPalette(r, g, b, a)
    }

    pub fn from_dmg_palette(palette: DmgPalette) -> Self {
        ColorPalette(
            DEFAULT_PALETTE[((palette >> 0) & 0x03) as usize],
            DEFAULT_PALETTE[((palette >> 2) & 0x03) as usize],
            DEFAULT_PALETTE[((palette >> 4) & 0x03) as usize],
            DEFAULT_PALETTE[((palette >> 6) & 0x03) as usize],
        )
    }
}

impl Index<usize> for ColorPalette {
    type Output = Color32;

    fn index(&self, idx: usize) -> &Self::Output {
        match idx {
            0 => &self.0,
            1 => &self.1,
            2 => &self.2,
            3 => &self.3,
            _ => unreachable!(),
        }
    }
}

// #[tracing::instrument]
pub fn draw_tile_data(data: &[u8], dmg_palette: DmgPalette) -> ColorImage {
    let palette = ColorPalette::from_dmg_palette(dmg_palette);
    let mut image = ColorImage::new([16 * 8, 24 * 8], Color32::WHITE);
    for i in 0..(16 * 24) {
        let data_idx = i * 16;
        let tile_array = &data[data_idx..data_idx + 16];
        for j in 0..8 {
            let byte_a = tile_array[j * 2];
            let byte_b = tile_array[j * 2 + 1];
            for bit in 0..8 {
                let bit_a = (byte_a.wrapping_shr(7 - bit)) & 0x01;
                let bit_b = (byte_b.wrapping_shr(7 - bit)) & 0x01;
                let color = (bit_b << 1) | bit_a;

                let px_y = (i / 16) * 8 + j;
                let px_x = (i % 16) * 8 + bit as usize;
                image[(px_x, px_y)] = palette[color as usize];
            }
        }
    }
    image
}

// #[tracing::instrument]
pub fn draw_bg_map(data: &[u8], tile_image: &ColorImage) -> ColorImage {
    let mut image = ColorImage::new([32 * 8, 32 * 8], Color32::WHITE);
    for (i, tile_idx) in data.iter().enumerate() {
        for y in 0..8 {
            for x in 0..8 {
                let ipx_y = (i / 32) * 8 + y;
                let ipx_x = (i % 32) * 8 + x;
                let tpx_y = (*tile_idx as usize / 16) * 8 + y;
                let tpx_x = (*tile_idx as usize % 16) * 8 + x;
                image[(ipx_x, ipx_y)] = tile_image[(tpx_x, tpx_y)];
            }
        }
    }
    image
}
