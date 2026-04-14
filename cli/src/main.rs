use clap::{Parser, Subcommand};
use directories::ProjectDirs;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Command as ProcCommand, Stdio};

const JIRA_URL: &str = "https://jira.rz.bankenit.de/jira";
const DEFAULT_PROJECT: &str = "ITGADT";
const ISSUE_TYPE_DEFAULT: &str = "10100";

#[derive(Parser)]
#[command(name = "jira")]
#[command(about = "LazyJira - Simple CLI for Jira Interactions", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: JiraCommand,
}

#[derive(Subcommand)]
enum JiraCommand {
    /// Show my assigned issues from cache
    ///
    /// Examples:
    ///   jira mine
    Mine,

    /// Show issues assigned to users (comma-separated)
    ///
    /// Examples:
    ///   jira from xcxa1b9
    ///   jira from xcxa1b4,xcxa1b9
    From { users: String },

    /// Interactive issue picker using fzf
    ///
    /// Examples:
    ///   jira pick
    Pick,

    /// Search issues via JQL
    ///
    /// Examples:
    ///   jira search 'project = ITGADT AND status != Done'
    ///   jira search 'assignee = currentUser() AND resolution = Unresolved'
    Search { jql: Option<String> },

    /// Create a new issue
    ///
    /// Examples:
    ///   jira create --name "fix pipeline"
    ///   jira create --name "fix pipeline" --project ITGADT --desc "some text"
    ///   jira create --name "fix pipeline" --component FSV/IS
    Create {
        #[arg(long)]
        name: String,
        #[arg(long)]
        project: Option<String>,
        #[arg(long)]
        component: Option<String>,
        #[arg(long)]
        desc: Option<String>,
        #[arg(long)]
        #[arg(default_value = ISSUE_TYPE_DEFAULT)]
        #[arg(long, hide = true)]
        r#type: String,
    },

    /// Search by component name
    ///
    /// Examples:
    ///   jira component "FSV/IS"
    Component { name: String },

    /// Transition a ticket (done/start/todo)
    ///
    /// Examples:
    ///   jira update start ITGADT-2836
    ///   jira update done ITGADT-2836
    Update { state: String, key: String },

    /// Check configuration and dependencies
    ///
    /// Examples:
    ///   jira doctor
    Doctor,

    /// Show this help message
    ///
    /// Examples:
    ///   jira about
    About,
}

fn get_config_dir() -> PathBuf {
    if let Some(proj_dirs) = ProjectDirs::from("jira", "lazyjira", "LazyJira") {
        proj_dirs.config_dir().to_path_buf()
    } else {
        env::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".jira")
    }
}

fn get_cache_dir() -> PathBuf {
    get_config_dir().join("cache")
}

fn get_env_file() -> PathBuf {
    get_config_dir().join("env")
}

fn get_cache_file() -> PathBuf {
    get_cache_dir().join("issues.json")
}

fn ensure_dirs() {
    let config_dir = get_config_dir();
    let cache_dir = get_cache_dir();
    fs::create_dir_all(&config_dir).ok();
    fs::create_dir_all(&cache_dir).ok();
}

fn load_token() -> Result<String, String> {
    ensure_dirs();
    let env_file = get_env_file();

    if let Ok(mut file) = File::open(&env_file) {
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| e.to_string())?;
        for line in contents.lines() {
            if line.starts_with("TOKEN=") {
                return Ok(line[6..].trim().to_string());
            }
        }
    }

    print!("Enter your Jira Bearer Token: ");
    std::io::stdout().flush().ok();

    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .map_err(|e| e.to_string())?;
    let token = input.trim().to_string();

    if token.is_empty() {
        return Err("No token provided. Aborting.".to_string());
    }

    let mut file = File::create(&env_file).map_err(|e| e.to_string())?;
    let _ = file.write_all(format!("TOKEN={}\n", token).as_bytes());
    println!("Token saved to {:?}", env_file);

    Ok(token)
}

fn create_client(token: &str) -> Client {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::AUTHORIZATION,
        format!("Bearer {}", token).parse().unwrap(),
    );
    headers.insert(
        reqwest::header::CONTENT_TYPE,
        "application/json".parse().unwrap(),
    );

    Client::builder()
        .default_headers(headers)
        .build()
        .expect("Failed to create HTTP client")
}

