use super::holidays::HolidayCal;
use super::time::Time;
use super::time::{CourseTime, TimeSpan};
use anyhow::Result;
use anyhow::{anyhow, Context};
use chrono::NaiveDate;
use serde_json::Value as JsonValue;

/// An intermediate representation of a course
#[derive(Debug)]
pub struct Course {
    pub name: String,
    pub location: String,
    pub notes: String,

    pub time: Vec<CourseTime>,
}

impl Course {
    pub fn from_json(
        raw: JsonValue,
        hcal: &HolidayCal,
        first_week_start: NaiveDate,
    ) -> Result<Self> {
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
        let points = raw["XF"]
            .as_f64()
            .ok_or(anyhow!("Cannot extract points of course"))?;
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
                .collect::<Result<Vec<_>>>()?
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

    pub fn from_final_exam_json(raw: JsonValue, first_week_start: NaiveDate) -> Result<Self> {
        let course_name = raw["KCM"]
            .as_str()
            .context("Cannot get course name in final exam")?
            .to_string();
        let teacher_name = raw["ZJJSXM"]
            .as_str()
            .context("Cannot get teacher name in final exam")?;

        // Something like `2025-06-22`
        let date_str = raw["KSRQ"].as_str().context("Cannot get final exam date")?;
        let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
            .context("Failed to parse final exam date")?;

        let days = (date - first_week_start).num_days();
        let week = days / 7 + 1;
        let weekday = days % 7 + 1;

        // Something like `16:00`
        let start_time_str = raw["KSKSSJ"]
            .as_str()
            .context("Cannot get exam start time")?;
        let end_time_str = raw["KSJSSJ"].as_str().context("Cannot get exam end time")?;
        let start_time = Time::from_str(start_time_str)?;
        let end_time = Time::from_str(end_time_str)?;
        let exam_timespan = TimeSpan::new(start_time, end_time);

        Ok(Course {
            name: format!("{} 期末考试", course_name),
            location: raw["JASMC"]
                .as_str()
                .unwrap_or("无法获取考试地点")
                .to_string(),
            notes: format!("任课教师：{}", teacher_name),
            time: vec![CourseTime::new(exam_timespan, weekday as u8, week as u8)],
        })
    }

    pub fn batch_from_json(
        raw: JsonValue,
        hcal: &HolidayCal,
        first_week_start: NaiveDate,
    ) -> Result<Vec<Self>> {
        let rows = &raw["datas"]["cxxszhxqkb"]["rows"]
            .as_array()
            .ok_or(anyhow!(
                "Expected array in course_json['datas']['cxxszhxqkb']['rows'], got {raw:#?}"
            ))?;

        let courses = rows
            .into_iter()
            .map(|c| Self::from_json(c.clone(), hcal, first_week_start))
            .collect::<Result<Vec<_>>>()?;
        Ok(courses)
    }

    pub fn batch_from_final_exam_json(
        raw: JsonValue,
        first_week_start: NaiveDate,
    ) -> Result<Vec<Self>> {
        let rows = raw["datas"]["cxxsksap"]["rows"]
            .as_array()
            .context("Cannot extract rows from final exam raw")?;

        let exams = rows
            .into_iter()
            .map(|exam| Self::from_final_exam_json(exam.clone(), first_week_start))
            .collect::<Result<Vec<_>>>()?;

        Ok(exams)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use json::JsonValue::Array as jsonArray;
    use std::fs::File;
    use std::io::Read;

    #[tokio::test]
    async fn test_course_from_json() {
        // Build hcal and first_week_start
        let hcal = HolidayCal::from_shuyz().await.unwrap();
        let first_week_start = NaiveDate::from_ymd_opt(2024, 9, 2).unwrap();

        // Read from ./example_course_data_1.txt
        let mut file = File::open("./src/schedule/example_course_data_1.txt").unwrap();
        // Read all its contents
        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();

        // Parse it as json
        let obj = json::parse(&content).unwrap();
        let rows = &obj["datas"]["cxxszhxqkb"]["rows"];
        let jsonArray(rows) = rows else {
            panic!("Not an array??");
        };
        for c in rows {
            let course = Course::from_json(c.clone(), &hcal, first_week_start);
            println!("{:#?}", course);
            let _course = course.unwrap();
        }
    }
}
