use std::fs::read_dir;
use std::io::Write;
use std::path::{Path, PathBuf};

use std::collections::HashMap;
use std::fs::read_to_string;

use crate::commons::{Date, IssueType, Priority, Sprint, Status, TimeEstimate};
use crate::jira::TimeTrackingJira;

use time::{Duration, Time};

use regex;
use serde;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_yaml;
use time::macros::format_description;

#[derive(Debug)]
pub struct ObsidianFile {
    path: PathBuf,
    content: String,
    properties: Option<Properties>,
}

impl ObsidianFile {
    pub fn read_file<P: AsRef<Path>>(path: P) -> Self {
        let mut complete_path = PathBuf::new();
        complete_path.push(crate::config::CONFIG.get_vault_path());
        complete_path.push(&path);
        complete_path.set_extension("md");

        let full_content = read_to_string(&complete_path).unwrap();
        let has_properties = full_content.starts_with("---");

        let content: String;
        let properties: Option<Properties>;

        if has_properties {
            let end_of_properties = full_content[3..].find("---").unwrap() + 3;
            properties = Some(serde_yaml::from_str(&full_content[3..end_of_properties]).unwrap());
            content = full_content[(end_of_properties + 4)..].to_owned();
        } else {
            properties = None;
            content = full_content.to_owned();
        }

        return ObsidianFile {
            path: path.as_ref().to_owned(),
            content,
            properties,
        };
    }

    pub fn save_file(&self) -> Result<(), ()> {
        let mut complete_path = PathBuf::new();
        complete_path.push(crate::config::CONFIG.get_vault_path());
        complete_path.push(&self.path);
        complete_path.set_extension("md");

        let mut file = std::fs::File::create(complete_path).unwrap();
        let mut content = String::new();
        if let Some(properties) = &self.properties {
            content.push_str("---\n");
            content.push_str(&format!("{}", serde_yaml::to_string(&properties).unwrap()));
            content.push_str("\n---\n")
        }
        file.write(content.as_bytes()).unwrap();

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Properties {
    #[serde(flatten)]
    jira: JiraProperties,
    #[serde(flatten)]
    other: HashMap<String, serde_yaml::Value>,
}

impl Properties {
    pub fn new(jira_props: JiraProperties) -> Self {
        Properties {
            jira: jira_props,
            other: HashMap::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JiraProperties {
    #[serde(deserialize_with = "Priority::deserialize_from_number")]
    #[serde(serialize_with = "Priority::serialize_to_number")]
    priority: Priority,
    status: Status,
    issue_type: crate::commons::IssueType,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    due_date: Option<Date>,
    jira_key: crate::jira::JiraKey,
    sprints: Vec<Sprint>,
    #[serde(flatten)]
    time_tracking: TimeTrackingObsidian,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    parent: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    children: Vec<String>,
}
// Consider using custom serialization and deserialization for parent and children

impl JiraProperties {
    pub fn from_jira_issue(issue: &crate::jira::JiraIssue) -> Self {
        let fields = issue.get_fields();
        JiraProperties {
            jira_key: issue.get_key().clone(),
            priority: *fields.get_priority(),
            issue_type: *fields.get_issue_type(),
            status: *fields.get_status(),
            due_date: fields.get_due_date().cloned(),
            time_tracking: TimeTrackingObsidian::from(fields.get_time_tracking()),
            sprints: fields.get_sprints().iter().cloned().collect(),
            parent: fields.get_parent().map(|x| format!("[[{}]]", x.get_name())),
            children: fields
                .get_children()
                .iter()
                .map(|x| format!("[[{}]]", x.get_name()))
                .collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TimeTrackingObsidian {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(rename = "time original estimation")]
    original: Option<crate::commons::TimeEstimate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(rename = "time spent")]
    spent: Option<crate::commons::TimeEstimate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(rename = "time left")]
    left: Option<crate::commons::TimeEstimate>,
}

impl TimeTrackingObsidian {
    pub fn new(original: Option<&str>, spent: Option<&str>, estimate_left: Option<&str>) -> Self {
        Self {
            original: original.map(|x| TimeEstimate::from(x)),
            left: spent.map(|x| TimeEstimate::from(x)),
            spent: estimate_left.map(|x| TimeEstimate::from(x)),
        }
    }
}

impl From<&TimeTrackingJira> for TimeTrackingObsidian {
    fn from(input: &TimeTrackingJira) -> TimeTrackingObsidian {
        TimeTrackingObsidian {
            original: input.get_time_original().cloned(),
            left: input.get_time_left().cloned(),
            spent: input.get_time_spent().cloned(),
        }
    }
}

struct LinkedFilename(String);

impl Serialize for LinkedFilename {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&format!("[[{}]]", self.0))
    }
}

impl<'de> Deserialize<'de> for LinkedFilename {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text: String = Deserialize::deserialize(deserializer).unwrap();
        Ok(LinkedFilename(
            text.trim_matches(|x| x == '[' || x == ']').to_owned(),
        ))
    }
}

impl JiraProperties {
    pub fn new(
        priority: Priority,
        due_date: Option<Date>,
        issue_type: IssueType,
        status: Status,
        jira_key: crate::jira::JiraKey,
        time_tracking: TimeTrackingObsidian,
        sprints: Vec<Sprint>,
        parent: Option<String>,
        children: Vec<String>,
    ) -> JiraProperties {
        JiraProperties {
            priority,
            due_date,
            issue_type,
            status,
            jira_key,
            time_tracking,
            sprints,
            parent,
            children,
        }
    }
}

pub fn get_all_notes<P: AsRef<Path>>(vault_path: P) -> Vec<ObsidianFile> {
    let mut all_notes: Vec<ObsidianFile> = Vec::new();
    let dir_walker = read_dir(vault_path).unwrap();

    for entry in dir_walker {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_dir() {
            all_notes.append(&mut get_all_notes(entry.path()));
        } else {
            let path = entry.path();
            if path.extension().map_or(true, |x| x != "md") {
                continue;
            }
            all_notes.push(ObsidianFile::read_file(path));
        }
    }

    return all_notes;
}

#[cfg(test)]
mod test {
    use crate::obsidian::task_file::TimeTrackingObsidian;

    #[test]
    fn create_file() {
        use super::{JiraProperties, ObsidianFile, Properties};
        let jira_props = JiraProperties {
            priority: crate::commons::Priority::High,
            due_date: Some(crate::commons::Date::from("2024-03-14")),
            status: crate::commons::Status::InProgress,
            issue_type: crate::commons::IssueType::Task,
            jira_key: crate::jira::JiraKey::new("MB-123"),
            sprints: vec![
                crate::commons::Sprint::new(String::from("Y24W12")),
                crate::commons::Sprint::new(String::from("Y24W14")),
            ],
            time_tracking: TimeTrackingObsidian {
                original: None,
                spent: None,
                left: None,
            },
            parent: Some(String::from("Problems/In The Water")),
            children: vec![String::from("First Problems")],
        };
        let properties = Properties {
            jira: jira_props,
            other: std::collections::HashMap::new(),
        };

        let mut path: std::path::PathBuf = ["created_file"].iter().collect();
        path.set_extension("md");
        let file = ObsidianFile {
            path,
            properties: Some(properties),
            content: String::from("Buenas noches gente"),
        };
        file.save_file();
    }
}
