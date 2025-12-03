use std::{fmt::Debug, sync::Arc};

use crate::{
    adapters::{course::Course, traits::School},
    plugins::holidays::HolidayPlugin,
};
use anyhow::Result;

pub mod holidays;

pub trait PlugIn: Sync + Send + Debug {
    /// After school adapter has got all the courses, before creating the calendar file.
    fn pre_generate_calendar<'a, 'b, 'c>(
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

impl PlugIn for Vec<Arc<dyn PlugIn>> {
    fn pre_generate_calendar<'a, 'b, 'c>(
        &self,
        school: &'a dyn School,
        courses: Vec<Course>,
    ) -> Vec<Course>
    where
        'b: 'c,
    {
        let mut result = courses;
        for plugin in self {
            result = plugin.pre_generate_calendar(school, result);
        }
        result
    }
}

pub async fn get_plugins() -> Result<Vec<Arc<dyn PlugIn>>> {
    Ok(vec![Arc::new(HolidayPlugin::new().await?)])
}
