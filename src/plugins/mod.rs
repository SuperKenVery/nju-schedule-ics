use std::sync::Arc;

use crate::{
    adapters::{course::Course, traits::School},
    plugins::holidays::HolidayPlugin,
};
use anyhow::Result;
use async_trait::async_trait;
use tracing::{info_span, instrument};

pub mod holidays;

#[async_trait]
pub trait PlugIn: Sync + Send {
    /// After school adapter has got all the courses, before creating the calendar file.

    async fn pre_generate_calendar<'a, 'b, 'c>(
        &self,
        _school: &'a dyn School,
        courses: Vec<Course>,
    ) -> Vec<Course>
    where
        'b: 'c,
    {
        courses
    }
}

#[async_trait]
impl PlugIn for Vec<Arc<dyn PlugIn>> {
    async fn pre_generate_calendar<'a, 'b, 'c>(
        &self,
        school: &'a dyn School,
        courses: Vec<Course>,
    ) -> Vec<Course>
    where
        'b: 'c,
    {
        let mut result = courses;
        for plugin in self {
            result = plugin.pre_generate_calendar(school, result).await;
        }
        result
    }
}

pub async fn get_plugins() -> Result<Vec<Arc<dyn PlugIn>>> {
    Ok(vec![Arc::new(HolidayPlugin::new().await?)])
}
