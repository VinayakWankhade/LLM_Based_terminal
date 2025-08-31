use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::process::{Command, Stdio};
use std::io::{BufReader, Write};
use std::path::{Path, PathBuf};
use chrono::{DateTime, Utc};
use tauri::State;
use std::sync::{Arc, Mutex};
use regex::Regex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellCompletion {
    pub text: String,
    pub display: String,
    pub description: Option<String>,
    pub completion_type: CompletionType,
    pub priority: i32,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompletionType {
    Command,
    File,
    Directory,
    Variable,
    Alias,
    Function,
    Argument,
    Flag,
    History,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptConfiguration {
    pub template: String,
    pub segments: Vec<PromptSegment>,
    pub colors: PromptColors,
    pub icons: PromptIcons,
    pub show_git: bool,
    pub show_duration: bool,
    pub show_exit_code: bool,
    pub multiline: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptSegment {
    pub name: String,
    pub enabled: bool,
    pub format: String,
    pub condition: Option<String>,
    pub color: Option<String>,
    pub background: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptColors {
    pub primary: String,
    pub secondary: String,
    pub success: String,
    pub warning: String,
    pub error: String,
    pub info: String,
    pub directory: String,
    pub git: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptIcons {
    pub directory: String,
    pub git_branch: String,
    pub git_modified: String,
    pub git_staged: String,
    pub git_untracked: String,
    pub success: String,
    pub error: String,
    pub lock: String,
    pub user: String,
    pub host: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandHistory {
    pub id: String,
    pub command: String,
    pub directory: String,
    pub timestamp: DateTime<Utc>,
    pub exit_code: Option<i32>,
    pub duration: Option<u64>,
    pub session_id: String,
    pub tags: Vec<String>,
    pub favorite: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellAlias {
    pub name: String,
    pub command: String,
    pub description: Option<String>,
    pub shell_specific: Option<String>,
    pub created_at: DateTime<Utc>,
    pub usage_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellFunction {
    pub name: String,
    pub body: String,
    pub description: Option<String>,
    pub parameters: Vec<String>,
    pub shell_specific: String,
    pub created_at: DateTime<Utc>,
    pub usage_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellVariable {
    pub name: String,
    pub value: String,
    pub var_type: VariableType,
    pub exported: bool,
    pub readonly: bool,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VariableType {
    String,
    Integer,
    Array,
    Hash,
    Path,
    Boolean,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellScript {
    pub id: String,
    pub name: String,
    pub content: String,
    pub language: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub executable: bool,
    pub auto_run: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitStatus {
    pub branch: Option<String>,
    pub ahead: u32,
    pub behind: u32,
    pub staged: u32,
    pub modified: u32,
    pub untracked: u32,
    pub conflicts: u32,
    pub stashes: u32,
    pub is_dirty: bool,
    pub is_detached: bool,
}

pub type ShellIntegrationManager = Arc<Mutex<ShellIntegrationState>>;

pub struct ShellIntegrationState {
    pub completions_cache: HashMap<String, Vec<ShellCompletion>>,
    pub history: VecDeque<CommandHistory>,
    pub aliases: HashMap<String, ShellAlias>,
    pub functions: HashMap<String, ShellFunction>,
    pub variables: HashMap<String, ShellVariable>,
    pub scripts: HashMap<String, ShellScript>,
    pub prompt_configs: HashMap<String, PromptConfiguration>,
    pub git_status_cache: HashMap<String, (GitStatus, DateTime<Utc>)>,
    pub max_history_size: usize,
}

impl ShellIntegrationState {
    pub fn new() -> Self {
        Self {
            completions_cache: HashMap::new(),
            history: VecDeque::new(),
            aliases: HashMap::new(),
            functions: HashMap::new(),
            variables: HashMap::new(),
            scripts: HashMap::new(),
            prompt_configs: HashMap::new(),
            git_status_cache: HashMap::new(),
            max_history_size: 10000,
        }
    }

    pub fn add_to_history(&mut self, history_item: CommandHistory) {
        self.history.push_front(history_item);
        if self.history.len() > self.max_history_size {
            self.history.pop_back();
        }
    }

    pub fn search_history(&self, query: &str, limit: usize) -> Vec<CommandHistory> {
        let query_lower = query.to_lowercase();
        self.history
            .iter()
            .filter(|item| item.command.to_lowercase().contains(&query_lower))
            .take(limit)
            .cloned()
            .collect()
    }

    pub fn get_completion_suggestions(
        &mut self,
        input: &str,
        _cursor_position: usize,
        shell_type: &str,
        current_dir: &str,
    ) -> Vec<ShellCompletion> {
        let cache_key = format!("{}:{}:{}", shell_type, current_dir, input);
        
        if let Some(cached) = self.completions_cache.get(&cache_key) {
            return cached.clone();
        }

        let mut suggestions = Vec::new();
        
        // Command completions
        suggestions.extend(self.get_command_completions(input));
        
        // File/directory completions
        suggestions.extend(self.get_file_completions(input, current_dir));
        
        // History completions
        suggestions.extend(self.get_history_completions(input));
        
        // Alias completions
        suggestions.extend(self.get_alias_completions(input));
        
        // Variable completions
        suggestions.extend(self.get_variable_completions(input));
        
        // Shell-specific completions
        match shell_type {
            "bash" => suggestions.extend(self.get_bash_completions(input, current_dir)),
            "zsh" => suggestions.extend(self.get_zsh_completions(input, current_dir)),
            "fish" => suggestions.extend(self.get_fish_completions(input, current_dir)),
            "pwsh" | "powershell" => suggestions.extend(self.get_powershell_completions(input, current_dir)),
            _ => {}
        }
        
        // Sort by priority and relevance
        suggestions.sort_by(|a, b| b.priority.cmp(&a.priority));
        suggestions.truncate(50); // Limit results
        
        // Cache the results
        self.completions_cache.insert(cache_key, suggestions.clone());
        suggestions
    }

    fn get_command_completions(&self, input: &str) -> Vec<ShellCompletion> {
        let mut completions = Vec::new();
        
        // Get commands from PATH
        if let Ok(path) = std::env::var("PATH") {
            let separator = if cfg!(windows) { ";" } else { ":" };
            for path_entry in path.split(separator) {
                if let Ok(entries) = std::fs::read_dir(path_entry) {
                    for entry in entries.flatten() {
                        if let Some(name) = entry.file_name().to_str() {
                            if name.starts_with(input) && name != input {
                                completions.push(ShellCompletion {
                                    text: name.to_string(),
                                    display: name.to_string(),
                                    description: Some("Command".to_string()),
                                    completion_type: CompletionType::Command,
                                    priority: 80,
                                    source: "PATH".to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
        
        completions
    }

    fn get_file_completions(&self, input: &str, current_dir: &str) -> Vec<ShellCompletion> {
        let mut completions = Vec::new();
        let path = Path::new(current_dir);
        
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.starts_with(input) && name != input {
                        let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
                        completions.push(ShellCompletion {
                            text: if is_dir { format!("{}/", name) } else { name.to_string() },
                            display: name.to_string(),
                            description: Some(if is_dir { "Directory" } else { "File" }.to_string()),
                            completion_type: if is_dir { CompletionType::Directory } else { CompletionType::File },
                            priority: 70,
                            source: "filesystem".to_string(),
                        });
                    }
                }
            }
        }
        
        completions
    }

    fn get_history_completions(&self, input: &str) -> Vec<ShellCompletion> {
        let mut completions = Vec::new();
        let mut seen = std::collections::HashSet::new();
        
        for item in &self.history {
            if item.command.starts_with(input) && seen.insert(item.command.clone()) {
                completions.push(ShellCompletion {
                    text: item.command.clone(),
                    display: item.command.clone(),
                    description: Some(format!("History - {}", item.timestamp.format("%Y-%m-%d %H:%M"))),
                    completion_type: CompletionType::History,
                    priority: 60,
                    source: "history".to_string(),
                });
                
                if completions.len() >= 10 {
                    break;
                }
            }
        }
        
        completions
    }

    fn get_alias_completions(&self, input: &str) -> Vec<ShellCompletion> {
        self.aliases
            .values()
            .filter(|alias| alias.name.starts_with(input))
            .map(|alias| ShellCompletion {
                text: alias.name.clone(),
                display: alias.name.clone(),
                description: alias.description.clone().or_else(|| Some(alias.command.clone())),
                completion_type: CompletionType::Alias,
                priority: 90,
                source: "aliases".to_string(),
            })
            .collect()
    }

    fn get_variable_completions(&self, input: &str) -> Vec<ShellCompletion> {
        if !input.starts_with('$') {
            return Vec::new();
        }
        
        let var_prefix = &input[1..];
        self.variables
            .values()
            .filter(|var| var.name.starts_with(var_prefix))
            .map(|var| ShellCompletion {
                text: format!("${}", var.name),
                display: format!("${}", var.name),
                description: var.description.clone().or_else(|| Some(var.value.clone())),
                completion_type: CompletionType::Variable,
                priority: 75,
                source: "variables".to_string(),
            })
            .collect()
    }

    fn get_bash_completions(&self, _input: &str, _current_dir: &str) -> Vec<ShellCompletion> {
        // Placeholder for bash-specific completions
        Vec::new()
    }

    fn get_zsh_completions(&self, _input: &str, _current_dir: &str) -> Vec<ShellCompletion> {
        // Placeholder for zsh-specific completions
        Vec::new()
    }

    fn get_fish_completions(&self, _input: &str, _current_dir: &str) -> Vec<ShellCompletion> {
        // Placeholder for fish-specific completions
        Vec::new()
    }

    fn get_powershell_completions(&self, _input: &str, _current_dir: &str) -> Vec<ShellCompletion> {
        // Placeholder for PowerShell-specific completions
        Vec::new()
    }

    pub fn get_git_status(&mut self, directory: &str) -> Option<GitStatus> {
        // Check cache first
        if let Some((status, timestamp)) = self.git_status_cache.get(directory) {
            if Utc::now().signed_duration_since(*timestamp).num_seconds() < 30 {
                return Some(status.clone());
            }
        }

        // Get fresh git status
        if let Ok(status) = self.fetch_git_status(directory) {
            self.git_status_cache.insert(directory.to_string(), (status.clone(), Utc::now()));
            Some(status)
        } else {
            None
        }
    }

    fn fetch_git_status(&self, directory: &str) -> Result<GitStatus, Box<dyn std::error::Error>> {
        let output = Command::new("git")
            .args(&["status", "--porcelain=v1", "--branch"])
            .current_dir(directory)
            .output()?;

        if !output.status.success() {
            return Err("Not a git repository".into());
        }

        let status_text = String::from_utf8_lossy(&output.stdout);
        let mut git_status = GitStatus {
            branch: None,
            ahead: 0,
            behind: 0,
            staged: 0,
            modified: 0,
            untracked: 0,
            conflicts: 0,
            stashes: 0,
            is_dirty: false,
            is_detached: false,
        };

        for line in status_text.lines() {
            if line.starts_with("##") {
                // Branch information
                if let Some(branch_info) = line.strip_prefix("## ") {
                    if let Some(branch) = branch_info.split("...").next() {
                        git_status.branch = Some(branch.to_string());
                    }
                }
            } else if line.len() >= 3 {
                let status_codes = &line[0..2];
                match status_codes {
                    "??" => git_status.untracked += 1,
                    "UU" | "AA" | "DD" => git_status.conflicts += 1,
                    _ => {
                        if status_codes.chars().nth(0).unwrap() != ' ' {
                            git_status.staged += 1;
                        }
                        if status_codes.chars().nth(1).unwrap() != ' ' {
                            git_status.modified += 1;
                        }
                    }
                }
            }
        }

        git_status.is_dirty = git_status.staged > 0 || git_status.modified > 0 || git_status.untracked > 0;

        Ok(git_status)
    }

    pub fn generate_prompt(&self, config: &PromptConfiguration, context: &crate::execution_context::ExecutionContext) -> String {
        let mut prompt = config.template.clone();
        
        // Replace basic placeholders
        prompt = prompt.replace("{pwd}", &context.directory_state.pwd);
        prompt = prompt.replace("{user}", &context.operating_system.username);
        prompt = prompt.replace("{hostname}", &context.operating_system.hostname);
        prompt = prompt.replace("{time}", &context.current_time.format("%H:%M:%S").to_string());
        
        // Git information
        if config.show_git {
            if let Some((git_status, _)) = self.git_status_cache.get(&context.directory_state.pwd) {
                let git_info = self.format_git_info(git_status, &config.colors, &config.icons);
                prompt = prompt.replace("{git}", &git_info);
            } else {
                prompt = prompt.replace("{git}", "");
            }
        }
        
        prompt
    }

    fn format_git_info(&self, git_status: &GitStatus, _colors: &PromptColors, icons: &PromptIcons) -> String {
        if let Some(branch) = &git_status.branch {
            let mut git_info = format!("{} {}", icons.git_branch, branch);
            
            if git_status.is_dirty {
                if git_status.modified > 0 {
                    git_info.push_str(&format!(" {}{}", icons.git_modified, git_status.modified));
                }
                if git_status.staged > 0 {
                    git_info.push_str(&format!(" {}{}", icons.git_staged, git_status.staged));
                }
                if git_status.untracked > 0 {
                    git_info.push_str(&format!(" {}{}", icons.git_untracked, git_status.untracked));
                }
            }
            
            git_info
        } else {
            String::new()
        }
    }
}

// Tauri commands
#[tauri::command]
pub async fn get_shell_completions(
    input: String,
    cursor_position: usize,
    shell_type: String,
    current_dir: String,
    integration_manager: State<'_, ShellIntegrationManager>,
) -> Result<Vec<ShellCompletion>, String> {
    let mut manager = integration_manager.lock().map_err(|e| e.to_string())?;
    Ok(manager.get_completion_suggestions(&input, cursor_position, &shell_type, &current_dir))
}

#[tauri::command]
pub async fn add_command_to_history(
    command: String,
    directory: String,
    session_id: String,
    exit_code: Option<i32>,
    duration: Option<u64>,
    integration_manager: State<'_, ShellIntegrationManager>,
) -> Result<(), String> {
    let mut manager = integration_manager.lock().map_err(|e| e.to_string())?;
    let history_item = CommandHistory {
        id: uuid::Uuid::new_v4().to_string(),
        command,
        directory,
        timestamp: Utc::now(),
        exit_code,
        duration,
        session_id,
        tags: Vec::new(),
        favorite: false,
    };
    manager.add_to_history(history_item);
    Ok(())
}

#[tauri::command]
pub async fn search_command_history(
    query: String,
    limit: usize,
    integration_manager: State<'_, ShellIntegrationManager>,
) -> Result<Vec<CommandHistory>, String> {
    let manager = integration_manager.lock().map_err(|e| e.to_string())?;
    Ok(manager.search_history(&query, limit))
}

#[tauri::command]
pub async fn add_shell_alias(
    name: String,
    command: String,
    description: Option<String>,
    shell_specific: Option<String>,
    integration_manager: State<'_, ShellIntegrationManager>,
) -> Result<(), String> {
    let mut manager = integration_manager.lock().map_err(|e| e.to_string())?;
    let alias = ShellAlias {
        name: name.clone(),
        command,
        description,
        shell_specific,
        created_at: Utc::now(),
        usage_count: 0,
    };
    manager.aliases.insert(name, alias);
    Ok(())
}

#[tauri::command]
pub async fn get_shell_aliases(
    integration_manager: State<'_, ShellIntegrationManager>,
) -> Result<Vec<ShellAlias>, String> {
    let manager = integration_manager.lock().map_err(|e| e.to_string())?;
    Ok(manager.aliases.values().cloned().collect())
}

#[tauri::command]
pub async fn get_git_status(
    directory: String,
    integration_manager: State<'_, ShellIntegrationManager>,
) -> Result<Option<GitStatus>, String> {
    let mut manager = integration_manager.lock().map_err(|e| e.to_string())?;
    Ok(manager.get_git_status(&directory))
}

#[tauri::command]
pub async fn create_shell_script(
    name: String,
    content: String,
    language: String,
    description: Option<String>,
    tags: Vec<String>,
    integration_manager: State<'_, ShellIntegrationManager>,
) -> Result<String, String> {
    let mut manager = integration_manager.lock().map_err(|e| e.to_string())?;
    let script_id = uuid::Uuid::new_v4().to_string();
    let script = ShellScript {
        id: script_id.clone(),
        name,
        content,
        language,
        description,
        tags,
        created_at: Utc::now(),
        modified_at: Utc::now(),
        executable: false,
        auto_run: false,
    };
    manager.scripts.insert(script_id.clone(), script);
    Ok(script_id)
}

#[tauri::command]
pub async fn get_shell_scripts(
    integration_manager: State<'_, ShellIntegrationManager>,
) -> Result<Vec<ShellScript>, String> {
    let manager = integration_manager.lock().map_err(|e| e.to_string())?;
    Ok(manager.scripts.values().cloned().collect())
}

#[tauri::command]
pub async fn generate_custom_prompt(
    config: PromptConfiguration,
    context: crate::execution_context::ExecutionContext,
    integration_manager: State<'_, ShellIntegrationManager>,
) -> Result<String, String> {
    let manager = integration_manager.lock().map_err(|e| e.to_string())?;
    Ok(manager.generate_prompt(&config, &context))
}
