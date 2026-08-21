use super::NJUUndergradAdaptor;

use crate::adapters::traits::{Credentials, Login, LoginSession};
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use derivative::Derivative;
use image::DynamicImage;
use reqwest::{Url, cookie::Jar};
use reqwest_middleware::ClientWithMiddleware;
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};
use reqwest_tracing::TracingMiddleware;
use sqlx::SqlitePool;
use sqlx::prelude::FromRow;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

#[async_trait]
impl Login for NJUUndergradAdaptor {
    async fn new_login_session(&self) -> Result<Box<dyn LoginSession>> {
        Ok(Box::new(Session::new(self.connection.clone())))
    }

    async fn get_cred_from_db(&self, session_id: &str) -> Option<Box<dyn Credentials>> {
        let connection = self.connection.lock().await;

        let mut cred = sqlx::query_as::<_, LoginCredential>("SELECT * FROM castgc WHERE key = ?")
            .bind(session_id)
            .fetch_one(&*connection)
            .await
            .ok()?;
        cred.last_access = chrono::Local::now().naive_local();

        Some(Box::new(cred))
    }

    async fn create_authenticated_client(
        &self,
        credentials: Box<dyn Credentials>,
    ) -> Result<ClientWithMiddleware> {
        let jar = Arc::new(Jar::default());

        let client = reqwest_middleware::ClientBuilder::new(
            reqwest::ClientBuilder::new()
                .cookie_provider(jar.clone())
                .user_agent("nju-schedule-ics")
                .timeout(std::time::Duration::from_secs(10))
                .build()?,
        )
        .with(RetryTransientMiddleware::new_with_policy(
            ExponentialBackoff::builder().build_with_max_retries(3),
        ))
        .with(TracingMiddleware::default())
        .build();

        let credentials: Box<LoginCredential> = credentials
            .downcast()
            .map_err(|_| anyhow!("Invalid login credentials (failed to downcast)"))?;
        jar.add_cookie_str(
            format!("CASTGC={}", credentials.value).as_str(),
            &Url::parse("https://authserver.nju.edu.cn").unwrap(),
        );

        let _ = client
            .get("https://ehall.nju.edu.cn/appShow?appId=4770397878132218")
            .send()
            .await?
            .text()
            .await?;

        Ok(client)
    }
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Session {
    #[derivative(Debug = "ignore")]
    db: Arc<Mutex<SqlitePool>>,
    id: String,
}

#[async_trait]
impl LoginSession for Session {
    fn get_captcha(&self) -> Option<&DynamicImage> {
        None
    }

    async fn login(
        &self,
        username: String,
        password: String,
        _captcha_answer: Option<String>,
    ) -> Result<Box<dyn Credentials>> {
        let castgc = nju_unified_auth::login(username, password).await?;

        Ok(Box::new(LoginCredential {
            key: self.id.clone(),
            value: castgc,
            last_access: chrono::Local::now().naive_local(),
        }))
    }

    fn session_id(&self) -> &str {
        &self.id
    }

    async fn save_cred_to_db(&self, cred: Box<dyn Credentials>) -> Result<String> {
        let cred: Box<LoginCredential> = cred
            .downcast()
            .map_err(|_| anyhow!("Got invalid credential when saving to db, downcasting failed"))?;
        let db_key = cred.key.clone();

        let connection = self.db.lock().await;
        sqlx::query("INSERT INTO castgc (key, value, last_access) VALUES ($1, $2, $3)")
            .bind(&db_key)
            .bind(cred.value)
            .bind(cred.last_access)
            .execute(&*connection)
            .await?;

        Ok(db_key)
    }
}

impl Session {
    pub fn new(db: Arc<Mutex<SqlitePool>>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            db,
        }
    }
}

#[derive(FromRow, Clone)]
pub struct LoginCredential {
    /// The session ID
    pub key: String,
    /// The CASTGC cookie
    pub value: String,
    /// Time last accessed
    pub last_access: chrono::NaiveDateTime,
}
