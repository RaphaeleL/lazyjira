use std::collections::HashMap;
use std::env;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub token: String,
    pub jira_url: String,
    pub default_project: String,
    pub issue_type: String,
}

impl Config {
    pub fn load() -> Result<Self, String> {
        let env_file = get_env_file();

        if !env_file.exists() {
            return Err("Config file not found. Please run 'jira init' first.".to_string());
        }

        let contents = std::fs::read_to_string(&env_file)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let mut map = HashMap::new();

        for line in contents.lines() {
            if let Some((k, v)) = line.split_once('=') {
                map.insert(k.trim().to_string(), v.trim().to_string());
            }
        }

        Ok(Config {
            token: map.get("TOKEN").cloned().unwrap_or_default(),
            jira_url: map.get("JIRA_URL").cloned().unwrap_or_default(),
            default_project: map.get("DEFAULT_PROJECT").cloned().unwrap_or_default(),
            issue_type: map.get("ISSUE_TYPE").cloned().unwrap_or_default(),
        })
    }

    pub fn get_config_dir() -> PathBuf {
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

    pub fn get_cache_dir() -> PathBuf {
        Self::get_config_dir().join("cache")
    }
}

fn get_env_file() -> PathBuf {
    Config::get_config_dir().join("env")
}
