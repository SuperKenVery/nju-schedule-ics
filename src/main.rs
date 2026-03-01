use anyhow::Result;
use tracing::Level;

#[cfg(feature = "web")]
use nju_schedule_ics::gui;

#[cfg(feature = "server")]
use nju_schedule_ics::server::main as server;

fn main() -> Result<()> {
    dioxus_logger::init(Level::INFO).expect("Failed to init logger");

    #[cfg(feature = "server")]
    server::server_start()?;

    #[cfg(feature = "web")]
    gui::start_app();

    Ok(())
}
