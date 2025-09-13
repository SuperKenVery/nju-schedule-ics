mod course;
mod db_schema;
mod login;

use diesel::{Connection, SqliteConnection};
use std::sync::{Arc, Mutex};

use crate::adapters::traits::School;

pub struct NJUBatchelorAdaptor {
    connection: Arc<Mutex<SqliteConnection>>,
}

impl School for NJUBatchelorAdaptor {
    fn new(db: SqliteConnection) -> Self
    where
        Self: Sized,
    {
        Self {
            connection: Arc::new(Mutex::new(db)),
        }
    }
}
