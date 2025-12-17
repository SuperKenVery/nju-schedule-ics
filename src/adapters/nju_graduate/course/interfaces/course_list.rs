//! 对应课表页面下面的表格。这里的课程时间机器可读性很差，但有校区信息。

use anyhow::{Context, Result};
use map_macro::hash_map;
use reqwest_middleware::ClientWithMiddleware;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Response {
    pub code: String,
    pub datas: Datas,
}

#[derive(Deserialize)]
pub struct Datas {
    pub xsjxrwcx: DataInner,
}

#[derive(Deserialize)]
pub struct DataInner {
    pub totalSize: i32,
    pub pageNumber: i32,
    pub pageSize: i32,
    pub rows: Vec<Row>,
}

#[derive(Deserialize)]
pub struct Row {
    /// 带班级号课程名称，比如`匹克球02`
    pub BJMC: String,
    /// 校区名字，比如`苏州校区`
    pub XQDM_DISPLAY: String,
    /// 上课人数，比如`"20"`
    pub XKRS: String,
    /// 开课单位，比如`体育科学研究所`
    pub KKDW_DISPLAY: String,
    /// 首次上课日期，比如`2025-09-18`
    pub SCSKRQ: Option<String>,
    /// 课程ID，比如`081200B71`
    pub KCDM: String,
}

impl Response {
    pub async fn from_req(client: &ClientWithMiddleware, semester_id: &str) -> Result<Self> {
        let form = hash_map! {
            "XNXQDM" => semester_id,
            "XH" => "",
            "pageNumber" => "1",
            "pageSize" => "100"
        };

        Ok(
            client.post("https://ehallapp.nju.edu.cn/gsapp/sys/wdkbapp/modules/xskcb/xsjxrwcx.do?_=1765716674587")
                .form(&form)
                .send().await?.json().await.context("Parsing course list for nju graduate student")?
        )
    }
}
