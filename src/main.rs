mod commons;
mod jira;
mod obsidian;

use std::io::Write;

fn main() {
    test_jira();
    // test_get_notes();
}

fn test_day_planner() {
    let output = obsidian::read_day_planner("test_vault/2024-02-14.md");

    for i in output {
        println!("{:?}", i);
    }
}

fn test_get_notes() {
    println!("Hello, hello");

    let notes = obsidian::get_all_notes("test_vault");

    for note in notes {
        println!("{:?}", note);
    }
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
