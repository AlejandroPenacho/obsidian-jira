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

#[derive(Deserialize, Debug)]
#[serde(from = "u8")]
pub enum Priority {
    VeryLow,
    Low,
    Medium,
    High,
    VeryHigh,
}
impl From<u8> for Priority {
    fn from(input: u8) -> Priority {
        use Priority::*;
        match input {
            1 => VeryLow,
            2 => Low,
            3 => Medium,
            4 => High,
            5 => VeryHigh,
            _ => panic!(),
        }
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
