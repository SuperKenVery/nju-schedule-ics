use anyhow::anyhow;
use anyhow::Result;
use chrono::{Datelike, Days, Local, NaiveDate};
use reqwest;
use serde_json;
use serde_json::Value as JsonValue;

use std::collections::HashSet;

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

pub struct HolidayCal {
    pub holidays: HashSet<NaiveDate>,
    /// 调休日期
    pub compdays: HashSet<NaiveDate>,
}

impl HolidayCal {
    pub async fn from_shuyz() -> Result<Self> {
        let holidays_json: JsonValue = reqwest::get(
            "https://www.shuyz.com/githubfiles/china-holiday-calender/master/holidayAPI.json",
        )
        .await?
        .json()
        .await?;

        let year = Local::now().year().to_string();
        let year_holiday = holidays_json["Years"][year]
            .as_array()
            .ok_or(anyhow!("Holiday API year not array"))?;

        let holidays = year_holiday
            .iter()
            .map(|holiday| {
                let start = NaiveDate::parse_from_str(
                    holiday["StartDate"]
                        .as_str()
                        .ok_or(anyhow!("Holiday API start date not str"))?,
                    "%Y-%m-%d",
                )?;
                let end = NaiveDate::parse_from_str(
                    holiday["EndDate"]
                        .as_str()
                        .ok_or(anyhow!("Holiday API end date not str"))?,
                    "%Y-%m-%d",
                )?;

                (0..=(end - start).num_days())
                    .map(|i| {
                        start
                            .checked_add_days(Days::new(i as u64))
                            .ok_or(anyhow!("Invalid date calculated from holiday api"))
                    })
                    .collect::<Result<Vec<_>>>()
            })
            .collect::<Result<Vec<Vec<_>>>>()?
            .concat()
            .into_iter()
            .collect();

        let compdays = year_holiday
            .iter()
            .map(|holiday| {
                holiday["CompDays"]
                    .as_array()
                    .ok_or(anyhow!("Holiday CompDays not array"))?
                    .iter()
                    .map(|date| {
                        Ok(NaiveDate::parse_from_str(
                            date.as_str()
                                .ok_or(anyhow!("Holiday API comp day not str"))?,
                            "%Y-%m-%d",
                        )?)
                    })
                    .collect::<Result<Vec<_>>>()
            })
            .collect::<Result<Vec<Vec<_>>>>()?
            .concat()
            .into_iter()
            .collect();

        Ok(HolidayCal { holidays, compdays })
    }

    pub fn is_holiday(&self, date: NaiveDate) -> bool {
        self.holidays.contains(&date)
    }

    pub fn is_compday(&self, date: NaiveDate) -> bool {
        self.compdays.contains(&date)
    }
}

#[cfg(test)]
mod test {
    use chrono::NaiveDate;

    use super::HolidayCal;

    #[tokio::test]
    async fn hcal_midautumn() {
        let hcal = HolidayCal::from_shuyz().await.unwrap();

        println!(
            "Holidays: {:?}\nCompdays: {:?}",
            hcal.holidays, hcal.compdays
        );

        assert!(hcal.is_holiday(NaiveDate::from_ymd_opt(2024, 9, 15).unwrap()));
        assert!(hcal.is_holiday(NaiveDate::from_ymd_opt(2024, 9, 16).unwrap()));
        assert!(hcal.is_holiday(NaiveDate::from_ymd_opt(2024, 9, 17).unwrap()));
        assert!(hcal.is_compday(NaiveDate::from_ymd_opt(2024, 9, 14).unwrap()));
    }
}
