use std::fs::read_dir;
use std::path::{Path, PathBuf};

use std::fs::read_to_string;

use serde_yaml;

#[derive(Debug)]
pub struct ObsidianFile {
    path: PathBuf,
    content: String,
    properties: Option<serde_yaml::Value>,
}

fn get_obsidian_file(path: PathBuf) -> ObsidianFile {
    let full_content = read_to_string(&path).unwrap();
    let has_properties = full_content.starts_with("---");

    let content: String;
    let properties: Option<serde_yaml::Value>;

    if has_properties {
        let property_end = full_content[3..].find("---").unwrap() + 3;
        properties = Some(serde_yaml::from_str(&full_content[3..property_end]).unwrap());
        content = full_content[(property_end + 4)..].to_owned();
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