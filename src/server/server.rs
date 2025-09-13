use super::error::AppError;
use super::log_format::LogTimePrefix;
use crate::server::config::Config;
use anyhow::Result;
use axum::{
    routing::{get, post},
    Router,
};
use derivative::Derivative;
use diesel::{Connection, SqliteConnection};
use dioxus::prelude::*;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Derivative)]
#[derivative(Debug, Clone)]
pub struct AppState {
    #[derivative(Debug = "ignore")]
    pub db: Arc<Mutex<SqliteConnection>>,
    pub site_url: String,
}

impl TryFrom<Config> for AppState {
    type Error = anyhow::Error;

    fn try_from(cfg: Config) -> Result<Self> {
        Ok(Self {
            db: Arc::new(Mutex::new(SqliteConnection::establish(&cfg.db_path)?)),
            site_url: cfg.site_url,
        })
    }
}

#[cfg(feature = "server")]
pub async fn server_start() -> Result<()> {
    use crate::gui::app::App;

    let config = Config::from_default().await?;
    let state: AppState = config.clone().try_into()?;

    // Build a custom axum router
    let router = axum::Router::new()
        .serve_dioxus_application(ServeConfigBuilder::new(), App)
        .with_state(state)
        .into_make_service();
    let listen_addr = dioxus::cli_config::fullstack_address_or_localhost();

    // And launch it!
    let listener = tokio::net::TcpListener::bind(listen_addr).await?;
    axum::serve(listener, router).await?;

    Ok(())
}