#[derive(Deserialize, Serialize, Debug)]
struct JiraSearchResponse {
    issues: Vec<JiraIssue>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct JiraIssue {
    key: String,
    fields: JiraFields,
}

#[derive(Deserialize, Serialize, Debug)]
struct JiraFields {
    summary: String,
    status: JiraStatus,
    #[serde(default)]
    components: Vec<JiraComponent>,
}

#[derive(Deserialize, Serialize, Debug)]
struct JiraStatus {
    name: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct JiraComponent {
    name: String,
}

#[derive(Deserialize, Debug)]
struct TransitionResponse {
    transitions: Vec<Transition>,
}

#[derive(Deserialize, Debug)]
struct Transition {
    id: String,
    name: String,
}

fn api_get(client: &Client, jql: &str) -> Result<JiraSearchResponse, String> {
    let encoded = urlencoding::encode(jql);
    let url = format!("{}/rest/api/2/search?jql={}", JIRA_URL, encoded);

    let resp = client.get(&url).send().map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().unwrap_or_default();
        return Err(format!("API error: {} - {}", status, text));
    }

    resp.json().map_err(|e| e.to_string())
}

fn api_post<T: Serialize>(client: &Client, endpoint: &str, body: &T) -> Result<String, String> {
    let url = format!("{}{}", JIRA_URL, endpoint);

    let resp = client
        .post(&url)
        .json(body)
        .send()
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().unwrap_or_default();
        return Err(format!("API error: {} - {}", status, text));
    }

    resp.text().map_err(|e| e.to_string())
}

fn cache_issues(client: &Client) -> Result<JiraSearchResponse, String> {
    let jql = "assignee = currentUser() ORDER BY updated DESC";
    let result = api_get(client, jql)?;
    let cache_file = get_cache_file();

    let json = serde_json::to_string(&result).map_err(|e| e.to_string())?;
    fs::write(&cache_file, json).map_err(|e| e.to_string())?;

    Ok(result)
}

fn load_cached_issues() -> Result<JiraSearchResponse, String> {
    let cache_file = get_cache_file();
    if !cache_file.exists() {
        return Err("No cache file".to_string());
    }

    let contents = fs::read_to_string(&cache_file).map_err(|e| e.to_string())?;
    serde_json::from_str(&contents).map_err(|e| e.to_string())
}

fn issues_from_cache() -> Vec<(String, String, String)> {
    match load_cached_issues() {
        Ok(resp) => resp
            .issues
            .iter()
            .map(|issue| {
                (
                    issue.key.clone(),
                    issue.fields.status.name.clone(),
                    issue.fields.summary.clone(),
                )
            })
            .collect(),
        Err(_) => vec![],
    }
}

fn get_status_names(state: &str) -> Option<&'static str> {
    match state {
        "done" => Some("Done"),
        "start" => Some("Started"),
        "todo" => Some("To Do"),
        _ => None,
    }
}

fn cmd_mine() {
    let issues = issues_from_cache();
    for (key, status, summary) in issues {
        println!("{} [{}] {}", key, status, summary);
    }
}

