use crate::commons::{Date, DateTime};
use std::path::Path;
use time::macros::format_description;
use time::{Duration, Time};

use std::fs::read_to_string;

#[derive(Debug)]
pub struct PlannedTask {
    start: Time,
    end: Time,
    date: Date,
    name: String,
    linked: bool,
    completed: bool,
}

pub fn read_period_times(
    start_date: &Date,
    end_date: &Date,
) -> std::collections::HashMap<String, time::Duration> {
    let planned_tasks = read_period_plan(start_date, end_date);

    let mut tasks: std::collections::HashMap<String, time::Duration> =
        std::collections::HashMap::new();

    for planned in planned_tasks {
        let name = planned.name;
        let duration = planned.end - planned.start;

        if tasks.contains_key(&name) {
            *tasks.get_mut(&name).unwrap() += duration;
        } else {
            tasks.insert(name, duration);
        }
    }

    tasks
}

pub fn read_period_plan(start_date: &Date, end_date: &Date) -> Vec<PlannedTask> {
    let mut output = Vec::new();

    for date in crate::commons::DateIterator::new(start_date, end_date) {
        let mut tasks = read_day_plan(&date);
        if let Some(mut tasks) = tasks {
            output.append(&mut tasks);
        }
    }

    output
}

pub fn read_day_plan(date: &Date) -> Option<Vec<PlannedTask>> {
    // println!("{:?}", date);
    let vault_path = crate::config::CONFIG.get_vault_path();
    let day_planner_path = crate::config::CONFIG.get_daily_notes_path();
    let date_string: String = date.clone().into();

    let mut total_path: std::path::PathBuf = [vault_path, day_planner_path, &date_string]
        .iter()
        .collect();

    total_path.set_extension("md");

    let time_format = format_description!("[hour]:[minute]");
    let mut output = Vec::new();
    let text = read_to_string(total_path).ok()?;
    /*
        Okay, so here comes the explanation:
        ~ - Matches the dash at the beginning of the task
        ~ \[(.)\]        Matches one character surrounded by [], so [ ] and [X] are valid
                         The inside is the capture group 1
        ~ \s+            Matches as much whitespace as needed
        ~ (\d+:\d\d)     Matches a time as capture group 2, like 9:30 or 18:15
        ~ \s*-\s*        A dash surrounded by arbitrary whitespace
        ~ (\d+:\d\d)     Again, matches a time, this time for capture group 3
        ~ \s*            As much whitespace as needed
        ~ (.*)           One last match of anything
    */
    let re = regex::Regex::new(r"- \[(.)\]\s+(\d+:\d\d)\s*-\s*(\d+:\d\d)\s*(.*)").unwrap();
    for line in text.lines() {
        let Some(capture) = re.captures(line) else {
            continue;
        };
        // println!("{:?}", capture);

        let completed = !(capture.get(1).unwrap().as_str() == " ");
        let start = Time::parse(capture.get(2).unwrap().as_str(), time_format).unwrap();
        let end = Time::parse(capture.get(3).unwrap().as_str(), time_format).unwrap();
        let mut name = capture.get(4).unwrap().as_str().to_owned();
        // This is a mess, do it right later
        let linked: bool;
        if &name[..2] == "[[" {
            name = name[2..name.len() - 2].to_owned();
            linked = true;
        } else {
            linked = false;
        }

        output.push(PlannedTask {
            start,
            end,
            date: date.clone(),
            name,
            linked,
            completed,
        })
    }

    Some(output)
}

#[cfg(test)]
mod test {
    #[test]
    fn read_simple_day() {
        use super::read_day_plan;
        let date = crate::commons::Date::from("2024-02-14");

        let tasks = read_day_plan(&date).unwrap();
        for task in tasks {
            println!("{:#?}", task);
        }
    }
    #[test]
    fn read_week() {
        use super::read_period_plan;
        let start_date = crate::commons::Date::from("2024-02-26");
        let end_date = crate::commons::Date::from("2024-03-01");

        let all_tasks = read_period_plan(&start_date, &end_date);

        for task in all_tasks {
            println!("{:#?}", task)
        }
    }

    #[test]
    fn get_week_time_allocation() {
        use super::read_period_times;
        let start_date = crate::commons::Date::from("2024-03-04");
        let end_date = crate::commons::Date::from("2024-03-08");

        let tasks = read_period_times(&start_date, &end_date);

        for pair in tasks.iter() {
            let name = pair.0;
            let duration = pair.1;
            let hours = duration.whole_hours();
            let minutes = duration.whole_minutes() - hours * 60;
            println!("{:0>2}:{:0>2}\t\t{}", hours, minutes, name)
        }
    }
}
