use anyhow::{Context, Result};
use reqwest_middleware::ClientWithMiddleware;
use serde::Deserialize;
use tracing::instrument;

#[derive(Deserialize, Debug)]
pub struct Response {
    pub code: String,
    pub datas: Datas,
}

#[derive(Deserialize, Debug)]
pub struct Datas {
    pub kfdxnxqcx: DataInner,
}

#[derive(Deserialize, Debug)]
pub struct DataInner {
    pub totalSize: i32,
    pub pageSize: i32,
    pub rows: Vec<Row>,
    // pub extParams: ExtParams,
}

#[derive(Deserialize, Debug)]
pub struct Row {
    /// 学年学期ID，比如`20251`
    pub XNXQDM: String,
    /// 学年学期ID，比如`20251`
    pub WID: String,
    /// 学期显示名称，比如`2024-2025学年 第二学期`
    pub XNXQDM_DISPLAY: String,
    /// 学期开始日期，比如`2025-06-27 00:00:00`
    pub KBKFRQ: String,
}

impl Response {
    #[instrument(err, ret)]
    pub async fn from_req(client: &ClientWithMiddleware) -> Result<Response> {
        client
            .post("https://ehallapp.nju.edu.cn/gsapp/sys/wdkbapp/modules/xskcb/kfdxnxqcx.do")
            .send()
            .await?
            .json()
            .await
            .context("Parsing response of all semesters for nju graduate student")
    }
}
