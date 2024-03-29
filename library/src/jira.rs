use reqwest;

use serde::{Deserialize, Deserializer, Serialize};
// use serde_json::Value;


use crate::commons::{Date, DateTime, IssueType, Priority, Sprint, Status, TimeEstimate};

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
    pub fn get_key(&self) -> &JiraKey {
        &self.key
    }
}

#[derive(Deserialize, Debug)]
pub struct JiraIssueFields {
    summary: String,
    #[serde(deserialize_with = "deserialize_description")]
    description: String,
    // #[serde(rename = "customfield_10035")]
    // story_points: Option<f32>,
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
    time_tracking: TimeTrackingJira,
    #[serde(deserialize_with = "Status::deserialize_from_jira")]
    status: Status,
    #[serde(rename = "customfield_10020")]
    #[serde(deserialize_with = "Sprint::deserialize_sprint_vec_from_jira")]
    sprints: Vec<Sprint>,
    #[serde(default)]
    parent: Option<IssueIdentifier>,
    #[serde(rename = "subtasks")]
    children: Vec<IssueIdentifier>,
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

    pub fn get_status(&self) -> &Status {
        &self.status
    }

    pub fn get_issue_type(&self) -> &IssueType {
        &self.issue_type
    }

    pub fn get_due_date(&self) -> Option<&Date> {
        self.due_date.as_ref()
    }

    pub fn get_time_tracking(&self) -> &TimeTrackingJira {
        &self.time_tracking
    }

    pub fn get_sprints(&self) -> &[Sprint] {
        &self.sprints
    }

    pub fn get_parent(&self) -> Option<&IssueIdentifier> {
        self.parent.as_ref()
    }
    pub fn get_children(&self) -> &[IssueIdentifier] {
        &self.children
    }
}

fn deserialize_description<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    Deserialize::deserialize(deserializer).or_else(|_| Ok(String::new()))
}

#[derive(Debug)]
pub struct IssueIdentifier {
    key: JiraKey,
    name: String,
}

impl IssueIdentifier {
    pub fn get_key(&self) -> &JiraKey {
        &self.key
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }
}

impl<'de> Deserialize<'de> for IssueIdentifier {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Mid {
            key: String,
            fields: MidFields,
        }
        #[derive(Deserialize)]
        struct MidFields {
            summary: String,
        }

        let mid: Mid = Deserialize::deserialize(deserializer)?;

        Ok(IssueIdentifier {
            key: JiraKey(mid.key),
            name: mid.fields.summary,
        })
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

// state is either closed, future

#[derive(Debug, Deserialize, Clone)]
pub struct TimeTrackingJira {
    #[serde(rename = "timeestimate")]
    #[serde(deserialize_with = "TimeEstimate::deserialize_from_secs")]
    left: Option<TimeEstimate>,
    #[serde(rename = "timeoriginalestimate")]
    #[serde(deserialize_with = "TimeEstimate::deserialize_from_secs")]
    original: Option<TimeEstimate>,
    #[serde(rename = "timespent")]
    #[serde(deserialize_with = "TimeEstimate::deserialize_from_secs")]
    spent: Option<TimeEstimate>,
}

impl TimeTrackingJira {
    pub fn get_time_left(&self) -> Option<&TimeEstimate> {
        self.left.as_ref()
    }
    pub fn get_time_original(&self) -> Option<&TimeEstimate> {
        self.original.as_ref()
    }
    pub fn get_time_spent(&self) -> Option<&TimeEstimate> {
        self.spent.as_ref()
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct JiraKey(String);

impl JiraKey {
    pub fn new(key: &str) -> Self {
        Self(key.to_owned())
    }
}

pub fn get_issues(max_results: u32) -> JiraResponse {
    let url = format!(
        "https://{}.atlassian.net/rest/api/2/search",
        crate::config::CONFIG.get_jira_url()
    );

    let client = reqwest::blocking::Client::new();

    let query = [
        ("maxResults", max_results.to_string()),
        (
            "jql",
            format!("assignee={}", crate::config::CONFIG.get_user_id(),),
        ),
    ];

    let auth_mail = crate::config::CONFIG.get_user_mail();
    let auth_token = crate::config::CONFIG.get_jira_token();

    client
        .get(url)
        .basic_auth(auth_mail, Some(auth_token))
        .query(&query)
        .send()
        .unwrap()
        .json::<JiraResponse>()
        .unwrap()
}
