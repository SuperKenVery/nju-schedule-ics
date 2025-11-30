use crate::adapters::nju_batchelor::NJUBatchelorAdaptor;
use crate::adapters::traits::{Credentials, LoginSession, School};
use crate::plugins::{PlugIn, get_plugins};
use crate::server::config::Config;
use anyhow::{Context, Result, anyhow, bail, ensure};
use axum::extract::FromRef;
use derivative::Derivative;
use dioxus::fullstack::FullstackContext;
use dioxus::fullstack::extract::{FromRequest, FromRequestParts};
use dioxus::html::u::part;
use dioxus::prelude::*;
use image::DynamicImage;
use sqlx::prelude::FromRow;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_sessions_sqlx_store::{SqliteStore, sqlx::SqlitePool};

#[derive(Derivative)]
#[derivative(Debug, Clone)]
pub struct ServerState {
    pub site_url: String,
    pub school_adapters: Arc<Mutex<HashMap<&'static str, Arc<dyn School>>>>,
    pub plugins: Arc<Vec<Arc<dyn PlugIn>>>,
}

impl ServerState {
    pub async fn from_config(cfg: Config, db: SqlitePool) -> Result<Self> {
        let mut school_adapters = HashMap::<&'static str, Arc<dyn School>>::new();
        school_adapters.insert(
            "南京大学本科生",
            Arc::new(NJUBatchelorAdaptor::new(Arc::new(Mutex::new(db.clone()))).await),
        );

        Ok(Self {
            site_url: cfg.site_url,
            school_adapters: Arc::new(Mutex::new(school_adapters)),
            plugins: Arc::new(get_plugins().await?),
        })
    }
}

impl FromRef<FullstackContext> for ServerState {
    fn from_ref(state: &FullstackContext) -> Self {
        state.extension::<ServerState>().unwrap()
    }
}

impl<S> FromRequestParts<S> for ServerState
where
    S: Sync + Send,
    ServerState: FromRef<S>,
{
    type Rejection = ServerFnError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> std::result::Result<Self, Self::Rejection> {
        Ok(ServerState::from_ref(state))
    }
}
