use clap::{Parser, Subcommand};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "jira")]
#[command(about = "LazyJira - Simple CLI for Jira Interactions", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: JiraCommand,
}

struct Config {
    token: String,
    jira_url: String,
    default_project: String,
    issue_type: String,
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

    /// Show JQL history
    ///
    /// Examples:
    ///   jira history
    History,

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
        r#type: Option<String>,
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

    /// Show all information about a ticket
    ///
    /// Examples:
    ///   jira show ITGADT-2836
    Show { key: String },

    /// Re-run the configuration setup
    ///
    /// Examples:
    ///   jira init
    Init,

    /// Check configuration and dependencies
    ///
    /// Examples:
    ///   jira doctor
    Doctor,
}

fn get_config_dir() -> PathBuf {
    let home = env::home_dir().unwrap_or_else(|| PathBuf::from("."));

    #[cfg(target_os = "macos")]
    {
        home.join("Library/Application Support/LazyJira")
    }

    #[cfg(target_os = "linux")]
    {
        home.join(".config/lazyjira")
    }

    #[cfg(target_os = "windows")]
    {
        env::var("APPDATA")
            .map(|p| PathBuf::from(p).join("LazyJira"))
            .unwrap_or_else(|_| home.join("AppData/Roaming/LazyJira"))
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        home.join(".lazyjira")
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

fn get_history_file() -> PathBuf {
    get_cache_dir().join("history.json")
}

fn ensure_dirs() {
    let config_dir = get_config_dir();
    let cache_dir = get_cache_dir();
    fs::create_dir_all(&config_dir).ok();
    fs::create_dir_all(&cache_dir).ok();
}

fn load_or_init_config() -> Result<Config, String> {
    ensure_dirs();
    let env_file = get_env_file();

    if env_file.exists() {
        let contents = std::fs::read_to_string(&env_file).map_err(|e| e.to_string())?;

        let mut map = HashMap::new();

        for line in contents.lines() {
            if let Some((k, v)) = line.split_once('=') {
                map.insert(k.trim().to_string(), v.trim().to_string());
            }
        }

        return Ok(Config {
            token: map.get("TOKEN").cloned().unwrap_or_default(),
            jira_url: map.get("JIRA_URL").cloned().unwrap_or_default(),
            default_project: map.get("DEFAULT_PROJECT").cloned().unwrap_or_default(),
            issue_type: map.get("ISSUE_TYPE").cloned().unwrap_or_default(),
        });
    }

    init_config()
}

fn init_config() -> Result<Config, String> {
    ensure_dirs();
    let env_file = get_env_file();

    // ---- INIT FLOW ----
    println!("Configuration setup:");

    let token = prompt("Jira Bearer Token")?;
    let jira_url = prompt("Jira URL")?;
    let project = prompt("Default Project")?;
    let issue_type = prompt("Issue Type ID")?;

    let content = format!(
        "TOKEN={}\nJIRA_URL={}\nDEFAULT_PROJECT={}\nISSUE_TYPE={}\n",
        token, jira_url, project, issue_type
    );

    std::fs::write(&env_file, content).map_err(|e| e.to_string())?;

    println!("Config saved to {:?}", env_file);

    Ok(Config {
        token,
        jira_url,
        default_project: project,
        issue_type,
    })
}

fn load_config() -> Option<Config> {
    let env_file = get_env_file();
    if !env_file.exists() {
        return None;
    }

    match std::fs::read_to_string(&env_file) {
        Ok(contents) => {
            let mut map = HashMap::new();
            for line in contents.lines() {
                if let Some((k, v)) = line.split_once('=') {
                    map.insert(k.trim().to_string(), v.trim().to_string());
                }
            }
            Some(Config {
                token: map.get("TOKEN").cloned().unwrap_or_default(),
                jira_url: map.get("JIRA_URL").cloned().unwrap_or_default(),
                default_project: map.get("DEFAULT_PROJECT").cloned().unwrap_or_default(),
                issue_type: map.get("ISSUE_TYPE").cloned().unwrap_or_default(),
            })
        }
        Err(_) => None,
    }
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

fn api_get(client: &Client, config: &Config, jql: &str) -> Result<JiraSearchResponse, String> {
    let encoded = urlencoding::encode(jql);
    let url = format!("{}/rest/api/2/search?jql={}", config.jira_url, encoded);

    let resp = client.get(&url).send().map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().unwrap_or_default();
        return Err(format!("API error: {} - {}", status, text));
    }

    resp.json().map_err(|e| e.to_string())
}

fn api_post<T: Serialize>(
    client: &Client,
    config: &Config,
    endpoint: &str,
    body: &T,
) -> Result<String, String> {
    let url = format!("{}{}", config.jira_url, endpoint);

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

fn cache_issues(client: &Client, config: &Config) -> Result<JiraSearchResponse, String> {
    let jql = "assignee = currentUser() ORDER BY updated DESC";
    let result = api_get(client, config, jql)?;
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

fn save_jql_history(jql: &str) {
    let history_file = get_history_file();
    let mut history: Vec<String> = if history_file.exists() {
        fs::read_to_string(&history_file)
            .ok()
            .and_then(|c| serde_json::from_str(&c).ok())
            .unwrap_or_default()
    } else {
        vec![]
    };

    if !history.contains(&jql.to_string()) {
        history.insert(0, jql.to_string());
        if history.len() > 50 {
            history.truncate(50);
        }
        fs::write(
            &history_file,
            serde_json::to_string(&history).unwrap_or_default(),
        )
        .ok();
    }
}

fn load_jql_history() -> Vec<String> {
    let history_file = get_history_file();
    if !history_file.exists() {
        return vec![];
    }
    fs::read_to_string(&history_file)
        .ok()
        .and_then(|c| serde_json::from_str(&c).ok())
        .unwrap_or_default()
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

fn prompt(label: &str) -> Result<String, String> {
    use std::io::{stdin, stdout, Write};

    print!("{}: ", label);
    stdout().flush().ok();

    let mut input = String::new();
    stdin().read_line(&mut input).map_err(|e| e.to_string())?;

    let val = input.trim().to_string();
    if val.is_empty() {
        return Err(format!("{} cannot be empty", label));
    }

    Ok(val)
}

fn get_status_names(state: &str) -> Option<&'static str> {
    match state {
        // English aliases to German transitions
        "done" => Some("Fertig"),
        "start" => Some("Wird Ausgeführt"),
        "todo" => Some("Zu erledigen"),
        "review" => Some("In Review"),
        "waiting" => Some("Waiting"),
        // German aliases
        "fertig" => Some("Fertig"),
        "wird-ausgefuhrt" | "wird-ausgeführt" => Some("Wird Ausgeführt"),
        "erledigen" | "zu-erledigen" => Some("Zu erledigen"),
        _ => None,
    }
}

fn print_available_commands(transitions: &[Transition]) {
    eprintln!("\nAvailable commands:");
    for t in transitions {
        match t.name.as_str() {
            "Done" | "Fertig" => eprintln!("  done           -> {}", t.name),
            "Started" | "Wird Ausgeführt" => eprintln!("  start          -> {}", t.name),
            "To Do" | "Zu erledigen" => eprintln!("  todo           -> {}", t.name),
            "In Review" => eprintln!("  review         -> {}", t.name),
            "Waiting" => eprintln!("  waiting        -> {}", t.name),
            other => eprintln!("  {}           -> {}", other.to_lowercase(), other),
        }
    }
}

fn cmd_mine(client: &Client, config: &Config) {
    if let Err(e) = cache_issues(client, config) {
        eprintln!("Failed to update cache: {}", e);
    }

    for (key, status, summary) in issues_from_cache() {
        println!("{} [{}] {}", key, status, summary);
    }
}

fn cmd_from(client: &Client, config: &Config, users: &str) {
    let jql = format!("assignee in ({}) ORDER BY updated DESC", users);
    save_jql_history(&jql);

    match api_get(client, config, &jql) {
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

fn cmd_search(client: &Client, config: &Config, jql: Option<String>) {
    let query = jql.unwrap_or_else(|| "assignee = currentUser()".to_string());
    save_jql_history(&query);

    match api_get(client, config, &query) {
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
    config: &Config,
    name: &str,
    project: Option<String>,
    component: Option<String>,
    desc: Option<String>,
    issue_type: Option<String>,
) {
    let project = project.unwrap_or_else(|| config.default_project.clone());
    let desc = desc.unwrap_or_default();
    let issue_type = issue_type.unwrap_or_else(|| config.issue_type.clone());

    let mut fields = HashMap::new();
    fields.insert("project".to_string(), json!({ "key": project }));
    fields.insert("summary".to_string(), json!(name));
    fields.insert("description".to_string(), json!(desc));
    fields.insert("issuetype".to_string(), json!({ "id": issue_type }));

    if let Some(c) = component {
        fields.insert("components".to_string(), json!([{ "name": c }]));
    }

    let payload = json!({ "fields": fields });

    match api_post(client, config, "/rest/api/2/issue", &payload) {
        Ok(response) => println!("{}", response),
        Err(e) => eprintln!("Error: {}", e),
    }

    cache_issues(client, config).ok();
}

fn cmd_component(client: &Client, name: &str, config: &Config) {
    let normalized = name.to_uppercase();
    let jql = format!("component = \"{}\" ORDER BY updated DESC", normalized);
    save_jql_history(&jql);

    match api_get(client, config, &jql) {
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

fn cmd_update(client: &Client, config: &Config, state: &str, key: &str) {
    let target_name = match get_status_names(state) {
        Some(name) => name,
        None => {
            eprintln!("Unknown state: {}", state);
            std::process::exit(1);
        }
    };

    let url = format!("{}/rest/api/2/issue/{}/transitions", config.jira_url, key);
    let resp = client.get(&url).send().unwrap();
    let trans_resp: TransitionResponse = resp.json().unwrap();

    let transition_id = trans_resp
        .transitions
        .iter()
        .find(|t| t.name == target_name)
        .map(|t| t.id.clone())
        .unwrap_or_else(|| {
            eprintln!("No transition found for '{}' on {}", state, key);
            print_available_commands(&trans_resp.transitions);
            std::process::exit(1);
        });

    let payload = json!({ "transition": { "id": transition_id } });

    client.post(&url).json(&payload).send().ok();

    println!("Updated Ticket {} to {}", key, state);

    cache_issues(client, config).ok();
}

fn cmd_show(client: &Client, config: &Config, key: &str) {
    let encoded = urlencoding::encode(key);
    let url = format!("{}/rest/api/2/issue/{}", config.jira_url, encoded);

    match client.get(&url).send() {
        Ok(resp) => {
            if !resp.status().is_success() {
                eprintln!("Error: {}", resp.status());
                return;
            }
            match resp.json::<serde_json::Value>() {
                Ok(issue) => {
                    let fields = &issue["fields"];

                    // Header
                    println!("\n{}: {}", &issue["key"], fields["summary"]);
                    println!("{}\n", "=".repeat(80));

                    // Type and Status
                    if let Some(issue_type) = fields["issuetype"]["name"].as_str() {
                        println!("Type: {}", issue_type);
                    }
                    if let Some(status) = fields["status"]["name"].as_str() {
                        println!("Status: {}", status);
                    }

                    // People
                    if let Some(assignee) = fields["assignee"].as_object() {
                        if let Some(name) = assignee.get("displayName").and_then(|v| v.as_str()) {
                            println!("Assignee: {}", name);
                        }
                    } else {
                        println!("Assignee: unassigned");
                    }
                    if let Some(reporter) = fields["reporter"]["displayName"].as_str() {
                        println!("Reporter: {}", reporter);
                    }

                    // Details
                    if let Some(priority) = fields["priority"]["name"].as_str() {
                        println!("Priority: {}", priority);
                    }
                    if let Some(project) = fields["project"]["name"].as_str() {
                        println!("Project: {}", project);
                    }

                    // Dates
                    if let Some(created) = fields["created"].as_str() {
                        println!("Created: {}", created);
                    }
                    if let Some(updated) = fields["updated"].as_str() {
                        println!("Updated: {}", updated);
                    }
                    if let Some(due) = fields["duedate"].as_str() {
                        println!("Due Date: {}", due);
                    }

                    // Description
                    if let Some(desc) = fields["description"].as_str() {
                        if !desc.is_empty() {
                            println!("\n{}", "-".repeat(80));
                            println!("Description:\n");
                            println!("{}", desc);
                        }
                    }

                    // Components
                    if let Some(components) = fields["components"].as_array() {
                        if !components.is_empty() {
                            println!("\n{}", "-".repeat(80));
                            println!("Components:");
                            for comp in components {
                                if let Some(name) = comp["name"].as_str() {
                                    println!("  • {}", name);
                                }
                            }
                        }
                    }

                    // Labels
                    if let Some(labels) = fields["labels"].as_array() {
                        if !labels.is_empty() {
                            println!("\n{}", "-".repeat(80));
                            println!("Labels:");
                            for label in labels {
                                if let Some(label_str) = label.as_str() {
                                    println!("  • {}", label_str);
                                }
                            }
                        }
                    }

                    // Issue Links
                    if let Some(links) = fields["issuelinks"].as_array() {
                        if !links.is_empty() {
                            println!("\n{}", "-".repeat(80));
                            println!("Related Issues:");
                            for link in links {
                                if let Some(link_type) = link["type"]["name"].as_str() {
                                    if let Some(out_link) = link["outwardIssue"].as_object() {
                                        if let Some(linked_key) = out_link["key"].as_str() {
                                            println!(
                                                "  {} -> {} {}",
                                                link_type,
                                                linked_key,
                                                out_link["fields"]["summary"]
                                            );
                                        }
                                    } else if let Some(in_link) = link["inwardIssue"].as_object() {
                                        if let Some(linked_key) = in_link["key"].as_str() {
                                            println!(
                                                "  {} <- {} {}",
                                                link_type, linked_key, in_link["fields"]["summary"]
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }

                    println!("\n{}\n", "=".repeat(80));
                }
                Err(e) => eprintln!("Error parsing response: {}", e),
            }
        }
        Err(e) => eprintln!("Error fetching ticket: {}", e),
    }
}

fn cmd_init() {
    if let Err(e) = init_config() {
        eprintln!("Failed to initialize config: {}", e);
    }
}

fn cmd_doctor() {
    let env_file = get_env_file();
    println!("Config file: {:?}", env_file);

    if let Some(config) = load_config() {
        println!(
            "Token: {}",
            if config.token.is_empty() {
                "not set"
            } else {
                "set"
            }
        );
        println!("Jira URL: {}", config.jira_url);
        println!("Default Project: {}", config.default_project);
        println!("Issue Type: {}", config.issue_type);
    } else {
        println!("Config: not found");
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

fn cmd_history() {
    let history = load_jql_history();
    if history.is_empty() {
        println!("No JQL history found");
    } else {
        for (i, jql) in history.iter().enumerate() {
            println!("{}: {}", i + 1, jql);
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Handle init command before loading config
    if let JiraCommand::Init = &cli.command {
        cmd_init();
        return Ok(());
    }

    let config = load_or_init_config()?;
    let client = create_client(&config.token);

    match cli.command {
        JiraCommand::Mine => cmd_mine(&client, &config),
        JiraCommand::From { users } => cmd_from(&client, &config, &users),
        JiraCommand::Search { jql } => cmd_search(&client, &config, jql),
        JiraCommand::Create {
            name,
            project,
            component,
            desc,
            r#type,
        } => cmd_create(&client, &config, &name, project, component, desc, r#type),
        JiraCommand::Component { name } => cmd_component(&client, &name, &config),
        JiraCommand::Update { state, key } => cmd_update(&client, &config, &state, &key),
        JiraCommand::Show { key } => cmd_show(&client, &config, &key),
        JiraCommand::Init => unreachable!(),
        JiraCommand::Doctor => cmd_doctor(),
        JiraCommand::History => cmd_history(),
    }

    Ok(())
}
