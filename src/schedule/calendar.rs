/* Generate iCalendar file (.ics) from Course */
use super::course::Course;
use ics::{Event as oriEvent, ICalendar,
    properties::{Description,Name,Geo,DtStart,DtEnd, Location}
};
use chrono::{DateTime, Utc, FixedOffset, TimeZone, Local};
use uuid::Uuid;
use std::error::Error;
use std::ops::Deref;
use super::location::get_geolocation;

#[derive(Debug)]
struct Event<'a>(oriEvent<'a>);

impl<'a> Event<'a> {
    fn from_course(course: Course, first_week_start: DateTime<Local>) -> Result<Vec<Event<'a>>,Box<dyn Error>> {
        const TIME_FMT: &str = "%Y%m%dT%H%M%S";
        let mut base_event = oriEvent::new(
            Uuid::new_v4().to_string(),
            chrono::Utc::now().format(TIME_FMT).to_string()
        );

        let geo=get_geolocation(&course.location);
        base_event.push(Name::new(course.name));
        base_event.push(Location::new(course.location));
        if let Some(geo)=geo{
            base_event.push(Geo::new(geo.to_ical_str()));
        }


        let mut results=vec![];
        for time in course.time{
            let mut event=base_event.clone();

            let (start,end)=time.to_datetime(first_week_start)?;
            event.push(DtStart::new(start.format(TIME_FMT).to_string()));
            event.push(DtEnd::new(end.format(TIME_FMT).to_string()));

            results.push(Event(event));
        }

        Ok(results)
    }

}

impl<'a> Deref for Event<'a>{
    type Target=oriEvent<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod test{
    use crate::schedule::course::Course;

    use super::*;
    use tokio;
    use super::super::getcourse;
    use super::super::super::login::LoginCredential;
    use std::fs::File;
    use std::io::Write;
    use std::process::Command;
    use std::io::stdin;

    #[test]
    fn test_uid(){
        let uid=Uuid::new_v4().to_string();
        println!("{}", uid);
    }

    async fn get_auth() -> LoginCredential {
        LoginCredential::new("PutYourOwn", "NotGonnaTellYou",
            |content| async move{
            let mut file=File::create("captcha.jpeg").unwrap();
            file.write_all(&content).unwrap();
            Command::new("open").arg("captcha.jpeg").spawn().unwrap();
            let mut input=String::new();
            stdin().read_line(&mut input).unwrap();
            // Remove tailing \n
            input.pop();
            input
        }).await.unwrap()
    }

    #[tokio::test]
    async fn test_to_calendar(){
        let auth=get_auth().await;
        let first_week_start=getcourse::get_first_week_start(&auth).await.unwrap();
        let courses=getcourse::get_course_raw(&auth).await.unwrap();
        let courses=json::parse(&courses).unwrap();
        let courses=&courses["datas"]["cxxszhxqkb"]["rows"];

        let json::JsonValue::Array(courses) = courses else{
            panic!("Not an array??");
        };

        for course in courses{
            let course=Course::from_json(course.clone()).unwrap();
            let events=Event::from_course(course,first_week_start).unwrap();
            for event in events{
                println!("{:#?}", event);
            }
        }
    }
}
