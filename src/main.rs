use anyhow::Result;
use nju_schedule_ics::server::log_format::LogTimePrefix;

#[cfg(feature = "web")]
use nju_schedule_ics::gui;

#[cfg(feature = "server")]
use nju_schedule_ics::server::server;

fn main() -> Result<()> {
    colog::default_builder()
        .format(colog::formatter(LogTimePrefix))
        .filter(Some("nju_schedule_ics"), log::LevelFilter::Debug)
        .init();

    #[cfg(feature = "server")]
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(server::server_start())
        .unwrap();

    #[cfg(feature = "web")]
    gui::start_app();

    Ok(())
}
