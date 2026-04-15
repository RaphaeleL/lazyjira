use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Issue {
    pub key: String,
    pub fields: Fields,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Fields {
    pub summary: String,
    pub description: Option<String>,
    pub status: Status,
    pub assignee: Option<User>,
    pub reporter: Option<User>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Status {
    pub name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct User {
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,

    pub name: Option<String>,
    pub email_address: Option<String>,
}

#[allow(dead_code)]
pub fn mock_issues() -> Vec<Issue> {
    vec![
        Issue {
            key: "PROJ-101".to_string(),
            fields: Fields {
                summary: "Implement user authentication".to_string(),
                description: Some("Add login and logout functionality with JWT tokens".to_string()),
                status: Status {
                    name: "In Progress".to_string(),
                },
                assignee: Some(User {
                    display_name: Some("John Doe".to_string()),
                    name: None,
                    email_address: None,
                }),
                reporter: Some(User {
                    display_name: Some("Jane Smith".to_string()),
                    name: None,
                    email_address: None,
                }),
            },
        },
        Issue {
            key: "PROJ-102".to_string(),
            fields: Fields {
                summary: "Fix navigation bug on mobile".to_string(),
                description: Some(
                    "The hamburger menu doesn't close when clicking outside".to_string(),
                ),
                status: Status {
                    name: "To Do".to_string(),
                },
                assignee: Some(User {
                    display_name: Some("Alice Johnson".to_string()),
                    name: None,
                    email_address: None,
                }),
                reporter: Some(User {
                    display_name: Some("Bob Wilson".to_string()),
                    name: None,
                    email_address: None,
                }),
            },
        },
        Issue {
            key: "PROJ-103".to_string(),
            fields: Fields {
                summary: "Add dark mode support".to_string(),
                description: Some(
                    "Implement theme switching with system preference detection".to_string(),
                ),
                status: Status {
                    name: "Done".to_string(),
                },
                assignee: Some(User {
                    display_name: Some("John Doe".to_string()),
                    name: None,
                    email_address: None,
                }),
                reporter: Some(User {
                    display_name: Some("Jane Smith".to_string()),
                    name: None,
                    email_address: None,
                }),
            },
        },
        Issue {
            key: "PROJ-104".to_string(),
            fields: Fields {
                summary: "Update dependencies".to_string(),
                description: Some("Upgrade all npm packages to latest versions".to_string()),
                status: Status {
                    name: "In Progress".to_string(),
                },
                assignee: Some(User {
                    display_name: Some("Charlie Brown".to_string()),
                    name: None,
                    email_address: None,
                }),
                reporter: Some(User {
                    display_name: Some("John Doe".to_string()),
                    name: None,
                    email_address: None,
                }),
            },
        },
        Issue {
            key: "PROJ-105".to_string(),
            fields: Fields {
                summary: "Write unit tests for API endpoints".to_string(),
                description: Some("Add test coverage for all REST API endpoints".to_string()),
                status: Status {
                    name: "To Do".to_string(),
                },
                assignee: None,
                reporter: Some(User {
                    display_name: Some("Alice Johnson".to_string()),
                    name: None,
                    email_address: None,
                }),
            },
        },
    ]
}
