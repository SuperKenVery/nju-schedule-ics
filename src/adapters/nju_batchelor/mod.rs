mod course;
use crate::adapters::traits::CalendarHelper;
use sqlx::SqlitePool;
mod login;
use crate::adapters::traits::School;
use anyhow::Result;
use async_trait::async_trait;
use derivative::Derivative;
use ics::{Standard, TimeZone};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

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
