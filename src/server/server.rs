use super::state::ServerState;
use crate::adapters::login_process::LoginProcessManagerLayer;
use crate::gui::app::App;
use crate::server::calendar::get_calendar_file;
use crate::server::config::Config;
use anyhow::Result;
use axum::Extension;
use dioxus::prelude::*;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tracing::debug;

pub fn server_start() -> Result<()> {
    use dioxus::server::axum::routing::{get, post};
    debug!("Current server working dir: {:?}", std::env::current_dir());
    dioxus_logger::initialize_default();

    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async move {
            let config = Config::from_default()?;
            let db = SqlitePool::connect(config.db_path.as_str()).await?;

            let session_store = SqliteStore::new(db);
            let state = ServerState::from_config(config, db.clone()).await?;

            let session_layer = LoginProcessManagerLayer {};

            dioxus::serve(|| {
                let session_layer = session_layer.clone();
                let state = state.clone();

                async move {
                    let router = dioxus::server::router(App)
                        .layer(session_layer.clone())
                        // .with_state(state);
                        .layer(Extension(state.clone()));

                    Ok(router)
                }
            });
        })
}
