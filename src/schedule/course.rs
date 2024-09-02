use anyhow::anyhow;
use json;
use json::JsonValue::Array as jsonArray;
use super::time::{TimeSpan, CourseTime};

#[derive(Debug)]
pub struct Course {
    pub name: String,
    pub location: String,
    pub notes: String,

    pub time: Vec<CourseTime>,
}

impl Course {
    pub fn from_json(raw: json::JsonValue) -> Result<Self, anyhow::Error> {
        // Notes
        let line_or_empty=|key: &str| {
            let content=raw[key].as_str();

            if let Some(content)=content{
                format!("{}\n", content)
            }else{
                "".to_string()
            }
        };
        let notes=line_or_empty("SKSM");
        let swaps=line_or_empty("TKJG");
        let final_exam=line_or_empty("QMKSXX");
        let class=line_or_empty("JXBMC");
        let teacher=line_or_empty("JSHS");
        let points=line_or_empty("XF");

        // Name and location
        let name=raw["KCM"].as_str()
            .ok_or("Cannot extract name").map_err(anyhow::Error::msg)?;
        let location=raw["JASMC"]
            .as_str()
            .unwrap_or("")      // 比如阅读课就会没有这个字段
            .replace("（合班）", "");

        // Time
        let start=raw["KSJC"].as_str()
                    .ok_or("Cannot extract start time").map_err(anyhow::Error::msg)?.parse::<u8>()?;
        let end=raw["JSJC"].as_str()
            .ok_or("Cannot extract end time").map_err(anyhow::Error::msg)?.parse::<u8>()?;
        let weekday=raw["SKXQ"].as_str()
            .ok_or("Cannot extract weekday").map_err(anyhow::Error::msg)?.parse::<u8>()?;

        let weeks=raw["SKZC"].as_str()
            .ok_or("Cannot extract weeks").map_err(anyhow::Error::msg)?;
        let times=if start!=0{
            weeks.chars().enumerate()
            .map(|(i, c)| (i, c))
            .filter(|(_, c)| *c=='1')
            .map(|(i, _c)| {
                let week=i+1;

                Ok(CourseTime::new(
                    TimeSpan::from_course_index_range(start, end)?,
                    weekday,
                    week as u8,
                ))
            })
            .collect::<Result<Vec<_>,anyhow::Error>>()?
        }else{  // 自由时间的课程，开始结束会设为0
            vec![]
        };

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
            let course=Course::from_json(c.clone());
            println!("{:#?}", course);
            let _course=course.unwrap();
        }
    }

}
