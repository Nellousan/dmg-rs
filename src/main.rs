mod cartridge;
mod clock;
mod disassembler;
mod dmg;
mod graphics;
mod gui;
mod joypad;
mod lr35902;
mod mmu;
mod ppu;
mod thread;
mod timer;
mod tracer;

extern crate getopts;

use dmg::DotMatrixGame;
use gui::Gui;
use std::{env, error, sync::mpsc::channel};
use thread::{DmgMessage, GuiMessage};
use tracing::Level;
use tracing_flame::FlameLayer;
use tracing_subscriber::{
    fmt::{self, writer::MakeWriterExt},
    prelude::__tracing_subscriber_SubscriberExt,
    util::SubscriberInitExt,
};

fn main() -> Result<(), Box<dyn error::Error>> {
    let (flame_layer, _guard) = FlameLayer::with_file("./tracing.folded").unwrap();
    tracing_subscriber::registry()
        .with(
            fmt::Layer::new()
                .with_writer(std::io::stdout.with_max_level(Level::DEBUG))
                .with_file(false)
                .with_line_number(false)
                .with_thread_ids(false)
                .compact(),
        )
        .with(flame_layer)
        .try_init()?;

    let (gui_tx, gui_rx) = channel::<GuiMessage>();
    let (dmg_tx, dmg_rx) = channel::<DmgMessage>();
    let tx_end = gui_tx.clone();

    let args: Vec<String> = env::args().collect();

    let handle = std::thread::spawn(move || {
        let mut dmg = DotMatrixGame::new_with_rom_path(&args[1], dmg_tx, gui_rx)?;
        dmg.start_game()
    });

    let mut options = eframe::NativeOptions::default();
    options.viewport.min_inner_size = Some(eframe::egui::Vec2::new(1280f32, 720f32));
    eframe::run_native(
        "DMG",
        options,
        Box::new(|cc| Box::new(Gui::new(cc, gui_tx, dmg_rx))),
    )?;

    tx_end.send(GuiMessage::Close)?;

    handle.join().map_err(|e| format!("{:?}", e))??;
    Ok(())
}
