use super::state::ServerState;
use crate::adapters::login_process::LoginProcessManagerLayer;
use crate::gui::app::App;
use crate::server::calendar::get_calendar_file;
use crate::server::config::Config;
use anyhow::Result;
use axum::Extension;
use dioxus::prelude::*;
use sqlx::migrate::MigrateDatabase;
use sqlx::{Sqlite, SqlitePool};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tower_cookies::CookieManagerLayer;
use tracing::debug;

pub fn server_start() -> Result<()> {
    debug!("Current server working dir: {:?}", std::env::current_dir());

    dioxus::serve(|| async move {
        let config = Config::from_default()?;

        if !Sqlite::database_exists(&config.db_path).await? {
            Sqlite::create_database(&config.db_path).await?;
        }
        let db = SqlitePool::connect(config.db_path.as_str()).await?;

        let state = ServerState::from_config(config, db.clone()).await?;

        let router = dioxus::server::router(App)
            .layer(LoginProcessManagerLayer::new())
            .layer(CookieManagerLayer::new())
            .layer(Extension(state));

        Ok(router)
    });
}
