use crate::nju::login::{LoginCredential, LoginOperation};
use log::error;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::sync::Arc;
use std::{collections::HashMap, str::FromStr};
use time::{Duration, OffsetDateTime};
use tokio::sync::Mutex;
use uuid::Uuid;

pub struct CookieDb {
    pool: SqlitePool,

    // We only store new login operations.
    // For completed logins, we directly fetch its
    // cookie from sqlite database.
    login_ops: HashMap<String, (OffsetDateTime, LoginOperation)>,
}

impl CookieDb {
    pub async fn new(path: &str) -> Result<Self, anyhow::Error> {
        let options = SqliteConnectOptions::from_str(path)?.create_if_missing(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;

        sqlx::query(
            "
            CREATE TABLE IF NOT EXISTS castgc
            (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                key TEXT UNIQUE NOT NULL,
                value TEXT NOT NULL,
                last_access DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            ",
        )
        .execute(&pool)
        .await?;

        let login_ops = HashMap::new();

        Ok(CookieDb { pool, login_ops })
    }

    async fn insert<K, V>(&mut self, key: K, value: V) -> Result<(), anyhow::Error>
    where
        K: ToString,
        V: ToString,
    {
        let key = key.to_string();
        let value = value.to_string();
        sqlx::query("INSERT INTO castgc (key, value) VALUES (?, ?)")
            .bind(key)
            .bind(value)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn get<K>(&self, key: K) -> Result<Option<String>, anyhow::Error>
    where
        K: ToString,
    {
        let key = key.to_string();

        let row: Option<(String,)> = sqlx::query_as("SELECT value FROM castgc WHERE key = ?")
            .bind(key.clone())
            .fetch_optional(&self.pool)
            .await?;

        let _updated =
            sqlx::query("UPDATE castgc SET last_access = CURRENT_TIMESTAMP WHERE key = ?")
                .bind(key)
                .execute(&self.pool)
                .await?;

        if let Some(row) = row {
            Ok(Some(row.0))
        } else {
            Ok(None)
        }
    }

    async fn get_all(&self) -> Result<Vec<(String, String, OffsetDateTime)>, anyhow::Error> {
        // Used to clean up unused cookies, so don't update last_access here

        let rows: Vec<(String, String, OffsetDateTime)> =
            sqlx::query_as("SELECT key, value, last_access FROM castgc")
                .fetch_all(&self.pool)
                .await?;

        Ok(rows)
    }

    pub async fn new_session(&mut self) -> Result<String, anyhow::Error> {
        let mut session = Uuid::new_v4().to_string();

        while let Ok(Some(_)) = self.get(&session).await {
            // UUID collision
            session = Uuid::new_v4().to_string();
        }

        let o = LoginOperation::start().await?;
        self.login_ops
            .insert(session.clone(), (OffsetDateTime::now_utc(), o));

        Ok(session)
    }

    pub async fn get_session_captcha(&self, session: &str) -> Result<Vec<u8>, anyhow::Error> {
        if let (
            _last_access,
            LoginOperation::WaitingVerificationCode {
                captcha,
                client: _,
                context: _,
            },
        ) = self
            .login_ops
            .get(session)
            .ok_or_else(|| anyhow::anyhow!("No such session"))?
        {
            Ok(captcha.clone())
        } else {
            Err(anyhow::anyhow!(
                "Session is not waiting for verification code"
            ))
        }
    }

    pub async fn session_login(
        &mut self,
        session: &str,
        username: &str,
        password: &str,
        captcha_answer: &str,
    ) -> Result<(), anyhow::Error> {
        let o = &self
            .login_ops
            .get(session)
            .ok_or_else(|| anyhow::anyhow!("No such session"))?
            .1;
        let o = o.finish(username, password, captcha_answer).await?;

        if let LoginOperation::Done(cred) = o {
            self.insert(session, cred.castgc).await?;
            self.login_ops.remove(session);
            Ok(())
        } else {
            Err(anyhow::anyhow!("LoginOperation is not Done after finish()"))
        }
    }

    pub async fn get_cred(&self, session: &str) -> Option<LoginCredential> {
        let castgc = self.get(session).await.ok()??;

        Some(LoginCredential::new(castgc))
    }

    // Clean up
    async fn cleanup_login_op(&mut self) -> Result<(), anyhow::Error> {
        // Clean up login operations older than 5 minutes
        let now = OffsetDateTime::now_utc();
        let mut to_remove = Vec::new();

        for (session, (last_access, _)) in &self.login_ops {
            if now - *last_access > Duration::minutes(5) {
                to_remove.push(session.clone());
            }
        }

        for session in to_remove {
            self.login_ops.remove(&session);
        }

        Ok(())
    }

    pub async fn remove_uuid(&self, uuid: &str) -> Result<(), anyhow::Error> {
        sqlx::query("DELETE FROM castgc WHERE key = ?")
            .bind(uuid)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn cleanup_cookie_db(&self) -> Result<(), anyhow::Error> {
        // Clean up cookies older than 1 year
        let now = OffsetDateTime::now_utc();
        let year = Duration::days(365);
        let rows = self.get_all().await?;
        for (key, _, last_access) in rows {
            if now - last_access > year {
                self.remove_uuid(&key).await?;
            }
        }

        Ok(())
    }

    pub fn start_cleanup_thread(db: Arc<Mutex<Self>>) {
        let sa = db.clone();
        tokio::spawn(async move {
            loop {
                // lock() returns err when another holder panicked
                // So just panic if lock() returns err
                let err = sa.lock().await.cleanup_login_op().await;
                if let Err(err) = err {
                    error!("Error in cleanup_login_op: {:?}", err);
                }
                tokio::time::sleep(std::time::Duration::from_secs(30 * 60)).await;
                // clean every 30min
            }
        });

        let sb = db.clone();
        tokio::spawn(async move {
            loop {
                let err = sb.lock().await.cleanup_cookie_db().await;
                if let Err(err) = err {
                    error!("Error in cleanup_cookie_db: {:?}", err);
                }
                tokio::time::sleep(std::time::Duration::from_secs(10 * 24 * 60 * 60)).await;
                // clean every 10days
            }
        });
    }
}
