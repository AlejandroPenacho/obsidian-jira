use serde::Deserialize;
use serde_yaml;
pub static CONFIG: std::sync::OnceLock<Config> = std::sync::OnceLock::new();

#[derive(Debug, Deserialize)]
pub struct Config {
    jira_url: String,
    user_mail: String,
    user_id: String,
    jira_token: String,
}

impl Config {
    pub fn new() -> Config {
        let file = std::fs::File::open("config.yaml").unwrap();
        serde_yaml::from_reader(file).unwrap()
    }
    pub fn get_jira_url(&self) -> &str {
        &self.jira_url
    }
    pub fn get_user_id(&self) -> &str {
        &self.user_id
    }
    pub fn get_user_mail(&self) -> &str {
        &self.user_mail
    }
    pub fn get_jira_token(&self) -> &str {
        &self.jira_token
    }
}
