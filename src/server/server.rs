use super::state::ServerState;
use crate::gui::app::App;
use crate::server::config::Config;
use anyhow::Result;
use tracing::debug;

pub fn server_start() -> Result<()> {
    debug!("Current server working dir: {:?}", std::env::current_dir());

    let config = Config::from_default()?;
    let state = ServerState::from_config(config)?;

    dioxus::LaunchBuilder::new().with_context(state).launch(App);

    Ok(())
}
