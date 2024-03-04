use anyhow::anyhow;
use json;
use json::JsonValue::Array as jsonArray;
use super::time::{TimeSpan, CourseTime};
use std::num::ParseIntError;

#[derive(Debug)]
pub struct Course {
    pub name: String,
    pub location: String,
    pub notes: String,

    pub time: Vec<CourseTime>,
}

impl Course {
    pub fn from_json(raw: json::JsonValue) -> Result<Self, anyhow::Error> {
        let notes=raw["SKSM"].as_str();
        let notes=if let Some(notes)=notes{
            format!("{}\n", notes)
        }else{
            "".to_string()
        };
        let swaps=raw["TKJG"].as_str();
        let swaps=if let Some(swaps)=swaps{
            format!("{}\n", swaps)
        }else{
            "".to_string()
        };
        let final_exam=raw["QMKSXX"].as_str();
        let final_exam=if let Some(final_exam)=final_exam{
            format!("{}\n", final_exam)
        }else{
            "".to_string()
        };
        let name=raw["KCM"].as_str()
            .ok_or("Cannot extract name").map_err(anyhow::Error::msg)?;
        let location=raw["JASMC"]
            .as_str()
            .unwrap_or("")      // 比如阅读课就会没有这个字段
            .replace("（合班）", "");

        let time=raw["ZCXQJCDD"].as_str
        ()
            .ok_or("Cannot extract time").map_err(anyhow::Error::msg)?;
        /* Example data:
         * 周三 2-4节 1-17周 仙Ⅱ-211,周五 3-4节 1-17周 仙Ⅱ-211
         * 自由时间 0-0节 7-17周 自由地点
         * 周五 7-8节 3周,7周,11周,15周 仙Ⅰ-108
         * 周四 3-4节 1-17周 仙Ⅱ-212,周四 9-10节 1-17周 基础实验楼乙124,125,周一 3-4节 1-17周 仙Ⅱ-212
         * 周三 3-4节 2-16周(双)
         */
        let time=time.replace("周,", "周|");
        /* Now they are:
         * 周三 2-4节 1-17周 仙Ⅱ-211,周五 3-4节 1-17周 仙Ⅱ-211
         * 自由时间 0-0节 7-17周 自由地点
         * 周五 7-8节 3周|7周|11周|15周 仙Ⅰ-108
         * 周四 3-4节 1-17周 仙Ⅱ-212,周四 9-10节 1-17周 基础实验楼乙124,125,周一 3-4节 1-17周 仙Ⅱ-212
         * 周三 3-4节 2-16周(双)
         */
        let time=time.replace(",周","##周");
        /* Now they are:
         * 周三 2-4节 1-17周 仙Ⅱ-211##周五 3-4节 1-17周 仙Ⅱ-211
         * 自由时间 0-0节 7-17周 自由地点
         * 周五 7-8节 3周|7周|11周|15周 仙Ⅰ-108
         * 周四 3-4节 1-17周 仙Ⅱ-212##周四 9-10节 1-17周 基础实验楼乙124,125##周一 3-4节 1-17周 仙Ⅱ-212
         * 周三 3-4节 2-16周(双)
         */
        let times=time.split("##").into_iter()
            .map(|time| {
                // Weekday
                let [weekday, time, weeks, _location]=time.split(" ").collect::<Vec<&str>>()[..] else {
                    return Err(anyhow!("Invalid time str: {:?}",time));
                };

                if weekday.contains("自由时间"){
                    return Ok(vec![]);
                }
                let weekday: Result<u8, anyhow::Error>=match weekday {
                    "周一" => Ok(1),
                    "周二" => Ok(2),
                    "周三" => Ok(3),
                    "周四" => Ok(4),
                    "周五" => Ok(5),
                    "周六" => Ok(6),
                    "周日" => Ok(7),
                    _ => Err("Invalid weekday").map_err(anyhow::Error::msg),
                };
                let weekday=weekday?;

                // Weeks
                let weeks=if weeks.contains("-"){
                    // Chinese character takes 3 bytes
                    let (weeks, cond): (&str, Box<dyn Fn(u8) -> bool>) = if weeks.contains("(双)"){
                        (&weeks[..weeks.len()-5], Box::new(|week| week%2==0))
                    }else if weeks.contains("(单)"){
                        (&weeks[..weeks.len()-5], Box::new(|week| week%2==1))
                    }else{
                        (&weeks[..], Box::new(|_| true))
                    };

                    let weeks=&weeks[0..weeks.len()-3];
                    let [start, end]=weeks
                        .split("-")
                        .map(|week| week.parse::<u8>())
                        .collect::<Result<Vec<_>,ParseIntError>>()?[..] else{
                            return Err("Invalid week range str").map_err(anyhow::Error::msg);
                        };
                    let weeks=(start..=end)
                        .filter(|week| cond(*week))
                        .collect::<Vec<u8>>();
                    weeks
                }else{
                    weeks
                        .split("|")
                        .map(|weekstr| &weekstr[..weekstr.len()-3])
                        .map(|week| week.parse::<u8>())
                        .collect::<Result<Vec<_>,ParseIntError>>()?
                };

                // Time
                // Remove the last character, which is "节" (takes 3 bytes)
                let time=&time[..time.len()-3];

                let [start, end]=time.split("-")
                    .map(|time| {
                        let time=time.parse::<u8>()?;
                        let time=TimeSpan::from_course_index(time);
                        time
                    })
                    .collect::<Result<Vec<_>,anyhow::Error>>()?[..] else{
                        return Err("Invalid time range str").map_err(anyhow::Error::msg);
                    };

                Ok(weeks.iter().map(|week|{
                    CourseTime::new(
                        TimeSpan::new(start.start,end.end),
                        weekday,
                        *week,
                    )
                }).collect::<Vec<_>>())
            })
            .collect::<Result<Vec<Vec<CourseTime>>,_>>()?
            .concat();

        Ok(
            Self {
                name: name.to_string(),
                location: location.to_string(),
                notes: format!("{}{}{}", notes, swaps, final_exam),
                time: times,
            }
        )
    }

    pub fn batch_from_json(raw: json::JsonValue) -> Result<Vec<Self>, anyhow::Error> {
        let rows=&raw["datas"]["cxxszhxqkb"]["rows"];
        let jsonArray(rows)=rows else{
            return Err("Not an array??").map_err(anyhow::Error::msg);
        };
        let courses=rows.into_iter()
            .map(|c| Self::from_json(c.clone()))
            .collect::<Result<Vec<_>,anyhow::Error>>()?;
        Ok(courses)
    }
}



#[cfg(test)]
mod test{
    use super::*;
    use std::fs::File;
    use json::JsonValue::Array as jsonArray;
    use std::io::Read;

    #[test]
    fn test_course_from_json(){
        // Read from ./example_course_data_1.txt
        let mut file=File::open("./src/schedule/example_course_data_1.txt").unwrap();
        // Read all its contents
        let mut content=String::new();
        file.read_to_string(&mut content).unwrap();

        // Parse it as json
        let obj=json::parse(&content).unwrap();
        let rows=&obj["datas"]["cxxszhxqkb"]["rows"];
        let jsonArray(rows)=rows else{
            panic!("Not an array??");
        };
        for c in rows{
            let course=Course::from_json(c.clone()).unwrap();
            println!("{:#?}", course);
        }
    }

}
