use std::env;

pub struct Config {
    pub jira_url: String,
    pub token: String,
    pub project: String,
}

impl Config {
    pub fn load() -> Self {
        let token = env::var("JIRA_TOKEN")
            .expect("JIRA_TOKEN not set");

        Self {
            jira_url: "https://jira.rz.bankenit.de/jira".to_string(),
            token,
            project: "ITGADT".to_string(),
        }
    }
}
