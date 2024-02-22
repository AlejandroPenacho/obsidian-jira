mod commons;
mod config;
mod jira;
mod obsidian;

use time::macros::date;

use std::io::Write;

fn main() {
    let _ = config::CONFIG.set(config::Config::new()).unwrap();
    // test_jira();
    // get_raw();
    create_many_notes("test_vault/jira");
    // test_create_note();
    // test_get_notes();
    // test_sprint(20);
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
        commons::IssueType::Task,
        commons::Status::InProgress,
        jira::JiraKey::new("MB-1004"),
        obsidian::TimeTrackingObsidian::new(Some("8:00"), Some("3:00"), Some("9:00")),
        vec![jira::Sprint::new("LA WEA".to_owned())],
        Some(String::from("HAGAN SITIO")),
        vec![String::from("ENEL"), String::from("JACUZZI")],
    );
    let properties = obsidian::Properties::new(jira_props);
    obsidian::create_obsidian_file("test_vault/replicant.md", Some(properties));
}

fn test_sprint(max_results: u32) -> () {
    let url = format!(
        "https://{}.atlassian.net/rest/agile/1.0/board/5/sprint",
        crate::config::CONFIG.get().unwrap().get_jira_url()
    );

    let client = reqwest::blocking::Client::new();

    let query = [
        ("maxResults", max_results.to_string()),
        /*
        (
            "jql",
            format!(
                "assignee={}",
                crate::config::CONFIG.get().unwrap().get_user_id(),
            ),
        ),
        */
    ];

    let auth_mail = crate::config::CONFIG.get().unwrap().get_user_mail();
    let auth_token = crate::config::CONFIG.get().unwrap().get_jira_token();

    let output = client
        .get(url)
        .basic_auth(auth_mail, Some(auth_token))
        .query(&query)
        .send()
        .unwrap()
        .json::<serde_json::Value>()
        .unwrap();

    println!("{:#?}", output);
}

fn get_raw() {
    let max_results = 200;
    let url = format!(
        "https://{}.atlassian.net/rest/api/2/search",
        crate::config::CONFIG.get().unwrap().get_jira_url()
    );

    let client = reqwest::blocking::Client::new();

    let query = [
        ("maxResults", max_results.to_string()),
        /*
        (
            "jql",
            format!(
                "assignee={}",
                crate::config::CONFIG.get().unwrap().get_user_id(),
            ),
        ),
        */
    ];

    let auth_mail = crate::config::CONFIG.get().unwrap().get_user_mail();
    let auth_token = crate::config::CONFIG.get().unwrap().get_jira_token();

    let output = client
        .get(url)
        .basic_auth(auth_mail, Some(auth_token))
        .query(&query)
        .send()
        .unwrap()
        .json::<serde_json::Value>()
        .unwrap();

    let mut file = std::fs::File::create("docs/example_issue_response_v2.json").unwrap();
    file.write(format!("{:#?}", output).as_bytes()).unwrap();
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
