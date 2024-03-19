use std::fs::read_dir;
use std::io::Write;
use std::path::{Path, PathBuf};

use std::fs::read_to_string;

use crate::commons::{Date, Priority, Sprint, Status, TimeEstimate};
use crate::jira::TimeTrackingJira;

use serde;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_yaml;

#[derive(Debug)]
pub struct TaskFile {
    path: PathBuf,
    content: String,
    properties: TaskProperties,
}

impl TaskFile {
    pub fn read<P: AsRef<Path>>(path: P) -> Self {
        let mut complete_path = PathBuf::new();
        complete_path.push(crate::config::CONFIG.get_vault_path());
        complete_path.push(&path);
        complete_path.set_extension("md");

        let full_content = read_to_string(&complete_path).unwrap();
        let _has_properties = full_content.starts_with("---");

        let end_of_properties = full_content[3..].find("---").unwrap() + 3;

        let properties: TaskProperties =
            serde_yaml::from_str(&full_content[3..end_of_properties]).unwrap();

        let content: String = full_content[(end_of_properties + 5)..].to_owned();

        return TaskFile {
            path: path.as_ref().to_owned(),
            content,
            properties,
        };
    }

    pub fn save(&self) -> Result<(), ()> {
        let mut complete_path = PathBuf::new();
        complete_path.push(crate::config::CONFIG.get_vault_path());
        complete_path.push(&self.path);
        complete_path.set_extension("md");

        let mut file = std::fs::File::create(complete_path).unwrap();
        let mut content = String::new();
        content.push_str("---\n");
        content.push_str(&format!(
            "{}",
            serde_yaml::to_string(&self.properties).unwrap()
        ));
        content.push_str("\n---\n");

        file.write(content.as_bytes()).unwrap();

        Ok(())
    }

    pub fn get_name(&self) -> String {
        self.path.file_stem().unwrap().to_str().unwrap().to_owned()
    }

    pub fn get_remaining_time(&self) -> time::Duration {
        self.properties
            .time_tracking
            .remaining
            .map(|x| x.0)
            .unwrap_or_default()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TaskProperties {
    #[serde(deserialize_with = "Priority::deserialize_from_number")]
    #[serde(serialize_with = "Priority::serialize_to_number")]
    priority: Priority,
    status: Status,
    // issue_type: crate::commons::IssueType,
    #[serde(default)]
    #[serde(rename = "due date")]
    #[serde(skip_serializing_if = "Option::is_none")]
    due_date: Option<Date>,
    // jira_key: crate::jira::JiraKey,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_sprints")]
    sprints: Vec<Sprint>,
    #[serde(flatten)]
    time_tracking: TimeTrackingObsidian,
    // #[serde(skip_serializing_if = "Option::is_none")]
    // #[serde(default)]
    // parent: Option<String>,
    // #[serde(skip_serializing_if = "Vec::is_empty")]
    // #[serde(default)]
    // children: Vec<String>,
}
// Consider using custom serialization and deserialization for parent and children

pub fn deserialize_sprints<'de, D: Deserializer<'de>>(
    deserialize: D,
) -> Result<Vec<Sprint>, D::Error> {
    Ok(Deserialize::deserialize(deserialize).unwrap_or(vec![]))
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TimeTrackingObsidian {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(rename = "original estimate")]
    original: Option<crate::commons::TimeEstimate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(rename = "spent time")]
    spent: Option<crate::commons::TimeEstimate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(rename = "remaining time")]
    remaining: Option<crate::commons::TimeEstimate>,
}

impl TimeTrackingObsidian {
    pub fn new(
        original: Option<&str>,
        spent: Option<&str>,
        estimate_remaining: Option<&str>,
    ) -> Self {
        Self {
            original: original.map(|x| TimeEstimate::from(x)),
            remaining: spent.map(|x| TimeEstimate::from(x)),
            spent: estimate_remaining.map(|x| TimeEstimate::from(x)),
        }
    }
}

