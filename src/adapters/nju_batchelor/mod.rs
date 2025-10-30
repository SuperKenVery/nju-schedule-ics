mod course;
use crate::{adapters::traits::CalendarHelper, schema as db_schema};
mod login;
use crate::adapters::traits::School;
use anyhow::Result;
use async_trait::async_trait;
use derivative::Derivative;
use diesel::SqliteConnection;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use ics::{Standard, TimeZone};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/");

#[derive(Derivative)]
#[derivative(Debug)]
pub struct NJUBatchelorAdaptor {
    #[derivative(Debug = "ignore")]
    connection: Arc<Mutex<SqliteConnection>>,
}

#[async_trait]
impl School for NJUBatchelorAdaptor {
    async fn new(db: Arc<Mutex<SqliteConnection>>) -> Self
    where
        Self: Sized,
    {
        {
            let mut conn = db.lock().await;
            run_migrations(&mut *conn);
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

fn run_migrations(connection: &mut impl MigrationHarness<diesel::sqlite::Sqlite>) {
    connection
        .run_pending_migrations(MIGRATIONS)
        .expect("Failed to run database migrations");
}
