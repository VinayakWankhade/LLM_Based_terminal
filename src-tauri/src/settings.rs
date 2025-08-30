use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Keybindings {
    pub open_ai_panel: String,
    pub open_workflows: String,
    pub split_vertical: String,
    pub split_horizontal: String,
    pub close_pane: String,
}

impl Default for Keybindings {
    fn default() -> Self {
        Keybindings {
            open_ai_panel: "Ctrl+Shift+A".into(),
            open_workflows: "Ctrl+Shift+W".into(),
            split_vertical: "Ctrl+Alt+V".into(),
            split_horizontal: "Ctrl+Alt+H".into(),
            close_pane: "Ctrl+Alt+X".into(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    pub theme: String,          // "dark" or "light"
    pub font_size: u16,         // terminal font size hint
    pub telemetry_enabled: bool,
    pub analytics_endpoint: Option<String>,
    pub keybindings: Keybindings,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            theme: "dark".into(),
            font_size: 14,
            telemetry_enabled: false,
            analytics_endpoint: None,
            keybindings: Keybindings::default(),
        }
    }
}

fn config_dir() -> PathBuf {
    let home = if cfg!(windows) {
        std::env::var("USERPROFILE").unwrap_or_else(|_| ".".into())
    } else {
        std::env::var("HOME").unwrap_or_else(|_| ".".into())
    };
    PathBuf::from(home).join(".warp-terminal")
}

fn settings_path() -> PathBuf { config_dir().join("settings.json") }

pub fn load_settings() -> Result<Settings, String> {
    let dir = config_dir();
    if !dir.exists() { fs::create_dir_all(&dir).map_err(|e| e.to_string())?; }
    let path = settings_path();
    if !path.exists() {
        let defaults = Settings::default();
        fs::write(&path, serde_json::to_string_pretty(&defaults).unwrap()).map_err(|e| e.to_string())?;
        return Ok(defaults)
    }
    let data = fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&data).map_err(|e| e.to_string())
}

pub fn save_settings(s: &Settings) -> Result<(), String> {
    let dir = config_dir();
    if !dir.exists() { fs::create_dir_all(&dir).map_err(|e| e.to_string())?; }
    let path = settings_path();
    fs::write(path, serde_json::to_string_pretty(s).unwrap()).map_err(|e| e.to_string())
}
