use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub workflows: Option<Vec<crate::workflows::Workflow>>, // optional bundled workflows
}

fn plugins_dir() -> PathBuf {
    let home = if cfg!(windows) {
        std::env::var("USERPROFILE").unwrap_or_else(|_| ".".into())
    } else {
        std::env::var("HOME").unwrap_or_else(|_| ".".into())
    };
    PathBuf::from(home).join(".warp-terminal").join("plugins")
}

pub fn list_plugins() -> Vec<PluginManifest> {
    let dir = plugins_dir();
    let mut out = Vec::new();
    if let Ok(entries) = fs::read_dir(&dir) {
        for e in entries.flatten() {
            if let Ok(meta) = e.metadata() { if meta.is_file() { if let Ok(s) = fs::read_to_string(e.path()) {
                if let Ok(m) = serde_json::from_str::<PluginManifest>(&s) { out.push(m); }
            }}}
        }
    }
    out
}
