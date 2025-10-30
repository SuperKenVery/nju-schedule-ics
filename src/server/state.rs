use crate::adapters::nju_batchelor::NJUBatchelorAdaptor;
use crate::adapters::traits::{Credentials, LoginSession, School};
use crate::server::config::Config;
use anyhow::{Context, Result, anyhow, bail, ensure};
use derivative::Derivative;
use diesel::prelude::*;
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
    pub school_adapters: Arc<Mutex<HashMap<&'static str, Arc<dyn School>>>>,
}

impl ServerState {
    pub async fn from_config(cfg: Config) -> Result<Self> {
        let db = Arc::new(Mutex::new(SqliteConnection::establish(&cfg.db_path)?));

        let mut school_adapters = HashMap::<&'static str, Arc<dyn School>>::new();
        school_adapters.insert(
            "南京大学本科生",
            Arc::new(NJUBatchelorAdaptor::new(db.clone()).await),
        );

        Ok(Self {
            db,
            site_url: cfg.site_url,
            unfinished_login_sessions: Arc::new(Mutex::new(HashMap::new())),
            school_adapters: Arc::new(Mutex::new(school_adapters)),
        })
    }
}

#[derive(Insertable, Clone)]
#[diesel(table_name = crate::schema::key_to_school)]
pub struct KeyToSchool {
    /// The key used for getting your schedule
    key: String,
    /// The name of school adapter api
    school: String,
}

#[derive(Queryable)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
struct RowKeyToSchool {
    id: Option<i32>,
    /// The key used for getting your schedule
    key: String,
    /// The name of school adapter api
    school: String,
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

        let school: Arc<dyn School> = state
            .school_adapters
            .lock()
            .await
            .get(&school_name.as_str())
            .context("No such school adapter")?
            .clone();
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
        db: Arc<Mutex<SqliteConnection>>,
    ) -> Result<()> {
        let Self::SelectedSchool { school, session } = self else {
            bail!("Not in SelectedSchool when calling `login`");
        };

        let cred = session.login(username, password, captcha_answer).await?;
        let cred_db_key = session.save_cred_to_db(cred.clone()).await?;

        // let correspondance = KeyToSchool {
        //     key: cred_db_key.clone(),
        //     school: school.name().to_string(),
        // };
        // let _inserted = diesel::insert_into(crate::schema::key_to_school::dsl::key_to_school)
        //     .values(correspondance)
        //     .execute(&mut *(db.lock().await))?;
        //
        // Put school api name in URL.

        *self = Self::Finished {
            school: school.clone(),
            credentials: cred.into(),
            cred_db_key,
        };

        Ok(())
    }
}
