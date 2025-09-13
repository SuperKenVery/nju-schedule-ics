mod course;
mod db_schema;
mod login;

use diesel::SqliteConnection;
use std::sync::{Arc, Mutex};

use crate::adapters::traits::School;

pub struct NJUBatchelorAdaptor {
    connection: Arc<Mutex<SqliteConnection>>,
}

impl School for NJUBatchelorAdaptor {
    fn new(db: Arc<Mutex<SqliteConnection>>) -> Self
    where
        Self: Sized,
    {
        Self { connection: db }
    }
}
