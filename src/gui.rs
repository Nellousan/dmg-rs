use std::sync::mpsc::{Receiver, Sender};

use eframe::{
    egui::{self, load::SizedTexture, RichText},
    epaint::{Color32, ColorImage, TextureHandle},
};

use crate::thread::{DmgMessage, GuiMessage};

pub struct Gui {
    color_image: ColorImage,
    texture_handle: TextureHandle,
    texture: SizedTexture,
    tx: Sender<GuiMessage>,
    rx: Receiver<DmgMessage>,
}

impl Gui {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        tx: Sender<GuiMessage>,
        rx: Receiver<DmgMessage>,
    ) -> Self {
        let raw_image_data = [255u8; 200 * 200 * 4];
        let color_image = ColorImage::from_rgba_unmultiplied([200, 200], &raw_image_data);
        let texture_handle =
            cc.egui_ctx
                .load_texture("Image", color_image.clone(), Default::default());
        let texture = egui::load::SizedTexture::from_handle(&texture_handle);

        Self {
            color_image,
            texture_handle,
            texture,
            tx,
            rx,
        }
    }
}

impl eframe::App for Gui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        for color in &mut self.color_image.pixels {
            *color = Color32::from_rgb(
                color.r().wrapping_add(3),
                color.g().wrapping_add(2),
                color.b().wrapping_add(1),
            );
        }

        self.texture_handle
            .set(self.color_image.clone(), Default::default());

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello World!");
            ui.label(RichText::new("Ouais").strong());
            ui.horizontal(|ui| {
                ui.label("hm");
                ui.label("Label");
                ui.image(self.texture);
            });
        });
        ctx.request_repaint();
    }
}
