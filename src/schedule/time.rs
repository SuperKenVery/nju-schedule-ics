/* Some utilities to deal with time
 * Time: packing hour and minute
 * TimeSpan: packing start and end time
 * CourseTime: packing time span, weekday and week
 */
use chrono::{DateTime, Duration, Local, LocalResult};

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct Time {
    hour: u8,
    minute: u8,
}

impl Time {
    pub fn new(hour: u8, minute: u8) -> Self {
        Self { hour, minute }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct TimeSpan {
    pub start: Time,
    pub end: Time,
}

impl TimeSpan {
    pub fn new(start: Time, end: Time) -> Self {
        Self { start, end }
    }

    pub fn from_course_index(idx: u8) -> Result<TimeSpan, anyhow::Error> {
        match idx {
            1 => Ok(TimeSpan::new(Time::new(8, 0), Time::new(8, 50))),
            2 => Ok(TimeSpan::new(Time::new(9, 0), Time::new(9, 50))),
            3 => Ok(TimeSpan::new(Time::new(10, 10), Time::new(11, 0))),
            4 => Ok(TimeSpan::new(Time::new(11, 10), Time::new(12, 0))),
            5 => Ok(TimeSpan::new(Time::new(14, 0), Time::new(14, 50))),
            6 => Ok(TimeSpan::new(Time::new(15, 0), Time::new(15, 50))),
            7 => Ok(TimeSpan::new(Time::new(16, 10), Time::new(17, 0))),
            8 => Ok(TimeSpan::new(Time::new(17, 10), Time::new(18, 0))),
            9 => Ok(TimeSpan::new(Time::new(18, 30), Time::new(19, 20))),
            10 => Ok(TimeSpan::new(Time::new(19, 30), Time::new(20, 20))),
            11 => Ok(TimeSpan::new(Time::new(20, 30), Time::new(21, 20))),
            12 => Ok(TimeSpan::new(Time::new(21, 30), Time::new(22, 20))),
            13 => Ok(TimeSpan::new(Time::new(22, 30), Time::new(23, 20))),
            _ => Err("Invalid time").map_err(anyhow::Error::msg),
        }
    }

    pub fn from_course_index_range(start: u8, end: u8) -> Result<TimeSpan, anyhow::Error> {
        let start = Self::from_course_index(start)?;
        let end = Self::from_course_index(end)?;

        Ok(TimeSpan::new(start.start, end.end))
    }
}

#[derive(Clone)]
pub struct CourseTime {
    span: TimeSpan,
    day: u8,  // 1 for Monday, 7 for Sunday
    week: u8, // 1 for the first week, 17 for the last week
}

impl CourseTime {
    pub fn new(span: TimeSpan, day: u8, week: u8) -> Self {
        Self { span, day, week }
    }

    pub fn to_datetime(
        &self,
        first_week_start: DateTime<Local>,
    ) -> Result<(DateTime<Local>, DateTime<Local>), anyhow::Error> {
        let [start, end] = [self.span.start, self.span.end];

        let date = (first_week_start
            + Duration::weeks((self.week - 1).into())
            + Duration::days((self.day - 1).into()))
        .date_naive();

        let start = date
            .and_hms_opt(start.hour.into(), start.minute.into(), 0)
            .ok_or("Cannot calculate start time")
            .map_err(anyhow::Error::msg)?
            .and_local_timezone(first_week_start.timezone());
        let LocalResult::Single(start) = start else {
            return Err("Cannot restore timezone").map_err(anyhow::Error::msg);
        };

        let stop = date
            .and_hms_opt(end.hour.into(), end.minute.into(), 0)
            .ok_or("Cannot calculate end time")
            .map_err(anyhow::Error::msg)?
            .and_local_timezone(first_week_start.timezone());
        let LocalResult::Single(stop) = stop else {
            return Err("Cannot restore timezone").map_err(anyhow::Error::msg);
        };

        Ok((start, stop))
    }
}

impl std::fmt::Display for CourseTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}-{}:{},第{}周 周{}",
            self.span.start.hour,
            self.span.start.minute,
            self.span.end.hour,
            self.span.end.minute,
            self.day,
            self.week
        )
    }
}
impl std::fmt::Debug for CourseTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
