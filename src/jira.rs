use reqwest;
use serde::Deserialize;
// use serde_json::Value;
use std::fs::read_to_string;

use crate::commons::{Date, DateTime, Priority};

#[derive(Deserialize)]
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

#[derive(Deserialize)]
pub struct JiraIssue {
    expand: String,
    fields: JiraIssueFields,
    id: String,
    key: String,
    #[serde(rename = "self")]
    url: String,
}

impl JiraIssue {
    pub fn get_fields(&self) -> &JiraIssueFields {
        &self.fields
    }
}

#[derive(Deserialize)]
pub struct JiraIssueFields {
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
    due_date: Option<Date>,
    #[serde(deserialize_with = "Priority::deserialize_from_jira_field")]
    priority: Priority,
    /*
    project: Project,
    status: Status
    */
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

#[derive(Deserialize)]
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

#[derive(Deserialize)]
pub struct IssueType {
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

pub fn get_issues(max_results: u32) -> JiraResponse {
    let client = reqwest::blocking::Client::new();
    let auth = get_jira_auth();
    client
        .get(format!(
            "https://barreau.atlassian.net/rest/api/2/search?maxResults={}",
            max_results,
        ))
        .basic_auth(auth.0, Some(auth.1))
        .send()
        .unwrap()
        .json::<JiraResponse>()
        .unwrap()
}
