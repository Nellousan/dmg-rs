mod cartridge;
mod gui;
mod lr35902;
mod mmu;

use gui::Gui;
use mmu::MMU;
use std::{env, error};
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

    let args: Vec<String> = env::args().collect();
    let cartridge = cartridge::from_file(&args[1])?;

    let _mmu = MMU::new(cartridge);

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "My egui App",
        native_options,
        Box::new(|cc| Box::new(Gui::new(cc))),
    )?;

    Ok(())
}
