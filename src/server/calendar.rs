use super::state::ServerState;
use crate::adapters::course::Course;
use crate::adapters::traits::{Login, School};
use crate::plugins::PlugIn;
use anyhow::Context;
use anyhow::Result;
use axum::extract::State;
use axum::http::{HeaderMap, HeaderValue, header};
use axum::response::IntoResponse;
use dioxus::fullstack::FromResponse;
use dioxus::prelude::*;
use ics::{ICalendar, Standard, TimeZone};
use std::sync::Arc;
use tracing::info;

struct CalendarRet(HeaderMap, Vec<u8>);

#[get("/calendar/{school_adapter}/{key}/schedule.ics", state: State<ServerState>)]
pub async fn get_calendar_file(school_adapter: String, key: String) -> Result<CalendarRet> {
    let school: Arc<dyn School> = state
        .school_adapters
        .lock()
        .await
        .get(&school_adapter.as_str())
        .context("No such school adapter")?
        .clone();
    let cred = school
        .get_cred_from_db(key.as_str())
        .await
        .context("No such key. URL might be wrong.")?;
    let client = school.create_authenticated_client(cred).await?;
    let courses = school.courses(&client).await?;

    let courses = state.plugins.pre_generate_calendar(&*school, courses);

    let calendar = calendar_from_courses(&*school, &courses)?;
    let mut calendar_bytes_buf = vec![];
    let writer = std::io::Cursor::new(&mut calendar_bytes_buf);
    calendar.write(writer)?;

    info!("Done generating calendar file");
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str("text/calendar")?,
    );

    Ok(CalendarRet(headers, calendar_bytes_buf))
}

impl FromResponse for CalendarRet {
    async fn from_response(
        res: dioxus_fullstack::ClientResponse,
    ) -> std::result::Result<Self, ServerFnError> {
        panic!("This is for calendar clients, never use it as server function.")
    }
}

impl IntoResponse for CalendarRet {
    fn into_response(self) -> axum::response::Response {
        (self.0, self.1).into_response()
    }
}

fn calendar_from_courses<'a, 'b: 'a>(
    school: &'b dyn School,
    courses: &[Course],
) -> Result<ICalendar<'a>> {
    let mut calendar = ICalendar::new("2.0", "南哪另一课表");

    let tzid = "schedule_tz";
    let tz = TimeZone::standard(tzid, Standard::new("19710101T000000", "+0000", "+0000"));
    calendar.add_timezone(tz);

    for course in courses {
        for event in course.to_events(school, tzid)? {
            calendar.add_event(event);
        }
    }

    Ok(calendar)
}
