use std::sync::{
    mpsc::{Receiver, Sender},
    Arc,
};

use eframe::{
    egui::{self, load::SizedTexture, Key},
    epaint::{ColorImage, TextureHandle},
};
use tracing::error;

use crate::{
    lr35902::{Register16, Register8, Registers},
    thread::{DmgMessage, GuiMessage},
};

struct State {
    registers: Registers,
    memory: Arc<[u8; 0xFFFF]>,
}

pub struct Gui {
    color_image: ColorImage,
    texture_handle: TextureHandle,
    texture: SizedTexture,
    tx: Sender<GuiMessage>,
    rx: Receiver<DmgMessage>,
    state: State,
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
            state: State {
                registers: Default::default(),
                memory: Arc::new([0u8; 0xFFFF]),
            },
        }
    }

    fn handle_dmg_message(&mut self, _ctx: &egui::Context) {
        while let Ok(message) = self.rx.try_recv() {
            match message {
                DmgMessage::RegistersStatus(registers) => self.state.registers = registers,
                DmgMessage::MemoryState(state) => self.state.memory = state,
                _ => (),
            }
        }
    }

    fn ui_registers(&self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.heading("Registers");
            ui.horizontal(|ui| {
                ui.monospace(format!(
                    "A {:#04X}",
                    self.state.registers.get_8(Register8::A)
                ));
                ui.monospace(format!(
                    "F {:#04X}",
                    self.state.registers.get_8(Register8::F)
                ));
            });
            ui.horizontal(|ui| {
                ui.monospace(format!(
                    "BC {:#06X}",
                    self.state.registers.get_16(Register16::BC)
                ));
                ui.monospace(format!(
                    "DE {:#06X}",
                    self.state.registers.get_16(Register16::DE)
                ));
            });
            ui.horizontal(|ui| {
                ui.monospace(format!(
                    "HL {:#06X}",
                    self.state.registers.get_16(Register16::HL)
                ));
                ui.monospace(format!(
                    "SP {:#06X}",
                    self.state.registers.get_16(Register16::SP)
                ));
            });
            ui.monospace(format!(
                "PC {:#06X}",
                self.state.registers.get_16(Register16::PC)
            ));
        });
    }

    fn ui_ram(&self, ui: &mut egui::Ui) {
        unimplemented!()
    }

    fn handle_inputs(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.key_pressed(Key::N)) {
            if let Err(_) = self.tx.send(GuiMessage::NextInstruction) {
                error!("Could not send Next Instruction message");
            }
        }
    }
}

impl eframe::App for Gui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.handle_dmg_message(ctx);
            self.handle_inputs(ctx);
            self.ui_registers(ui);
        });
        ctx.request_repaint();
    }
}
