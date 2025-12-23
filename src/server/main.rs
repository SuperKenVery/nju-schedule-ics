use super::state::ServerState;
use crate::adapters::login_process::LoginProcessManagerLayer;
use crate::gui::app::App;
use crate::server::config::Config;
use anyhow::Result;
use axum::error_handling::HandleErrorLayer;
use axum::http::StatusCode;
use axum::{Extension, Json};
use sqlx::migrate::MigrateDatabase;
use sqlx::{Sqlite, SqlitePool};
use tower::{BoxError, ServiceBuilder};
use tower_cookies::CookieManagerLayer;
use tower_http::compression::CompressionLayer;
use tracing::{Level, debug, info};

pub fn server_start() -> Result<()> {
    dioxus_logger::init(Level::INFO).expect("Failed to init logger");
    info!("Current server working dir: {:?}", std::env::current_dir());
    info!(
        "Listening on: {:?}",
        dioxus_cli_config::fullstack_address_or_localhost()
    );

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
            .layer(Extension(state))
            .layer(CompressionLayer::new().zstd(true).gzip(true));

        Ok(router)
    });
}
