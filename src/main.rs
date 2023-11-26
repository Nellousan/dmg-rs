mod cartridge;
mod dmg;
mod gui;
mod lr35902;
mod mmu;
mod thread;

use dmg::DotMatrixGame;
use gui::Gui;
use std::{env, error, sync::mpsc::channel};
use thread::{DmgMessage, GuiMessage};
use tracing::Level;
use tracing_subscriber::{
    fmt::{self, writer::MakeWriterExt},
    prelude::__tracing_subscriber_SubscriberExt,
    util::SubscriberInitExt,
};

fn main() -> Result<(), Box<dyn error::Error>> {
    tracing_subscriber::registry()
        .with(
            fmt::Layer::new()
                .with_writer(std::io::stdout.with_max_level(Level::DEBUG))
                .with_file(false)
                .with_line_number(false)
                .with_thread_ids(false)
                .compact(),
        )
        .try_init()?;

    let (gui_tx, gui_rx) = channel::<GuiMessage>();
    let (dmg_tx, dmg_rx) = channel::<DmgMessage>();
    let tx_end = gui_tx.clone();

    let args: Vec<String> = env::args().collect();

    let mut dmg = DotMatrixGame::new_with_rom_path(&args[1], dmg_tx, gui_rx)?;

    std::thread::spawn(move || dmg.start_game());

    eframe::run_native(
        "My egui App",
        eframe::NativeOptions::default(),
        Box::new(|cc| Box::new(Gui::new(cc, gui_tx, dmg_rx))),
    )?;

    tx_end.send(GuiMessage::Close)?;

    Ok(())
}