fn cmd_from(client: &Client, users: &str) {
    let jql = format!("assignee in ({}) ORDER BY updated DESC", users);
    match api_get(client, &jql) {
        Ok(resp) => {
            for issue in resp.issues {
                println!(
                    "{} [{}] {}",
                    issue.key, issue.fields.status.name, issue.fields.summary
                );
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}

fn cmd_pick(client: &Client) {
    if let Err(e) = cache_issues(client) {
        eprintln!("Failed to cache issues: {}", e);
    }

    let issues = issues_from_cache();
    let items: Vec<String> = issues
        .iter()
        .map(|(key, status, summary)| format!("{} [{}] {}", key, status, summary))
        .collect();

    if items.is_empty() {
        eprintln!("No issues in cache");
        return;
    }

    let input = items.join("\n");
    let child = ProcCommand::new("fzf")
        .arg("--prompt=jira> ")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn();

    match child {
        Ok(mut process) => {
            if let Some(ref mut stdin) = process.stdin.take() {
                let _ = stdin.write_all(input.as_bytes());
            }
            let output = process.wait_with_output().ok();
            if let Some(out) = output {
                let selected = String::from_utf8_lossy(&out.stdout);
                let key = selected.trim().split_whitespace().next().unwrap_or("");
                if !key.is_empty() {
                    println!("{}", key);
                }
            }
        }
        Err(e) => eprintln!("fzf error: {}", e),
    }
}

fn cmd_search(client: &Client, jql: Option<String>) {
    let query = jql.unwrap_or_else(|| "assignee = currentUser()".to_string());

    match api_get(client, &query) {
        Ok(resp) => {
            for issue in resp.issues {
                println!(
                    "{} [{}] {}",
                    issue.key, issue.fields.status.name, issue.fields.summary
                );
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}

fn cmd_create(
    client: &Client,
    name: &str,
    project: Option<String>,
    component: Option<String>,
    desc: Option<String>,
    issue_type: &str,
) {
    let project = project.unwrap_or_else(|| DEFAULT_PROJECT.to_string());
    let desc = desc.unwrap_or_default();
    let component = component.clone();

    let mut fields: HashMap<String, serde_json::Value> = HashMap::new();
    fields.insert("project".to_string(), json!({ "key": project }));
    fields.insert("summary".to_string(), json!(name));
    fields.insert("description".to_string(), json!(desc));
    fields.insert("issuetype".to_string(), json!({ "id": issue_type }));

    if component.is_some() {
        fields.insert(
            "components".to_string(),
            json!([{ "name": component.unwrap() }]),
        );
    }

    let payload = json!({ "fields": fields });

    match api_post(client, "/rest/api/2/issue", &payload) {
        Ok(response) => println!("{}", response),
        Err(e) => eprintln!("Error: {}", e),
    }

    cache_issues(client).ok();
}

fn cmd_component(client: &Client, name: &str) {
    let normalized = name.to_uppercase();
    let jql = format!("component = \"{}\" ORDER BY updated DESC", normalized);

    match api_get(client, &jql) {
        Ok(resp) => {
            for issue in resp.issues {
                println!(
                    "{} [{}] {}",
                    issue.key, issue.fields.status.name, issue.fields.summary
                );
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}

fn cmd_update(client: &Client, state: &str, key: &str) {
    let target_name = match get_status_names(state) {
        Some(name) => name,
        None => {
            eprintln!("Unknown state: {}", state);
            std::process::exit(1);
        }
    };

    let url = format!("{}/rest/api/2/issue/{}/transitions", JIRA_URL, key);

    let resp = client
        .get(&url)
        .send()
        .map_err(|e| e.to_string())
        .expect("Failed to get transitions");

    let trans_resp: TransitionResponse = resp
        .json()
        .map_err(|e| e.to_string())
        .expect("Failed to parse transitions");

    let transition_id = trans_resp
        .transitions
        .iter()
        .find(|t| t.name.as_str() == target_name)
        .map(|t| t.id.clone());

    let transition_id = match transition_id {
        Some(id) => id,
        None => {
            eprintln!("No transition found for '{}' on {}", state, key);
            std::process::exit(1);
        }
    };

    let payload = json!({ "transition": { "id": transition_id } });
    let url = format!("{}/rest/api/2/issue/{}/transitions", JIRA_URL, key);

    let resp = client
        .post(&url)
        .json(&payload)
        .send()
        .map_err(|e| e.to_string());

    match resp {
        Ok(_) => {
            println!("Updated Ticket {} to {}", key, state);
            cache_issues(client).ok();
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}

fn cmd_doctor() {
    let env_file = get_env_file();
    if env_file.exists() {
        println!("Token: found");
    } else {
        println!("Token: not found. Create one in your Jira");
    }

    let cache_file = get_cache_file();
    if cache_file.exists() {
        match load_cached_issues() {
            Ok(resp) => println!("Cache items: {}", resp.issues.len()),
            Err(_) => println!("Cache items: broken cache"),
        }
    } else {
        println!("Cache: no cache");
    }
}

fn cmd_help() {
    println!(
        r#"LazyJira CLI - Simple CLI for Jira Interactions

Usage: jira [command] [options]

Core Commands:
  mine
  from userA[,userB,...]
  search '<JQL>'
  component <name>
  pick
  update <done|start|todo> [KEY]
  doctor
  create --name "title" [--project KEY] [--desc TEXT]
  about

Examples:
  jira create --name "fix pipeline"
  jira update start ITGADT-2836
  jira update done ITGADT-2836
  jira search 'project = ITGADT AND status != Done'
  jira mine
  jira pick
  jira component "FSV/IS""#
    );
}

fn main() {
    let cli = Cli::parse();

    let token = match load_token() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    let client = create_client(&token);

    match cli.command {
        JiraCommand::Mine => cmd_mine(),
        JiraCommand::From { users } => cmd_from(&client, &users),
        JiraCommand::Pick => cmd_pick(&client),
        JiraCommand::Search { jql } => cmd_search(&client, jql),
        JiraCommand::Create {
            name,
            project,
            component,
            desc,
            r#type,
        } => cmd_create(&client, &name, project, component, desc, &r#type),
        JiraCommand::Component { name } => cmd_component(&client, &name),
        JiraCommand::Update { state, key } => cmd_update(&client, &state, &key),
        JiraCommand::Doctor => cmd_doctor(),
        JiraCommand::About => cmd_help(),
    }
}
