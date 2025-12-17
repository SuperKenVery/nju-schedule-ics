//! 对应课表页课程表，课程时间机器可读性好，但缺乏校区信息。
use std::collections::HashMap;

use anyhow::{Context, Result};
use chrono::{Duration, FixedOffset, NaiveDate, NaiveTime, Utc};
use derivative::Derivative;
use map_macro::hash_map;
use reqwest_middleware::ClientWithMiddleware;
use serde::Deserialize;

use crate::adapters::course::Course;

#[derive(Deserialize)]
pub struct Response {
    pub code: String,
    pub datas: Data,
}

#[derive(Deserialize)]
pub struct Data {
    pub xspkjgcx: DataInner,
}

#[derive(Deserialize)]
pub struct DataInner {
    pub totalSize: i32,
    pub pageSize: i32,
    pub rows: Vec<Row>,
    // extParams: ,
}

#[derive(Derivative, Deserialize, Clone)]
#[derivative(Debug)]
pub struct Row {
    #[derivative(Debug = "ignore")]
    /// 带班级号的课程名称，比如`新时代中国特色社会主义理论与实践（19）`
    pub BJMC: String,
    /// 课程名称，比如`新时代中国特色社会主义理论与实践`
    pub KCMC: String,
    #[derivative(Debug = "ignore")]
    /// 上课的周次，是一个文本的bitmap，比如`000111111111111111000000000000`
    pub ZCBH: String,
    #[derivative(Debug = "ignore")]
    /// 不知道是什么时间，比如`2025-06-23 00:00:00`
    pub CZSJ: String,
    #[derivative(Debug = "ignore")]
    /// 学期名，比如`20251`表示2025-2026上学期
    pub XNXQDM: String,
    /// 开始节次， 比如5
    pub KSJCDM: i32,
    /// 结束节次，比如6
    pub JSJCDM: i32,
    /// 开始时间，比如`1400`表示14:00
    pub KSSJ: i32,
    /// 结束时间，比如`1450`表示14:50
    pub JSSJ: i32,
    /// 星期几上课，比如2表示星期二
    pub XQ: i32,
    /// 上课地点，比如`苏教B207`
    pub JASMC: String,
    #[derivative(Debug = "ignore")]
    /// 上课地点ID，比如`S01B207`
    pub JASDM: Option<String>,
    /// 教师姓名
    pub JSXM: String,
    // pub KBBZ: Option<String>,    // 可能是 课表备注
    // pub BZ: Option<String>,      // 可能是 备注
    /// 选课备注
    pub XKBZ: Option<String>,
    #[derivative(Debug = "ignore")]
    /// 课程ID，比如`081200B71`
    pub KCDM: String,
}

impl Response {
    pub async fn from_req(client: &ClientWithMiddleware, semester_id: &str) -> Result<Self> {
        let form = hash_map! {
            "XNXQDM" => semester_id,
            "XH" => "",
        };

        Ok(client
            .post("https://ehallapp.nju.edu.cn/gsapp/sys/wdkbapp/modules/xskcb/xspkjgcx.do")
            .form(&form)
            .send()
            .await?
            .json()
            .await
            .context("Parsing schedule courses for nju graduate")?)
    }
}

impl Row {
    pub fn to_course(
        &self,
        courseid_to_campus: &HashMap<String, String>,
        semester_start: &NaiveDate,
    ) -> Course {
        let (start, end) = self.get_time();
        let times: Vec<_> = self
            .get_dates(semester_start)
            .iter()
            .map(|date| {
                let offset = FixedOffset::east_opt(8 * 60 * 60).expect("UTF+8 offset out of bound");
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
            .collect();

        Course {
            name: self.KCMC.clone(),
            time: times,
            location: Some(self.JASMC.clone()),
            geo: None,
            campus: courseid_to_campus.get(&self.KCDM).cloned(),
            notes: vec![
                format!("教师：{}", self.JSXM.clone()),
                format!(
                    "选课备注：{}",
                    self.XKBZ
                        .as_ref()
                        .cloned()
                        .unwrap_or_else(|| "无备注".to_string())
                ),
            ],
        }
    }

    /// Get the start and end time of this course in [`NaiveTime`].
    fn get_time(&self) -> (NaiveTime, NaiveTime) {
        // Parse self.KSSJ and JSSJ
        // KSSJ and JSSJ are in HHMM format, e.g., 1400 for 14:00, 1450 for 14:50
        let start_hour = self.KSSJ / 100;
        let start_minute = self.KSSJ % 100;
        let end_hour = self.JSSJ / 100;
        let end_minute = self.JSSJ % 100;

        let start_time = NaiveTime::from_hms_opt(start_hour as u32, start_minute as u32, 0)
            .expect("Invalid start time");
        let end_time = NaiveTime::from_hms_opt(end_hour as u32, end_minute as u32, 0)
            .expect("Invalid end time");

        (start_time, end_time)
    }

    /// Given semester start, use [`self.ZCBH`] and [`self.XQ`] to get a list of dates
    /// Semester always starts at Monday.
    /// XQ is day of week (1-7, Monday-Sunday), so we convert to 0-6 for calculation
    fn get_dates(&self, semester_start: &NaiveDate) -> Vec<NaiveDate> {
        // XQ is day of week (1-7, Monday-Sunday)
        let day_from_monday = self.XQ - 1;

        // For each week in ZCBH (which is a bitmap string like "000111111111111118000000000000")
        self.ZCBH
            .chars()
            .enumerate()
            .filter(|(_, char)| *char == '1')
            .map(|(week_index, _)| {
                // Calculate the date for this week
                let days_from_start = (week_index as i64) * 7 + (day_from_monday as i64);
                *semester_start + Duration::days(days_from_start)
            })
            .collect()
    }
}
