use crate::adapters::all_school_adapters::school_adapter_from_name;
use crate::adapters::traits::{Credentials, LoginSession, School};
use crate::server::config::Config;
use anyhow::{Context, Result, anyhow, bail};
use derivative::Derivative;
use diesel::{Connection, SqliteConnection};
use dioxus::prelude::*;
use image::DynamicImage;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Derivative)]
#[derivative(Debug, Clone)]
pub struct ServerState {
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

impl ServerState {
    pub fn from_config(cfg: Config) -> Result<Self> {
        Ok(Self {
            db: Arc::new(Mutex::new(SqliteConnection::establish(&cfg.db_path)?)),
            site_url: cfg.site_url,
            unfinished_login_sessions: Arc::new(Mutex::new(HashMap::new())),
            // db: SqlitePool::connect(&cfg.db_path).await?,
        })
    }
}

#[derive(Derivative)]
#[derivative(Debug, Clone)]
pub enum UnfinishedLoginSession {
    Started,
    SelectedSchool {
        school: Arc<dyn School>,
        session: Arc<dyn LoginSession>,
    },
    Finished {
        school: Arc<dyn School>,
        #[derivative(Debug = "ignore")]
        credentials: Arc<dyn Credentials>,
    },
}

#[cfg(feature = "server")]
impl UnfinishedLoginSession {
    pub async fn select_school(&mut self, school_name: String) -> Result<()> {
        use anyhow::Context;

        let Self::Started = self else {
            bail!("Not in Started state when calling `select_school`");
        };
        // ensure!(self == Self::Started, "Not in started");
        let FromContext(state): FromContext<ServerState> = extract().await?;

        let school: Arc<dyn School> = school_adapter_from_name(&school_name, state.db)
            .context("No such school adapter")?
            .into();
        *self = Self::SelectedSchool {
            school: school.clone(),
            session: school.new_login_session().await?.into(),
        };
        Ok(())
    }

    pub async fn get_captcha(&self) -> Result<&DynamicImage> {
        let Self::SelectedSchool { session, .. } = self else {
            bail!("Not in SelectedSchool when calling `get_captcha`");
        };

        Ok(session.get_captcha())
    }

    pub async fn login(
        &mut self,
        username: String,
        password: String,
        captcha_answer: String,
    ) -> Result<()> {
        let Self::SelectedSchool { school, session } = self else {
            bail!("Not in SelectedSchool when calling `login`");
        };

        let cred = session.login(username, password, captcha_answer).await?;

        *self = Self::Finished {
            school: school.clone(),
            credentials: cred.into(),
        };

        Ok(())
    }
}
