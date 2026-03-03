//! Datastructures for parsing response of current semester.
//! URL: https://ehallapp.nju.edu.cn/jwapp/sys/wdkb/modules/jshkcb/dqxnxq.do
#![allow(non_snake_case)]

use anyhow::{Context, Result};
use reqwest_middleware::ClientWithMiddleware;
use serde::Deserialize;
use tracing::instrument;

#[derive(Deserialize, Debug)]
pub struct Response {
    pub datas: Data,
    pub code: String,
}

#[derive(Deserialize, Debug)]
pub struct Data {
    pub dqxnxq: Dqxnxq,
}

#[derive(Deserialize, Debug)]
pub struct Dqxnxq {
    pub totalSize: i32,
    pub rows: Vec<Semester>,
}

#[derive(Deserialize, Debug)]
pub struct Semester {
    /// Semester year e.g. "2025-2026"
    pub XNDM: String,
    /// Semester full name e.g. "2025-2026-1" (for first half)
    pub DM: String,
    /// Semester half e.g. "1" for first half
    pub XQDM: String,
    /// Display name e.g. "2025-2026学年 第1学期"
    pub MC: String,
}

impl Response {
    #[instrument(ret)]
    pub async fn from_req(client: &ClientWithMiddleware) -> Result<Self> {
        client
            .get("https://ehallapp.nju.edu.cn/jwapp/sys/wdkb/modules/jshkcb/dqxnxq.do")
            .send()
            .await?
            .json()
            .await
            .context("Parsing current semester for nju undergrad")
    }
}
