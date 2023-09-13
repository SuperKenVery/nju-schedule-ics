use super::login::LoginCredential;
use std::collections::HashMap;
use reqwest::{cookie::CookieStore, header::HeaderValue};
use std::sync::Arc;
use json;
use chrono::{DateTime, Datelike, Local};
// use crate::schedule::course::Course;
use anyhow;
use reqwest_middleware::ClientWithMiddleware;
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};


fn build_client(auth: &LoginCredential) -> Result<ClientWithMiddleware,anyhow::Error> {
    let cookie_store=Arc::new(reqwest::cookie::Jar::default());

    let cookie=HeaderValue::from_str(&format!("CASTGC={}",auth.castgc)).unwrap();

    cookie_store.set_cookies(
        &mut vec![&cookie].into_iter(),
        &"https://authserver.nju.edu.cn".try_into().unwrap()
    );

    let reqwest_client=reqwest::ClientBuilder::new()
        .cookie_provider(cookie_store.clone())
        // Redirect by default <=10. It seems we need 9, fine.
        .user_agent("rust-reqwest/0.11.18")
        .timeout(std::time::Duration::from_secs(3))
        .build()?;

    let mut retry=ExponentialBackoff::builder()
        .build_with_max_retries(10);
    retry.max_retry_interval=std::time::Duration::from_secs(0);

    let client=reqwest_middleware::ClientBuilder::new(reqwest_client)
        .with(RetryTransientMiddleware::new_with_policy(retry))
        .build();

    Ok(client)
}

pub async fn get_course_raw(auth: &LoginCredential) -> Result<String, anyhow::Error> {
    let client=build_client(auth)?;

    /* We'll be redirected to authserver. As we have CASTGC, we'll be
     * thrown back immediately with the needed cookies.
     * Then we'll get some needed cookies from ehallapp.nju.edu.cn.
     *
     * At last, we'll have enough cookies to request for all courses.
    */
    let _=client.get("https://ehall.nju.edu.cn/appShow?appId=4770397878132218")
        .send()
        .await?;

    println!("Done login");

    let semesters=client.post("https://ehallapp.nju.edu.cn/jwapp/sys/wdkb/modules/jshkcb/dqxnxq.do")
        .send().await?
        .text().await?;
    let semesters=json::parse(&semesters)?;
    let latest_semester=semesters["datas"]["dqxnxq"]["rows"][0]["DM"].as_str()
        .ok_or("Cannot resolve the latest semester")
        .map_err(anyhow::Error::msg)?;

    let mut form = HashMap::new();
    form.insert("XNXQDM".to_string(), latest_semester);
    form.insert("pageSize".into(), "9999");
    form.insert("pageNumber".into(), "1");

    let resp=client.post("https://ehallapp.nju.edu.cn/jwapp/sys/wdkb/modules/xskcb/cxxszhxqkb.do")
        .form(&form)
        .send().await?
        .text().await?;

    Ok(resp)
}

// pub async fn get_course_info(auth: &LoginCredential) -> Result<Vec<Course>, anyhow::Error> {


//     // https://ehallapp.nju.edu.cn/jwapp/sys/wdkb/modules/jshkcb/cxkcdgxx.do
//     todo!()
// }

pub async fn get_first_week_start(auth: &LoginCredential) -> Result<DateTime<Local>,anyhow::Error> {
    let client=build_client(auth)?;

    let week_info=client.get("https://wx.nju.edu.cn/njukb/wap/default/classes")
        .send().await?
        .text().await?;
    let week_info=json::parse(&week_info)?;

    let name=&week_info["d"]["dateInfo"]["name"]; // "2023-2024学年上学期 第1周"
    let [_semester, week_name]=name
        .as_str()
        .ok_or("Cannot read semester and week name")
        .map_err(anyhow::Error::msg)?
        .split(" ").collect::<Vec<&str>>()[..] else {
        return Err(anyhow::Error::msg("Invalid dateInfo name"));
    };
    let week_num_str=&week_name[3..week_name.len()-3];
    let week_num=week_num_str.parse::<u8>()?;

    // Get local date from chrono
    let local_date=chrono::Local::now();

    // What day is it today?
    let weekday=local_date.weekday().num_days_from_monday();

    // Rewind local_date to Monday
    let monday=local_date-chrono::Duration::days(weekday as i64);

    // Rewind monday to the Monday of the first week
    let first_week_start=monday-chrono::Duration::weeks(week_num as i64-1);

    Ok(first_week_start)

}

#[cfg(test)]
mod test{
    use super::*;
    use tokio;
    use std::io::stdin;
    use std::process::Command;
    use std::fs::File;
    use std::io::Write;

    async fn get_auth() -> LoginCredential {
        LoginCredential::from_login("PutYourOwn", "NotGonnaTellYou",
            |content| async move{
            let mut file=File::create("captcha.jpeg").unwrap();
            file.write_all(&content).unwrap();
            Command::new("open").arg("captcha.jpeg").spawn().unwrap();
            let mut input=String::new();
            stdin().read_line(&mut input).unwrap();
            // Remove tailing \n
            input.pop();
            input
        }).await.unwrap()
    }

    #[tokio::test]
    async fn get_course_raw_works(){
        let auth=get_auth().await;
        // let result=get_course_raw(&auth).await.unwrap();
        // println!("{}", result);
        let client=build_client(&auth).unwrap();

        let week_info=client.get("https://wx.nju.edu.cn/njukb/wap/default/classes")
            .send().await.unwrap();
        println!("{:?}",week_info);
    }

    #[tokio::test]
    async fn get_first_week_start_works(){
        let auth=get_auth().await;
        let result=get_first_week_start(&auth).await.unwrap();
        println!("{}", result);
    }
}
