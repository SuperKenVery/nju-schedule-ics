use super::getcourse::{get_course_raw, get_final_exams_raw, get_first_week_start};
use crate::schedule::time::{CourseTime, TimeSpan};
use crate::schedule::{course::Course, holidays::HolidayCal};
use anyhow::anyhow;
use chrono::NaiveDate;
use json::JsonValue::Array as JsonArray;
use reqwest_middleware::ClientWithMiddleware;

pub async fn get_all_courses(
    client: &ClientWithMiddleware,
    hcal: &HolidayCal,
) -> Result<Vec<Course>, anyhow::Error> {
    let first_week_start = get_first_week_start(client).await?;

    let courses_raw = get_course_raw(client).await?;
    let courses_raw = json::parse(&courses_raw)?;
    let rows = &courses_raw["datas"]["cxxszhxqkb"]["rows"];
    let JsonArray(rows) = rows else {
        return Err(anyhow!("Course rows not an array"));
    };
    let courses = rows
        .into_iter()
        .map(|c| Course::from_nju_course_json(c.clone(), hcal, first_week_start.clone()))
        .collect::<Result<Vec<_>, anyhow::Error>>()?;

    let exams_raw = get_final_exams_raw(client).await?;
    let exams_raw = json::parse(&exams_raw)?;
    let rows = &exams_raw["datas"]["cxxsksap"]["rows"];

    Err(anyhow!("Not implemented"))
}

impl Course {
    pub fn from_nju_course_json(
        raw: json::JsonValue,
        hcal: &HolidayCal,
        first_week_start: NaiveDate,
    ) -> Result<Self, anyhow::Error> {
        // Notes
        let line_or_empty = |key: &str| {
            let content = raw[key].as_str();

            if let Some(content) = content {
                format!("{}\n", content)
            } else {
                "".to_string()
            }
        };
        let notes = line_or_empty("SKSM");
        let swaps = line_or_empty("TKJG");
        let final_exam = raw["QMKSXX"].as_str();
        let final_exam = if let Some(fexinfo) = final_exam {
            format!("期末考试 {}\n", fexinfo)
        } else {
            "".to_string()
        };
        let class = line_or_empty("JXBMC");
        let teacher = line_or_empty("JSHS");
        let points = Into::<f32>::into(
            raw["XF"]
                .as_number()
                .ok_or(anyhow!("Cannot extract points"))?,
        );
        let points = format!("{}学分\n", points);

        // Name and location
        let name = raw["KCM"]
            .as_str()
            .ok_or("Cannot extract name")
            .map_err(anyhow::Error::msg)?;
        let location = raw["JASMC"]
            .as_str()
            .unwrap_or("") // 比如阅读课就会没有这个字段
            .replace("（合班）", "");

        // Time
        let start = raw["KSJC"]
            .as_str()
            .ok_or("Cannot extract start time")
            .map_err(anyhow::Error::msg)?
            .parse::<u8>()?;
        let end = raw["JSJC"]
            .as_str()
            .ok_or("Cannot extract end time")
            .map_err(anyhow::Error::msg)?
            .parse::<u8>()?;
        let weekday = raw["SKXQ"]
            .as_str()
            .ok_or("Cannot extract weekday")
            .map_err(anyhow::Error::msg)?
            .parse::<u8>()?;

        let weeks = raw["SKZC"]
            .as_str()
            .ok_or("Cannot extract weeks")
            .map_err(anyhow::Error::msg)?;
        let times = if start != 0 {
            weeks
                .chars()
                .enumerate()
                .map(|(i, c)| (i, c))
                .filter(|(_, c)| *c == '1')
                .map(|(i, _c)| {
                    let week = i + 1;

                    Ok(CourseTime::new(
                        TimeSpan::from_course_index_range(start, end)?,
                        weekday,
                        week as u8,
                    ))
                })
                .filter_map(|t| {
                    if let Ok(t) = t {
                        let date = t.to_naivedate(first_week_start);
                        if let Ok(date) = date {
                            if hcal.is_holiday(date) {
                                None
                            } else {
                                Some(Ok(t))
                            }
                        } else {
                            Some(Err(anyhow!("Failed to convert CourseTime to NaiveDate")))
                        }
                    } else {
                        Some(t) // Propagate error to `collect`
                    }
                })
                .collect::<Result<Vec<_>, anyhow::Error>>()?
        } else {
            // 自由时间的课程，开始结束会设为0
            vec![]
        };

        Ok(Self {
            name: name.to_string(),
            location: location.to_string(),
            notes: format!(
                "{}{}{}{}{}{}",
                notes, class, teacher, points, swaps, final_exam
            ),
            time: times,
        })
    }

    pub fn from_nju_exam_json(
        raw: json::JsonValue,
        first_week_start: NaiveDate,
    ) -> Result<Self, anyhow::Error> {
        let start = raw["KSRQ"];
    }
}
