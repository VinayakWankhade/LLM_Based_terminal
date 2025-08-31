use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalSession {
    pub session_id: String,
    pub name: String,
    pub created_at: u64,
    pub last_accessed: u64,
    pub working_directory: PathBuf,
    pub environment_variables: HashMap<String, String>,
    pub command_history: Vec<String>,
    pub scrollback_buffer: Vec<String>,
    pub panes: Vec<TerminalPane>,
    pub active_pane_id: Option<String>,
    pub layout: PaneLayout,
    pub tabs: Vec<TerminalTab>,
    pub active_tab_index: usize,
    pub status: SessionStatus,
    pub metadata: SessionMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalPane {
    pub pane_id: String,
    pub title: String,
    pub working_directory: PathBuf,
    pub command_history: VecDeque<String>,
    pub scrollback_buffer: VecDeque<String>,
    pub current_command: Option<String>,
    pub process_id: Option<u32>,
    pub status: PaneStatus,
    pub position: PanePosition,
    pub size: PaneSize,
    pub split_info: Option<SplitInfo>,
    pub created_at: u64,
    pub last_activity: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalTab {
    pub tab_id: String,
    pub title: String,
    pub icon: Option<String>,
    pub closable: bool,
    pub session_id: String,
    pub created_at: u64,
    pub last_accessed: u64,
    pub is_pinned: bool,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SessionStatus {
    Active,
    Inactive,
    Suspended,
    Restored,
    Terminated,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PaneStatus {
    Active,
    Inactive,
    Running,
    Idle,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanePosition {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaneSize {
    pub rows: u16,
    pub columns: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitInfo {
    pub split_type: SplitType,
    pub parent_pane_id: Option<String>,
    pub child_panes: Vec<String>,
    pub split_ratio: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SplitType {
    Horizontal,
    Vertical,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaneLayout {
    pub layout_type: LayoutType,
    pub root_pane: String,
    pub splits: Vec<Split>,
    pub focus_order: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LayoutType {
    Single,
    TwoColumn,
    TwoRow,
    ThreeColumn,
    ThreeRow,
    Grid,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Split {
    pub split_id: String,
    pub split_type: SplitType,
    pub ratio: f32,
    pub first_pane: String,
    pub second_pane: String,
    pub resizable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub tags: Vec<String>,
    pub description: Option<String>,
    pub project_path: Option<PathBuf>,
    pub git_branch: Option<String>,
    pub custom_properties: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSnapshot {
    pub snapshot_id: String,
    pub session_id: String,
    pub name: String,
    pub created_at: u64,
    pub session_data: TerminalSession,
    pub screenshot: Option<String>, // Base64 encoded image
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionTemplate {
    pub template_id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub pane_layout: PaneLayout,
    pub initial_commands: HashMap<String, Vec<String>>, // pane_id -> commands
    pub environment_variables: HashMap<String, String>,
    pub working_directories: HashMap<String, PathBuf>, // pane_id -> directory
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabGroup {
    pub group_id: String,
    pub name: String,
    pub color: String,
    pub tabs: Vec<String>, // tab_ids
    pub collapsible: bool,
    pub collapsed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    pub workspace_id: String,
    pub name: String,
    pub default_session_template: Option<String>,
    pub tab_groups: Vec<TabGroup>,
    pub global_environment: HashMap<String, String>,
    pub startup_sessions: Vec<String>,
    pub layout_preferences: LayoutPreferences,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutPreferences {
    pub default_split_type: SplitType,
    pub default_split_ratio: f32,
    pub minimum_pane_size: PaneSize,
    pub tab_position: TabPosition,
    pub show_tab_numbers: bool,
    pub show_pane_borders: bool,
    pub pane_border_style: BorderStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TabPosition {
    Top,
    Bottom,
    Left,
    Right,
    Hidden,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BorderStyle {
    None,
    Solid,
    Dashed,
    Dotted,
    Double,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalEvent {
    pub event_type: TerminalEventType,
    pub session_id: String,
    pub pane_id: Option<String>,
    pub tab_id: Option<String>,
    pub timestamp: u64,
    pub data: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TerminalEventType {
    SessionCreated,
    SessionDestroyed,
    SessionSuspended,
    SessionResumed,
    PaneCreated,
    PaneDestroyed,
    PaneSplit,
    PaneResized,
    PaneFocused,
    TabCreated,
    TabClosed,
    TabSwitched,
    CommandExecuted,
    ProcessStarted,
    ProcessEnded,
}

pub struct AdvancedTerminalManager {
    sessions: Arc<Mutex<HashMap<String, TerminalSession>>>,
    snapshots: Arc<Mutex<HashMap<String, SessionSnapshot>>>,
    templates: Arc<Mutex<HashMap<String, SessionTemplate>>>,
    workspaces: Arc<Mutex<HashMap<String, WorkspaceConfig>>>,
    active_session_id: Arc<Mutex<Option<String>>>,
    event_history: Arc<Mutex<VecDeque<TerminalEvent>>>,
    event_sender: Arc<Mutex<Option<mpsc::UnboundedSender<TerminalEvent>>>>,
    next_session_id: Arc<Mutex<u64>>,
    next_pane_id: Arc<Mutex<u64>>,
    next_tab_id: Arc<Mutex<u64>>,
}

impl AdvancedTerminalManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            snapshots: Arc::new(Mutex::new(HashMap::new())),
            templates: Arc::new(Mutex::new(HashMap::new())),
            workspaces: Arc::new(Mutex::new(HashMap::new())),
            active_session_id: Arc::new(Mutex::new(None)),
            event_history: Arc::new(Mutex::new(VecDeque::with_capacity(1000))),
            event_sender: Arc::new(Mutex::new(None)),
            next_session_id: Arc::new(Mutex::new(1)),
            next_pane_id: Arc::new(Mutex::new(1)),
            next_tab_id: Arc::new(Mutex::new(1)),
        }
    }

    pub async fn start_event_monitoring(&self) -> Result<mpsc::UnboundedReceiver<TerminalEvent>, String> {
        let (tx, rx) = mpsc::unbounded_channel();

        {
            let mut sender = self.event_sender.lock().unwrap();
            *sender = Some(tx);
        }

        Ok(rx)
    }

    fn emit_event(&self, event: TerminalEvent) {
        // Add to history
        {
            let mut history = self.event_history.lock().unwrap();
            if history.len() >= 1000 {
                history.pop_front();
            }
            history.push_back(event.clone());
        }

        // Send to subscribers
        if let Some(ref sender) = *self.event_sender.lock().unwrap() {
            let _ = sender.send(event);
        }
    }

    fn generate_session_id(&self) -> String {
        let mut next_id = self.next_session_id.lock().unwrap();
        let id = *next_id;
        *next_id += 1;
        format!("session_{}", id)
    }

    fn generate_pane_id(&self) -> String {
        let mut next_id = self.next_pane_id.lock().unwrap();
        let id = *next_id;
        *next_id += 1;
        format!("pane_{}", id)
    }

    fn generate_tab_id(&self) -> String {
        let mut next_id = self.next_tab_id.lock().unwrap();
        let id = *next_id;
        *next_id += 1;
        format!("tab_{}", id)
    }

    // Session Management
    pub fn create_session(&self, name: Option<String>, template_id: Option<String>) -> Result<String, String> {
        let session_id = self.generate_session_id();
        let pane_id = self.generate_pane_id();
        let tab_id = self.generate_tab_id();

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let default_pane = TerminalPane {
            pane_id: pane_id.clone(),
            title: "Terminal".to_string(),
            working_directory: std::env::current_dir().unwrap_or_default(),
            command_history: VecDeque::new(),
            scrollback_buffer: VecDeque::new(),
            current_command: None,
            process_id: None,
            status: PaneStatus::Active,
            position: PanePosition {
                x: 0.0,
                y: 0.0,
                width: 1.0,
                height: 1.0,
            },
            size: PaneSize {
                rows: 24,
                columns: 80,
            },
            split_info: None,
            created_at: timestamp,
            last_activity: timestamp,
        };

        let default_tab = TerminalTab {
            tab_id: tab_id.clone(),
            title: name.clone().unwrap_or_else(|| "Terminal".to_string()),
            icon: None,
            closable: true,
            session_id: session_id.clone(),
            created_at: timestamp,
            last_accessed: timestamp,
            is_pinned: false,
            color: None,
        };

        let layout = PaneLayout {
            layout_type: LayoutType::Single,
            root_pane: pane_id.clone(),
            splits: Vec::new(),
            focus_order: vec![pane_id.clone()],
        };

        let session = TerminalSession {
            session_id: session_id.clone(),
            name: name.unwrap_or_else(|| format!("Session {}", session_id)),
            created_at: timestamp,
            last_accessed: timestamp,
            working_directory: std::env::current_dir().unwrap_or_default(),
            environment_variables: std::env::vars().collect(),
            command_history: Vec::new(),
            scrollback_buffer: Vec::new(),
            panes: vec![default_pane],
            active_pane_id: Some(pane_id),
            layout,
            tabs: vec![default_tab],
            active_tab_index: 0,
            status: SessionStatus::Active,
            metadata: SessionMetadata {
                tags: Vec::new(),
                description: None,
                project_path: None,
                git_branch: None,
                custom_properties: HashMap::new(),
            },
        };

        {
            let mut sessions = self.sessions.lock().unwrap();
            sessions.insert(session_id.clone(), session);
        }

        {
            let mut active_session = self.active_session_id.lock().unwrap();
            *active_session = Some(session_id.clone());
        }

        self.emit_event(TerminalEvent {
            event_type: TerminalEventType::SessionCreated,
            session_id: session_id.clone(),
            pane_id: None,
            tab_id: None,
            timestamp,
            data: HashMap::new(),
        });

        Ok(session_id)
    }

    pub fn get_session(&self, session_id: &str) -> Option<TerminalSession> {
        let sessions = self.sessions.lock().unwrap();
        sessions.get(session_id).cloned()
    }

    pub fn get_all_sessions(&self) -> Vec<TerminalSession> {
        let sessions = self.sessions.lock().unwrap();
        sessions.values().cloned().collect()
    }

    pub fn destroy_session(&self, session_id: &str) -> Result<(), String> {
        {
            let mut sessions = self.sessions.lock().unwrap();
            if !sessions.contains_key(session_id) {
                return Err(format!("Session {} not found", session_id));
            }
            sessions.remove(session_id);
        }

        // Update active session if this was the active one
        {
            let mut active_session = self.active_session_id.lock().unwrap();
            if active_session.as_ref() == Some(&session_id.to_string()) {
                *active_session = None;
            }
        }

        self.emit_event(TerminalEvent {
            event_type: TerminalEventType::SessionDestroyed,
            session_id: session_id.to_string(),
            pane_id: None,
            tab_id: None,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            data: HashMap::new(),
        });

        Ok(())
    }

    pub fn set_active_session(&self, session_id: &str) -> Result<(), String> {
        {
            let sessions = self.sessions.lock().unwrap();
            if !sessions.contains_key(session_id) {
                return Err(format!("Session {} not found", session_id));
            }
        }

        {
            let mut active_session = self.active_session_id.lock().unwrap();
            *active_session = Some(session_id.to_string());
        }

        // Update session's last accessed time
        {
            let mut sessions = self.sessions.lock().unwrap();
            if let Some(session) = sessions.get_mut(session_id) {
                session.last_accessed = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
            }
        }

        Ok(())
    }

    pub fn get_active_session_id(&self) -> Option<String> {
        let active_session = self.active_session_id.lock().unwrap();
        active_session.clone()
    }

    // Pane Management
    pub fn create_pane(&self, session_id: &str, working_directory: Option<PathBuf>) -> Result<String, String> {
        let pane_id = self.generate_pane_id();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let new_pane = TerminalPane {
            pane_id: pane_id.clone(),
            title: "Terminal".to_string(),
            working_directory: working_directory.unwrap_or_else(|| std::env::current_dir().unwrap_or_default()),
            command_history: VecDeque::new(),
            scrollback_buffer: VecDeque::new(),
            current_command: None,
            process_id: None,
            status: PaneStatus::Active,
            position: PanePosition {
                x: 0.0,
                y: 0.0,
                width: 1.0,
                height: 1.0,
            },
            size: PaneSize {
                rows: 24,
                columns: 80,
            },
            split_info: None,
            created_at: timestamp,
            last_activity: timestamp,
        };

        {
            let mut sessions = self.sessions.lock().unwrap();
            if let Some(session) = sessions.get_mut(session_id) {
                session.panes.push(new_pane);
                session.layout.focus_order.push(pane_id.clone());
            } else {
                return Err(format!("Session {} not found", session_id));
            }
        }

        self.emit_event(TerminalEvent {
            event_type: TerminalEventType::PaneCreated,
            session_id: session_id.to_string(),
            pane_id: Some(pane_id.clone()),
            tab_id: None,
            timestamp,
            data: HashMap::new(),
        });

        Ok(pane_id)
    }

    pub fn split_pane(&self, session_id: &str, pane_id: &str, split_type: SplitType, ratio: f32) -> Result<String, String> {
        let new_pane_id = self.generate_pane_id();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get_mut(session_id) {
            // Find the pane to split
            let pane_index = session.panes.iter()
                .position(|p| p.pane_id == pane_id)
                .ok_or_else(|| format!("Pane {} not found", pane_id))?;

            // Clone necessary data before mutating to avoid borrowing issues
            let original_position = session.panes[pane_index].position.clone();
            let original_size = session.panes[pane_index].size.clone();
            let working_directory = session.panes[pane_index].working_directory.clone();
            
            // Calculate new pane positions and sizes
            let (pos1, pos2, size1, size2) = self.calculate_split_layout(
                &original_position,
                &original_size,
                &split_type,
                ratio
            );

            // Update original pane
            session.panes[pane_index].position = pos1;
            session.panes[pane_index].size = size1;
            session.panes[pane_index].split_info = Some(SplitInfo {
                split_type: split_type.clone(),
                parent_pane_id: None,
                child_panes: vec![new_pane_id.clone()],
                split_ratio: ratio,
            });

            // Create new pane
            let new_pane = TerminalPane {
                pane_id: new_pane_id.clone(),
                title: "Terminal".to_string(),
                working_directory,
                command_history: VecDeque::new(),
                scrollback_buffer: VecDeque::new(),
                current_command: None,
                process_id: None,
                status: PaneStatus::Active,
                position: pos2,
                size: size2,
                split_info: Some(SplitInfo {
                    split_type: split_type.clone(),
                    parent_pane_id: Some(pane_id.to_string()),
                    child_panes: Vec::new(),
                    split_ratio: 1.0 - ratio,
                }),
                created_at: timestamp,
                last_activity: timestamp,
            };

            session.panes.push(new_pane);
            session.layout.focus_order.push(new_pane_id.clone());

            // Add split to layout
            session.layout.splits.push(Split {
                split_id: format!("split_{}_{}", pane_id, new_pane_id),
                split_type,
                ratio,
                first_pane: pane_id.to_string(),
                second_pane: new_pane_id.clone(),
                resizable: true,
            });

            self.emit_event(TerminalEvent {
                event_type: TerminalEventType::PaneSplit,
                session_id: session_id.to_string(),
                pane_id: Some(pane_id.to_string()),
                tab_id: None,
                timestamp,
                data: [("new_pane_id".to_string(), serde_json::Value::String(new_pane_id.clone()))]
                    .into_iter().collect(),
            });

            Ok(new_pane_id)
        } else {
            Err(format!("Session {} not found", session_id))
        }
    }

    fn calculate_split_layout(
        &self,
        original_pos: &PanePosition,
        original_size: &PaneSize,
        split_type: &SplitType,
        ratio: f32,
    ) -> (PanePosition, PanePosition, PaneSize, PaneSize) {
        match split_type {
            SplitType::Horizontal => {
                let first_height = original_pos.height * ratio;
                let second_height = original_pos.height * (1.0 - ratio);

                let pos1 = PanePosition {
                    x: original_pos.x,
                    y: original_pos.y,
                    width: original_pos.width,
                    height: first_height,
                };

                let pos2 = PanePosition {
                    x: original_pos.x,
                    y: original_pos.y + first_height,
                    width: original_pos.width,
                    height: second_height,
                };

                let size1 = PaneSize {
                    rows: (original_size.rows as f32 * ratio) as u16,
                    columns: original_size.columns,
                };

                let size2 = PaneSize {
                    rows: (original_size.rows as f32 * (1.0 - ratio)) as u16,
                    columns: original_size.columns,
                };

                (pos1, pos2, size1, size2)
            }
            SplitType::Vertical => {
                let first_width = original_pos.width * ratio;
                let second_width = original_pos.width * (1.0 - ratio);

                let pos1 = PanePosition {
                    x: original_pos.x,
                    y: original_pos.y,
                    width: first_width,
                    height: original_pos.height,
                };

                let pos2 = PanePosition {
                    x: original_pos.x + first_width,
                    y: original_pos.y,
                    width: second_width,
                    height: original_pos.height,
                };

                let size1 = PaneSize {
                    rows: original_size.rows,
                    columns: (original_size.columns as f32 * ratio) as u16,
                };

                let size2 = PaneSize {
                    rows: original_size.rows,
                    columns: (original_size.columns as f32 * (1.0 - ratio)) as u16,
                };

                (pos1, pos2, size1, size2)
            }
            SplitType::None => {
                // No split, return original values
                (original_pos.clone(), original_pos.clone(), original_size.clone(), original_size.clone())
            }
        }
    }

    pub fn close_pane(&self, session_id: &str, pane_id: &str) -> Result<(), String> {
        let mut sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get_mut(session_id) {
            // Don't allow closing the last pane
            if session.panes.len() <= 1 {
                return Err("Cannot close the last pane".to_string());
            }

            // Remove the pane
            let pane_index = session.panes.iter()
                .position(|p| p.pane_id == pane_id)
                .ok_or_else(|| format!("Pane {} not found", pane_id))?;

            session.panes.remove(pane_index);

            // Update active pane if necessary
            if session.active_pane_id.as_ref() == Some(&pane_id.to_string()) {
                session.active_pane_id = session.panes.first().map(|p| p.pane_id.clone());
            }

            // Remove from focus order
            session.layout.focus_order.retain(|id| id != pane_id);

            // Remove related splits
            session.layout.splits.retain(|split| {
                split.first_pane != pane_id && split.second_pane != pane_id
            });

            self.emit_event(TerminalEvent {
                event_type: TerminalEventType::PaneDestroyed,
                session_id: session_id.to_string(),
                pane_id: Some(pane_id.to_string()),
                tab_id: None,
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                data: HashMap::new(),
            });

            Ok(())
        } else {
            Err(format!("Session {} not found", session_id))
        }
    }

    pub fn focus_pane(&self, session_id: &str, pane_id: &str) -> Result<(), String> {
        let mut sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get_mut(session_id) {
            // Verify pane exists
            if !session.panes.iter().any(|p| p.pane_id == pane_id) {
                return Err(format!("Pane {} not found", pane_id));
            }

            session.active_pane_id = Some(pane_id.to_string());

            self.emit_event(TerminalEvent {
                event_type: TerminalEventType::PaneFocused,
                session_id: session_id.to_string(),
                pane_id: Some(pane_id.to_string()),
                tab_id: None,
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                data: HashMap::new(),
            });

            Ok(())
        } else {
            Err(format!("Session {} not found", session_id))
        }
    }

    // Tab Management
    pub fn create_tab(&self, session_id: &str, title: Option<String>) -> Result<String, String> {
        let tab_id = self.generate_tab_id();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let new_tab = TerminalTab {
            tab_id: tab_id.clone(),
            title: title.unwrap_or_else(|| "New Tab".to_string()),
            icon: None,
            closable: true,
            session_id: session_id.to_string(),
            created_at: timestamp,
            last_accessed: timestamp,
            is_pinned: false,
            color: None,
        };

        {
            let mut sessions = self.sessions.lock().unwrap();
            if let Some(session) = sessions.get_mut(session_id) {
                session.tabs.push(new_tab);
            } else {
                return Err(format!("Session {} not found", session_id));
            }
        }

        self.emit_event(TerminalEvent {
            event_type: TerminalEventType::TabCreated,
            session_id: session_id.to_string(),
            pane_id: None,
            tab_id: Some(tab_id.clone()),
            timestamp,
            data: HashMap::new(),
        });

        Ok(tab_id)
    }

    pub fn close_tab(&self, session_id: &str, tab_index: usize) -> Result<(), String> {
        let mut sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get_mut(session_id) {
            if tab_index >= session.tabs.len() {
                return Err("Tab index out of bounds".to_string());
            }

            // Don't allow closing the last tab
            if session.tabs.len() <= 1 {
                return Err("Cannot close the last tab".to_string());
            }

            let tab = session.tabs.remove(tab_index);

            // Update active tab index if necessary
            if session.active_tab_index >= tab_index && session.active_tab_index > 0 {
                session.active_tab_index -= 1;
            }

            self.emit_event(TerminalEvent {
                event_type: TerminalEventType::TabClosed,
                session_id: session_id.to_string(),
                pane_id: None,
                tab_id: Some(tab.tab_id),
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                data: HashMap::new(),
            });

            Ok(())
        } else {
            Err(format!("Session {} not found", session_id))
        }
    }

    pub fn switch_tab(&self, session_id: &str, tab_index: usize) -> Result<(), String> {
        let mut sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get_mut(session_id) {
            if tab_index >= session.tabs.len() {
                return Err("Tab index out of bounds".to_string());
            }

            session.active_tab_index = tab_index;
            session.tabs[tab_index].last_accessed = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            self.emit_event(TerminalEvent {
                event_type: TerminalEventType::TabSwitched,
                session_id: session_id.to_string(),
                pane_id: None,
                tab_id: Some(session.tabs[tab_index].tab_id.clone()),
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                data: [("tab_index".to_string(), serde_json::Value::Number(tab_index.into()))]
                    .into_iter().collect(),
            });

            Ok(())
        } else {
            Err(format!("Session {} not found", session_id))
        }
    }

    // Session Snapshots and Restoration
    pub fn create_snapshot(&self, session_id: &str, name: Option<String>, notes: Option<String>) -> Result<String, String> {
        let session = {
            let sessions = self.sessions.lock().unwrap();
            sessions.get(session_id).cloned()
                .ok_or_else(|| format!("Session {} not found", session_id))?
        };

        let snapshot_id = format!("snapshot_{}_{}", session_id, SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs());

        let snapshot = SessionSnapshot {
            snapshot_id: snapshot_id.clone(),
            session_id: session_id.to_string(),
            name: name.unwrap_or_else(|| format!("Snapshot of {}", session.name)),
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            session_data: session,
            screenshot: None, // Would be implemented to capture terminal output
            notes,
        };

        {
            let mut snapshots = self.snapshots.lock().unwrap();
            snapshots.insert(snapshot_id.clone(), snapshot);
        }

        Ok(snapshot_id)
    }

    pub fn restore_session(&self, snapshot_id: &str) -> Result<String, String> {
        let snapshot = {
            let snapshots = self.snapshots.lock().unwrap();
            snapshots.get(snapshot_id).cloned()
                .ok_or_else(|| format!("Snapshot {} not found", snapshot_id))?
        };

        let new_session_id = self.generate_session_id();
        let mut restored_session = snapshot.session_data;
        restored_session.session_id = new_session_id.clone();
        restored_session.status = SessionStatus::Restored;
        restored_session.last_accessed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Clear runtime state
        for pane in &mut restored_session.panes {
            pane.current_command = None;
            pane.process_id = None;
            pane.status = PaneStatus::Inactive;
        }

        {
            let mut sessions = self.sessions.lock().unwrap();
            sessions.insert(new_session_id.clone(), restored_session);
        }

        self.emit_event(TerminalEvent {
            event_type: TerminalEventType::SessionResumed,
            session_id: new_session_id.clone(),
            pane_id: None,
            tab_id: None,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            data: [("snapshot_id".to_string(), serde_json::Value::String(snapshot_id.to_string()))]
                .into_iter().collect(),
        });

        Ok(new_session_id)
    }

    pub fn get_snapshots(&self, session_id: Option<&str>) -> Vec<SessionSnapshot> {
        let snapshots = self.snapshots.lock().unwrap();
        
        if let Some(session_id) = session_id {
            snapshots.values()
                .filter(|snapshot| snapshot.session_id == session_id)
                .cloned()
                .collect()
        } else {
            snapshots.values().cloned().collect()
        }
    }

    // Session Templates
    pub fn create_template(&self, session_id: &str, template_name: String, category: String) -> Result<String, String> {
        let session = {
            let sessions = self.sessions.lock().unwrap();
            sessions.get(session_id).cloned()
                .ok_or_else(|| format!("Session {} not found", session_id))?
        };

        let template_id = format!("template_{}", template_name.replace(' ', "_").to_lowercase());

        // Extract initial commands from session history
        let mut initial_commands = HashMap::new();
        for pane in &session.panes {
            let commands: Vec<String> = pane.command_history
                .iter()
                .take(5) // Take first 5 commands as initial commands
                .cloned()
                .collect();
            if !commands.is_empty() {
                initial_commands.insert(pane.pane_id.clone(), commands);
            }
        }

        // Extract working directories
        let working_directories: HashMap<String, PathBuf> = session.panes
            .iter()
            .map(|pane| (pane.pane_id.clone(), pane.working_directory.clone()))
            .collect();

        let template = SessionTemplate {
            template_id: template_id.clone(),
            name: template_name,
            description: format!("Template created from session {}", session.name),
            category,
            pane_layout: session.layout.clone(),
            initial_commands,
            environment_variables: session.environment_variables.clone(),
            working_directories,
            tags: session.metadata.tags.clone(),
        };

        {
            let mut templates = self.templates.lock().unwrap();
            templates.insert(template_id.clone(), template);
        }

        Ok(template_id)
    }

    pub fn get_templates(&self) -> Vec<SessionTemplate> {
        let templates = self.templates.lock().unwrap();
        templates.values().cloned().collect()
    }

    pub fn apply_template(&self, template_id: &str, session_name: Option<String>) -> Result<String, String> {
        let template = {
            let templates = self.templates.lock().unwrap();
            templates.get(template_id).cloned()
                .ok_or_else(|| format!("Template {} not found", template_id))?
        };

        // Create new session based on template
        let session_id = self.create_session(session_name, Some(template_id.to_string()))?;

        // Apply template configuration
        {
            let mut sessions = self.sessions.lock().unwrap();
            if let Some(session) = sessions.get_mut(&session_id) {
                session.environment_variables.extend(template.environment_variables);
                session.metadata.tags = template.tags;
                
                // Update pane working directories
                for pane in &mut session.panes {
                    if let Some(working_dir) = template.working_directories.get(&pane.pane_id) {
                        pane.working_directory = working_dir.clone();
                    }
                }
            }
        }

        Ok(session_id)
    }

    // Utility Functions
    pub fn suspend_session(&self, session_id: &str) -> Result<(), String> {
        let mut sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get_mut(session_id) {
            session.status = SessionStatus::Suspended;

            self.emit_event(TerminalEvent {
                event_type: TerminalEventType::SessionSuspended,
                session_id: session_id.to_string(),
                pane_id: None,
                tab_id: None,
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                data: HashMap::new(),
            });

            Ok(())
        } else {
            Err(format!("Session {} not found", session_id))
        }
    }

    pub fn resume_session(&self, session_id: &str) -> Result<(), String> {
        let mut sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get_mut(session_id) {
            session.status = SessionStatus::Active;
            session.last_accessed = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            self.emit_event(TerminalEvent {
                event_type: TerminalEventType::SessionResumed,
                session_id: session_id.to_string(),
                pane_id: None,
                tab_id: None,
                timestamp: session.last_accessed,
                data: HashMap::new(),
            });

            Ok(())
        } else {
            Err(format!("Session {} not found", session_id))
        }
    }

    pub fn get_event_history(&self) -> Vec<TerminalEvent> {
        let history = self.event_history.lock().unwrap();
        history.iter().cloned().collect()
    }

    pub fn export_session(&self, session_id: &str) -> Result<String, String> {
        let session = {
            let sessions = self.sessions.lock().unwrap();
            sessions.get(session_id).cloned()
                .ok_or_else(|| format!("Session {} not found", session_id))?
        };

        serde_json::to_string_pretty(&session)
            .map_err(|e| format!("Failed to serialize session: {}", e))
    }

    pub fn import_session(&self, json_data: &str) -> Result<String, String> {
        let session: TerminalSession = serde_json::from_str(json_data)
            .map_err(|e| format!("Failed to parse session JSON: {}", e))?;

        let new_session_id = self.generate_session_id();
        let mut imported_session = session;
        imported_session.session_id = new_session_id.clone();

        {
            let mut sessions = self.sessions.lock().unwrap();
            sessions.insert(new_session_id.clone(), imported_session);
        }

        Ok(new_session_id)
    }
}
