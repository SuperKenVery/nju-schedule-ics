mod getcourse;
mod interfaces;
mod location;

use super::NJUUndergradAdaptor;
use crate::adapters::course::Course;
use crate::adapters::nju_undergrad::course::getcourse::get_courses;
use crate::adapters::traits::CoursesProvider;
use anyhow::Result;
use async_trait::async_trait;
use reqwest_middleware::ClientWithMiddleware;

#[async_trait]
impl CoursesProvider for NJUUndergradAdaptor {
    async fn courses(&self, client: &ClientWithMiddleware) -> Result<Vec<Course>> {
        get_courses(client).await
    }
}
