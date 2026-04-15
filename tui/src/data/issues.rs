use crate::model::issue::Issue;
use crate::{
    cache::{load_cache, save_cache},
    jira::JiraClient,
};

pub fn load_issues(jira: &JiraClient, cache_path: &str) -> Vec<Issue> {
    let jql = "assignee = currentUser() ORDER BY updated DESC";

    let data = jira.search(jql);

    save_cache(cache_path, &data);

    let issues: Vec<Issue> = serde_json::from_value(data["issues"].clone()).unwrap_or_default();

    issues
}

#[allow(dead_code)]
pub fn load_from_cache(cache_path: &str) -> Vec<Issue> {
    let data = load_cache(cache_path).unwrap_or_default();

    serde_json::from_value(data["issues"].clone()).unwrap_or_default()
}
