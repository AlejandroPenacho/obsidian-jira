use std::fs::read_dir;
use std::path::{Path, PathBuf};

use std::collections::HashMap;
use std::fs::read_to_string;

use crate::commons::{Date, Priority};

use time::{Duration, Time};

use regex;
use serde;
use serde::Deserialize;
use serde_yaml;
use time::macros::format_description;

#[derive(Debug)]
pub struct ObsidianFile {
    path: PathBuf,
    content: String,
    properties: Option<Properties>,
}

#[derive(Deserialize, Debug)]
pub struct Properties {
    #[serde(flatten)]
    jira: JiraProperties,
    #[serde(flatten)]
    other: HashMap<String, serde_yaml::Value>,
}

#[derive(Deserialize, Debug)]
pub struct JiraProperties {
    // #[serde(deserialize_with = "crate::commons::get_priority_from_number")]
    priority: Priority,
    #[serde(default)]
    due_date: Option<Date>,
}

fn get_obsidian_file(path: PathBuf) -> ObsidianFile {
    let full_content = read_to_string(&path).unwrap();
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
        path,
        content,
        properties,
    };
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
            all_notes.push(get_obsidian_file(path));
        }
    }

    return all_notes;
}

#[derive(Debug)]
pub struct PlannedTask {
    start: Time,
    end: Time,
    length: Duration,
    name: String,
    linked: bool,
    completed: bool,
}

pub fn read_day_planner<P: AsRef<Path>>(file: P) -> Vec<PlannedTask> {
    let time_format = format_description!("[hour]:[minute]");
    let mut output = Vec::new();
    let text = read_to_string(file).unwrap();
    /*
        Okay, so here comes the explanation:
        ~ - Matches the dash at the beginning of the task
        ~ \[(.)\]        Matches one character surrounded by [], so [ ] and [X] are valid
                         The inside is the capture group 1
        ~ \s+            Matches as much whitespace as needed
        ~ (\d+:\d\d)     Matches a time as capture group 2, like 9:30 or 18:15
        ~ \s*-\s*        A dash surrounded by arbitrary whitespace
        ~ (\d+:\d\d)     Again, matches a time, this time for capture group 3
        ~ \s*            As much whitespace as needed
        ~ (.*)           One last match of anything
    */
    let re = regex::Regex::new(r"- \[(.)\]\s+(\d+:\d\d)\s*-\s*(\d+:\d\d)\s*(.*)").unwrap();
    for line in text.lines() {
        let Some(capture) = re.captures(line) else {
            continue;
        };
        println!("{:?}", capture);

        let completed = capture.get(1).unwrap().as_str() == " ";
        let start = Time::parse(capture.get(2).unwrap().as_str(), time_format).unwrap();
        let end = Time::parse(capture.get(3).unwrap().as_str(), time_format).unwrap();
        let length = end - start;
        let mut name = capture.get(4).unwrap().as_str().to_owned();
        // This is a mess, do it right later
        let linked: bool;
        if &name[..2] == "[[" {
            name = name[2..name.len() - 2].to_owned();
            linked = true;
        } else {
            linked = false;
        }

        output.push(PlannedTask {
            start,
            end,
            length,
            name,
            linked,
            completed,
        })
    }

    output
}
