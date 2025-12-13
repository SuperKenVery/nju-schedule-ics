use crate::adapters::nju_graduate::course::utils::group_by;
use crate::adapters::{course::Course, nju_graduate::NJUGraduateAdapter, traits::CoursesProvider};
use anyhow::Result;
use async_trait::async_trait;
use chrono::{Duration, NaiveDateTime};
use interfaces::all_semesters::Response as AllSemesters;
use interfaces::courses::{Response as CoursesResponse, Row as SplittedCourse};
use reqwest_middleware::ClientWithMiddleware;
use std::cmp::Ordering;
use std::collections::HashMap;

mod interfaces;
mod utils;

/// Get the current semester.
///
/// It works by:
/// 1. Fetch all semesters
/// 2. Find the latest semester whose start date is not later than today + 14 days.
///
/// This is because people want to see their schedule before the semester actually starts.
async fn get_curr_semester(client: &ClientWithMiddleware) -> Result<String> {
    let all_semesters = AllSemesters::from_req(client).await?;
    let semesters = all_semesters.datas.kfdxnxqcx.rows;

    let now = chrono::Local::now().naive_local();
    let cutoff = now + Duration::days(14); // Today + 14 days

    semesters
        .into_iter()
        .filter_map(|semester| {
            NaiveDateTime::parse_from_str(&semester.KBKFRQ, "%Y-%m-%d %H:%M:%S")
                .ok()
                .map(|start_date| (start_date, semester.XNXQDM))
        }) // Parse start time
        .filter(|(start_date, _)| *start_date <= cutoff) // Filter those before today+14d
        .max_by_key(|(start_date, _)| *start_date) // Take the latest one
        .map(|(_, semester_id)| semester_id) // Take semester id
        .ok_or_else(|| anyhow::anyhow!("No valid semesters found"))
}

/// The graduate student API returns courses by classes, so e.g. a course from `8:00` to `9:50` would be returned as
/// - A course from `8:00` to `8:50`
/// - Another course from `9:00` to `9:50`
/// So we need to combine them together.
///
/// For all SplittedCourse with a same name, we
/// 1. Sort all their times, based on weekday and start end time.
/// 2. Combine adjacent ones. To detect this, use JSJCDM and KSJCDM (end and start class index)
/// 3. Now we have
///     1) In which weeks we have this class
///     2) The time(s) we have it in one week (adjacent ones combined)
///  4. We can generate a Course object.
async fn merge_courses(raw_courses: Vec<SplittedCourse>) -> Vec<Course> {
    let courses_by_name = group_by(raw_courses, |c| c.BJMC.clone());

    let merged_courses = courses_by_name
        .into_iter()
        .map(|(name, mut courses)| {
            courses.sort_by(|a, b| {
                match a.XQ.cmp(&b.XQ) {
                    // Compare weekday first
                    Ordering::Equal => {
                        a.KSJCDM.cmp(&b.KSJCDM) // If same, compare start index
                    }
                    result => result,
                }
            });

            let mut used = vec![false; courses.len()];
            let mut weekday_startidx_to_course =
                group_by(courses.iter().enumerate(), |(_idx, c)| (c.XQ, c.KSJCDM));

            let mut combined_courses = vec![];
            for (idx, course) in courses.iter().enumerate() {
                // iterate through courses of the same name
                if used[idx] {
                    continue;
                }

                let mut curr_course = course.clone();
                used[idx] = true;
                while let Some(consecutive) =
                    weekday_startidx_to_course.remove(&(course.XQ, course.JSJCDM))
                {
                    // Here, consecutive's length must be 1 (there can't be 2 courses with same weekday and start idx and name!)
                    let (cidx, consecutive) = consecutive[0]; // We would never insert vec![] here, a push always follows insert
                    used[cidx] = true;
                    curr_course.JSJCDM = consecutive.JSJCDM;
                    curr_course.JSSJ = consecutive.JSSJ
                }
                combined_courses.push(curr_course);
            }

            // We would return Vec of SplittedCourse (but the time is actually combined) here
            // We flatten and convert to one Course in next step
            combined_courses
        })
        .map(|cc| {
            // Courses have been combined up till here, it's still a Vec cause e.g. there could be both this course on Monday and Thursday
            // We'll combine those into a [`Course`] and put all those times into the `time` attribute.
            todo!()
        });

    todo!()
}

#[async_trait]
impl CoursesProvider for NJUGraduateAdapter {
    async fn courses(&self, client: &ClientWithMiddleware) -> Result<Vec<Course>> {
        let curr_semester_id = get_curr_semester(client).await?;
        let courses = CoursesResponse::from_req(client, &curr_semester_id).await?;

        todo!()
    }
}
