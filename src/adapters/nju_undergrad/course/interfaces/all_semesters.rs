//! Datastructures for parsing response of semester informations.
//! URL: https://ehallapp.nju.edu.cn/jwapp/sys/wdkb/modules/jshkcb/cxjcs.do
//!
//! This includes the start date of every semester.
#![allow(non_snake_case)]

use anyhow::{Context, Result};
use reqwest_middleware::ClientWithMiddleware;
use serde::Deserialize;
use tracing::instrument;

#[derive(Deserialize, Debug)]
pub struct Response {
    pub datas: Datas,
    pub code: String,
}

#[derive(Deserialize, Debug)]
pub struct Datas {
    pub cxjcs: Cxjcs,
}

#[derive(Deserialize, Debug)]
pub struct Cxjcs {
    pub totalSize: i32,
    pub rows: Vec<Semester>,
}

#[derive(Deserialize, Debug)]
pub struct Semester {
    /// Semester year e.g. "2025-2026"
    pub XN: String,
    /// Semester half e.g. "1" for first half
    pub XQ: String,
    /// Semester start date e.g. "2025-08-25 00:00:00"
    pub XQKSRQ: String,
}

impl Response {
    #[instrument(ret, err)]
    pub async fn from_req(client: &ClientWithMiddleware) -> Result<Self> {
        client
            .get("https://ehallapp.nju.edu.cn/jwapp/sys/wdkb/modules/jshkcb/cxjcs.do")
            .send()
            .await?
            .json()
            .await
            .context("Parsing all semesters for nju undergrad")
    }
}
