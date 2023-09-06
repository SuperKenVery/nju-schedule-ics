use std::error::Error;
use json;
use super::time::{TimeSpan, CourseTime,Ts,Time};
use std::num::ParseIntError;

#[derive(Debug)]
pub struct Course {
    pub name: String,
    pub location: String,
    pub notes: String,

    pub time: Vec<CourseTime>,
}

impl Course {
    pub fn from_json(raw: json::JsonValue) -> Result<Self, Box<dyn Error>> {
        let name=raw["KCM"].as_str().ok_or("Cannot extract name")?;
        let location=raw["JASMC"]
            .as_str().ok_or("Cannot extract location")?
            .replace("（合班）", "");

        let time=raw["YPSJDD"].as_str().ok_or("Cannot extract time")?;
        /* Example data:
         * 周三 2-4节 1-17周 仙Ⅱ-211,周五 3-4节 1-17周 仙Ⅱ-211
         * 自由时间 0-0节 7-17周 自由地点
         * 周五 7-8节 3周,7周,11周,15周 仙Ⅰ-108
         * 周四 3-4节 1-17周 仙Ⅱ-212,周四 9-10节 1-17周 基础实验楼乙124,125,周一 3-4节 1-17周 仙Ⅱ-212
         */
        let time=time.replace("周,", "周|");
        /* Now they are:
         * 周三 2-4节 1-17周 仙Ⅱ-211,周五 3-4节 1-17周 仙Ⅱ-211
         * 自由时间 0-0节 7-17周 自由地点
         * 周五 7-8节 3周|7周|11周|15周 仙Ⅰ-108
         * 周四 3-4节 1-17周 仙Ⅱ-212,周四 9-10节 1-17周 基础实验楼乙124,125,周一 3-4节 1-17周 仙Ⅱ-212
         */
        let time=time.replace(",周","##周");
        /* Now they are:
         * 周三 2-4节 1-17周 仙Ⅱ-211,周五 3-4节 1-17周 仙Ⅱ-211
         * 自由时间 0-0节 7-17周 自由地点
         * 周五 7-8节 3周|7周|11周|15周 仙Ⅰ-108
         * 周四 3-4节 1-17周 仙Ⅱ-212##周四 9-10节 1-17周 基础实验楼乙124,125##周一 3-4节 1-17周 仙Ⅱ-212
         */
        let times=time.split("##").into_iter()
            .map(|time| {
                // Weekday
                let [weekday, time, weeks, location]=time.split(" ").collect::<Vec<&str>>()[..] else {
                    return Err::<_,Box<dyn Error>>("Invalid weekday range str".into());
                };

                if weekday.contains("自由时间"){
                    return Ok(vec![]);
                }
                let weekday: Result<u8, Box<dyn Error>>=match weekday {
                    "周一" => Ok(1),
                    "周二" => Ok(2),
                    "周三" => Ok(3),
                    "周四" => Ok(4),
                    "周五" => Ok(5),
                    "周六" => Ok(6),
                    "周日" => Ok(7),
                    _ => Err("Invalid weekday".into()),
                };
                let weekday=weekday?;

                // Weeks
                let weeks=if weeks.contains("-"){
                    // Chinese character takes 3 bytes
                    let weeks=&weeks[0..weeks.len()-3];
                    let [start, end]=weeks
                        .split("-")
                        .map(|week| week.parse::<u8>())
                        .collect::<Result<Vec<_>,ParseIntError>>()?[..] else{
                            return Err::<_,Box<dyn Error>>("Invalid week range str".into());
                        };
                    let weeks=(start..=end).collect::<Vec<u8>>();
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
                    .collect::<Result<Vec<_>,Box<dyn Error>>>()?[..] else{
                        return Err::<_,Box<dyn Error>>("Invalid time range str".into());
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
                // TODO: Fetch nodes
                notes: "".to_string(),
                time: times,
            }
        )
    }
}



#[cfg(test)]
mod test{
    use super::*;
    use std::fs::File;
    use json::JsonValue::Array as jsonArray;
    use std::io::Read;

    #[test]
    fn test_ts_macro(){
        let ts=Ts!("01:2", "3:4");
        assert_eq!(ts, TimeSpan::new(Time::new(1, 2), Time::new(3, 4)));

        let ts=Ts!("10:30", "11:30");
        assert_eq!(ts, TimeSpan::new(Time::new(10, 30), Time::new(11, 30)));
    }

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
