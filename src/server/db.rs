// Use redis with its persistence feature as the db

// use redis::{Client,AsyncCommands};
use sqlx::sqlite::{SqliteConnectOptions,SqlitePool, SqlitePoolOptions};
use std::str::FromStr;

pub struct CookieDb{
    // connection: redis::aio::Connection,
    pool: SqlitePool,
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

        Ok(CookieDb { pool })
    }

    pub async fn insert<K,V>(&mut self, key: K, value: V) -> Result<(),anyhow::Error>
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

    pub async fn get<K>(&mut self, key: K) -> Result<Option<String>,anyhow::Error>
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
            Err(anyhow::Error::msg("Not found"))
        }
    }

    pub async fn get_all(&self) -> Result<Vec<(String,String)>,anyhow::Error> {
        let rows: Vec<(String,String)> = sqlx::query_as("SELECT key, value FROM castgc")
            .fetch_all(&self.pool)
            .await?;

        Ok(rows)
    }


}
