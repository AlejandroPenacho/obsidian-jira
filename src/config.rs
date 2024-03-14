use serde::Deserialize;
use serde_yaml;

pub static CONFIG: Config = Config {
    lock: std::sync::OnceLock::new(),
};

pub struct Config {
    lock: std::sync::OnceLock<ConfigInner>,
}

impl Config {
    fn get_inner(&self) -> &ConfigInner {
        &self.lock.get_or_init(|| ConfigInner::read_config_file())
    }

    pub fn get_jira_url(&self) -> &str {
        &self.get_inner().jira_url
    }

    pub fn get_week(&self) -> Option<u8> {
        self.get_inner().week
    }

    pub fn get_user_id(&self) -> &str {
        &self.get_inner().user_id
    }

    pub fn get_user_mail(&self) -> &str {
        &self.get_inner().user_mail
    }

    pub fn get_jira_token(&self) -> &str {
        &self.get_inner().jira_token
    }

    pub fn get_vault_path(&self) -> &str {
        &self.get_inner().vault_path
    }

    pub fn get_daily_notes_path(&self) -> &str {
        &self.get_inner().daily_notes_path
    }

    pub fn get_project_path(&self) -> &str {
        &self.get_inner().project_path
    }
}

#[derive(Debug, Deserialize)]
pub struct ConfigInner {
    jira_url: String,
    user_mail: String,
    user_id: String,
    jira_token: String,
    vault_path: String,
    daily_notes_path: String,
    project_path: String,
    #[serde(default)]
    week: Option<u8>,
}

impl ConfigInner {
    pub fn read_config_file() -> ConfigInner {
        let file = std::fs::File::open("config.yaml").unwrap();
        serde_yaml::from_reader(file).unwrap()
    }
}
