use super::interfaces;
use crate::adapters::{
    course::Course, course::GeoLocation, nju_batchelor::course::interfaces::all_semesters,
};
use anyhow::{Result, anyhow, bail};
use chrono::{NaiveDate, Utc};
use reqwest::Client;
use reqwest_middleware::ClientWithMiddleware;

pub async fn get_courses(client: &ClientWithMiddleware) -> Result<Vec<Course>> {
    let current_semester = get_current_semester_id(client).await?;
    let courses = interfaces::courses::Response::from_req(client, &current_semester).await?;
    let semester_start = get_semester_start(client, &current_semester).await?;

    Ok(courses
        .datas
        .cxxszhxqkb
        .rows
        .into_iter()
        .map(|course_json| course_json.to_course(&semester_start))
        .collect())
}

async fn get_current_semester_id(client: &ClientWithMiddleware) -> Result<String> {
    let mut curr_semester = interfaces::curr_semester::Response::from_req(client).await?;
    let curr_semester_id = curr_semester
        .datas
        .dqxnxq
        .rows
        .pop()
        .ok_or(anyhow!("Getting current semester returned nothing"))?
        .DM;

    Ok(curr_semester_id)
}

async fn get_semester_start(
    client: &ClientWithMiddleware,
    current_semester_id: &str,
) -> Result<chrono::NaiveDate> {
    let all_semesters = interfaces::all_semesters::Response::from_req(client).await?;

    let curr_semester_info = all_semesters
        .datas
        .cxjcs
        .rows
        .iter()
        .find(|semester| format!("{}-{}", semester.XN, semester.XQ).as_str() == current_semester_id)
        .ok_or(anyhow!("Current semester not found in semester infos"))?;

    let [start_date, _start_time] = curr_semester_info.XQKSRQ.split(" ").collect::<Vec<_>>()[..]
    else {
        bail!(
            "Failed to parse semester start: {}",
            curr_semester_info.XQKSRQ
        )
    };
    let start_date = NaiveDate::parse_from_str(start_date, "%Y-%m-%d")?;

    Ok(start_date)
}

impl interfaces::courses::Course {
    pub fn to_course(self, semester_start: &chrono::NaiveDate) -> Course {
        let time = self.get_time();
        let all_course_times = match time {
            Some((start, end)) => self
                .get_dates(semester_start)
                .iter()
                .map(|date| {
                    let offset = chrono::FixedOffset::east_opt(8 * 60 * 60).unwrap();
                    (
                        date.and_time(start)
                            .and_local_timezone(offset)
                            .unwrap()
                            .with_timezone(&Utc),
                        date.and_time(end)
                            .and_local_timezone(offset)
                            .unwrap()
                            .with_timezone(&Utc),
                    )
                })
                .collect(),
            None => vec![],
        };

        Course {
            name: self.KCM,
            time: all_course_times,
            location: Some(self.JASMC.clone()),
            geo: GeoLocation::from_name_and_campus(&self.JASMC, &self.XXXQDM_DISPLAY),
            campus: Some(self.XXXQDM_DISPLAY),
            notes: vec![
                format!("班级: {}", self.JXBMC),
                format!("教师: {}", self.JSHS),
                format!("上课班级: {}", self.SKBJ),
            ],
        }
    }

    fn get_time(&self) -> Option<(chrono::NaiveTime, chrono::NaiveTime)> {
        if self.KSJC == 0 || self.JSJC == 0 {
            // 自由时间课程
            return None;
        }

        const START_TIMES: [(u32, u32); 13] = [
            (8, 0),
            (9, 0),
            (10, 10),
            (11, 10),
            (14, 0),
            (15, 0),
            (16, 10),
            (17, 10),
            (18, 30),
            (19, 30),
            (20, 30),
            (21, 30),
            (22, 30),
        ];
        const END_TIMES: [(u32, u32); 13] = [
            (8, 50),
            (9, 50),
            (11, 0),
            (12, 0),
            (14, 50),
            (15, 50),
            (17, 0),
            (18, 0),
            (19, 20),
            (20, 20),
            (21, 20),
            (22, 20),
            (23, 20),
        ];

        let start_hour_minute = START_TIMES.get((self.KSJC - 1) as usize)?;
        let end_hour_minute = END_TIMES.get((self.JSJC - 1) as usize)?;
        let start_time =
            chrono::NaiveTime::from_hms_opt(start_hour_minute.0, start_hour_minute.1, 0).unwrap();
        let end_time =
            chrono::NaiveTime::from_hms_opt(end_hour_minute.0, end_hour_minute.1, 0).unwrap();

        Some((start_time, end_time))
    }

    fn get_dates(&self, semester_start: &chrono::NaiveDate) -> Vec<chrono::NaiveDate> {
        let week = chrono::Duration::days(7);
        let day = chrono::Duration::days(1);

        self.SKZC
            .chars()
            .enumerate()
            .filter_map(|(idx, have_course)| {
                if have_course == '1' {
                    Some(semester_start.clone() + week * (idx as i32) + day * (self.SKXQ - 1))
                } else {
                    None
                }
            })
            .collect()
    }
}
