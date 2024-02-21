mod commons;
mod config;
mod jira;
mod obsidian;

use time::macros::date;

use std::io::Write;

fn main() {
    let _ = config::CONFIG.set(config::Config::new()).unwrap();
    // test_jira();
    // create_many_notes("test_vault/jira");
    // test_create_note();
    test_get_notes();
    // println!("{:?}", config::CONFIG.get().unwrap());
}

fn test_day_planner() {
    let output = obsidian::read_day_planner("test_vault/2024-02-14.md");

    for i in output {
        println!("{:?}", i);
    }
}

fn test_get_notes() {
    println!("Hello, hello");

    let notes = obsidian::get_all_notes("test_vault/jira");

    for note in notes {
        println!("{:#?}", note);
    }
}

fn test_create_note() {
    let jira_props = obsidian::JiraProperties::new(
        commons::Priority::High,
        Some(commons::Date::new(date!(1975 - 04 - 12))),
        // None,
        commons::Status::InProgress,
        jira::JiraKey::new("MB-1004"),
        obsidian::TimeTrackingObsidian::new(Some("8:00"), Some("3:00"), Some("9:00")),
    );
    let properties = obsidian::Properties::new(jira_props);
    obsidian::create_obsidian_file("test_vault/replicant.md", Some(properties));
}

fn test_jira() {
    let response = jira::get_issues(30);
    // let all_issues = response.get("issues").unwrap().as_array().unwrap();
    println!("{:#?}", response);

    for issue in response.get_issues() {
        let summary = issue.get_fields().get_summary();
        let created = issue.get_fields().get_creation_date();
        let user = issue
            .get_fields()
            .get_reporter()
            .map(|x| x.get_display_name());
        let priority = issue.get_fields().get_priority();
        println!("{:#?}", issue);
    }

    /*
    let format = time::macros::format_description!(
        "[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond][offset_hour][offset_minute]"
    );
    let date = time::OffsetDateTime::parse("2024-02-09T19:15:59.009+0100", format);
    println!("{:?}", date)
    */
}

use std::path::PathBuf;
fn create_many_notes<P: AsRef<str>>(folder: P) {
    std::fs::create_dir(folder.as_ref()).unwrap();
    let response = jira::get_issues(20);
    let issues = response.get_issues().iter().map(|x| {
        let jira_props = obsidian::JiraProperties::from_jira_issue(x);
        let name = x.get_fields().get_summary().to_owned();
        let props = obsidian::Properties::new(jira_props);
        (name, props)
    });

    for (name, props) in issues {
        let path = PathBuf::new()
            .join(folder.as_ref())
            .join(name)
            .with_extension("md");
        println!("{:?}", path);
        obsidian::create_obsidian_file(path, Some(props))
    }
}
