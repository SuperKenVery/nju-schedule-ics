use anyhow::Result;
use tracing::error;

#[cfg(feature = "web")]
use nju_schedule_ics::gui;

#[cfg(feature = "server")]
use nju_schedule_ics::server::server;

fn main() -> Result<()> {
    dioxus::logger::initialize_default();

    #[cfg(feature = "server")]
    match tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(server::server_start())
    {
        Ok(_) => (),
        Err(error) => error!("Error running server: {}", error),
    }

    #[cfg(feature = "web")]
    gui::start_app();

    Ok(())
}
