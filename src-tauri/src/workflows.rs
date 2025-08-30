use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::PathBuf, time::{SystemTime, UNIX_EPOCH}};
use regex::Regex;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkflowParam {
    pub name: String,
    pub description: Option<String>,
    pub required: bool,
    pub default: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub command: String,
    pub params: Vec<WorkflowParam>,
    pub tags: Vec<String>,
    pub created_at: u64,
    pub updated_at: u64,
}

fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64
}

fn workflows_dir() -> PathBuf {
    let home = if cfg!(windows) {
        std::env::var("USERPROFILE").unwrap_or_else(|_| ".".into())
    } else {
        std::env::var("HOME").unwrap_or_else(|_| ".".into())
    };
    PathBuf::from(home).join(".warp-terminal")
}

fn workflows_path() -> PathBuf { workflows_dir().join("workflows.json") }

fn ensure_default_file() -> std::io::Result<()> {
    let dir = workflows_dir();
    if !dir.exists() { fs::create_dir_all(&dir)?; }
    let path = workflows_path();
    if !path.exists() {
        let defaults = vec![
            Workflow {
                id: uuid::Uuid::new_v4().to_string(),
                name: "List files".into(),
                description: Some("List files in current directory".into()),
                command: "ls -la".into(),
                params: vec![],
                tags: vec!["files".into()],
                created_at: now_ms(),
                updated_at: now_ms(),
            },
            Workflow {
                id: uuid::Uuid::new_v4().to_string(),
                name: "Search in files".into(),
                description: Some("Search recursively for a pattern".into()),
                command: "grep -R {{pattern}} .".into(),
                params: vec![WorkflowParam { name: "pattern".into(), description: Some("Text to search".into()), required: true, default: None }],
                tags: vec!["search".into()],
                created_at: now_ms(),
                updated_at: now_ms(),
            },
        ];
        let json = serde_json::to_string_pretty(&defaults).unwrap();
        fs::write(path, json)?;
    }
    Ok(())
}

pub fn load_all() -> Result<Vec<Workflow>, String> {
    ensure_default_file().map_err(|e| e.to_string())?;
    let data = fs::read_to_string(workflows_path()).map_err(|e| e.to_string())?;
    let mut list: Vec<Workflow> = serde_json::from_str(&data).map_err(|e| e.to_string())?;
    // Merge plugin workflows if present
    let plugins = crate::plugins::list_plugins();
    for p in plugins {
        if let Some(mut ws) = p.workflows { list.append(&mut ws); }
    }
    Ok(list)
}

fn save_all(items: &Vec<Workflow>) -> Result<(), String> {
    let json = serde_json::to_string_pretty(items).map_err(|e| e.to_string())?;
    fs::write(workflows_path(), json).map_err(|e| e.to_string())
}

pub fn upsert(mut wf: Workflow) -> Result<Workflow, String> {
    let mut list = load_all()?;
    let mut found = false;
    for item in &mut list {
        if item.id == wf.id { *item = wf.clone(); item.updated_at = now_ms(); found = true; break; }
    }
    if !found {
        if wf.id.is_empty() { wf.id = uuid::Uuid::new_v4().to_string(); }
        wf.created_at = now_ms(); wf.updated_at = wf.created_at;
        list.push(wf.clone());
    }
    save_all(&list)?;
    Ok(wf)
}

pub fn delete(id: &str) -> Result<(), String> {
    let mut list = load_all()?;
    list.retain(|w| w.id != id);
    save_all(&list)
}

pub fn get(id: &str) -> Result<Workflow, String> {
    let list = load_all()?;
    list.into_iter().find(|w| w.id == id).ok_or_else(|| "Workflow not found".into())
}

pub fn render_command(command: &str, params: &HashMap<String, String>) -> String {
    let re = Regex::new(r"\{\{\s*([a-zA-Z0-9_\-]+)\s*\}\}").unwrap();
    re.replace_all(command, |caps: &regex::Captures| {
        let key = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        params.get(key).cloned().unwrap_or_else(|| format!("{{{{{}}}}}", key))
    }).to_string()
}
