use crate::config::Config;
use crate::constants::DEV_MODE;
use crate::model::issue::Issue;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

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
        if DEV_MODE {
            return self.mock_search(jql);
        }

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

    pub fn mock_search(&self, jql: &str) -> serde_json::Value {
        let issues = vec![
            Issue {
                key: "PROJ-101".to_string(),
                fields: crate::model::issue::Fields {
                    summary: "Implement user authentication".to_string(),
                    description: Some(
                        "Add login and logout functionality with JWT tokens".to_string(),
                    ),
                    status: crate::model::issue::Status {
                        name: "In Progress".to_string(),
                    },
                    assignee: Some(crate::model::issue::User {
                        display_name: Some("John Doe".to_string()),
                        name: None,
                        email_address: None,
                    }),
                    reporter: Some(crate::model::issue::User {
                        display_name: Some("Jane Smith".to_string()),
                        name: None,
                        email_address: None,
                    }),
                },
            },
            Issue {
                key: "PROJ-102".to_string(),
                fields: crate::model::issue::Fields {
                    summary: "Fix navigation bug on mobile".to_string(),
                    description: Some(
                        "The hamburger menu doesn't close when clicking outside".to_string(),
                    ),
                    status: crate::model::issue::Status {
                        name: "To Do".to_string(),
                    },
                    assignee: Some(crate::model::issue::User {
                        display_name: Some("Alice Johnson".to_string()),
                        name: None,
                        email_address: None,
                    }),
                    reporter: Some(crate::model::issue::User {
                        display_name: Some("Bob Wilson".to_string()),
                        name: None,
                        email_address: None,
                    }),
                },
            },
            Issue {
                key: "PROJ-103".to_string(),
                fields: crate::model::issue::Fields {
                    summary: "Add dark mode support".to_string(),
                    description: Some(
                        "Implement theme switching with system preference detection".to_string(),
                    ),
                    status: crate::model::issue::Status {
                        name: "Done".to_string(),
                    },
                    assignee: Some(crate::model::issue::User {
                        display_name: Some("John Doe".to_string()),
                        name: None,
                        email_address: None,
                    }),
                    reporter: Some(crate::model::issue::User {
                        display_name: Some("Jane Smith".to_string()),
                        name: None,
                        email_address: None,
                    }),
                },
            },
            Issue {
                key: "PROJ-104".to_string(),
                fields: crate::model::issue::Fields {
                    summary: "Update dependencies".to_string(),
                    description: Some("Upgrade all npm packages to latest versions".to_string()),
                    status: crate::model::issue::Status {
                        name: "In Progress".to_string(),
                    },
                    assignee: Some(crate::model::issue::User {
                        display_name: Some("Charlie Brown".to_string()),
                        name: None,
                        email_address: None,
                    }),
                    reporter: Some(crate::model::issue::User {
                        display_name: Some("John Doe".to_string()),
                        name: None,
                        email_address: None,
                    }),
                },
            },
            Issue {
                key: "PROJ-105".to_string(),
                fields: crate::model::issue::Fields {
                    summary: "Write unit tests for API endpoints".to_string(),
                    description: Some("Add test coverage for all REST API endpoints".to_string()),
                    status: crate::model::issue::Status {
                        name: "To Do".to_string(),
                    },
                    assignee: None,
                    reporter: Some(crate::model::issue::User {
                        display_name: Some("Alice Johnson".to_string()),
                        name: None,
                        email_address: None,
                    }),
                },
            },
        ];

        let issues_json: Vec<serde_json::Value> = issues
            .into_iter()
            .map(|i| serde_json::to_value(i).unwrap())
            .collect();
        serde_json::json!({
            "issues": issues_json,
            "total": issues_json.len()
        })
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
        if DEV_MODE {
            return Ok(format!("Created {} in DEV_MODE", summary));
        }

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

        match self
            .client
            .post(&url)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
        {
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
        if DEV_MODE {
            return Ok(vec![
                Transition {
                    id: "11".to_string(),
                    name: "To Do".to_string(),
                },
                Transition {
                    id: "21".to_string(),
                    name: "In Progress".to_string(),
                },
                Transition {
                    id: "31".to_string(),
                    name: "Done".to_string(),
                },
            ]);
        }

        let url = format!(
            "{}/rest/api/2/issue/{}/transitions",
            self.config.jira_url, issue_key
        );

        match self
            .client
            .get(&url)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .send()
        {
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
        if DEV_MODE {
            return Ok(());
        }

        let url = format!(
            "{}/rest/api/2/issue/{}/transitions",
            self.config.jira_url, issue_key
        );
        let payload = json!({ "transition": { "id": transition_id } });

        match self
            .client
            .post(&url)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
        {
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
