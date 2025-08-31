use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::process::Command;
use tokio::sync::mpsc;
use tokio::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitRepository {
    pub path: PathBuf,
    pub name: String,
    pub remote_url: Option<String>,
    pub current_branch: String,
    pub status: GitStatus,
    pub last_commit: Option<GitCommit>,
    pub stash_count: usize,
    pub ahead: usize,
    pub behind: usize,
    pub is_dirty: bool,
    pub conflicts: Vec<String>,
    pub submodules: Vec<GitSubmodule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitStatus {
    pub staged: Vec<GitFileStatus>,
    pub unstaged: Vec<GitFileStatus>,
    pub untracked: Vec<String>,
    pub ignored: Vec<String>,
    pub conflicted: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitFileStatus {
    pub path: String,
    pub status: GitFileChange,
    pub additions: usize,
    pub deletions: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GitFileChange {
    Added,
    Modified,
    Deleted,
    Renamed,
    Copied,
    Unmerged,
    TypeChanged,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitCommit {
    pub hash: String,
    pub short_hash: String,
    pub author: String,
    pub email: String,
    pub message: String,
    pub timestamp: u64,
    pub files_changed: usize,
    pub insertions: usize,
    pub deletions: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitBranch {
    pub name: String,
    pub is_current: bool,
    pub is_remote: bool,
    pub upstream: Option<String>,
    pub last_commit: Option<GitCommit>,
    pub ahead: usize,
    pub behind: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitSubmodule {
    pub name: String,
    pub path: String,
    pub url: String,
    pub branch: Option<String>,
    pub status: SubmoduleStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SubmoduleStatus {
    Uninitialized,
    Initialized,
    Updated,
    OutOfDate,
    Modified,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitRemote {
    pub name: String,
    pub fetch_url: String,
    pub push_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitStash {
    pub index: usize,
    pub branch: String,
    pub message: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageServer {
    pub id: String,
    pub name: String,
    pub language: String,
    pub command: Vec<String>,
    pub working_directory: Option<PathBuf>,
    pub environment: HashMap<String, String>,
    pub status: LspStatus,
    pub capabilities: LspCapabilities,
    pub initialization_options: Option<serde_json::Value>,
    pub settings: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LspStatus {
    Stopped,
    Starting,
    Running,
    Error,
    Crashed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspCapabilities {
    pub hover: bool,
    pub completion: bool,
    pub signature_help: bool,
    pub goto_definition: bool,
    pub goto_references: bool,
    pub document_symbols: bool,
    pub workspace_symbols: bool,
    pub code_actions: bool,
    pub formatting: bool,
    pub range_formatting: bool,
    pub rename: bool,
    pub diagnostics: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspDiagnostic {
    pub file_path: String,
    pub line: usize,
    pub column: usize,
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub source: Option<String>,
    pub code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Information,
    Hint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Debugger {
    pub id: String,
    pub name: String,
    pub language: String,
    pub adapter: DebugAdapter,
    pub status: DebuggerStatus,
    pub current_session: Option<DebugSession>,
    pub breakpoints: Vec<Breakpoint>,
    pub variables: HashMap<String, DebugVariable>,
    pub call_stack: Vec<StackFrame>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugAdapter {
    pub adapter_type: String,
    pub command: Vec<String>,
    pub port: Option<u16>,
    pub host: Option<String>,
    pub env: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DebuggerStatus {
    Stopped,
    Starting,
    Running,
    Paused,
    Debugging,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugSession {
    pub id: String,
    pub program: String,
    pub arguments: Vec<String>,
    pub working_directory: String,
    pub environment: HashMap<String, String>,
    pub started_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Breakpoint {
    pub id: String,
    pub file_path: String,
    pub line: usize,
    pub condition: Option<String>,
    pub hit_condition: Option<String>,
    pub log_message: Option<String>,
    pub enabled: bool,
    pub verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugVariable {
    pub name: String,
    pub value: String,
    pub variable_type: String,
    pub scope: VariableScope,
    pub children: Vec<DebugVariable>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VariableScope {
    Local,
    Global,
    Parameter,
    Return,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackFrame {
    pub id: usize,
    pub name: String,
    pub file_path: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub language: String,
    pub framework: Option<String>,
    pub tags: Vec<String>,
    pub files: Vec<TemplateFile>,
    pub post_creation_commands: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateFile {
    pub path: String,
    pub content: String,
    pub is_template: bool, // If true, content contains variables like {{project_name}}
    pub executable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfiguration {
    pub name: String,
    pub command: Vec<String>,
    pub working_directory: Option<PathBuf>,
    pub environment: HashMap<String, String>,
    pub pre_build_commands: Vec<String>,
    pub post_build_commands: Vec<String>,
    pub watch_patterns: Vec<String>,
    pub ignore_patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConfiguration {
    pub name: String,
    pub command: Vec<String>,
    pub test_pattern: Option<String>,
    pub coverage_enabled: bool,
    pub parallel: bool,
    pub timeout: Option<u64>,
    pub environment: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub name: String,
    pub status: TestStatus,
    pub duration: Duration,
    pub message: Option<String>,
    pub file_path: Option<String>,
    pub line: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TestStatus {
    Passed,
    Failed,
    Skipped,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevToolsEvent {
    pub event_type: DevToolsEventType,
    pub timestamp: u64,
    pub details: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DevToolsEventType {
    GitStatusChanged,
    LspServerStarted,
    LspServerStopped,
    DiagnosticsUpdated,
    DebugSessionStarted,
    DebugSessionStopped,
    BreakpointHit,
    BuildStarted,
    BuildCompleted,
    TestsStarted,
    TestsCompleted,
}

pub struct DevToolsManager {
    git_repositories: Arc<Mutex<HashMap<String, GitRepository>>>,
    language_servers: Arc<Mutex<HashMap<String, LanguageServer>>>,
    debuggers: Arc<Mutex<HashMap<String, Debugger>>>,
    project_templates: Arc<Mutex<HashMap<String, ProjectTemplate>>>,
    build_configs: Arc<Mutex<HashMap<String, BuildConfiguration>>>,
    test_configs: Arc<Mutex<HashMap<String, TestConfiguration>>>,
    diagnostics: Arc<Mutex<Vec<LspDiagnostic>>>,
    event_history: Arc<Mutex<VecDeque<DevToolsEvent>>>,
    event_sender: Arc<Mutex<Option<mpsc::UnboundedSender<DevToolsEvent>>>>,
}

impl DevToolsManager {
    pub fn new() -> Self {
        Self {
            git_repositories: Arc::new(Mutex::new(HashMap::new())),
            language_servers: Arc::new(Mutex::new(HashMap::new())),
            debuggers: Arc::new(Mutex::new(HashMap::new())),
            project_templates: Arc::new(Mutex::new(HashMap::new())),
            build_configs: Arc::new(Mutex::new(HashMap::new())),
            test_configs: Arc::new(Mutex::new(HashMap::new())),
            diagnostics: Arc::new(Mutex::new(Vec::new())),
            event_history: Arc::new(Mutex::new(VecDeque::with_capacity(1000))),
            event_sender: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn start_event_monitoring(&self) -> Result<mpsc::UnboundedReceiver<DevToolsEvent>, String> {
        let (tx, rx) = mpsc::unbounded_channel();

        {
            let mut sender = self.event_sender.lock().unwrap();
            *sender = Some(tx);
        }

        Ok(rx)
    }

    fn emit_event(&self, event: DevToolsEvent) {
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

    // Git Integration
    pub async fn discover_git_repositories(&self, base_path: &PathBuf) -> Result<Vec<String>, String> {
        let mut discovered = Vec::new();
        let mut entries = fs::read_dir(base_path).await
            .map_err(|e| format!("Failed to read directory: {}", e))?;

        while let Some(entry) = entries.next_entry().await
            .map_err(|e| format!("Failed to read entry: {}", e))? {
            
            let path = entry.path();
            if path.is_dir() {
                let git_dir = path.join(".git");
                if git_dir.exists() {
                    if let Ok(repo) = self.load_git_repository(&path).await {
                        discovered.push(repo.name.clone());
                    }
                }
            }
        }

        Ok(discovered)
    }

    pub async fn load_git_repository(&self, path: &PathBuf) -> Result<GitRepository, String> {
        let name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let current_branch = self.get_git_current_branch(path).await?;
        let status = self.get_git_status(path).await?;
        let remote_url = self.get_git_remote_url(path).await.ok();
        let last_commit = self.get_git_last_commit(path).await.ok();
        let stash_count = self.get_git_stash_count(path).await.unwrap_or(0);
        let (ahead, behind) = self.get_git_ahead_behind(path).await.unwrap_or((0, 0));

        let is_dirty = !status.staged.is_empty() || !status.unstaged.is_empty() || !status.untracked.is_empty();
        let conflicts = status.conflicted.clone();
        let submodules = self.get_git_submodules(path).await.unwrap_or_default();

        let repository = GitRepository {
            path: path.clone(),
            name,
            remote_url,
            current_branch,
            status,
            last_commit,
            stash_count,
            ahead,
            behind,
            is_dirty,
            conflicts,
            submodules,
        };

        {
            let mut repos = self.git_repositories.lock().unwrap();
            repos.insert(repository.name.clone(), repository.clone());
        }

        self.emit_event(DevToolsEvent {
            event_type: DevToolsEventType::GitStatusChanged,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            details: [("repository".to_string(), serde_json::Value::String(repository.name.clone()))]
                .into_iter().collect(),
        });

        Ok(repository)
    }

    async fn get_git_current_branch(&self, path: &PathBuf) -> Result<String, String> {
        let output = Command::new("git")
            .args(&["rev-parse", "--abbrev-ref", "HEAD"])
            .current_dir(path)
            .output()
            .await
            .map_err(|e| format!("Failed to get current branch: {}", e))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err("Failed to get current branch".to_string())
        }
    }

    async fn get_git_status(&self, path: &PathBuf) -> Result<GitStatus, String> {
        let output = Command::new("git")
            .args(&["status", "--porcelain=v1"])
            .current_dir(path)
            .output()
            .await
            .map_err(|e| format!("Failed to get git status: {}", e))?;

        if !output.status.success() {
            return Err("Failed to get git status".to_string());
        }

        let mut status = GitStatus {
            staged: Vec::new(),
            unstaged: Vec::new(),
            untracked: Vec::new(),
            ignored: Vec::new(),
            conflicted: Vec::new(),
        };

        let output_str = String::from_utf8_lossy(&output.stdout);
        for line in output_str.lines() {
            if line.len() < 3 {
                continue;
            }

            let status_chars: Vec<char> = line.chars().collect();
            let index_status = status_chars[0];
            let worktree_status = status_chars[1];
            let file_path = &line[3..];

            // Handle conflicts
            if index_status == 'U' || worktree_status == 'U' ||
               (index_status == 'A' && worktree_status == 'A') ||
               (index_status == 'D' && worktree_status == 'D') {
                status.conflicted.push(file_path.to_string());
                continue;
            }

            // Handle staged changes
            match index_status {
                'A' => status.staged.push(GitFileStatus {
                    path: file_path.to_string(),
                    status: GitFileChange::Added,
                    additions: 0,
                    deletions: 0,
                }),
                'M' => status.staged.push(GitFileStatus {
                    path: file_path.to_string(),
                    status: GitFileChange::Modified,
                    additions: 0,
                    deletions: 0,
                }),
                'D' => status.staged.push(GitFileStatus {
                    path: file_path.to_string(),
                    status: GitFileChange::Deleted,
                    additions: 0,
                    deletions: 0,
                }),
                'R' => status.staged.push(GitFileStatus {
                    path: file_path.to_string(),
                    status: GitFileChange::Renamed,
                    additions: 0,
                    deletions: 0,
                }),
                'C' => status.staged.push(GitFileStatus {
                    path: file_path.to_string(),
                    status: GitFileChange::Copied,
                    additions: 0,
                    deletions: 0,
                }),
                _ => {}
            }

            // Handle unstaged changes
            match worktree_status {
                'M' => status.unstaged.push(GitFileStatus {
                    path: file_path.to_string(),
                    status: GitFileChange::Modified,
                    additions: 0,
                    deletions: 0,
                }),
                'D' => status.unstaged.push(GitFileStatus {
                    path: file_path.to_string(),
                    status: GitFileChange::Deleted,
                    additions: 0,
                    deletions: 0,
                }),
                '?' => status.untracked.push(file_path.to_string()),
                '!' => status.ignored.push(file_path.to_string()),
                _ => {}
            }
        }

        Ok(status)
    }

    async fn get_git_remote_url(&self, path: &PathBuf) -> Result<String, String> {
        let output = Command::new("git")
            .args(&["remote", "get-url", "origin"])
            .current_dir(path)
            .output()
            .await
            .map_err(|e| format!("Failed to get remote URL: {}", e))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err("No remote origin found".to_string())
        }
    }

    async fn get_git_last_commit(&self, path: &PathBuf) -> Result<GitCommit, String> {
        let output = Command::new("git")
            .args(&["log", "-1", "--pretty=format:%H|%h|%an|%ae|%s|%ct", "--numstat"])
            .current_dir(path)
            .output()
            .await
            .map_err(|e| format!("Failed to get last commit: {}", e))?;

        if !output.status.success() {
            return Err("Failed to get last commit".to_string());
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = output_str.lines().collect();
        
        if lines.is_empty() {
            return Err("No commit found".to_string());
        }

        let commit_line = lines[0];
        let parts: Vec<&str> = commit_line.split('|').collect();
        
        if parts.len() < 6 {
            return Err("Invalid commit format".to_string());
        }

        let mut insertions = 0;
        let mut deletions = 0;
        let mut files_changed = 0;

        // Parse numstat output
        for line in lines.iter().skip(1) {
            if line.trim().is_empty() {
                continue;
            }
            let stat_parts: Vec<&str> = line.split_whitespace().collect();
            if stat_parts.len() >= 2 {
                if let (Ok(ins), Ok(del)) = (stat_parts[0].parse::<usize>(), stat_parts[1].parse::<usize>()) {
                    insertions += ins;
                    deletions += del;
                    files_changed += 1;
                }
            }
        }

        Ok(GitCommit {
            hash: parts[0].to_string(),
            short_hash: parts[1].to_string(),
            author: parts[2].to_string(),
            email: parts[3].to_string(),
            message: parts[4].to_string(),
            timestamp: parts[5].parse().unwrap_or(0),
            files_changed,
            insertions,
            deletions,
        })
    }

    async fn get_git_stash_count(&self, path: &PathBuf) -> Result<usize, String> {
        let output = Command::new("git")
            .args(&["stash", "list"])
            .current_dir(path)
            .output()
            .await
            .map_err(|e| format!("Failed to get stash count: {}", e))?;

        if output.status.success() {
            let count = String::from_utf8_lossy(&output.stdout).lines().count();
            Ok(count)
        } else {
            Ok(0)
        }
    }

    async fn get_git_ahead_behind(&self, path: &PathBuf) -> Result<(usize, usize), String> {
        let output = Command::new("git")
            .args(&["rev-list", "--left-right", "--count", "HEAD...@{upstream}"])
            .current_dir(path)
            .output()
            .await
            .map_err(|e| format!("Failed to get ahead/behind count: {}", e))?;

        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let parts: Vec<&str> = output_str.trim().split_whitespace().collect();
            if parts.len() == 2 {
                let ahead = parts[0].parse().unwrap_or(0);
                let behind = parts[1].parse().unwrap_or(0);
                return Ok((ahead, behind));
            }
        }

        Ok((0, 0))
    }

    async fn get_git_submodules(&self, path: &PathBuf) -> Result<Vec<GitSubmodule>, String> {
        let output = Command::new("git")
            .args(&["submodule", "status"])
            .current_dir(path)
            .output()
            .await
            .map_err(|e| format!("Failed to get submodules: {}", e))?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let mut submodules = Vec::new();
        let output_str = String::from_utf8_lossy(&output.stdout);
        
        for line in output_str.lines() {
            if line.is_empty() {
                continue;
            }

            let status = match line.chars().next() {
                Some('-') => SubmoduleStatus::Uninitialized,
                Some(' ') => SubmoduleStatus::Updated,
                Some('+') => SubmoduleStatus::Modified,
                Some('U') => SubmoduleStatus::OutOfDate,
                _ => SubmoduleStatus::Initialized,
            };

            let parts: Vec<&str> = line[1..].split_whitespace().collect();
            if parts.len() >= 2 {
                submodules.push(GitSubmodule {
                    name: parts[1].to_string(),
                    path: parts[1].to_string(),
                    url: "".to_string(), // Would need to parse .gitmodules
                    branch: None,
                    status,
                });
            }
        }

        Ok(submodules)
    }

    pub async fn git_commit(&self, repo_name: &str, message: &str, files: Vec<String>) -> Result<String, String> {
        let repo_path = {
            let repos = self.git_repositories.lock().unwrap();
            repos.get(repo_name)
                .map(|r| r.path.clone())
                .ok_or_else(|| format!("Repository {} not found", repo_name))?
        };

        // Add files
        for file in &files {
            let output = Command::new("git")
                .args(&["add", file])
                .current_dir(&repo_path)
                .output()
                .await
                .map_err(|e| format!("Failed to add file {}: {}", file, e))?;

            if !output.status.success() {
                return Err(format!("Failed to add file {}", file));
            }
        }

        // Commit
        let output = Command::new("git")
            .args(&["commit", "-m", message])
            .current_dir(&repo_path)
            .output()
            .await
            .map_err(|e| format!("Failed to commit: {}", e))?;

        if output.status.success() {
            // Refresh repository status
            let _ = self.load_git_repository(&repo_path).await;
            Ok("Commit successful".to_string())
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(format!("Commit failed: {}", error))
        }
    }

    pub async fn git_push(&self, repo_name: &str, remote: &str, branch: &str) -> Result<String, String> {
        let repo_path = {
            let repos = self.git_repositories.lock().unwrap();
            repos.get(repo_name)
                .map(|r| r.path.clone())
                .ok_or_else(|| format!("Repository {} not found", repo_name))?
        };

        let output = Command::new("git")
            .args(&["push", remote, branch])
            .current_dir(&repo_path)
            .output()
            .await
            .map_err(|e| format!("Failed to push: {}", e))?;

        if output.status.success() {
            let _ = self.load_git_repository(&repo_path).await;
            Ok("Push successful".to_string())
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(format!("Push failed: {}", error))
        }
    }

    pub async fn git_pull(&self, repo_name: &str) -> Result<String, String> {
        let repo_path = {
            let repos = self.git_repositories.lock().unwrap();
            repos.get(repo_name)
                .map(|r| r.path.clone())
                .ok_or_else(|| format!("Repository {} not found", repo_name))?
        };

        let output = Command::new("git")
            .args(&["pull"])
            .current_dir(&repo_path)
            .output()
            .await
            .map_err(|e| format!("Failed to pull: {}", e))?;

        if output.status.success() {
            let _ = self.load_git_repository(&repo_path).await;
            Ok("Pull successful".to_string())
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(format!("Pull failed: {}", error))
        }
    }

    // Language Server Protocol (LSP) Integration
    pub fn register_language_server(&self, language_server: LanguageServer) -> Result<String, String> {
        let server_id = language_server.id.clone();
        
        {
            let mut servers = self.language_servers.lock().unwrap();
            servers.insert(server_id.clone(), language_server);
        }

        Ok(server_id)
    }

    pub async fn start_language_server(&self, server_id: &str) -> Result<(), String> {
        let mut server = {
            let servers = self.language_servers.lock().unwrap();
            servers.get(server_id).cloned()
                .ok_or_else(|| format!("Language server {} not found", server_id))?
        };

        server.status = LspStatus::Starting;

        // Update status
        {
            let mut servers = self.language_servers.lock().unwrap();
            servers.insert(server_id.to_string(), server.clone());
        }

        // Start LSP server process
        let mut cmd = Command::new(&server.command[0]);
        if server.command.len() > 1 {
            cmd.args(&server.command[1..]);
        }

        if let Some(ref working_dir) = server.working_directory {
            cmd.current_dir(working_dir);
        }

        for (key, value) in &server.environment {
            cmd.env(key, value);
        }

        match cmd.spawn() {
            Ok(_child) => {
                server.status = LspStatus::Running;
                
                {
                    let mut servers = self.language_servers.lock().unwrap();
                    servers.insert(server_id.to_string(), server);
                }

                self.emit_event(DevToolsEvent {
                    event_type: DevToolsEventType::LspServerStarted,
                    timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                    details: [("server_id".to_string(), serde_json::Value::String(server_id.to_string()))]
                        .into_iter().collect(),
                });

                Ok(())
            }
            Err(e) => {
                server.status = LspStatus::Error;
                
                {
                    let mut servers = self.language_servers.lock().unwrap();
                    servers.insert(server_id.to_string(), server);
                }

                Err(format!("Failed to start language server: {}", e))
            }
        }
    }

    pub fn stop_language_server(&self, server_id: &str) -> Result<(), String> {
        let mut servers = self.language_servers.lock().unwrap();
        if let Some(server) = servers.get_mut(server_id) {
            server.status = LspStatus::Stopped;
            
            self.emit_event(DevToolsEvent {
                event_type: DevToolsEventType::LspServerStopped,
                timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                details: [("server_id".to_string(), serde_json::Value::String(server_id.to_string()))]
                    .into_iter().collect(),
            });
            
            Ok(())
        } else {
            Err(format!("Language server {} not found", server_id))
        }
    }

    pub fn add_diagnostic(&self, diagnostic: LspDiagnostic) {
        {
            let mut diagnostics = self.diagnostics.lock().unwrap();
            diagnostics.push(diagnostic);
        }

        self.emit_event(DevToolsEvent {
            event_type: DevToolsEventType::DiagnosticsUpdated,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            details: HashMap::new(),
        });
    }

    pub fn get_diagnostics(&self, file_path: Option<&str>) -> Vec<LspDiagnostic> {
        let diagnostics = self.diagnostics.lock().unwrap();
        
        if let Some(path) = file_path {
            diagnostics.iter()
                .filter(|d| d.file_path == path)
                .cloned()
                .collect()
        } else {
            diagnostics.clone()
        }
    }

    pub fn clear_diagnostics(&self, file_path: Option<&str>) {
        let mut diagnostics = self.diagnostics.lock().unwrap();
        
        if let Some(path) = file_path {
            diagnostics.retain(|d| d.file_path != path);
        } else {
            diagnostics.clear();
        }
    }

    // Build System Integration
    pub fn add_build_configuration(&self, config: BuildConfiguration) -> Result<String, String> {
        let config_name = config.name.clone();
        
        {
            let mut configs = self.build_configs.lock().unwrap();
            configs.insert(config_name.clone(), config);
        }

        Ok(config_name)
    }

    pub async fn run_build(&self, config_name: &str) -> Result<String, String> {
        let config = {
            let configs = self.build_configs.lock().unwrap();
            configs.get(config_name).cloned()
                .ok_or_else(|| format!("Build configuration {} not found", config_name))?
        };

        self.emit_event(DevToolsEvent {
            event_type: DevToolsEventType::BuildStarted,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            details: [("config".to_string(), serde_json::Value::String(config_name.to_string()))]
                .into_iter().collect(),
        });

        // Run pre-build commands
        for cmd_str in &config.pre_build_commands {
            let parts: Vec<&str> = cmd_str.split_whitespace().collect();
            if !parts.is_empty() {
                let mut cmd = Command::new(parts[0]);
                if parts.len() > 1 {
                    cmd.args(&parts[1..]);
                }
                
                if let Some(ref working_dir) = config.working_directory {
                    cmd.current_dir(working_dir);
                }

                for (key, value) in &config.environment {
                    cmd.env(key, value);
                }

                let output = cmd.output().await
                    .map_err(|e| format!("Failed to run pre-build command: {}", e))?;

                if !output.status.success() {
                    let error = String::from_utf8_lossy(&output.stderr);
                    return Err(format!("Pre-build command failed: {}", error));
                }
            }
        }

        // Run main build command
        let mut cmd = Command::new(&config.command[0]);
        if config.command.len() > 1 {
            cmd.args(&config.command[1..]);
        }

        if let Some(ref working_dir) = config.working_directory {
            cmd.current_dir(working_dir);
        }

        for (key, value) in &config.environment {
            cmd.env(key, value);
        }

        let output = cmd.output().await
            .map_err(|e| format!("Failed to run build command: {}", e))?;

        let success = output.status.success();
        let result_message = if success {
            String::from_utf8_lossy(&output.stdout).to_string()
        } else {
            String::from_utf8_lossy(&output.stderr).to_string()
        };

        // Run post-build commands if build succeeded
        if success {
            for cmd_str in &config.post_build_commands {
                let parts: Vec<&str> = cmd_str.split_whitespace().collect();
                if !parts.is_empty() {
                    let mut cmd = Command::new(parts[0]);
                    if parts.len() > 1 {
                        cmd.args(&parts[1..]);
                    }
                    
                    if let Some(ref working_dir) = config.working_directory {
                        cmd.current_dir(working_dir);
                    }

                    for (key, value) in &config.environment {
                        cmd.env(key, value);
                    }

                    let _ = cmd.output().await; // Don't fail build if post-build fails
                }
            }
        }

        self.emit_event(DevToolsEvent {
            event_type: DevToolsEventType::BuildCompleted,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            details: [
                ("config".to_string(), serde_json::Value::String(config_name.to_string())),
                ("success".to_string(), serde_json::Value::Bool(success)),
            ].into_iter().collect(),
        });

        if success {
            Ok(result_message)
        } else {
            Err(result_message)
        }
    }

    // Test Integration
    pub fn add_test_configuration(&self, config: TestConfiguration) -> Result<String, String> {
        let config_name = config.name.clone();
        
        {
            let mut configs = self.test_configs.lock().unwrap();
            configs.insert(config_name.clone(), config);
        }

        Ok(config_name)
    }

    pub async fn run_tests(&self, config_name: &str) -> Result<Vec<TestResult>, String> {
        let config = {
            let configs = self.test_configs.lock().unwrap();
            configs.get(config_name).cloned()
                .ok_or_else(|| format!("Test configuration {} not found", config_name))?
        };

        self.emit_event(DevToolsEvent {
            event_type: DevToolsEventType::TestsStarted,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            details: [("config".to_string(), serde_json::Value::String(config_name.to_string()))]
                .into_iter().collect(),
        });

        let mut cmd = Command::new(&config.command[0]);
        if config.command.len() > 1 {
            cmd.args(&config.command[1..]);
        }

        for (key, value) in &config.environment {
            cmd.env(key, value);
        }

        let start_time = std::time::Instant::now();
        let output = cmd.output().await
            .map_err(|e| format!("Failed to run tests: {}", e))?;
        let duration = start_time.elapsed();

        let success = output.status.success();
        let output_str = String::from_utf8_lossy(&output.stdout);

        // Simple test result parsing - would be more sophisticated in real implementation
        let mut results = Vec::new();
        for line in output_str.lines() {
            if line.contains("PASS") || line.contains("FAIL") || line.contains("SKIP") {
                let status = if line.contains("PASS") {
                    TestStatus::Passed
                } else if line.contains("FAIL") {
                    TestStatus::Failed
                } else {
                    TestStatus::Skipped
                };

                results.push(TestResult {
                    name: line.to_string(),
                    status,
                    duration,
                    message: None,
                    file_path: None,
                    line: None,
                });
            }
        }

        // If no specific test results found, create a summary result
        if results.is_empty() {
            results.push(TestResult {
                name: "Test Suite".to_string(),
                status: if success { TestStatus::Passed } else { TestStatus::Failed },
                duration,
                message: Some(output_str.to_string()),
                file_path: None,
                line: None,
            });
        }

        self.emit_event(DevToolsEvent {
            event_type: DevToolsEventType::TestsCompleted,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            details: [
                ("config".to_string(), serde_json::Value::String(config_name.to_string())),
                ("success".to_string(), serde_json::Value::Bool(success)),
                ("test_count".to_string(), serde_json::Value::Number(results.len().into())),
            ].into_iter().collect(),
        });

        Ok(results)
    }

    // Project Templates
    pub fn add_project_template(&self, template: ProjectTemplate) -> Result<String, String> {
        let template_id = template.id.clone();
        
        {
            let mut templates = self.project_templates.lock().unwrap();
            templates.insert(template_id.clone(), template);
        }

        Ok(template_id)
    }

    pub async fn create_project_from_template(
        &self,
        template_id: &str,
        project_name: &str,
        target_path: &PathBuf,
        variables: HashMap<String, String>,
    ) -> Result<(), String> {
        let template = {
            let templates = self.project_templates.lock().unwrap();
            templates.get(template_id).cloned()
                .ok_or_else(|| format!("Template {} not found", template_id))?
        };

        let project_path = target_path.join(project_name);
        fs::create_dir_all(&project_path).await
            .map_err(|e| format!("Failed to create project directory: {}", e))?;

        // Create files from template
        for template_file in &template.files {
            let file_path = project_path.join(&template_file.path);
            
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent).await
                    .map_err(|e| format!("Failed to create directory: {}", e))?;
            }

            let content = if template_file.is_template {
                self.replace_template_variables(&template_file.content, &variables, project_name)
            } else {
                template_file.content.clone()
            };

            fs::write(&file_path, content).await
                .map_err(|e| format!("Failed to write file {}: {}", template_file.path, e))?;

            #[cfg(unix)]
            if template_file.executable {
                use std::os::unix::fs::PermissionsExt;
                let metadata = fs::metadata(&file_path).await
                    .map_err(|e| format!("Failed to get file metadata: {}", e))?;
                let mut permissions = metadata.permissions();
                permissions.set_mode(0o755);
                fs::set_permissions(&file_path, permissions).await
                    .map_err(|e| format!("Failed to set file permissions: {}", e))?;
            }
        }

        // Run post-creation commands
        for cmd_str in &template.post_creation_commands {
            let parts: Vec<&str> = cmd_str.split_whitespace().collect();
            if !parts.is_empty() {
                let mut cmd = Command::new(parts[0]);
                if parts.len() > 1 {
                    cmd.args(&parts[1..]);
                }
                cmd.current_dir(&project_path);

                let _ = cmd.output().await; // Don't fail if post-creation commands fail
            }
        }

        Ok(())
    }

    fn replace_template_variables(&self, content: &str, variables: &HashMap<String, String>, project_name: &str) -> String {
        let mut result = content.replace("{{project_name}}", project_name);
        
        for (key, value) in variables {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }

        result
    }

    // Getters
    pub fn get_git_repositories(&self) -> Vec<GitRepository> {
        let repos = self.git_repositories.lock().unwrap();
        repos.values().cloned().collect()
    }

    pub fn get_git_repository(&self, repo_name: &str) -> Option<GitRepository> {
        let repos = self.git_repositories.lock().unwrap();
        repos.get(repo_name).cloned()
    }

    pub fn get_language_servers(&self) -> Vec<LanguageServer> {
        let servers = self.language_servers.lock().unwrap();
        servers.values().cloned().collect()
    }

    pub fn get_project_templates(&self) -> Vec<ProjectTemplate> {
        let templates = self.project_templates.lock().unwrap();
        templates.values().cloned().collect()
    }

    pub fn get_build_configurations(&self) -> Vec<BuildConfiguration> {
        let configs = self.build_configs.lock().unwrap();
        configs.values().cloned().collect()
    }

    pub fn get_test_configurations(&self) -> Vec<TestConfiguration> {
        let configs = self.test_configs.lock().unwrap();
        configs.values().cloned().collect()
    }

    pub fn get_event_history(&self) -> Vec<DevToolsEvent> {
        let history = self.event_history.lock().unwrap();
        history.iter().cloned().collect()
    }
}
