use super::state::ServerState;
use crate::gui::app::App;
use crate::server::calendar::get_calendar_file;
use crate::server::config::Config;
use anyhow::Result;
use axum::routing::get;
use dioxus::prelude::*;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tracing::debug;

pub fn server_start() -> Result<()> {
    debug!("Current server working dir: {:?}", std::env::current_dir());
    dioxus_logger::initialize_default();

    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async move {
            let config = Config::from_default()?;
            let state = ServerState::from_config(config).await?;
            let state2 = state.clone();

            let ip = dioxus::cli_config::server_ip()
                .unwrap_or_else(|| IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
            let port = dioxus::cli_config::server_port().unwrap_or(8080);
            let address = SocketAddr::new(ip, port);
            let listener = tokio::net::TcpListener::bind(address).await.unwrap();

            // dioxus's server state isn't compatible with axum's with_state (axum handler can't see dioxus state, and vice versa.)
            // so we'll have to pass two states (one in [`with_state`] another in [`ServeConfigBuilder::context_providers`]).
            // ServerState contains mostly `Arc`s (except for site_url), so it's safe to clone (everything still shared).
            let router = axum::Router::new()
                // serve_dioxus_application adds routes to server side render the application, serve static assets, and register server functions
                .serve_dioxus_application(
                    ServeConfigBuilder::default().context_providers(Arc::new(vec![Box::new(
                        move || Box::new(state2.clone()),
                    )])),
                    App,
                )
                .route(
                    "/calendar/:school_api/:key/schedule.ics",
                    get(get_calendar_file),
                )
                .with_state(state.clone())
                .into_make_service();
            axum::serve(listener, router).await.unwrap();

            Result::<(), anyhow::Error>::Ok(())
        })
}
