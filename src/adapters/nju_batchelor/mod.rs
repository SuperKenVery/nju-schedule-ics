mod course;
mod db_schema;
mod login;

use crate::adapters::traits::School;
use derivative::Derivative;
use diesel::SqliteConnection;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct NJUBatchelorAdaptor {
    #[derivative(Debug = "ignore")]
    connection: Arc<Mutex<SqliteConnection>>,
}

impl School for NJUBatchelorAdaptor {
    fn new(db: Arc<Mutex<SqliteConnection>>) -> Self
    where
        Self: Sized,
    {
        Self { connection: db }
    }

    fn name(&self) -> &str {
        "南京大学本科生"
    }
}
