use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::constants::DEFAULT_CONFIG;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Config {
    pub name: String,
    pub purpose: String,
    pub api_base: String,
    pub auth: String,
    pub safe_actions: Vec<String>,
    pub risky_actions: Vec<String>,
    pub docs: Vec<String>,
    pub support: String,
    pub limits: String,
    pub ignore: Vec<String>,
}

pub fn load_config(root: &Path, config_path: Option<&str>) -> Result<Config, String> {
    let file = root.join(config_path.unwrap_or(DEFAULT_CONFIG));
    if !file.exists() {
        return Ok(Config::default());
    }

    let text = fs::read_to_string(&file)
        .map_err(|error| format!("Could not read {}: {error}", display_path(&file)))?;
    serde_json::from_str(&text)
        .map_err(|error| format!("Could not parse {} as JSON: {error}", display_path(&file)))
}

pub fn write_default_config(root: &Path, config_path: Option<&str>) -> Result<PathBuf, String> {
    let file = root.join(config_path.unwrap_or(DEFAULT_CONFIG));
    let sample = Config {
        name: root
            .file_name()
            .map(|value| value.to_string_lossy().to_string())
            .unwrap_or_else(|| "my-api".to_string()),
        purpose: "This API helps agents use this project.".to_string(),
        api_base: "/api".to_string(),
        auth: "Use the auth rules in the API docs.".to_string(),
        safe_actions: vec!["GET health and status data.".to_string()],
        risky_actions: vec!["Ask before write or delete calls.".to_string()],
        docs: Vec::new(),
        support: String::new(),
        limits: "Use a slow pace.".to_string(),
        ignore: Vec::new(),
    };

    let text = serde_json::to_string_pretty(&sample)
        .map_err(|error| format!("Could not write sample config: {error}"))?;
    let mut handle = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&file)
        .map_err(|error| format!("Could not write {}: {error}", display_path(&file)))?;
    handle
        .write_all(format!("{text}\n").as_bytes())
        .map_err(|error| format!("Could not write {}: {error}", display_path(&file)))?;
    Ok(file)
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().to_string()
}
