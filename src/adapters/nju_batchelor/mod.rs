mod login;

use diesel::{Connection, SqliteConnection};
use std::sync::Mutex;

use crate::adapters::traits::School;

pub struct NJUBatchelorAdaptor {
    connection: Mutex<SqliteConnection>,
}

impl School for NJUBatchelorAdaptor {
    fn new(db: SqliteConnection) -> Self
    where
        Self: Sized,
    {
        Self {
            connection: Mutex::new(db),
        }
    }
}
