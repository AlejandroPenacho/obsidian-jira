use std::fs::read_to_string;

use reqwest;
use serde::Deserialize;
use serde_json::Value;

use time;

// Note to self: this should be a TryFrom, but I do not want to look up the error types
#[derive(Deserialize, Debug)]
#[serde(from = "&str")]
struct DateTime(time::OffsetDateTime);

impl From<&str> for DateTime {
    fn from(input: &str) -> DateTime {
        let format = time::macros::format_description!(
            "[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond][offset_hour][offset_minute]"
        );
        let date = time::OffsetDateTime::parse(input, format).unwrap();
        return DateTime(date);
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct JiraResponse {
    expand: String,
    issues: Vec<JiraIssue>,
    max_results: u32,
    start_at: u32,
    total: u32,
}

#[derive(Deserialize)]
struct JiraIssue {
    expand: String,
    fields: JiraIssueFields,
    id: String,
    key: String,
    #[serde(rename = "self")]
    url: String,
}

#[derive(Deserialize)]
struct JiraIssueFields {
    summary: String,
    description: Option<String>,
    #[serde(rename = "customfield_10035")]
    expected_time: Option<f32>,
    #[serde(rename = "issuetype")]
    issue_type: IssueType,
    creator: User,
    reporter: Option<User>,
    assignee: Option<User>,
    created: DateTime,
    updated: DateTime,
    #[serde(rename = "duedate")]
    due_data: Option<String>,
    /*
    priority: IssuePriority,
    project: Project,
    status: Status
    */
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct User {
    account_id: String,
    display_name: String,
    #[serde(rename = "self")]
    url: String,
}

#[derive(Deserialize)]
struct IssueType {
    id: String,
    name: String,
    #[serde(rename = "self")]
    url: String,
}

fn get_jira_auth() -> (String, String) {
    (
        "alejpr@kth.se".to_owned(),
        read_to_string("token.txt").unwrap(),
    )
}

fn main() {
    let client = reqwest::blocking::Client::new();
    let auth = get_jira_auth();
    let response = client
        .get("https://barreau.atlassian.net/rest/api/2/search?maxResults=10")
        .basic_auth(auth.0, Some(auth.1))
        .send()
        .unwrap()
        .json::<JiraResponse>()
        .unwrap();

    // let all_issues = response.get("issues").unwrap().as_array().unwrap();

    for issue in response.issues {
        let summary = issue.fields.summary;
        let created = issue.fields.created;
        let user = issue.fields.reporter.map(|x| x.display_name);

        println!("{:?} => {:?}\n\t{:?}\n", user, created, summary);
    }

    /*
    let format = time::macros::format_description!(
        "[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond][offset_hour][offset_minute]"
    );
    let date = time::OffsetDateTime::parse("2024-02-09T19:15:59.009+0100", format);
    println!("{:?}", date)
    */
}
