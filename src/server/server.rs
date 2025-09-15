use super::error::AppError;
use crate::server::config::Config;
use crate::{adapters::traits::School, gui::app::App};
use anyhow::Result;
use axum::{
    Router,
    routing::{get, post},
};
use derivative::Derivative;
use diesel::{Connection, SqliteConnection};
use dioxus::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::debug;

#[derive(Derivative)]
#[derivative(Debug, Clone)]
pub struct AppState {
    #[derivative(Debug = "ignore")]
    pub db: Arc<Mutex<SqliteConnection>>,
    pub site_url: String,
    /// An unfinished login session. Its lifecycle is:
    /// - Lifecycle starts when the user access this website. The user gets a session ID now.
    /// - Then the user selects a school API, and logins with username and password.
    /// - After that, we associate the session ID with a [`Credentials`], storing that in DB.
    /// - Now this session is no longer an "unfinished login session", and is removed from this HashMap.
    pub unfinished_login_sessions: Arc<Mutex<HashMap<String, UnfinishedLoginSession>>>,
}

impl AppState {
    fn from_config(cfg: Config) -> Result<Self> {
        Ok(Self {
            db: Arc::new(Mutex::new(SqliteConnection::establish(&cfg.db_path)?)),
            site_url: cfg.site_url,
            unfinished_login_sessions: Arc::new(Mutex::new(HashMap::new())),
            // db: SqlitePool::connect(&cfg.db_path).await?,
        })
    }
}

pub fn server_start() -> Result<()> {
    debug!("Current server working dir: {:?}", std::env::current_dir());

    let config = Config::from_default()?;
    let state: AppState = AppState::from_config(config)?;

    dioxus::LaunchBuilder::new().with_context(state).launch(App);

    Ok(())
}

#[derive(Debug, Clone)]
pub struct UnfinishedLoginSession {
    pub selected_school_api: Option<Arc<dyn School>>,
    pub client: Option<reqwest::Client>,
}

impl UnfinishedLoginSession {
    pub fn new() -> Self {
        Self {
            selected_school_api: None,
            client: None,
        }
    }
}
