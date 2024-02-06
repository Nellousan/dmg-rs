use std::sync::{
    mpsc::{Receiver, Sender},
    Arc,
};

use eframe::{
    egui::{self, Key, Window},
    epaint::{Color32, ColorImage, TextureHandle},
};
use tracing::{debug, error};

use crate::{
    disassembler,
    graphics::{draw_bg_map, draw_tile_data},
    lr35902::{Register16, Register8, Registers},
    ppu::PixelBuffer,
    thread::{DmgMessage, GuiMessage},
};

struct State {
    registers: Registers,
    memory: Arc<[u8; 0x10000]>,
}

pub struct Gui {
    tile_texture_handle: TextureHandle,
    bg_map_texture_handle: TextureHandle,
    screen_texture_handle: TextureHandle,
    tx: Sender<GuiMessage>,
    rx: Receiver<DmgMessage>,
    state: State,
    rom_label_content: String,
    ram_label_content: String,
    memory_label_content: String,
}

impl Gui {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        tx: Sender<GuiMessage>,
        rx: Receiver<DmgMessage>,
    ) -> Self {
        let tile_image = ColorImage::new([16 * 8, 24 * 8], Color32::WHITE);
        let bg_map_image = ColorImage::new([32 * 8, 32 * 8], Color32::WHITE);
        let screen_image = ColorImage::new([160, 140], Color32::WHITE);
        let tile_texture_handle =
            cc.egui_ctx
                .load_texture("TileData", tile_image, Default::default());
        let bg_map_texture_handle =
            cc.egui_ctx
                .load_texture("BGMapData", bg_map_image, Default::default());
        let screen_texture_handle =
            cc.egui_ctx
                .load_texture("ScreenData", screen_image, Default::default());
        cc.egui_ctx.set_pixels_per_point(1.3f32);

        Self {
            tile_texture_handle,
            bg_map_texture_handle,
            screen_texture_handle,
            tx,
            rx,
            state: State {
                registers: Default::default(),
                memory: Arc::new([0u8; 0x10000]),
            },
            rom_label_content: "".to_string(),
            ram_label_content: "".to_string(),
            memory_label_content: "".to_string(),
        }
    }

    fn update_memory_state(&mut self, _ctx: &egui::Context, state: Arc<[u8; 65536]>) {
        let tile_image = draw_tile_data(&state[0x8000..=0x97FF], state[0xFF47]);
        let bg_map_image = draw_bg_map(&state[0x9800..=0x9BFF], &tile_image);

        self.rom_label_content =
            self.format_ram_label(&self.state.memory[0xC000..0xD000], 0xC000, 0x1000);
        self.ram_label_content =
            self.format_ram_label(&self.state.memory[0xD000..0xE000], 0xD000, 0x1000);
        self.memory_label_content = self.format_ram_label(&*self.state.memory, 0x0000, 0x10000);
        self.tile_texture_handle.set(tile_image, Default::default());
        self.bg_map_texture_handle
            .set(bg_map_image, Default::default());
        self.state.memory = state;
    }

    fn update_screen_texture(&mut self, _ctx: &egui::Context, pixel_buffer: Arc<PixelBuffer>) {
        let mut image = ColorImage::new([160, 144], Color32::WHITE);
        for (i, pixel) in pixel_buffer.iter().enumerate() {
            image[(i % 160, i / 160)] = pixel.clone();
        }

        self.screen_texture_handle.set(image, Default::default());
    }

    fn handle_dmg_message(&mut self, ctx: &egui::Context) {
        while let Ok(message) = self.rx.try_recv() {
            match message {
                DmgMessage::RegistersStatus(registers) => self.state.registers = registers,
                DmgMessage::MemoryState(state) => self.update_memory_state(ctx, state),
                DmgMessage::Render(pixel_buffer) => self.update_screen_texture(ctx, pixel_buffer),
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

    fn format_ram_label(&self, section: &[u8], offset: u16, length: usize) -> String {
        let mut res = format!("      00 01 02 03 04 05 06 07 08 09 0A 0B 0C 0D 0E 0F");
        for i in 0x0..length / 0x10 {
            let mut line = format!("\n{:0>4X} ", i * 0x10 + offset as usize);
            for j in 0x0..0x10 {
                line.push_str(format!(" {:02X}", section[(i * 0x10 + j) as usize]).as_str());
            }
            res.push_str(line.as_str());
        }

        res
    }

    fn ui_ram(&self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.heading("Work RAM");
            egui::CollapsingHeader::new("Expand")
                .id_source("collapse1")
                .show(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .id_source("scroll1")
                        .min_scrolled_height(128f32)
                        .show(ui, |ui| {
                            ui.monospace(&self.rom_label_content);
                        });
                });
            ui.add_space(10f32);
            ui.heading("External RAM");
            egui::CollapsingHeader::new("Expand")
                .id_source("collapse2")
                .show(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .id_source("scroll2")
                        .min_scrolled_height(128f32)
                        .show(ui, |ui| {
                            ui.monospace(&self.ram_label_content);
                        });
                });
        });
    }

    fn ui_disassemble(&self, ui: &mut egui::Ui) {
        let instrucions = disassembler::disassemble(
            self.state.registers.get_16(Register16::PC),
            &*self.state.memory,
            10,
        );

        ui.heading("Disassembled Instructions");
        egui::ScrollArea::vertical()
            .id_source("scroll_disass")
            .min_scrolled_height(128f32)
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    for instruction in instrucions {
                        ui.monospace(format!(
                            "{:04X} {}    ",
                            instruction.address, instruction.mnemonic
                        ));
                    }
                });
            });
    }

    fn ui_vram(&self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.heading("Video RAM");
            egui::CollapsingHeader::new("Expand")
                .id_source("collapse_vram")
                .show(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .id_source("scroll_vram")
                        .show(ui, |ui| {
                            ui.vertical(|ui| {
                                ui.vertical(|ui| {
                                    ui.label("Background Map");
                                    ui.add(
                                        egui::Image::new(egui::load::SizedTexture::from_handle(
                                            &self.bg_map_texture_handle,
                                        ))
                                        .fit_to_original_size(1f32),
                                    );
                                });
                                ui.vertical(|ui| {
                                    ui.label("Tile Data");
                                    ui.add(
                                        egui::Image::new(egui::load::SizedTexture::from_handle(
                                            &self.tile_texture_handle,
                                        ))
                                        .fit_to_original_size(1.5),
                                    );
                                });
                            })
                        });
                });
        });
    }

    fn ui_screen(&self, ui: &mut egui::Ui) {
        ui.add(
            egui::Image::new(egui::load::SizedTexture::from_handle(
                &self.screen_texture_handle,
            ))
            .fit_to_original_size(2f32),
        );
    }

    fn handle_inputs(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.key_pressed(Key::N)) {
            if let Err(_) = self.tx.send(GuiMessage::NextInstruction) {
                error!("Could not send Next Instruction message");
            }
            if let Err(_) = self.tx.send(GuiMessage::RequestState) {
                error!("Could not send State Request message");
            }
        }
        if ctx.input(|i| i.key_pressed(Key::S)) {
            if let Err(_) = self.tx.send(GuiMessage::StepMode(true)) {
                error!("Could not send Stop message");
            }
        }
        if ctx.input(|i| i.key_pressed(Key::C)) {
            if let Err(_) = self.tx.send(GuiMessage::StepMode(false)) {
                error!("Could not send Continue message");
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
                self.ui_screen(ui);
                self.ui_ram(ui);
                self.ui_vram(ui);
            });
            egui::Window::new("Memory")
                .default_open(false)
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical()
                        .id_source("mem_window_scroll")
                        .min_scrolled_height(128f32)
                        .show(ui, |ui| {
                            ui.monospace(&self.memory_label_content);
                        });
                })
        });
        ctx.request_repaint();

        if let Err(_) = self.tx.send(GuiMessage::RequestState) {
            error!("Could not send state request to DMG.")
        }
    }
}
