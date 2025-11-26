//! 调休插件

use std::collections::{HashMap, HashSet};

use crate::adapters::{course::Course, traits::School};
use crate::plugins::PlugIn;
use anyhow::Result;
use chrono::{DateTime, NaiveDate, Utc};
use serde::Deserialize;

#[derive(Debug)]
pub struct HolidayPlugin {
    holidays: HashSet<NaiveDate>,
    compensate_days: HashSet<NaiveDate>,
}

impl HolidayPlugin {
    pub async fn new() -> Result<Self> {
        let holidays = reqwest::get(
            "https://www.shuyz.com/githubfiles/china-holiday-calender/master/holidayAPI.json",
        )
        .await?
        .json::<ShuyzHolidayResponse>()
        .await?;

        let mut holiday_dates = HashSet::new();
        let mut compensate_dates = HashSet::new();

        for (_, year_holidays) in holidays.Years {
            for holiday in year_holidays {
                // Parse holiday dates
                let start_date = NaiveDate::parse_from_str(&holiday.StartDate, "%Y-%m-%d")?;
                let end_date = NaiveDate::parse_from_str(&holiday.EndDate, "%Y-%m-%d")?;

                // Add all dates in the holiday range
                let mut current_date = start_date;
                while current_date <= end_date {
                    holiday_dates.insert(current_date);
                    current_date = current_date
                        .succ_opt()
                        .expect("Holiday is beyond the last representable day");
                }

                // Parse compensate days
                for comp_day in &holiday.CompDays {
                    let comp_date = NaiveDate::parse_from_str(comp_day, "%Y-%m-%d")?;
                    compensate_dates.insert(comp_date);
                }
            }
        }

        Ok(HolidayPlugin {
            holidays: holiday_dates,
            compensate_days: compensate_dates,
        })
    }

    /// Check if a DateTime<Utc> falls on a holiday in UTC+8 timezone
    pub fn is_in_holiday(&self, datetime: &DateTime<Utc>) -> bool {
        // Convert UTC to UTC+8
        let offset = chrono::FixedOffset::east_opt(8 * 3600).expect("8 hours offset out of bound");
        let utc_plus_8 = datetime.with_timezone(&offset);
        let naive_date = utc_plus_8.date_naive();

        // Check if the date is in holidays
        self.holidays.contains(&naive_date)
    }
}

impl PlugIn for HolidayPlugin {
    fn pre_generate_calendar<'a, 'b, 'c>(
        &self,
        _school: &'a dyn School,
        courses: Vec<Course>,
    ) -> Vec<Course>
    where
        'b: 'c,
    {
        // Filter out courses that fall on holidays/
        courses
            .into_iter()
            .map(|mut course| {
                // Filter out time slots that fall on holidays
                let filtered_times: Vec<(DateTime<Utc>, DateTime<Utc>)> = course
                    .time
                    .into_iter()
                    .filter(|(start_time, _)| !self.is_in_holiday(start_time))
                    .collect();

                // Update the course with filtered times
                course.time = filtered_times;
                course
            })
            .collect()
    }
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
struct ShuyzHolidayResponse {
    Name: String,
    Timezone: String,
    Years: HashMap<i32, Vec<ShuyzHoliday>>,
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
struct ShuyzHoliday {
    Name: String,
    StartDate: String,
    EndDate: String,
    CompDays: Vec<String>,
}
