use super::login::LoginCredential;
use chrono::{Datelike, Days, Local, NaiveDate, Utc};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::Arc;
// use crate::schedule::course::Course;
use anyhow::{anyhow, Context, Result};
use log::debug;
use reqwest_middleware::ClientWithMiddleware;
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};

async fn build_client(auth: &LoginCredential) -> Result<ClientWithMiddleware> {
    let cookie_store = Arc::new(reqwest::cookie::Jar::default());
    cookie_store.add_cookie_str(
        &format!("CASTGC={}", auth.castgc),
        &"https://authserver.nju.edu.cn".try_into().unwrap(),
    );

    let reqwest_client = reqwest::ClientBuilder::new()
        .cookie_provider(cookie_store.clone())
        // Redirect by default <=10. It seems we need 9, fine.
        .user_agent("rust-reqwest/0.11.18")
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    let retry = ExponentialBackoff::builder().build_with_max_retries(10);

    let client = reqwest_middleware::ClientBuilder::new(reqwest_client)
        .with(RetryTransientMiddleware::new_with_policy(retry))
        .build();

    /* Get the cookies for ehall.nju.edu.cn
     *
     * We'll be redirected to authserver. As we have CASTGC, we'll be
     * thrown back immediately with the needed cookies.
     * Then we'll get some needed cookies from ehallapp.nju.edu.cn.
     *
     * At last, we'll have enough cookies to request for all courses.
     */
    let _ = client
        .get("https://ehall.nju.edu.cn/appShow?appId=4770397878132218")
        .send()
        .await?;

    Ok(client)
}

pub async fn get_curr_semester(client: &ClientWithMiddleware) -> Result<String> {
    let semesters: JsonValue = client
        .post("https://ehallapp.nju.edu.cn/jwapp/sys/wdkb/modules/jshkcb/dqxnxq.do")
        .send()
        .await?
        .json()
        .await?;
    let latest_semester = semesters["datas"]["dqxnxq"]["rows"][0]["DM"]
        .as_str()
        .context("Cannot resolve the latest semester")?;

    Ok(latest_semester.to_string())
}

pub async fn get_course_raw(auth: &LoginCredential) -> Result<String> {
    let client = build_client(auth).await?;

    let curr_semester = get_curr_semester(&client).await?;
    let form = HashMap::from([
        ("XNXQDM", curr_semester.as_str()),
        ("pageSize", "9999"),
        ("pageNumber", "1"),
    ]);

    let resp = client
        .post("https://ehallapp.nju.edu.cn/jwapp/sys/wdkb/modules/xskcb/cxxszhxqkb.do")
        .form(&form)
        .send()
        .await?
        .text()
        .await?;

    Ok(resp)
}

pub async fn get_final_exams_raw(auth: &LoginCredential) -> Result<JsonValue> {
    let client = build_client(auth).await?;

    let curr_semester = get_curr_semester(&client).await?;
    let form = HashMap::from([(
        "requestParamStr",
        format!(
            r#"{{"XNXQDM":"{}","*order":"-KSRQ,-KSSJMS"}}"#,
            curr_semester
        ),
    )]);

    let final_exam_info: JsonValue = client
        .post("https://ehallapp.nju.edu.cn/jwapp/sys/studentWdksapApp/WdksapController/cxxsksap.do")
        .form(&form)
        .send()
        .await?
        .json()
        .await
        .context("Parsing json of final exam")?;

    Ok(final_exam_info)
}

pub async fn get_first_week_start(auth: &LoginCredential) -> Result<NaiveDate> {
    let client = build_client(auth).await?;

    let semester_info: JsonValue = client
        .get("https://ehallapp.nju.edu.cn/jwapp/sys/wdkb/modules/jshkcb/cxjcs.do")
        .send()
        .await?
        .json()
        .await
        .context("Parsing json from NJU semester info")?;

    let current_date = Utc::now().naive_local().date();
    let semester_start = semester_info["datas"]["cxjcs"]["rows"]
        .as_array()
        .ok_or(anyhow!("Semester info not array"))?
        .iter()
        .map(parse_semester_info)
        .find_map(|date| {
            let date = date.ok()?;
            if current_date.signed_duration_since(date).num_seconds() >= 0 {
                Some(date)
            } else {
                None
            }
        })
        .ok_or(anyhow!(
            "No semester start found, semester info: {semester_info:#?}"
        ))?;

    Ok(semester_start)
}

/// Parse semester info from NJU backend into a date
///
/// Argument:
///     - info: &JsonValue - A semester info from NJU backend
///         - The backend returns all semesters; you should take one and feed to this function.
///         - The json structure should be like:
/// ```json
/// {
///     "XQKSRQ": "2025-02-17 00:00:00"
/// }
/// ```
fn parse_semester_info(info: &JsonValue) -> Result<NaiveDate> {
    let name = &info["XQKSRQ"]; // "2025-02-17 00:00:00"
    let [date, _time] = name
        .as_str()
        .ok_or(anyhow!("Cannot read semester and week name"))?
        .split(" ")
        .collect::<Vec<&str>>()[..]
    else {
        return Err(anyhow!("Failed to parse date for semester start"));
    };

    let semester_start = NaiveDate::parse_from_str(date, "%Y-%m-%d")?;
    Ok(semester_start)
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs::File;
    use std::io::stdin;
    use std::io::Write;
    use std::process::Command;
    use tokio;

    async fn get_auth() -> LoginCredential {
        LoginCredential::from_login("Username", "Password", |content| async move {
            let mut file = File::create("captcha.jpeg").unwrap();
            file.write_all(&content).unwrap();
            Command::new("open").arg("captcha.jpeg").spawn().unwrap();
            let mut input = String::new();
            stdin().read_line(&mut input).unwrap();
            // Remove tailing \n
            input.pop();
            println!("Got captcha `{}`", input);
            input
        })
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn get_course_raw_works() {
        let auth = get_auth().await;
        let client = build_client(&auth).await.unwrap();

        let week_info = client
            .get("https://wx.nju.edu.cn/njukb/wap/default/classes")
            .send()
            .await
            .unwrap();
        println!("{:?}", week_info);
    }

    #[tokio::test]
    async fn get_first_week_start_works() {
        let auth = get_auth().await;
        let result = get_first_week_start(&auth).await.unwrap();
        println!("{}", result);
        assert_eq!(result, NaiveDate::from_ymd_opt(2025, 8, 25).unwrap());
    }
}