impl From<&TimeTrackingJira> for TimeTrackingObsidian {
    fn from(input: &TimeTrackingJira) -> TimeTrackingObsidian {
        TimeTrackingObsidian {
            original: input.get_time_original().cloned(),
            remaining: input.get_time_left().cloned(),
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

pub struct TaskFilter {
    sprints: Option<Vec<Sprint>>,
    path: PathBuf,
    recursive: bool,
}

impl TaskFilter {
    pub fn new() -> Self {
        Self {
            sprints: None,
            path: PathBuf::new(),
            recursive: false,
        }
    }

    pub fn set_sprints(&mut self, sprints: &[Sprint]) -> &mut Self {
        self.sprints = Some(sprints.to_owned());
        self
    }

    pub fn set_path<P: Into<PathBuf>>(&mut self, path: P) -> &mut Self {
        self.path = path.into();
        self
    }

    pub fn get_tasks(&self) -> Vec<TaskFile> {
        use crate::config::CONFIG;
        let mut complete_path = PathBuf::new();
        complete_path.push(CONFIG.get_vault_path());
        complete_path.push(self.path.clone());

        let mut output = Vec::new();
        self.read_directory(complete_path, &mut output).unwrap();
        output
    }

    fn task_fulfills_criteria(&self, task_file: &TaskFile) -> bool {
        if let Some(sprints) = &self.sprints {
            if sprints
                .iter()
                .all(|f_sprint| !task_file.properties.sprints.contains(f_sprint))
            {
                return false;
            }
        }

        true
    }

    fn read_directory(
        &self,
        complete_path: PathBuf,
        files: &mut Vec<TaskFile>,
    ) -> Result<(), String> {
        println!("{:?}", complete_path);
        for entry in read_dir(&complete_path).unwrap() {
            let entry = entry.unwrap();
            let entry_path = entry.path();
            if entry.file_type().unwrap().is_dir() {
                if self.recursive {
                    self.read_directory(entry_path, files).unwrap();
                }
            } else {
                let reduced_path = entry_path
                    .strip_prefix(crate::config::CONFIG.get_vault_path())
                    .unwrap();

                let task_file = TaskFile::read(reduced_path);
                if self.task_fulfills_criteria(&task_file) {
                    files.push(task_file)
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {

    #[test]
    fn create_file() {
        use super::{TaskFile, TaskProperties, TimeTrackingObsidian};
        let properties = TaskProperties {
            priority: crate::commons::Priority::High,
            due_date: Some(crate::commons::Date::from("2024-03-14")),
            status: crate::commons::Status::InProgress,
            // issue_type: crate::commons::IssueType::Task,
            // jira_key: crate::jira::JiraKey::new("MB-123"),
            sprints: vec![
                crate::commons::Sprint::new(String::from("Y24W12")),
                crate::commons::Sprint::new(String::from("Y24W14")),
            ],
            time_tracking: TimeTrackingObsidian {
                original: None,
                spent: None,
                remaining: None,
            },
            // parent: Some(String::from("Problems/In The Water")),
            // children: vec![String::from("First Problems")],
        };

        let mut path: std::path::PathBuf = ["created_file"].iter().collect();
        path.set_extension("md");
        let file = TaskFile {
            path,
            properties,
            content: String::from("Buenas noches gente"),
        };
        file.save();
    }

    #[test]
    fn read_file() {
        use super::TaskFile;
        let file = TaskFile::read("read_file");
        println!("{:#?}", file);
    }

    #[test]
    fn get_all_project_tasks() {
        let mut task_filter = super::TaskFilter::new();
        task_filter.set_path(crate::config::CONFIG.get_project_path());

        for task in task_filter.get_tasks() {
            println!("{:#?}", task);
        }
    }

    #[test]
    fn get_sprint_tasks() {
        let mut task_filter = super::TaskFilter::new();
        task_filter
            .set_sprints(&[crate::commons::Sprint::new("Y24W10".to_owned())])
            .set_path(crate::config::CONFIG.get_project_path());

        for task in task_filter.get_tasks() {
            println!("{:#?}", task);
        }
    }

    fn fmt_time(time: time::Duration, with_symbol: bool) -> String {
        let abs_time: time::Duration;
        if time.is_negative() {
            abs_time = -time;
        } else {
            abs_time = time;
        }

        let hours = abs_time.whole_hours();
        let minutes = abs_time.whole_minutes() - 60 * hours;

        let symbol: String;
        if !with_symbol {
            symbol = String::new();
        } else if time.is_negative() {
            symbol = String::from("-")
        } else {
            symbol = String::from("+")
        };

        format!("{}{:>2}:{:0>2}", symbol, hours, minutes)
    }
}
