use crate::adapters::nju_graduate::course::utils::group_by;
use crate::adapters::{course::Course, nju_graduate::NJUGraduateAdapter, traits::CoursesProvider};
use anyhow::Result;
use async_trait::async_trait;
use chrono::{Duration, NaiveDate, NaiveDateTime};
use interfaces::all_semesters::Response as AllSemesters;
use interfaces::course_list::{Response as CourseTableResponse, Row as CourseWithCampus};
use interfaces::courses::{Response as CoursesResponse, Row as SplittedCourse};
use reqwest_middleware::ClientWithMiddleware;
use std::cmp::Ordering;
use std::collections::HashMap;
use tracing::{Level, event};

mod interfaces;
mod utils;

/// Get the current semester.
///
/// It works by:
/// 1. Fetch all semesters
/// 2. Find the latest semester whose start date is not later than today + 14 days.
///
/// This is because people want to see their schedule before the semester actually starts.
async fn get_curr_semester(client: &ClientWithMiddleware) -> Result<(NaiveDateTime, String)> {
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
        .ok_or_else(|| anyhow::anyhow!("No valid semesters found"))
}

/// Merge courses returned by API.
///
/// `raw_courses`: The courses returned by school API.
/// `coursename_to_campus`: A [`HashMap`] mapping from course ID (KCDM) to campus display name.
///
/// # Why
/// The graduate student API returns courses by classes, so e.g. a course from `8:00` to `9:50` would be returned as
/// - A course from `8:00` to `8:50`
/// - Another course from `9:00` to `9:50`
///
/// So we need to combine them together.
///
/// # How it works
/// For all SplittedCourse with a same name, we
/// 1. Sort all their times, based on weekday and start end time.
/// 2. Combine adjacent ones.
///     - This is determined if they are on same weekday, have same name, and one's start index equals to the other's end index.
///     - For class index, we use `JSJCDM` and `KSJCDM` (end and start class index)
/// 3. For the combined courses, we have
///     1) In which weeks we have this class
///     2) The time(s) we have it in one week (adjacent ones combined)
///  4. We can generate a Course object.
async fn merge_courses(
    raw_courses: Vec<SplittedCourse>,
    // courseid_to_campus: HashMap<String, String>,
) -> Vec<SplittedCourse> {
    let courses_by_name = group_by(raw_courses, |c| c.BJMC.clone());

    courses_by_name
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
                    weekday_startidx_to_course.remove(&(curr_course.XQ, curr_course.JSJCDM + 1))
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
            // We flatten and convert to Course in next step
            combined_courses
        })
        .flatten()
        .collect()
}

#[async_trait]
impl CoursesProvider for NJUGraduateAdapter {
    async fn courses(&self, client: &ClientWithMiddleware) -> Result<Vec<Course>> {
        let (semester_start, curr_semester_id) = get_curr_semester(client).await?;

        let courses = CoursesResponse::from_req(client, &curr_semester_id).await?;
        let merged_courses = merge_courses(courses.datas.xspkjgcx.rows).await;

        let course_list = CourseTableResponse::from_req(client, &curr_semester_id).await?;
        let courseid_to_campus = build_cid_to_campus_map(course_list.datas.xsjxrwcx.rows);

        let courses = merged_courses
            .iter()
            .map(|x| x.to_course(&courseid_to_campus, &semester_start.date()))
            .collect();

        Ok(courses)
    }
}

fn build_cid_to_campus_map(courses: Vec<CourseWithCampus>) -> HashMap<String, String> {
    let mut result = HashMap::new();

    for course in courses {
        result.insert(course.KCDM, course.XQDM_DISPLAY);
    }

    result
}
