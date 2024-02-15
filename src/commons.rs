use serde::{self, Deserialize, Deserializer};
use time;
// Note to self: this should be a TryFrom, but I do not want to look up the error types
#[derive(Deserialize, Debug)]
#[serde(from = "&str")]
pub struct DateTime(time::OffsetDateTime);

impl From<&str> for DateTime {
    fn from(input: &str) -> DateTime {
        let format = time::macros::format_description!(
            "[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond][offset_hour][offset_minute]"
        );
        let date = time::OffsetDateTime::parse(input, format).unwrap();
        return DateTime(date);
    }
}

#[derive(Deserialize, Debug)]
#[serde(from = "&str")]
pub struct Date(time::Date);
impl From<&str> for Date {
    fn from(input: &str) -> Date {
        let format = time::macros::format_description!("[year]-[month]-[day]");
        let date = time::Date::parse(input, format).unwrap();
        return Date(date);
    }
}

#[derive(Debug)]
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
        let input = serde_json::value::Number::deserialize(deserializer)
            .unwrap()
            .as_u64()
            .unwrap();
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

    pub fn deserialize_from_jira_field<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let input: serde_json::Map<std::string::String, serde_json::Value> =
            serde_json::value::Map::deserialize(deserializer).unwrap();

        let value: &serde_json::Value = input.get("id").unwrap();
        let stringed = value.as_str().unwrap();
        let number: u8 = stringed.parse().unwrap();

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

/*
Eventually figure this out

pub fn get_priority_from_number<'de, D>(deserializer: D) -> Result<Priority, D::Error>
where
    D: Deserializer<'de>,
{
    let output = deserializer.deserialize_u8(visitor)?;
    return Ok(Priority::Medium);
}
*/
