use reqwest::blocking::Client;
use crate::config::Config;
use crate::model::issue::Issue;

pub struct JiraClient {
    client: Client,
    config: Config,
}

impl JiraClient {
    pub fn new(config: Config) -> Self {
        let client = Client::new();
        Self { client, config }
    }

    fn auth_header(&self) -> String {
        format!("Bearer {}", self.config.token)
    }

    pub fn search(&self, jql: &str) -> serde_json::Value {
        let url = format!(
            "{}/rest/api/2/search?jql={}",
            self.config.jira_url,
            urlencoding::encode(jql)
        );

        self.client
            .get(url)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .send()
            .unwrap()
            .json()
            .unwrap()
    }

    pub fn search_issues(&self, jql: &str) -> Vec<Issue> {
        let value = self.search(jql);

        value["issues"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|v| serde_json::from_value(v.clone()).ok())
            .collect()
    }

}
