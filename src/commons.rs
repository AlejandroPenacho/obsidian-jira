use serde::{self, Deserialize, Deserializer, Serialize, Serializer};
use time;
// Note to self: this should be a TryFrom, but I do not want to look up the error types
#[derive(Deserialize, Clone, Debug)]
#[serde(from = "&str")]
pub struct DateTime(time::OffsetDateTime);

impl From<&str> for DateTime {
    fn from(input: &str) -> DateTime {
        let format = time::macros::format_description!(
            "[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond][offset_hour][offset_minute]"
        );
        if input.chars().last().unwrap() == 'Z' {
            let mut new_input = input[..input.len() - 1].to_owned();
            new_input.push_str("+0000");
            return DateTime(time::OffsetDateTime::parse(&new_input, format).unwrap());
        } else {
            return DateTime(time::OffsetDateTime::parse(input, format).unwrap());
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(from = "&str")]
#[serde(into = "String")]
pub struct Date(time::Date);

impl From<&str> for Date {
    fn from(input: &str) -> Date {
        let format = time::macros::format_description!("[year]-[month]-[day]");
        let date = time::Date::parse(input, format).unwrap();
        return Date(date);
    }
}

impl Date {
    pub fn new(date: time::Date) -> Self {
        Date(date)
    }
}

impl Into<String> for Date {
    fn into(self) -> String {
        self.0.to_string()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Priority {
    VeryLow,
    Low,
    Medium,
    High,
    VeryHigh,
}

impl Priority {
    pub fn deserialize_from_number<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let input: u8 = Deserialize::deserialize(deserializer).unwrap();
        /*
        let input = serde_json::value::Number::deserialize(deserializer)
            .unwrap()
            .as_u64()
            .unwrap();
        */
        use Priority::*;
        Ok(match input {
            1 => VeryHigh,
            2 => High,
            3 => Medium,
            4 => Low,
            5 => VeryLow,
            _ => panic!(),
        })
    }
    pub fn serialize_to_number<S: Serializer>(
        input: &Self,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        use Priority::*;
        let number = match input {
            VeryLow => 5,
            Low => 4,
            Medium => 3,
            High => 2,
            VeryHigh => 1,
        };
        serializer.serialize_u8(number)
    }

    pub fn deserialize_from_jira_field<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Intermediate {
            id: String,
        }
        let input: Intermediate = Deserialize::deserialize(deserializer).unwrap();
        let number: u8 = input.id.parse().unwrap();
        /*
        let input: serde_json::Map<std::string::String, serde_json::Value> =
            serde_json::value::Map::deserialize(deserializer).unwrap();

        let value: &serde_json::Value = input.get("id").unwrap();
        let stringed = value.as_str().unwrap();
        let number: u8 = stringed.parse().unwrap();
        */

        use Priority::*;
        Ok(match number {
            1 => VeryHigh,
            2 => High,
            3 => Medium,
            4 => Low,
            5 => VeryLow,
            _ => panic!(),
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub enum Status {
    #[serde(rename = "To Do")]
    ToDo,
    #[serde(rename = "In Progress")]
    InProgress,
    Blocked,
    Done,
}

impl Status {
    pub fn deserialize_from_jira<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Intermediate {
            name: String,
        }
        let intermediate: Intermediate = Deserialize::deserialize(deserializer).unwrap();
        use Status::*;
        Ok(match intermediate.name.as_str() {
            "To Do" => ToDo,
            "In Progress" => InProgress,
            "Blocked" => Blocked,
            "Done" => Done,
            _ => panic!(),
        })
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum IssueType {
    Story,
    Task,
    SubTask,
    Epic,
}

impl IssueType {
    pub fn deserialize_from_jira<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Intermediate {
            name: String,
        }
        let intermediate: Intermediate = Deserialize::deserialize(deserializer).unwrap();
        use IssueType::*;
        Ok(match intermediate.name.as_str() {
            "Story" => Story,
            "Task" => Task,
            "Sub-task" => SubTask,
            "Epic" => Epic,
            _ => panic!(),
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Copy)]
#[serde(into = "String")]
#[serde(from = "&str")]
pub struct TimeEstimate(time::Duration);

impl Into<String> for TimeEstimate {
    fn into(self) -> String {
        let total_minutes = self.0.whole_minutes();
        let hours = total_minutes / 60;
        let minutes = total_minutes - hours * 60;
        format!("{}:{:0>2}", hours, minutes)
    }
}
impl From<&str> for TimeEstimate {
    fn from(input: &str) -> Self {
        let mut splitted = input.split(":");
        let hours = splitted.next().unwrap().parse::<i64>().unwrap();
        let minutes = match splitted.next() {
            None => 0,
            Some(x) => x.parse::<i64>().unwrap(),
        };
        Self(time::Duration::minutes(hours * 60 + minutes))
    }
}

impl TimeEstimate {
    pub fn from_secs(input: i64) -> Self {
        TimeEstimate(time::Duration::seconds(input as i64))
    }
    pub fn to_secs(&self) -> i64 {
        self.0.whole_seconds()
    }

    pub fn deserialize_from_secs<'de, D>(deserializer: D) -> Result<Option<Self>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let Some(seconds) = Deserialize::deserialize(deserializer).ok() else {
            return Ok(None);
        };
        Ok(Some(Self::from_secs(seconds)))
    }
}
