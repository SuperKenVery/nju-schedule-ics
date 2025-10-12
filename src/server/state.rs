use crate::adapters::all_school_adapters::school_adapter_from_name;
use crate::adapters::traits::{Credentials, LoginSession, School};
use crate::server::config::Config;
use anyhow::{Context, Result, anyhow, bail, ensure};
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

/// An unfinished login session.
///
/// 1. This is created when the user accesses the website.
/// 2. Then, when user selects a school this becomes [`Self::SelectedSchool`].
///    At this point, we request a login session from the school adapter.
/// 3. Then, when the user enters username, password and captcha answer,
///    we request login and put the credentials in [`Self::Finished`].
///    - We also call [`LoginSession.save_cred_to_db`] to persistent this login.
///    - We also save the correspondance between the db key and school adapter api. (TODO)
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
        cred_db_key: String,
    },
}

#[cfg(feature = "server")]
impl UnfinishedLoginSession {
    /// Set the school adapter for this session
    pub async fn select_school(&mut self, school_name: String) -> Result<()> {
        use anyhow::Context;

        ensure!(matches!(self, Self::Started), "Not in started");
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

    /// Get the captcha image content
    pub async fn get_captcha(&self) -> Result<&DynamicImage> {
        let Self::SelectedSchool { session, .. } = self else {
            bail!("Not in SelectedSchool when calling `get_captcha`");
        };

        Ok(session.get_captcha())
    }

    /// Login with username, password and captcha answer
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
        let cred_db_key = session.save_cred_to_db(cred.clone()).await?;
        todo!("save the correspondance between the db key and school adapter api");

        *self = Self::Finished {
            school: school.clone(),
            credentials: cred.into(),
            cred_db_key,
        };

        Ok(())
    }
}
