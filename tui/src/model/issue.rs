use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Issue {
    pub key: String,
    pub fields: Fields,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Fields {
    pub summary: String,
    pub description: Option<String>,
    pub status: Status,
    pub assignee: Option<User>,
    pub reporter: Option<User>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Status {
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct User {
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,

    pub name: Option<String>,   // fallback (older Jira APIs)
    pub email_address: Option<String>,
}
