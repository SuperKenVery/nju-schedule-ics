use sqlx::sqlite::{SqliteConnectOptions,SqlitePool, SqlitePoolOptions};
use std::{str::FromStr, collections::HashMap};
use uuid::Uuid;
use crate::nju::login::{LoginOperation, LoginCredential};

pub struct CookieDb{
    pool: SqlitePool,

    // We only store new login operations.
    // For completed logins, we directly fetch its
    // cookie from sqlite database.
    login_ops: HashMap<String, LoginOperation>,
}

impl CookieDb {
    pub async fn new(path: &str) -> Result<Self,anyhow::Error> {
        let options=SqliteConnectOptions::from_str(path)?
            .create_if_missing(true);

        let pool=SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options).await?;

        sqlx::query("
            CREATE TABLE IF NOT EXISTS castgc
            (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                key TEXT UNIQUE NOT NULL,
                value TEXT NOT NULL
            )
            ")
            .execute(&pool)
            .await?;

        let login_ops=HashMap::new();

        Ok(CookieDb { pool, login_ops })
    }

    async fn insert<K,V>(&mut self, key: K, value: V) -> Result<(),anyhow::Error>
    where
        K: ToString,
        V: ToString,
    {
        let key=key.to_string();
        let value=value.to_string();
        sqlx::query("INSERT INTO castgc (key, value) VALUES (?, ?)")
            .bind(key)
            .bind(value)
            .execute(&self.pool)
            .await?;

        Ok(())

    }

    async fn get<K>(&self, key: K) -> Result<Option<String>,anyhow::Error>
    where
        K: ToString,
    {
        let key=key.to_string();

        let row: Option<(String,)> = sqlx::query_as("SELECT value FROM castgc WHERE key = ?")
            .bind(key)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row)=row {
            Ok(Some(row.0))
        } else {
            Ok(None)
        }
    }

    async fn get_all(&self) -> Result<Vec<(String,String)>,anyhow::Error> {
        let rows: Vec<(String,String)> = sqlx::query_as("SELECT key, value FROM castgc")
            .fetch_all(&self.pool)
            .await?;

        Ok(rows)
    }

    pub async fn new_session(&mut self) -> Result<String,anyhow::Error> {
        let mut session=Uuid::new_v4().to_string();

        while let Ok(Some(_))=self.get(&session).await {
            // UUID collision
            session=Uuid::new_v4().to_string();
        }

        let o=LoginOperation::start().await?;
        self.login_ops.insert(session.clone(), o);

        Ok(session)
    }

    pub async fn get_session_captcha(&self, session: &str) -> Result<Vec<u8>,anyhow::Error> {
        if let LoginOperation::WaitingVerificationCode{
            captcha,client: _,context: _
        } = self.login_ops.get(session).ok_or_else(|| anyhow::anyhow!("No such session"))? {
            Ok(captcha.clone())
        } else {
            Err(anyhow::anyhow!("Session is not waiting for verification code"))
        }
    }

    pub async fn session_login(&mut self, session: &str, username: &str, password: &str, captcha_answer: &str) -> Result<(),anyhow::Error> {
        let o=self.login_ops.get_mut(session).ok_or_else(|| anyhow::anyhow!("No such session"))?;
        let o=o.finish(username,password,captcha_answer).await?;

        if let LoginOperation::Done(cred)=o {
            self.insert(session, cred.castgc).await?;
            self.login_ops.remove(session);
            Ok(())
        } else {
            Err(anyhow::anyhow!("LoginOperation is not Done after finish()"))
        }
    }

    pub async fn get_cred(&self, session: &str) -> Option<LoginCredential> {
        let castgc=self
            .get(session)
            .await
            .ok()??;

        Some(LoginCredential::new(castgc))
    }


}
