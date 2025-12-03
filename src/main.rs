use anyhow::Result;

#[cfg(feature = "web")]
use nju_schedule_ics::gui;

#[cfg(feature = "server")]
use nju_schedule_ics::server::server;

fn main() -> Result<()> {
    #[cfg(feature = "server")]
    server::server_start()?;

    #[cfg(feature = "web")]
    gui::start_app();

    Ok(())
}
