//! Defines the [`Course`] struct and how it converts to an iCalendar file.

use anyhow::Result;
use chrono::{DateTime, Utc};
use ics::{
    Event,
    components::{Parameter, Property},
    parameters::TzIDParam,
    properties::{Description, DtEnd, DtStart, Geo, Location, Summary},
};
use uuid::Uuid;

use crate::adapters::traits::School;

/// A course
#[derive(Debug, Clone)]
pub struct Course {
    /// Course name
    pub name: String,
    /// All times of course, including each one across the semester.
    /// Format is `Vec<(start_time, end_time)>`.
    pub time: Vec<(DateTime<Utc>, DateTime<Utc>)>,
    /// The location of this course.
    pub location: Option<String>,
    /// The latitide and longtitude of the course location.
    pub geo: Option<GeoLocation>,
    /// The campus of this course
    pub campus: Option<String>,
    /// Additional notes.
    ///
    /// This would be in the notes area of calendar event, and you can
    /// include anything like notes, teacher, notice or whatsoever.
    ///
    /// When displayed, the vec of string will be concatenated with
    /// new lines (each one in its own line)
    pub notes: Vec<String>,
}

/// Location of a course
#[derive(Debug, Clone, Copy)]
pub struct GeoLocation {
    latitude: f64,
    longitude: f64,
}

impl GeoLocation {
    pub fn new(latitude: f64, longitude: f64) -> Self {
        Self {
            latitude,
            longitude,
        }
    }

    pub fn to_ical_str(&self) -> String {
        format!("{};{}", self.latitude, self.longitude)
    }

    pub fn to_apple_location_str(&self) -> String {
        format!("geo:{},{}", self.latitude, self.longitude)
    }
}

const TIME_FMT: &str = "%Y%m%dT%H%M%S";
impl Course {
    pub fn to_events<'a>(&self, school: &dyn School, tzid: &str) -> Result<Vec<Event<'a>>> {
        Ok(self
            .time
            .iter()
            .map(|time| {
                let mut event = Event::new(
                    Uuid::new_v4().to_string(),
                    chrono::Utc::now().format(TIME_FMT).to_string(),
                );

                // Name
                event.push(Summary::new(self.name.clone()));

                // Location
                if let Some(location) = self.location.clone() {
                    event.push(Location::new(format!(
                        "{}\\n{}",
                        location,
                        school.school_name()
                    )));
                    if let Some(geo) = self.geo {
                        event.push(Geo::new(geo.to_ical_str()));
                        let mut apple_addr = Property::new(
                            "X-APPLE-STRUCTURED-LOCATION",
                            geo.to_apple_location_str(),
                        );
                        apple_addr.add(Parameter::new(
                            "X-ADDRESS",
                            school.school_name().to_string(),
                        ));
                        apple_addr.add(Parameter::new("X-TITLE", location.clone()));
                        event.push(apple_addr);
                    }
                }

                // Notes
                let mut notes = "".to_string();
                if let Some(campus) = &self.campus {
                    notes += format!("{}\n", campus).as_str();
                }
                for note in &self.notes {
                    notes += format!("{}\n", note).as_str();
                }
                event.push(Description::new(notes.replace("\n", "\\n")));

                let timezone = TzIDParam::new(tzid.to_string());

                let mut start = DtStart::new(time.0.format(TIME_FMT).to_string());
                start.add(timezone.clone());
                event.push(start);

                let mut end = DtEnd::new(time.1.format(TIME_FMT).to_string());
                end.add(timezone.clone());
                event.push(end);

                event
            })
            .collect())
    }
}
