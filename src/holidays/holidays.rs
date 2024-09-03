use reqwest;
use scraper::{Html, Selector};
use std::collections::HashMap;
use chrono::NaiveDate;

/*
 * 接口使用说明
 * 调用is_holiday函数传入年月日，返回布尔值，代表该日期是否为节假日
 * 
 * 注：
 * 由于本人Rust水平较弱，并不能够完全读懂项目代码，故尝试写这样的一个函数接口
 * 希望能够帮到这个项目
 * 
 * NJU电子商务 AritxOnly 2024.9.3
 */

// 判断日期是否在两个日期之间
fn is_between(date: (u32, u32, u32), start: (u32, u32, u32), end: (u32, u32, u32)) -> bool {
    let date = NaiveDate::from_ymd_opt(date.0 as i32, date.1, date.2);
    let start = NaiveDate::from_ymd_opt(start.0 as i32, start.1, start.2);
    let end = NaiveDate::from_ymd_opt(end.0 as i32, end.1, end.2);

    match (date, start, end) {
        (Some(d), Some(s), Some(e)) => d >= s && d <= e,
        _ => false, // 如果任何日期无效，返回 false
    }
}

// 获取节假日信息
async fn fetch_holidays() -> Result<HashMap<String, HashMap<String, String>>, Box<dyn std::error::Error>> {
    let url = "https://www.beijing.gov.cn/so/topics/1100000088/holiday.html";
    let body = reqwest::get(url).await?.text().await?;

    let document = Html::parse_document(&body);
    let table_selector = Selector::parse("div.holiday div.result-op table.c-table tbody tr").unwrap();

    let mut holidays = HashMap::new();

    for row in document.select(&table_selector).skip(1) { // skip the header row
        let td_selector = Selector::parse("td").unwrap();
        let mut cells = row.select(&td_selector);
        let holiday = cells.next().unwrap().inner_html();
        let mut info = HashMap::new();
        info.insert("放假时间".to_string(), cells.next().unwrap().inner_html());
        info.insert("调休上班日期".to_string(), cells.next().unwrap().inner_html());
        info.insert("放假天数".to_string(), cells.next().unwrap().inner_html());

        holidays.insert(holiday, info);
    }

    Ok(holidays)
}

// 判断是否为节假日
async fn is_holiday(year: u32, month: u32, day: u32) -> bool { // 传入u32类型
    let holidays = fetch_holidays().await.unwrap();
    for (holiday, info) in &holidays {
        let range = info.get("放假时间").unwrap();
        let days = range.split("~").collect::<Vec<&str>>();
        let start = days[0].trim();
        let end = days[1].trim();
        let start_date = date_parser(start).unwrap();
        let end_date = date_parser(end).unwrap();
        if is_between((year, month, day), start_date, end_date) {
            return true;
        }
    }
    false
}

// 将含中文的日期转化为tuple类型
fn date_parser(date: &str) -> Option<(u32, u32, u32)> {
    let date_str = date.replace("年", "-").replace("月", "-").replace("日", "");
    let date_vec = date_str.split("-").collect::<Vec<&str>>();
    if date_vec.len() == 3 {
        let year = date_vec[0].parse::<u32>().unwrap();
        let month = date_vec[1].parse::<u32>().unwrap();
        let day = date_vec[2].parse::<u32>().unwrap();
        return Some((year, month, day));
    } else if date_vec.len() == 2 {
        let month = date_vec[0].parse::<u32>().unwrap();
        let day = date_vec[1].parse::<u32>().unwrap();
        let year = 2024;    // 默认为2024
        return Some((year, month, day));
    }
    None
}

// 测试用途
// #[tokio::main]
// async fn main() {
//     println!("{:?}", is_holiday(2024, 9, 20).await);
//     println!("{:?}", is_holiday(2024, 10, 1).await);
// }