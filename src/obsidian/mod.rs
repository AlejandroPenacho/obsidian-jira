pub mod planner;
pub mod task_file;

use std::collections::HashMap;

#[derive(Debug, Clone)]
struct TaskTimes {
    in_sprint: bool,
    remaining_time: time::Duration,
    uncompleted_time: time::Duration,
    completed_time: time::Duration,
}

fn get_sprint_balance(year: i32, iso_week: u8) -> HashMap<String, TaskTimes> {
    let mut sprint_tasks_filter = task_file::TaskFilter::new();

    let sprint = crate::commons::Sprint::from(
        format!("Y{}W{:0<2}", &year.to_string()[2..], iso_week).as_str(),
    );
    let first_day = time::Date::from_iso_week_date(2024, iso_week, time::Weekday::Monday)
        .unwrap()
        .into();
    let last_day = time::Date::from_iso_week_date(2024, iso_week, time::Weekday::Sunday)
        .unwrap()
        .into();

    let sprint_tasks = sprint_tasks_filter
        .set_sprints(&[sprint])
        .set_path(crate::config::CONFIG.get_project_path())
        .get_tasks();

    let sprint_schedule = planner::TaskSchedule::new(&first_day, &last_day);

    let mut task_times: HashMap<String, TaskTimes> = HashMap::new();

    for task in sprint_tasks {
        let task_name = task.get_name();
        let remaining_time = task.get_remaining_time();
        let uncompleted_time = sprint_schedule
            .get_task_time_allocation(&task_name)
            .map(|x| x.get_uncompleted_time())
            .unwrap_or(time::Duration::ZERO);

        let completed_time = sprint_schedule
            .get_task_time_allocation(&task_name)
            .map(|x| x.get_completed_time())
            .unwrap_or(time::Duration::ZERO);

        task_times.insert(
            task_name,
            TaskTimes {
                in_sprint: true,
                remaining_time,
                uncompleted_time,
                completed_time,
            },
        );
    }

    for (task_name, time_allocation) in sprint_schedule.iter_time_allocations() {
        if task_times.contains_key(task_name) {
            continue;
        }
        task_times.insert(
            task_name.clone(),
            TaskTimes {
                in_sprint: false,
                remaining_time: time::Duration::ZERO,
                uncompleted_time: time_allocation.get_uncompleted_time(),
                completed_time: time_allocation.get_completed_time(),
            },
        );
    }

    task_times
}

fn fmt_duration(input: time::Duration, signed: bool) -> String {
    let x: time::Duration;
    let sign: &str;
    if input.is_positive() {
        x = input;
        sign = "+";
    } else {
        x = -input;
        sign = "-";
    }
    let hours = x.whole_hours();
    let minutes = x.whole_minutes() - 60 * hours;

    if signed {
        format!("{}{:>2}:{:0>2}", sign, hours, minutes)
    } else {
        format!("{:>2}:{:0>2}", hours, minutes)
    }
}

fn print_task_balance(name: &str, task_times: &TaskTimes, name_size: usize) {
    if task_times.in_sprint {
        println!(
            "{:<4$}   {:>5}   {:>5}   {:>6}",
            name,
            fmt_duration(task_times.remaining_time, false),
            fmt_duration(
                task_times.uncompleted_time + task_times.completed_time,
                false
            ),
            fmt_duration(
                task_times.uncompleted_time + task_times.completed_time - task_times.remaining_time,
                true
            ),
            name_size
        )
    } else {
        println!(
            "{:<3$}   {:>5}   {:>5}",
            name,
            String::from("  /  "),
            fmt_duration(
                task_times.uncompleted_time + task_times.completed_time,
                false
            ),
            name_size
        )
    }
}

pub fn print_sprint_balance(year: i32, iso_week: u8) {
    let sprint_data = get_sprint_balance(year, iso_week);
    let mut sprint_data: Vec<(String, TaskTimes)> = sprint_data
        .iter()
        .map(|(a, b)| (a.clone(), b.clone()))
        .collect();

    let sprint_remaining_time: time::Duration = sprint_data
        .iter()
        .filter_map(|(_, task_times)| {
            if task_times.in_sprint {
                Some(task_times.remaining_time)
            } else {
                None
            }
        })
        .sum();

    let sprint_allocated_time: time::Duration = sprint_data
        .iter()
        .filter_map(|(_, task_times)| {
            if task_times.in_sprint {
                Some(task_times.uncompleted_time + task_times.completed_time)
            } else {
                None
            }
        })
        .sum();

    let total_allocated_time: time::Duration = sprint_data
        .iter()
        .map(|(_, task_times)| task_times.uncompleted_time + task_times.completed_time)
        .sum();

    use std::cmp::Ordering::*;

    sprint_data.sort_by(|a, b| match (a.1.in_sprint, b.1.in_sprint) {
        (true, true) => match a.1.remaining_time.cmp(&b.1.remaining_time) {
            Equal => a.0.cmp(&b.0),
            x => return x,
        },
        (true, false) => Greater,
        (false, true) => Less,
        (false, false) => a.0.partial_cmp(&b.0).unwrap(),
    });

    let max_task_name = sprint_data.iter().map(|x| x.0.len()).max().unwrap() + 5;
    let separator = std::iter::repeat("-")
        .take(max_task_name + 27)
        .collect::<String>();

    println!(
        "{:<4$}   {:<5}   {:<5}   {:<6}",
        "Task name", "Rem", "Alloc", "Diff", max_task_name
    );
    println!("{}", separator);
    for task in sprint_data.iter().rev().filter(|x| x.1.in_sprint) {
        print_task_balance(&task.0, &task.1, max_task_name)
    }

    println!("{}", separator);
    println!(
        "{:<4$}   {:>5}   {:>5}   {:>6}",
        "",
        fmt_duration(sprint_remaining_time, false),
        fmt_duration(sprint_allocated_time, false),
        fmt_duration(sprint_allocated_time - sprint_remaining_time, true),
        max_task_name
    );

    println!();

    for task in sprint_data.iter().rev().filter(|x| !x.1.in_sprint) {
        print_task_balance(&task.0, &task.1, max_task_name)
    }
    println!("{}", separator);
    println!(
        "{:<3$}   {:<5}   {:<5}",
        "     ",
        "",
        fmt_duration(total_allocated_time, false),
        max_task_name
    );
}
