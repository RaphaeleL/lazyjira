use reqwest::blocking::Client;
use crate::config::Config;
use crate::model::issue::Issue;
use std::collections::HashMap;
use serde_json::json;
use serde::{Deserialize, Serialize};

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

    pub fn jira_url(&self) -> &str {
        &self.config.jira_url
    }

    pub fn create_issue(
        &self,
        summary: &str,
        project: Option<String>,
        component: Option<String>,
        description: Option<String>,
        issue_type: Option<String>,
    ) -> Result<String, String> {
        let project = project.unwrap_or_else(|| self.config.default_project.clone());
        let description = description.unwrap_or_default();
        let issue_type = issue_type.unwrap_or_else(|| self.config.issue_type.clone());

        let mut fields = HashMap::new();
        fields.insert("project".to_string(), json!({ "key": project }));
        fields.insert("summary".to_string(), json!(summary));
        fields.insert("description".to_string(), json!(description));
        fields.insert("issuetype".to_string(), json!({ "id": issue_type }));

        if let Some(c) = component {
            if !c.is_empty() {
                fields.insert("components".to_string(), json!([{ "name": c }]));
            }
        }

        let payload = json!({ "fields": fields });
        let url = format!("{}/rest/api/2/issue", self.config.jira_url);

        match self.client
            .post(&url)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .json(&payload)
            .send() {
            Ok(resp) => {
                if resp.status().is_success() {
                    resp.text().map_err(|e| e.to_string())
                } else {
                    Err(format!("HTTP {}", resp.status()))
                }
            }
            Err(e) => Err(e.to_string()),
        }
    }

    pub fn get_transitions(&self, issue_key: &str) -> Result<Vec<Transition>, String> {
        let url = format!("{}/rest/api/2/issue/{}/transitions", self.config.jira_url, issue_key);
        
        match self.client
            .get(&url)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .send() {
            Ok(resp) => {
                if resp.status().is_success() {
                    match resp.json::<TransitionResponse>() {
                        Ok(trans_resp) => Ok(trans_resp.transitions),
                        Err(e) => Err(e.to_string()),
                    }
                } else {
                    Err(format!("HTTP {}", resp.status()))
                }
            }
            Err(e) => Err(e.to_string()),
        }
    }

    pub fn transition_issue(&self, issue_key: &str, transition_id: &str) -> Result<(), String> {
        let url = format!("{}/rest/api/2/issue/{}/transitions", self.config.jira_url, issue_key);
        let payload = json!({ "transition": { "id": transition_id } });

        match self.client
            .post(&url)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .json(&payload)
            .send() {
            Ok(resp) => {
                if resp.status().is_success() {
                    Ok(())
                } else {
                    Err(format!("HTTP {}", resp.status()))
                }
            }
            Err(e) => Err(e.to_string()),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Transition {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct TransitionResponse {
    pub transitions: Vec<Transition>,
}
