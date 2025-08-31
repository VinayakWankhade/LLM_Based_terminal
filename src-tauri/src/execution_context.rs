use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use chrono::{DateTime, Utc};
use tauri::State;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryState {
    pub pwd: String,
    pub home: String,
    pub previous: Option<String>,
    pub bookmarks: Vec<DirectoryBookmark>,
    pub recent_directories: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryBookmark {
    pub name: String,
    pub path: String,
    pub created_at: DateTime<Utc>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperatingSystem {
    pub platform: String,
    pub version: Option<String>,
    pub architecture: String,
    pub hostname: String,
    pub username: String,
    pub is_admin: bool,
    pub uptime: Option<u64>,
    pub cpu_info: CpuInfo,
    pub memory_info: MemoryInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuInfo {
    pub cores: usize,
    pub brand: String,
    pub frequency: Option<u64>,
    pub usage_percent: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    pub total: u64,
    pub available: u64,
    pub used: u64,
    pub usage_percent: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellInfo {
    pub name: String,
    pub version: String,
    pub path: String,
    pub pid: Option<u32>,
    pub parent_pid: Option<u32>,
    pub config_files: Vec<String>,
    pub features: ShellFeatures,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellFeatures {
    pub completion: bool,
    pub history: bool,
    pub job_control: bool,
    pub aliases: bool,
    pub functions: bool,
    pub variables: bool,
    pub scripting: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentContext {
    pub variables: HashMap<String, String>,
    pub path_entries: Vec<String>,
    pub locale: String,
    pub timezone: String,
    pub color_support: ColorSupport,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorSupport {
    pub colors_16: bool,
    pub colors_256: bool,
    pub truecolor: bool,
    pub color_scheme: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    pub directory_state: DirectoryState,
    pub operating_system: OperatingSystem,
    pub current_time: DateTime<Utc>,
    pub shell: ShellInfo,
    pub environment: EnvironmentContext,
    pub selected_text: Vec<String>,
    pub active_processes: Vec<ProcessInfo>,
    pub network_status: NetworkStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub command: String,
    pub cpu_usage: f32,
    pub memory_usage: u64,
    pub status: String,
    pub start_time: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStatus {
    pub interfaces: Vec<NetworkInterface>,
    pub active_connections: Vec<NetworkConnection>,
    pub dns_servers: Vec<String>,
    pub proxy_settings: Option<ProxySettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    pub name: String,
    pub ip_addresses: Vec<String>,
    pub mac_address: String,
    pub status: String,
    pub bytes_sent: u64,
    pub bytes_received: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConnection {
    pub local_address: String,
    pub remote_address: String,
    pub protocol: String,
    pub status: String,
    pub process_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxySettings {
    pub http_proxy: Option<String>,
    pub https_proxy: Option<String>,
    pub ftp_proxy: Option<String>,
    pub no_proxy: Option<String>,
}

pub type ExecutionContextManager = Arc<Mutex<ExecutionContextState>>;

pub struct ExecutionContextState {
    pub contexts: HashMap<String, ExecutionContext>,
    pub active_session: Option<String>,
}

impl ExecutionContextState {
    pub fn new() -> Self {
        Self {
            contexts: HashMap::new(),
            active_session: None,
        }
    }

    pub fn create_context(&mut self, session_id: String) -> tauri::Result<()> {
        let context = self.build_execution_context()?;
        self.contexts.insert(session_id.clone(), context);
        self.active_session = Some(session_id);
        Ok(())
    }

    pub fn get_context(&self, session_id: &str) -> Option<&ExecutionContext> {
        self.contexts.get(session_id)
    }

    pub fn update_context(&mut self, session_id: &str, context: ExecutionContext) {
        self.contexts.insert(session_id.to_string(), context);
    }

    pub fn refresh_context(&mut self, session_id: &str) -> tauri::Result<()> {
        if let Some(existing) = self.contexts.get(session_id) {
            let mut updated = self.build_execution_context()?;
            // Preserve user-specific data
            updated.selected_text = existing.selected_text.clone();
            updated.directory_state.bookmarks = existing.directory_state.bookmarks.clone();
            updated.directory_state.recent_directories = existing.directory_state.recent_directories.clone();
            
            self.contexts.insert(session_id.to_string(), updated);
        }
        Ok(())
    }

    fn build_execution_context(&self) -> tauri::Result<ExecutionContext> {
        Ok(ExecutionContext {
            directory_state: self.get_directory_state()?,
            operating_system: self.get_operating_system()?,
            current_time: Utc::now(),
            shell: self.get_shell_info()?,
            environment: self.get_environment_context()?,
            selected_text: Vec::new(),
            active_processes: self.get_active_processes()?,
            network_status: self.get_network_status()?,
        })
    }

    fn get_directory_state(&self) -> tauri::Result<DirectoryState> {
        let pwd = env::current_dir()
            .map_err(|e| tauri::Error::Io(e))?
            .to_string_lossy()
            .to_string();
            
        let home = env::var("HOME")
            .or_else(|_| env::var("USERPROFILE"))
            .unwrap_or_else(|_| "~".to_string());

        Ok(DirectoryState {
            pwd,
            home,
            previous: None,
            bookmarks: Vec::new(),
            recent_directories: Vec::new(),
        })
    }

    fn get_operating_system(&self) -> tauri::Result<OperatingSystem> {
        let platform = env::consts::OS.to_string();
        let architecture = env::consts::ARCH.to_string();
        let hostname = gethostname::gethostname()
            .to_string_lossy()
            .to_string();
        let username = env::var("USER")
            .or_else(|_| env::var("USERNAME"))
            .unwrap_or_else(|_| "unknown".to_string());

        Ok(OperatingSystem {
            platform,
            version: None,
            architecture,
            hostname,
            username,
            is_admin: self.check_admin_privileges(),
            uptime: None,
            cpu_info: self.get_cpu_info(),
            memory_info: self.get_memory_info(),
        })
    }

    fn get_shell_info(&self) -> tauri::Result<ShellInfo> {
        let shell_path = env::var("SHELL")
            .or_else(|_| env::var("ComSpec"))
            .unwrap_or_else(|_| "/bin/sh".to_string());
            
        let name = PathBuf::from(&shell_path)
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        Ok(ShellInfo {
            name: name.clone(),
            version: "unknown".to_string(),
            path: shell_path,
            pid: None,
            parent_pid: None,
            config_files: self.get_shell_config_files(&name),
            features: self.get_shell_features(&name),
        })
    }

    fn get_environment_context(&self) -> tauri::Result<EnvironmentContext> {
        let variables: HashMap<String, String> = env::vars().collect();
        let path_entries: Vec<String> = env::var("PATH")
            .unwrap_or_default()
            .split(if cfg!(windows) { ';' } else { ':' })
            .map(|s| s.to_string())
            .collect();

        Ok(EnvironmentContext {
            variables,
            path_entries,
            locale: env::var("LANG").unwrap_or_else(|_| "en_US.UTF-8".to_string()),
            timezone: env::var("TZ").unwrap_or_else(|_| "UTC".to_string()),
            color_support: self.detect_color_support(),
        })
    }

    fn get_active_processes(&self) -> tauri::Result<Vec<ProcessInfo>> {
        // Placeholder - would use system crates like sysinfo for real implementation
        Ok(Vec::new())
    }

    fn get_network_status(&self) -> tauri::Result<NetworkStatus> {
        // Placeholder - would use network system crates for real implementation
        Ok(NetworkStatus {
            interfaces: Vec::new(),
            active_connections: Vec::new(),
            dns_servers: Vec::new(),
            proxy_settings: None,
        })
    }

    fn check_admin_privileges(&self) -> bool {
        #[cfg(windows)]
        {
            // Windows admin check would go here
            false
        }
        #[cfg(unix)]
        {
            unsafe { libc::geteuid() == 0 }
        }
        #[cfg(not(any(windows, unix)))]
        {
            false
        }
    }

    fn get_cpu_info(&self) -> CpuInfo {
        CpuInfo {
            cores: num_cpus::get(),
            brand: "Unknown".to_string(),
            frequency: None,
            usage_percent: 0.0,
        }
    }

    fn get_memory_info(&self) -> MemoryInfo {
        MemoryInfo {
            total: 0,
            available: 0,
            used: 0,
            usage_percent: 0.0,
        }
    }

    fn get_shell_config_files(&self, shell_name: &str) -> Vec<String> {
        match shell_name {
            "bash" => vec![".bashrc".to_string(), ".bash_profile".to_string(), ".profile".to_string()],
            "zsh" => vec![".zshrc".to_string(), ".zprofile".to_string()],
            "fish" => vec!["config.fish".to_string()],
            "pwsh" | "powershell" => vec!["Microsoft.PowerShell_profile.ps1".to_string()],
            _ => Vec::new(),
        }
    }

    fn get_shell_features(&self, shell_name: &str) -> ShellFeatures {
        match shell_name {
            "bash" | "zsh" | "fish" => ShellFeatures {
                completion: true,
                history: true,
                job_control: true,
                aliases: true,
                functions: true,
                variables: true,
                scripting: true,
            },
            "pwsh" | "powershell" => ShellFeatures {
                completion: true,
                history: true,
                job_control: true,
                aliases: true,
                functions: true,
                variables: true,
                scripting: true,
            },
            _ => ShellFeatures {
                completion: false,
                history: false,
                job_control: false,
                aliases: false,
                functions: false,
                variables: false,
                scripting: false,
            },
        }
    }

    fn detect_color_support(&self) -> ColorSupport {
        let term = env::var("TERM").unwrap_or_default();
        let colorterm = env::var("COLORTERM").unwrap_or_default();

        ColorSupport {
            colors_16: !term.is_empty(),
            colors_256: term.contains("256") || term.contains("xterm"),
            truecolor: colorterm.contains("truecolor") || colorterm.contains("24bit"),
            color_scheme: env::var("COLORFGBG").unwrap_or_else(|_| "default".to_string()),
        }
    }
}

// Tauri commands
#[tauri::command]
pub async fn get_execution_context(
    session_id: String,
    context_manager: State<'_, ExecutionContextManager>,
) -> Result<Option<ExecutionContext>, String> {
    let manager = context_manager.lock().map_err(|e| e.to_string())?;
    Ok(manager.get_context(&session_id).cloned())
}

#[tauri::command]
pub async fn create_execution_context(
    session_id: String,
    context_manager: State<'_, ExecutionContextManager>,
) -> Result<(), String> {
    let mut manager = context_manager.lock().map_err(|e| e.to_string())?;
    manager.create_context(session_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn refresh_execution_context(
    session_id: String,
    context_manager: State<'_, ExecutionContextManager>,
) -> Result<(), String> {
    let mut manager = context_manager.lock().map_err(|e| e.to_string())?;
    manager.refresh_context(&session_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_selected_text(
    session_id: String,
    selected_text: Vec<String>,
    context_manager: State<'_, ExecutionContextManager>,
) -> Result<(), String> {
    let mut manager = context_manager.lock().map_err(|e| e.to_string())?;
    if let Some(context) = manager.contexts.get_mut(&session_id) {
        context.selected_text = selected_text;
    }
    Ok(())
}

#[tauri::command]
pub async fn add_directory_bookmark(
    session_id: String,
    name: String,
    path: String,
    tags: Vec<String>,
    context_manager: State<'_, ExecutionContextManager>,
) -> Result<(), String> {
    let mut manager = context_manager.lock().map_err(|e| e.to_string())?;
    if let Some(context) = manager.contexts.get_mut(&session_id) {
        let bookmark = DirectoryBookmark {
            name,
            path,
            created_at: Utc::now(),
            tags,
        };
        context.directory_state.bookmarks.push(bookmark);
    }
    Ok(())
}

#[tauri::command]
pub async fn get_directory_bookmarks(
    session_id: String,
    context_manager: State<'_, ExecutionContextManager>,
) -> Result<Vec<DirectoryBookmark>, String> {
    let manager = context_manager.lock().map_err(|e| e.to_string())?;
    if let Some(context) = manager.get_context(&session_id) {
        Ok(context.directory_state.bookmarks.clone())
    } else {
        Ok(Vec::new())
    }
}

#[tauri::command]
pub async fn update_current_directory(
    session_id: String,
    new_path: String,
    context_manager: State<'_, ExecutionContextManager>,
) -> Result<(), String> {
    let mut manager = context_manager.lock().map_err(|e| e.to_string())?;
    if let Some(context) = manager.contexts.get_mut(&session_id) {
        context.directory_state.previous = Some(context.directory_state.pwd.clone());
        context.directory_state.pwd = new_path.clone();
        
        // Add to recent directories
        if !context.directory_state.recent_directories.contains(&new_path) {
            context.directory_state.recent_directories.insert(0, new_path);
            context.directory_state.recent_directories.truncate(20); // Keep last 20
        }
    }
    Ok(())
}
