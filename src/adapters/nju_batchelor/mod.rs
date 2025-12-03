mod course;
use crate::adapters::traits::CalendarHelper;
use sqlx::SqlitePool;
mod login;
use crate::adapters::traits::School;
use async_trait::async_trait;
use derivative::Derivative;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct NJUBatchelorAdaptor {
    #[derivative(Debug = "ignore")]
    connection: Arc<Mutex<SqlitePool>>,
}

#[async_trait]
impl School for NJUBatchelorAdaptor {
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
        "南京大学本科生"
    }
}

impl CalendarHelper for NJUBatchelorAdaptor {
    fn school_name(&self) -> &str {
        "南京大学"
    }
}
