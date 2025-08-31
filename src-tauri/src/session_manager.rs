use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::path::PathBuf;
use std::fs;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::terminal::TerminalManager;
use crate::terminal_types::{TerminalType, TerminalCapabilities};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
    pub terminal_type: TerminalType,
    pub working_dir: String,
    pub shell: String,
    pub environment: HashMap<String, String>,
    pub is_detached: bool,
    pub window_title: Option<String>,
    pub tabs: Vec<TabInfo>,
    pub active_tab_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabInfo {
    pub id: String,
    pub title: String,
    pub working_dir: String,
    pub shell: String,
    pub panes: Vec<PaneInfo>,
    pub active_pane_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaneInfo {
    pub id: String,
    pub terminal_id: String,
    pub working_dir: String,
    pub command_history: Vec<String>,
    pub scrollback_lines: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSnapshot {
    pub session_info: SessionInfo,
    pub scrollback_data: HashMap<String, Vec<String>>, // pane_id -> scrollback lines
    pub environment_state: HashMap<String, String>,
}

pub struct SessionManager {
    sessions: Arc<Mutex<HashMap<String, SessionInfo>>>,
    session_storage_dir: PathBuf,
    terminal_manager: Arc<Mutex<TerminalManager>>,
}

impl SessionManager {
    pub fn new(terminal_manager: Arc<Mutex<TerminalManager>>) -> Self {
        let storage_dir = Self::get_storage_dir();
        if !storage_dir.exists() {
            let _ = fs::create_dir_all(&storage_dir);
        }

        SessionManager {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            session_storage_dir: storage_dir,
            terminal_manager,
        }
    }

    fn get_storage_dir() -> PathBuf {
        let home = if cfg!(windows) {
            std::env::var("USERPROFILE").unwrap_or_else(|_| ".".to_string())
        } else {
            std::env::var("HOME").unwrap_or_else(|_| ".".to_string())
        };
        PathBuf::from(home).join(".warp-terminal").join("sessions")
    }

    /// Create a new named session
    pub async fn create_session(&self, name: String, shell: Option<String>, working_dir: Option<String>) -> Result<String, String> {
        let session_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        
        let terminal_type = TerminalType::from_env();
        let shell = shell.unwrap_or_else(|| self.get_default_shell());
        let working_dir = working_dir.unwrap_or_else(|| self.get_current_working_dir());

        // Create initial tab and pane
        let tab_id = Uuid::new_v4().to_string();
        let pane_id = Uuid::new_v4().to_string();
        
        // Create terminal through terminal manager  
        let default_size = crate::pty::TerminalSize { cols: 80, rows: 24, pixel_width: 0, pixel_height: 0 };
        let terminal_id = self.terminal_manager
            .lock()
            .await
            .create_terminal(default_size, Some(shell.clone()), Some(working_dir.clone()))
            .map_err(|e| e.to_string())?;

        let pane_info = PaneInfo {
            id: pane_id.clone(),
            terminal_id,
            working_dir: working_dir.clone(),
            command_history: Vec::new(),
            scrollback_lines: 0,
        };

        let tab_info = TabInfo {
            id: tab_id.clone(),
            title: "Terminal".to_string(),
            working_dir: working_dir.clone(),
            shell: shell.clone(),
            panes: vec![pane_info],
            active_pane_id: Some(pane_id),
        };

        let session_info = SessionInfo {
            id: session_id.clone(),
            name: name.clone(),
            created_at: now,
            last_accessed: now,
            terminal_type,
            working_dir,
            shell,
            environment: std::env::vars().collect(),
            is_detached: false,
            window_title: Some(format!("Warp Terminal - {}", name)),
            tabs: vec![tab_info],
            active_tab_id: Some(tab_id),
        };

        // Store session
        self.sessions.lock().await.insert(session_id.clone(), session_info.clone());
        
        // Persist session
        self.persist_session(&session_info).await?;

        Ok(session_id)
    }

    /// Attach to an existing session
    pub async fn attach_session(&self, session_id: &str) -> Result<SessionInfo, String> {
        let mut sessions = self.sessions.lock().await;
        
        if let Some(session) = sessions.get_mut(session_id) {
            session.is_detached = false;
            session.last_accessed = Utc::now();
            
            // Restore terminal connections if needed
            self.restore_session_terminals(session).await?;
            
            Ok(session.clone())
        } else {
            // Try loading from persistence
            if let Some(session_info) = self.load_session_from_disk(session_id).await? {
                sessions.insert(session_id.to_string(), session_info.clone());
                Ok(session_info)
            } else {
                Err("Session not found".to_string())
            }
        }
    }

    /// Detach from a session (keep it running in background)
    pub async fn detach_session(&self, session_id: &str) -> Result<(), String> {
        let mut sessions = self.sessions.lock().await;
        
        if let Some(session) = sessions.get_mut(session_id) {
            session.is_detached = true;
            session.last_accessed = Utc::now();
            
            // Persist current state
            self.persist_session(session).await?;
            
            Ok(())
        } else {
            Err("Session not found".to_string())
        }
    }

    /// Kill a session permanently
    pub async fn kill_session(&self, session_id: &str) -> Result<(), String> {
        let mut sessions = self.sessions.lock().await;
        
        if let Some(session) = sessions.remove(session_id) {
            // Close all terminals in the session
            for tab in &session.tabs {
                for pane in &tab.panes {
                    let _ = self.terminal_manager
                        .lock()
                        .await
                        .close_terminal(&pane.terminal_id);
                }
            }
            
            // Remove from disk
            let session_file = self.session_storage_dir.join(format!("{}.json", session_id));
            let _ = fs::remove_file(session_file);
            
            Ok(())
        } else {
            Err("Session not found".to_string())
        }
    }

    /// List all available sessions
    pub async fn list_sessions(&self) -> Vec<SessionInfo> {
        self.sessions.lock().await.values().cloned().collect()
    }

    /// Rename a session
    pub async fn rename_session(&self, session_id: &str, new_name: String) -> Result<(), String> {
        let mut sessions = self.sessions.lock().await;
        
        if let Some(session) = sessions.get_mut(session_id) {
            session.name = new_name;
            session.last_accessed = Utc::now();
            
            self.persist_session(session).await?;
            Ok(())
        } else {
            Err("Session not found".to_string())
        }
    }

    /// Create a snapshot of a session for backup/restore
    pub async fn create_session_snapshot(&self, session_id: &str) -> Result<SessionSnapshot, String> {
        let sessions = self.sessions.lock().await;
        
        if let Some(session) = sessions.get(session_id) {
            let mut scrollback_data = HashMap::new();
            
            // Collect scrollback data from all panes (simplified for now)
            for tab in &session.tabs {
                for pane in &tab.panes {
                    // For now, just use empty scrollback data
                    scrollback_data.insert(pane.id.clone(), vec![]);
                }
            }
            
            Ok(SessionSnapshot {
                session_info: session.clone(),
                scrollback_data,
                environment_state: std::env::vars().collect(),
            })
        } else {
            Err("Session not found".to_string())
        }
    }

    /// Restore session from snapshot
    pub async fn restore_session_snapshot(&self, snapshot: SessionSnapshot) -> Result<String, String> {
        let session_id = snapshot.session_info.id.clone();
        
        // Store session info
        self.sessions.lock().await.insert(session_id.clone(), snapshot.session_info.clone());
        
        // Recreate terminals and restore scrollback
        for tab in &snapshot.session_info.tabs {
            for pane in &tab.panes {
                let default_size = crate::pty::TerminalSize { cols: 80, rows: 24, pixel_width: 0, pixel_height: 0 };
                let _terminal_id = self.terminal_manager
                    .lock()
                    .await
                    .create_terminal(default_size, None, Some(pane.working_dir.clone()))
                    .map_err(|e| e.to_string())?;
                
                // Restore scrollback if available
                if let Some(scrollback_lines) = snapshot.scrollback_data.get(&pane.id) {
                    for _line in scrollback_lines {
                        // Simplified - would need proper terminal write implementation
                    }
                }
            }
        }
        
        // Persist restored session
        self.persist_session(&snapshot.session_info).await?;
        
        Ok(session_id)
    }

    /// Add a new tab to an existing session
    pub async fn add_tab_to_session(&self, session_id: &str, title: Option<String>) -> Result<String, String> {
        let mut sessions = self.sessions.lock().await;
        
        if let Some(session) = sessions.get_mut(session_id) {
            let tab_id = Uuid::new_v4().to_string();
            let pane_id = Uuid::new_v4().to_string();
            
            let default_size = crate::pty::TerminalSize { cols: 80, rows: 24, pixel_width: 0, pixel_height: 0 };
            let terminal_id = self.terminal_manager
                .lock()
                .await
                .create_terminal(default_size, Some(session.shell.clone()), Some(session.working_dir.clone()))
                .map_err(|e| e.to_string())?;

            let pane_info = PaneInfo {
                id: pane_id.clone(),
                terminal_id,
                working_dir: session.working_dir.clone(),
                command_history: Vec::new(),
                scrollback_lines: 0,
            };

            let tab_info = TabInfo {
                id: tab_id.clone(),
                title: title.unwrap_or_else(|| format!("Tab {}", session.tabs.len() + 1)),
                working_dir: session.working_dir.clone(),
                shell: session.shell.clone(),
                panes: vec![pane_info],
                active_pane_id: Some(pane_id),
            };

            session.tabs.push(tab_info);
            session.last_accessed = Utc::now();
            
            self.persist_session(session).await?;
            Ok(tab_id)
        } else {
            Err("Session not found".to_string())
        }
    }

    /// Split a pane in a session
    pub async fn split_pane(&self, session_id: &str, tab_id: &str, pane_id: &str, _direction: String) -> Result<String, String> {
        let mut sessions = self.sessions.lock().await;
        
        if let Some(session) = sessions.get_mut(session_id) {
            if let Some(tab) = session.tabs.iter_mut().find(|t| t.id == tab_id) {
                if tab.panes.iter().any(|p| p.id == pane_id) {
                    let new_pane_id = Uuid::new_v4().to_string();
                    
                    let default_size = crate::pty::TerminalSize { cols: 40, rows: 24, pixel_width: 0, pixel_height: 0 };
                    let terminal_id = self.terminal_manager
                        .lock()
                        .await
                        .create_terminal(default_size, Some(session.shell.clone()), Some(tab.working_dir.clone()))
                        .map_err(|e| e.to_string())?;

                    let new_pane = PaneInfo {
                        id: new_pane_id.clone(),
                        terminal_id,
                        working_dir: tab.working_dir.clone(),
                        command_history: Vec::new(),
                        scrollback_lines: 0,
                    };

                    tab.panes.push(new_pane);
                    tab.active_pane_id = Some(new_pane_id.clone());
                    session.last_accessed = Utc::now();
                    
                    self.persist_session(session).await?;
                    return Ok(new_pane_id);
                }
            }
        }
        
        Err("Session, tab, or pane not found".to_string())
    }

    async fn persist_session(&self, session: &SessionInfo) -> Result<(), String> {
        let session_file = self.session_storage_dir.join(format!("{}.json", session.id));
        let json_data = serde_json::to_string_pretty(session)
            .map_err(|e| format!("Failed to serialize session: {}", e))?;
        
        fs::write(session_file, json_data)
            .map_err(|e| format!("Failed to write session file: {}", e))
    }

    async fn load_session_from_disk(&self, session_id: &str) -> Result<Option<SessionInfo>, String> {
        let session_file = self.session_storage_dir.join(format!("{}.json", session_id));
        
        if session_file.exists() {
            let json_data = fs::read_to_string(session_file)
                .map_err(|e| format!("Failed to read session file: {}", e))?;
            
            let session_info: SessionInfo = serde_json::from_str(&json_data)
                .map_err(|e| format!("Failed to deserialize session: {}", e))?;
            
            Ok(Some(session_info))
        } else {
            Ok(None)
        }
    }

    async fn restore_session_terminals(&self, session: &SessionInfo) -> Result<(), String> {
        // This would recreate terminals for detached sessions
        // Implementation depends on whether terminals can be truly persisted
        // For now, we'll create new terminals
        
        for tab in &session.tabs {
            for pane in &tab.panes {
                // Check if terminal still exists (simplified)
                let default_size = crate::pty::TerminalSize { cols: 80, rows: 24, pixel_width: 0, pixel_height: 0 };
                let _new_terminal_id = self.terminal_manager
                    .lock()
                    .await
                    .create_terminal(default_size, Some(session.shell.clone()), Some(pane.working_dir.clone()))
                    .map_err(|e| e.to_string())?;
            }
        }
        
        Ok(())
    }

    fn get_default_shell(&self) -> String {
        if cfg!(windows) {
            std::env::var("SHELL").unwrap_or_else(|_| "powershell.exe".to_string())
        } else {
            std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string())
        }
    }

    fn get_current_working_dir(&self) -> String {
        std::env::current_dir()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    }
}

// Session-related commands for Tauri
#[tauri::command]
pub async fn create_session(
    name: String,
    shell: Option<String>,
    working_dir: Option<String>
) -> Result<String, String> {
    // This would need to be integrated with the main app state
    // For now, return a placeholder
    Ok("session-placeholder".to_string())
}

#[tauri::command]
pub async fn list_sessions() -> Result<Vec<SessionInfo>, String> {
    // Placeholder implementation
    Ok(vec![])
}

#[tauri::command]
pub async fn attach_session(_session_id: String) -> Result<SessionInfo, String> {
    // Placeholder implementation
    Err("Not implemented".to_string())
}

#[tauri::command]
pub async fn detach_session(_session_id: String) -> Result<(), String> {
    // Placeholder implementation
    Ok(())
}

#[tauri::command]
pub async fn kill_session(_session_id: String) -> Result<(), String> {
    // Placeholder implementation
    Ok(())
}
