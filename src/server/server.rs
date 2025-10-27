use super::state::ServerState;
use crate::gui::app::App;
use crate::server::config::Config;
use crate::server::subscription::get_calendar_file;
use anyhow::Result;
use axum::routing::get;
use dioxus::prelude::*;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tracing::debug;

pub fn server_start() -> Result<()> {
    debug!("Current server working dir: {:?}", std::env::current_dir());

    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async move {
            let config = Config::from_default()?;
            let state = ServerState::from_config(config)?;

            let ip = dioxus::cli_config::server_ip()
                .unwrap_or_else(|| IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
            let port = dioxus::cli_config::server_port().unwrap_or(8080);
            let address = SocketAddr::new(ip, port);
            let listener = tokio::net::TcpListener::bind(address).await.unwrap();
            let router = axum::Router::new()
                // serve_dioxus_application adds routes to server side render the application, serve static assets, and register server functions
                .serve_dioxus_application(
                    ServeConfigBuilder::default().context_providers(Arc::new(vec![Box::new(
                        move || Box::new(state.clone()),
                    )])),
                    App,
                )
                .route("/:school_api/:key/schedule.ics", get(get_calendar_file))
                .into_make_service();
            axum::serve(listener, router).await.unwrap();

            Result::<(), anyhow::Error>::Ok(())
        })
}
