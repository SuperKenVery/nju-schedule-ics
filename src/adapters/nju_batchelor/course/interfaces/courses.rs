//! Datastructures for pasrsing response of get courses.
//! URL: https://ehallapp.nju.edu.cn/jwapp/sys/wdkb/modules/xskcb/cxxszhxqkb.do
#![allow(non_snake_case)]

use anyhow::Result;
use map_macro::hash_map;
use reqwest::Client;
use reqwest_middleware::ClientWithMiddleware;
use serde::Deserialize;
use serde_with::{DisplayFromStr, serde_as};

#[derive(Deserialize)]
pub struct Response {
    pub datas: Data,
    pub code: String,
}

#[derive(Deserialize)]
pub struct Data {
    pub cxxszhxqkb: Cxxszhxqkb,
}

#[derive(Deserialize)]
pub struct Cxxszhxqkb {
    pub pageSize: i32,
    pub pageNumber: i32,
    pub totalSize: i32,
    pub rows: Vec<Course>,
}

#[serde_as]
#[derive(Deserialize)]
pub struct Course {
    /// Course name
    pub KCM: String,
    /// Teacher
    pub SKJS: String,
    /// Teacher with work number e.g. "1507810 王可 "
    pub JSHS: String,
    /// Location id e.g. "XⅠ-106" for 仙1-106
    pub JASDM: String,
    /// Location display name e.g. "仙Ⅰ-106"
    pub JASMC: String,
    /// Class e.g. 形势与政策16班"
    pub JXBMC: String,
    /// Classes that attend this class e.g. "2022计算机学院计算机科学与技术（拔尖计划）,2022计算机学院信息与计算科学（强基计划）,2022计算机学院金融工程（计算机金融实验班）,2022计算机学院计算机科学与技术"
    pub SKBJ: String,
    /// Course days display name e.g. "周二 5-6节 3周, 7周, 11周, 15周 仙Ⅰ-106"
    pub YPSJDD: String,
    /// Course days display name without spaces e.g. "周二 5-6节 3周,7周,11周,15周 仙Ⅰ-106"
    pub ZCXQJCDD: String,
    /// Credits
    pub XF: f32,
    /// On which course index (1-based) does this course start? (开始节次)
    /// For free-time courses, this would be "0".
    #[serde_as(as = "DisplayFromStr")]
    pub KSJC: i32,
    /// On which course index (1-based) does this course end? (结束节次)
    /// For free-time courses, this would be "0".
    #[serde_as(as = "DisplayFromStr")]
    pub JSJC: i32,
    /// The weekday this course occurs (上课星期) e.g. "2" for Tuesday
    #[serde_as(as = "DisplayFromStr")]
    pub SKXQ: i32,
    /// What weeks does this cours happen? Like a bit table.
    /// e.g. "001000100010001000000000000000"
    pub SKZC: String,
    /// Campus ID
    ///
    /// Known IDs:
    /// - 仙林=3
    #[serde_as(as = "DisplayFromStr")]
    pub XXXQDM: i32,
    /// Campus name e.g. "仙林校区"
    pub XXXQDM_DISPLAY: String,
}

impl Response {
    /// Create a courses response by sending the request.
    ///
    /// semester_id: e.g. "2025-2026-1" for first half of 2025-2026.
    pub async fn from_req(client: &ClientWithMiddleware, semester_id: &str) -> Result<Self> {
        let form = hash_map! {
            "XNXQDM" => semester_id,
            "pageSize" => "9999",
            "pageNumber" => "1",
        };

        Ok(client
            .post("https://ehallapp.nju.edu.cn/jwapp/sys/wdkb/modules/xskcb/cxxszhxqkb.do")
            .form(&form)
            .send()
            .await?
            .json()
            .await?)
    }
}
