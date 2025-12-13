use anyhow::Result;
use map_macro::hash_map;
use reqwest_middleware::ClientWithMiddleware;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Response {
    pub code: String,
    pub data: Data,
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

#[derive(Deserialize, Clone)]
pub struct Row {
    /// 带班级号的课程名称，比如`新时代中国特色社会主义理论与实践（19）`
    pub BJMC: String,
    /// 课程名称，比如`新时代中国特色社会主义理论与实践`
    pub KCMC: String,
    /// 上课的周次，是一个文本的bitmap，比如`000111111111111111000000000000`
    pub ZCBH: String,
    /// 不知道是什么时间，比如`2025-06-23 00:00:00`
    pub CZSJ: String,
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
            .await?)
    }
}
