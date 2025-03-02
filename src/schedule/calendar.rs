//! Generate iCalendar file (.ics) from Course

use super::location::get_geolocation;
use super::{course::Course, holidays::HolidayCal};
use crate::nju::getcourse;
use crate::nju::login::LoginCredential;
use anyhow::anyhow;
use chrono::NaiveDate;
use ics::{
    components::{Parameter, Property},
    parameters::TzIDParam,
    properties::{Description, DtEnd, DtStart, Geo, Location, Summary},
    Event as oriEvent, ICalendar, Standard, TimeZone,
};
use std::ops::Deref;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Event<'a>(oriEvent<'a>);

impl<'a> Event<'a> {
    /// Convert our course into a list of iCalendar events
    /// It's a list because Course has not-simple recurrent rules
    fn from_course(
        course: &Course,
        first_week_start: NaiveDate,
        tzid: &'a str,
    ) -> Result<Vec<Event<'a>>, anyhow::Error> {
        const TIME_FMT: &str = "%Y%m%dT%H%M%S";

        let mut results = vec![];
        for time in &course.time {
            let mut event = oriEvent::new(
                Uuid::new_v4().to_string(),
                chrono::Utc::now().format(TIME_FMT).to_string(),
            );

            event.push(Summary::new(course.name.clone()));

            let geo = get_geolocation(&course.location);
            event.push(Location::new(format!("{}\\n南京大学", course.location)));
            if let Some(geo) = geo {
                event.push(Geo::new(geo.to_ical_str()));
                let mut apple_addr =
                    Property::new("X-APPLE-STRUCTURED-LOCATION", geo.to_apple_location_str());
                apple_addr.add(Parameter::new("X-ADDRESS", "南京大学"));
                apple_addr.add(Parameter::new("X-TITLE", course.location.clone()));
                event.push(apple_addr);
            }
            let (start, end) = time.to_datetime(first_week_start)?;
            let tz = TzIDParam::new(tzid);
            let mut start = DtStart::new(start.format(TIME_FMT).to_string());
            start.add(tz.clone());
            let mut end = DtEnd::new(end.format(TIME_FMT).to_string());
            end.add(tz.clone());
            event.push(start);
            event.push(end);

            if course.notes.is_empty() == false {
                event.push(Description::new(course.notes.clone().replace("\n", "\\n")));
            }

            results.push(Event(event));
        }

        Ok(results)
    }
}

impl<'a> Deref for Event<'a> {
    type Target = oriEvent<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// The final calendar that would be served to the clients
#[derive(Debug)]
pub struct Calendar<'a>(ICalendar<'a>);

impl<'a> Calendar<'a> {
    pub fn with_events(events: Vec<Event<'a>>) -> Self {
        let mut cal = ICalendar::new("2.0", "南哪另一课表");
        for event in events {
            cal.add_event(event.0);
        }
        Self(cal)
    }

    pub async fn from_login(
        cred: LoginCredential,
        hcal: &HolidayCal,
    ) -> Result<Calendar<'a>, anyhow::Error> {
        let first_week_start = getcourse::get_first_week_start(&cred).await?;
        let courses = crate::nju::getcourse::get_course_raw(&cred).await?;
        let courses = crate::schedule::course::Course::batch_from_json(
            json::parse(&courses)?,
            hcal,
            first_week_start,
        )?;

        let tzid = "NJU";
        let tz = TimeZone::standard(tzid, Standard::new("19710101T000000", "+0800", "+0800"));

        let events = courses
            .iter()
            .map(|c| Event::from_course(c, first_week_start, tzid))
            .collect::<Result<Vec<Vec<Event>>, anyhow::Error>>()?
            .concat();

        let mut calendar = Self::with_events(events);
        calendar.0.add_timezone(tz);

        Ok(calendar)
    }

    pub async fn from_test() -> Result<Calendar<'a>, anyhow::Error> {
        let first_week_start = NaiveDate::from_ymd_opt(2024, 9, 2)
            .ok_or(anyhow!("Failed to construct first_week_start NaiveDate"))?;

        let hcal = HolidayCal::from_shuyz().await?;
        let courses = std::fs::read_to_string("src/nju/example.json")?;
        let courses = crate::schedule::course::Course::batch_from_json(
            json::parse(&courses)?,
            &hcal,
            first_week_start,
        )?;

        let tzid = "NJU";
        let tz = TimeZone::standard(tzid, Standard::new("19710101T000000", "+0800", "+0800"));

        let events = courses
            .iter()
            .map(|c| Event::from_course(c, first_week_start, tzid))
            .collect::<Result<Vec<Vec<Event>>, anyhow::Error>>()?
            .concat();

        let mut calendar = Self::with_events(events);
        calendar.0.add_timezone(tz);

        Ok(calendar)
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, anyhow::Error> {
        let cal = &self.0;
        let mut buf = vec![];
        let writer = std::io::Cursor::new(&mut buf);
        cal.write(writer)?;

        Ok(buf)
    }
}

impl<'a> Deref for Calendar<'a> {
    type Target = ICalendar<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod test {
    use crate::schedule::course::Course;
    use crate::schedule::holidays::HolidayCal;

    use super::*;
    use crate::nju::getcourse;
    use crate::nju::login::LoginCredential;
    use std::fs::File;
    use std::io::stdin;
    use std::io::Write;
    use std::process::Command;
    use tokio;

    async fn get_auth() -> LoginCredential {
        LoginCredential::from_login("PutYourOwn", "NotGonnaTellYou", |content| async move {
            let mut file = File::create("captcha.jpeg").unwrap();
            file.write_all(&content).unwrap();
            Command::new("open").arg("captcha.jpeg").spawn().unwrap();
            let mut input = String::new();
            stdin().read_line(&mut input).unwrap();
            // Remove tailing \n
            input.pop();
            input
        })
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn test_to_calendar() {
        let hcal = HolidayCal::from_shuyz().await.unwrap();
        let auth = get_auth().await;
        let first_week_start = getcourse::get_first_week_start(&auth).await.unwrap();
        let courses = getcourse::get_course_raw(&auth).await.unwrap();
        let courses = json::parse(&courses).unwrap();
        let courses = &courses["datas"]["cxxszhxqkb"]["rows"];

        let json::JsonValue::Array(courses) = courses else {
            panic!("Not an array??");
        };

        for course in courses {
            let course = Course::from_json(course.clone(), &hcal, first_week_start).unwrap();
            let events = Event::from_course(&course, first_week_start, "NJU").unwrap();
            for event in events {
                println!("{:#?}", event);
            }
        }
    }
}
