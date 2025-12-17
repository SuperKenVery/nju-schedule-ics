//! Datastructures for parsing response of get final exams.
//! URL: https://ehallapp.nju.edu.cn/jwapp/sys/studentWdksapApp/WdksapController/cxxsksap.do
#![allow(non_snake_case)]

use anyhow::Result;
use map_macro::hash_map;
use reqwest_middleware::ClientWithMiddleware;
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct Response {
    pub code: String,
    pub datas: Data,
}

#[derive(Deserialize)]
pub struct Data {
    pub cxxsksap: DataInner,
}

#[derive(Deserialize)]
pub struct DataInner {
    pub pageSize: i32,
    pub pageNumber: i32,
    pub totalSize: i32,
    pub rows: Vec<Row>,
}

#[derive(Deserialize)]
pub struct Row {
    /// 考试地点，比如`仙Ⅱ-105`
    pub JASMC: String,
    /// 考试开始时间，比如`14:00`
    pub KSKSSJ: String,
    /// 考试结束时间，比如`16:00`
    pub KSJSSJ: String,
    /// 学号
    pub XH: String,
    /// 考试日期，比如`2026-01-07`
    pub KSRQ: String,
    /// 课程名
    pub KCM: String,
    /// 教师姓名
    pub ZJJSXM: String,
}

impl Response {
    /// Create a final exams response by sending the request.
    ///
    /// semester_id: e.g. "2025-2026-1" for first half of 2025-2026.
    pub async fn from_req(client: &ClientWithMiddleware, semester_id: &str) -> Result<Self> {
        let request_param = json!({
            "XNXQDM": semester_id,
            "*order": "-KSRQ,-KSSJMS"
        });
        let form = hash_map! {
            "requestParamStr" => request_param.to_string(),
        };

        Ok(client
            .post("https://ehallapp.nju.edu.cn/jwapp/sys/studentWdksapApp/WdksapController/cxxsksap.do")
            .form(&form)
            .send()
            .await?
            .json()
            .await?)
    }
}
