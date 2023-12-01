use std::sync::{
    mpsc::{Receiver, Sender},
    Arc,
};

use eframe::{
    egui::{self, load::SizedTexture, Key},
    epaint::{ColorImage, TextureHandle, Vec2},
};
use tracing::error;

use crate::{
    disassembler,
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
        cc.egui_ctx.set_pixels_per_point(1.3f32);
        // cc.egui_ctx
        //     .style_mut(|style| style.spacing.item_spacing = Vec2 { x: 5f32, y: 5f32 });
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

    fn format_ram_label(&self, section: &[u8], offset: u16) -> String {
        if section.len() != 0x1000 {
            return format!("Malformed section slice, length is {}", section.len());
        }

        let mut res = format!("      00 01 02 03 04 05 06 07 08 09 0A 0B 0C 0D 0E 0F");
        for i in 0..0x100 {
            let mut line = format!("\n{:0>4X} ", i * 0x10 + offset);
            for j in 0x0..0x10 {
                line.push_str(format!(" {:02X}", section[(i * 0x10 + j) as usize]).as_str());
            }
            res.push_str(line.as_str());
        }

        res
    }

    fn ui_ram(&self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.heading("Work Ram");
            egui::CollapsingHeader::new("Expand")
                .id_source("collapse1")
                .show(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .id_source("scroll1")
                        .min_scrolled_height(128f32)
                        .show(ui, |ui| {
                            ui.monospace(
                                self.format_ram_label(&self.state.memory[0xC000..0xD000], 0xC000),
                            );
                        });
                });
            ui.add_space(10f32);
            ui.heading("External Ram");
            egui::CollapsingHeader::new("Expand")
                .id_source("collapse2")
                .show(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .id_source("scroll2")
                        .min_scrolled_height(128f32)
                        .show(ui, |ui| {
                            ui.monospace(
                                self.format_ram_label(&self.state.memory[0xD000..0xE000], 0xD000),
                            );
                        });
                });
        });
    }

    fn ui_disassemble(&self, ui: &mut egui::Ui) {
        let instrucions = disassembler::disassemble(
            self.state.registers.get_16(Register16::PC),
            &self.state.memory[0x0000..0x8000],
            10,
        );

        ui.heading("Disassembled Code.");
        egui::ScrollArea::vertical()
            .id_source("scroll_disass")
            .min_scrolled_height(128f32)
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    for instruction in instrucions.iter() {
                        ui.monospace(format!(
                            "{:04X} {}    ",
                            instruction.address, instruction.mnemonic
                        ));
                    }
                });
            });
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
        self.handle_dmg_message(ctx);
        self.handle_inputs(ctx);
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    self.ui_registers(ui);
                    self.ui_disassemble(ui);
                });
                self.ui_ram(ui);
            })
        });
        ctx.request_repaint();

        if let Err(_) = self.tx.send(GuiMessage::RequestState) {
            error!("Could not send state request to DMG.")
        }
    }
}
