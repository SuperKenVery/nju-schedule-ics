/*
 * 获取免修不免考的课程
 */

use super::login::LoginCredential;
use super::getcourse::build_client;
use anyhow::anyhow;
use reqwest_middleware::ClientWithMiddleware;
use std::collections::HashMap;

/**
 * Attempted to build a function to get exampted course
 * 免修不免考获取模块（尝试版）
 * 2025.2.28 AritxOnly
 */

pub async fn get_exampted_raw(auth: &LoginCredential) -> Result<String, anyhow::Error> {
    let client = build_client(auth)?;

    // 首先访问主页面获取必要的cookies
    let _ = client
        .get("https://ehallapp.nju.edu.cn/jwapp/sys/mtxkbl/*default/index.do")
        .send()
        .await?;

    // 获取免修不免考课程数据
    let form = HashMap::from([
        ("pageSize", "9999"),
        ("pageNumber", "1"),
    ]);

    let resp = client
        .post("https://ehallapp.nju.edu.cn/jwapp/sys/mtxkbl/modules/mtsq/cxmtxkxx.do")
        .form(&form)
        .send()
        .await?
        .text()
        .await?;

    Ok(resp)
}

pub async fn get_exampted_data(resp: &String) -> Result<Vec<String>, anyhow::Error> {
    let mut course_names = Vec::new();
    
    let document = Document::from(resp);
    
    let table = document.find(Attr("id", "tablemtsq-index-table")).next();

    if let Some(table_node) = table {
        for row in table_node.find(Name("tr")) {
            if let Some(cell) = row.find(Name("td")).filter(|td| td.text() == "课程名").next() {
                if let Some(course_name) = cell.next_sibling() {
                    course_names.push(course_name.text());
                }
            }
        }
    }

    Ok(course_names)
}