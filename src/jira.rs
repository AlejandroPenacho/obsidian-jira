use reqwest;
use serde::{Deserialize, Deserializer};
// use serde_json::Value;
use std::fs::read_to_string;

use std::io::Write;

use crate::commons::{Date, DateTime, IssueType, Priority, Status};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JiraResponse {
    expand: String,
    issues: Vec<JiraIssue>,
    max_results: u32,
    start_at: u32,
    total: u32,
}

impl JiraResponse {
    pub fn get_issues(&self) -> &[JiraIssue] {
        &self.issues
    }
}

#[derive(Deserialize, Debug)]
pub struct JiraIssue {
    expand: String,
    fields: JiraIssueFields,
    id: String,
    key: JiraKey,
    #[serde(rename = "self")]
    url: String,
}

impl JiraIssue {
    pub fn get_fields(&self) -> &JiraIssueFields {
        &self.fields
    }
}

#[derive(Deserialize, Debug)]
pub struct JiraIssueFields {
    summary: String,
    // description: Option<String>,
    #[serde(rename = "customfield_10035")]
    story_points: Option<f32>,
    #[serde(rename = "issuetype")]
    #[serde(deserialize_with = "IssueType::deserialize_from_jira")]
    issue_type: IssueType,
    creator: User,
    reporter: Option<User>,
    assignee: Option<User>,
    created: DateTime,
    updated: DateTime,
    #[serde(rename = "duedate")]
    due_date: Option<Date>,
    #[serde(deserialize_with = "Priority::deserialize_from_jira_field")]
    priority: Priority,
    #[serde(flatten)]
    time_tracking: TimeTracking,
    #[serde(deserialize_with = "Status::deserialize_from_jira")]
    status: Status,
    #[serde(rename = "customfield_10020")]
    #[serde(deserialize_with = "JiraBoard::deserialize_vec_from_jira")]
    boards: Vec<JiraBoard>,
    #[serde(deserialize_with = "JiraKey::deserialize_parent")]
    #[serde(rename = "parent")]
    #[serde(default)]
    parent_key: Option<JiraKey>,
}

impl JiraIssueFields {
    pub fn get_summary(&self) -> &str {
        &self.summary
    }

    pub fn get_creation_date(&self) -> &DateTime {
        &self.created
    }

    pub fn get_reporter(&self) -> Option<&User> {
        self.reporter.as_ref()
    }

    pub fn get_priority(&self) -> &Priority {
        &self.priority
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct User {
    account_id: String,
    display_name: String,
    #[serde(rename = "self")]
    url: String,
}

impl User {
    pub fn get_display_name(&self) -> &str {
        &self.display_name
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JiraBoard {
    id: u8,
    board_id: u8,
    start_date: DateTime,
    end_date: DateTime,
    complete_data: Option<DateTime>,
    name: String,
    state: String,
    goal: String,
}

impl JiraBoard {
    fn deserialize_vec_from_jira<'de, D>(deserializer: D) -> Result<Vec<JiraBoard>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let output: Vec<JiraBoard> =
            Deserialize::deserialize(deserializer).unwrap_or_else(|_| Vec::new());
        Ok(output)
    }
}

// state is either closed, future

#[derive(Debug, Deserialize)]
pub struct TimeTracking {
    #[serde(rename = "timeestimate")]
    time_estimate: Option<u32>,
    #[serde(rename = "timeoriginalestimate")]
    time_original_estimate: Option<u32>,
    #[serde(rename = "timespent")]
    time_spent: Option<u32>,
}

#[derive(Deserialize, Debug)]
pub struct JiraKey(String);

impl JiraKey {
    fn deserialize_parent<'de, D>(deserializer: D) -> Result<Option<JiraKey>, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Parent {
            key: String,
        }
        let mid: Parent = Deserialize::deserialize(deserializer)?;
        Ok(Some(JiraKey(mid.key)))
    }
}

fn get_jira_auth() -> (String, String) {
    (
        "alejpr@kth.se".to_owned(),
        read_to_string("token.txt").unwrap(),
    )
}

pub fn get_issues(max_results: u32) -> JiraResponse {
    let client = reqwest::blocking::Client::new();
    let query = [
        ("maxResults", max_results.to_string()),
        ("jql", String::from("assignee=635bd5b8fe5ff375235a8a6c")),
    ];
    let auth = get_jira_auth();
    client
        .get("https://barreau.atlassian.net/rest/api/3/search")
        .basic_auth(auth.0, Some(auth.1))
        .query(&query)
        .send()
        .unwrap()
        .json::<JiraResponse>()
        .unwrap()
}
