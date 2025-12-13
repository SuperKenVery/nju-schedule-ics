//! 南京大学研究生 适配模块

use std::sync::Arc;

use async_trait::async_trait;
use derivative::Derivative;
use sqlx::SqlitePool;
use tokio::sync::Mutex;

use crate::adapters::traits::{CalendarHelper, School};
mod course;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct NJUGraduateAdapter {
    #[derivative(Debug = "ignore")]
    connection: Arc<Mutex<SqlitePool>>,
}

#[async_trait]
impl School for NJUGraduateAdapter {
    async fn new(db: Arc<Mutex<SqlitePool>>) -> Self
    where
        Self: Sized,
    {
        {
            let database = db.lock().await;
            let _res = sqlx::query(
                "CREATE TABLE IF NOT EXISTS castgc (
                    key TEXT PRIMARY KEY,
                    value TEXT NOT NULL,
                    last_access TEXT NOT NULL
                )",
            )
            .execute(&*database)
            .await
            .expect("Failed to ensure table exists");
        }
        Self { connection: db }
    }

    fn adapter_name(&self) -> &str {
        "南京大学研究生"
    }
}

impl CalendarHelper for NJUGraduateAdapter {
    fn school_name(&self) -> &str {
        "南京大学"
    }
}
