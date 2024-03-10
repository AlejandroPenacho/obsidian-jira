use crate::commons::{Date, DateTime};
use std::collections::HashMap;
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

#[derive(Default, Debug)]
pub struct TimeAllocation {
    uncompleted_time: time::Duration,
    completed_time: time::Duration,
}

impl TimeAllocation {
    pub fn get_uncompleted_time(&self) -> time::Duration {
        self.uncompleted_time
    }
    pub fn get_completed_time(&self) -> time::Duration {
        self.completed_time
    }
}

#[derive(Debug)]
pub struct TaskSchedule {
    planned_tasks: Vec<PlannedTask>,
    time_allocations: HashMap<String, TimeAllocation>,
}

impl TaskSchedule {
    pub fn new(start_date: &Date, end_date: &Date) -> Self {
        let mut planned_tasks = Vec::new();

        for date in crate::commons::DateIterator::new(start_date, end_date) {
            let mut tasks = read_day_plan(&date);
            if let Some(mut tasks) = tasks {
                planned_tasks.append(&mut tasks);
            }
        }

        let mut time_allocations = HashMap::new();
        for planned_task in planned_tasks.iter() {
            let allocation = time_allocations
                .entry(planned_task.name.clone())
                .or_insert_with(|| TimeAllocation::default());

            if planned_task.completed {
                allocation.completed_time += planned_task.end - planned_task.start;
            } else {
                allocation.uncompleted_time += planned_task.end - planned_task.start;
            }
        }
        Self {
            planned_tasks,
            time_allocations,
        }
    }

    pub fn get_task_time_allocation(&self, task_name: &str) -> Option<&TimeAllocation> {
        self.time_allocations.get(task_name)
    }

    pub fn iter_time_allocations(&self) -> impl Iterator<Item = (&String, &TimeAllocation)> {
        self.time_allocations.iter()
    }
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
        let start_date = crate::commons::Date::from("2024-02-26");
        let end_date = crate::commons::Date::from("2024-03-01");

        let schedule = super::TaskSchedule::new(&start_date, &end_date);

        println!("{:#?}", schedule);
    }
}
