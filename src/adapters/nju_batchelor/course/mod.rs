mod getcourse;
mod interfaces;
mod location;

use super::NJUBatchelorAdaptor;
use crate::adapters::traits::{Course, CoursesProvider};

impl CoursesProvider for NJUBatchelorAdaptor {
    fn courses(&self, client: &reqwest::Client) -> Vec<Course> {
        todo!()
    }
}
