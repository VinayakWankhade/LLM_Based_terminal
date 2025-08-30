use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};
use regex::Regex;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    pub id: String,
    pub text: String,
    pub timestamp: u64,
    pub working_dir: String,
    pub exit_code: Option<i32>,
    pub duration_ms: Option<u64>,
    pub shell_type: ShellType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ShellType {
    PowerShell,
    Bash,
    Zsh,
    Fish,
    Cmd,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptInfo {
    pub shell_type: ShellType,
    pub working_dir: String,
    pub user: String,
    pub hostname: String,
    pub prompt_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandSuggestion {
    pub command: String,
    pub description: String,
    pub frequency: u32,
    pub last_used: u64,
}

pub struct ShellHooks {
    pub session_id: String,
    command_history: VecDeque<Command>,
    current_command: Option<Command>,
    prompt_patterns: HashMap<ShellType, Vec<Regex>>,
    shell_type: ShellType,
    current_prompt: Option<PromptInfo>,
    working_dir: String,
    max_history_size: usize,
    output_buffer: String,
}

impl ShellHooks {
    pub fn new(session_id: String, shell_type: ShellType, working_dir: String) -> Self {
        let mut hooks = ShellHooks {
            session_id,
            command_history: VecDeque::new(),
            current_command: None,
            prompt_patterns: HashMap::new(),
            shell_type: shell_type.clone(),
            current_prompt: None,
            working_dir,
            max_history_size: 1000,
            output_buffer: String::new(),
        };

        hooks.init_prompt_patterns();
        hooks
    }

    fn init_prompt_patterns(&mut self) {
        // PowerShell prompts
        let ps_patterns = vec![
            // Standard PowerShell prompt: PS C:\Users\user>
            Regex::new(r"^PS\s+([A-Za-z]:[\\\/][^>]*|[~\/][^>]*)>\s*$").unwrap(),
            // Custom PowerShell prompts
            Regex::new(r"^PowerShell\s+.*>\s*$").unwrap(),
            // Azure Cloud Shell
            Regex::new(r"^Azure:\w+@Azure:~\$\s*$").unwrap(),
        ];
        self.prompt_patterns.insert(ShellType::PowerShell, ps_patterns);

        // Bash prompts
        let bash_patterns = vec![
            // Standard bash: user@hostname:~/path$
            Regex::new(r"^[^@\s]+@[^:\s]+:[^$]*\$\s*$").unwrap(),
            // Simple bash: $
            Regex::new(r"^\$\s*$").unwrap(),
            // Root bash: #
            Regex::new(r"^#\s*$").unwrap(),
        ];
        self.prompt_patterns.insert(ShellType::Bash, bash_patterns);

        // Zsh prompts
        let zsh_patterns = vec![
            // Oh-my-zsh style
            Regex::new(r"^[^@\s]+@[^:\s]+:[^%]*%\s*$").unwrap(),
            // Simple zsh
            Regex::new(r"^%\s*$").unwrap(),
        ];
        self.prompt_patterns.insert(ShellType::Zsh, zsh_patterns);

        // Fish prompts
        let fish_patterns = vec![
            Regex::new(r"^[^@\s]+@[^:\s]+\s+[^>]*>\s*$").unwrap(),
        ];
        self.prompt_patterns.insert(ShellType::Fish, fish_patterns);

        // Windows CMD
        let cmd_patterns = vec![
            // C:\Users\user>
            Regex::new(r"^[A-Za-z]:[\\\/][^>]*>\s*$").unwrap(),
        ];
        self.prompt_patterns.insert(ShellType::Cmd, cmd_patterns);
    }

    pub fn process_output(&mut self, data: &str) {
        self.output_buffer.push_str(data);
        
        // Process complete lines
        while let Some(newline_pos) = self.output_buffer.find('\n') {
            let line = self.output_buffer[..newline_pos].trim_end_matches('\r').to_string();
            self.output_buffer.drain(..=newline_pos);
            
            self.process_line(&line);
        }
        
        // Also check the current buffer for prompts (in case prompt doesn't end with newline)
        if !self.output_buffer.trim().is_empty() {
            let buffer_copy = self.output_buffer.clone();
            self.check_for_prompt(&buffer_copy);
        }
    }

    fn process_line(&mut self, line: &str) {
        // Skip empty lines
        if line.trim().is_empty() {
            return;
        }

        // Check if this line contains a prompt
        if self.check_for_prompt(line) {
            // If we have a current command, it just finished
            if let Some(mut cmd) = self.current_command.take() {
                cmd.duration_ms = Some(
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64 - cmd.timestamp
                );
                self.add_to_history(cmd);
            }
            return;
        }

        // Check if this looks like a command being executed
        if self.current_prompt.is_some() && !line.starts_with(' ') {
            // This might be a command
            if self.looks_like_command(line) {
                let cmd = Command {
                    id: Uuid::new_v4().to_string(),
                    text: line.to_string(),
                    timestamp: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64,
                    working_dir: self.working_dir.clone(),
                    exit_code: None,
                    duration_ms: None,
                    shell_type: self.shell_type.clone(),
                };
                self.current_command = Some(cmd);
            }
        }
    }

    fn check_for_prompt(&mut self, line: &str) -> bool {
        let clean_line = self.strip_ansi_codes(line);
        
        if let Some(patterns) = self.prompt_patterns.get(&self.shell_type) {
            for pattern in patterns {
                if pattern.is_match(&clean_line) {
                    self.parse_prompt_info(&clean_line);
                    return true;
                }
            }
        }

        // Also check other shell types in case shell type detection was wrong
        for (shell_type, patterns) in &self.prompt_patterns {
            if *shell_type != self.shell_type {
                for pattern in patterns {
                    if pattern.is_match(&clean_line) {
                        self.shell_type = shell_type.clone();
                        self.parse_prompt_info(&clean_line);
                        return true;
                    }
                }
            }
        }

        false
    }

    fn parse_prompt_info(&mut self, prompt_line: &str) {
        match self.shell_type {
            ShellType::PowerShell => {
                if let Some(caps) = Regex::new(r"^PS\s+([A-Za-z]:[\\\/][^>]*|[~\/][^>]*)>\s*$")
                    .unwrap()
                    .captures(prompt_line)
                {
                    if let Some(path) = caps.get(1) {
                        self.working_dir = path.as_str().to_string();
                    }
                }
            }
            ShellType::Bash | ShellType::Zsh => {
                if let Some(caps) = Regex::new(r"^([^@\s]+)@([^:\s]+):([^$%]*)([$%])\s*$")
                    .unwrap()
                    .captures(prompt_line)
                {
                    let user = caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
                    let hostname = caps.get(2).map(|m| m.as_str().to_string()).unwrap_or_default();
                    let working_dir = caps.get(3).map(|m| m.as_str().to_string()).unwrap_or_default();
                    
                    self.working_dir = if working_dir.starts_with('~') {
                        working_dir
                    } else {
                        working_dir
                    };

                    self.current_prompt = Some(PromptInfo {
                        shell_type: self.shell_type.clone(),
                        working_dir: self.working_dir.clone(),
                        user,
                        hostname,
                        prompt_text: prompt_line.to_string(),
                    });
                }
            }
            _ => {}
        }
    }

    fn looks_like_command(&self, line: &str) -> bool {
        let clean_line = line.trim();
        
        // Skip if line is empty or starts with common non-command prefixes
        if clean_line.is_empty() 
            || clean_line.starts_with('#')  // Comment
            || clean_line.starts_with("//") // Comment
            || clean_line.starts_with('>') // Continuation prompt
        {
            return false;
        }

        // PowerShell specific
        if matches!(self.shell_type, ShellType::PowerShell) {
            // Check for PowerShell cmdlets and common commands
            return clean_line.split_whitespace().next()
                .map(|first_word| {
                    first_word.contains('-') || // PowerShell cmdlets (Get-Process, Set-Location)
                    ["cd", "ls", "dir", "pwd", "echo", "cat", "type", "mkdir", "rmdir", "del", "copy", "move"]
                        .contains(&first_word.to_lowercase().as_str())
                })
                .unwrap_or(false);
        }

        // Unix shells
        true // For now, assume most lines in Unix shells are commands
    }

    fn strip_ansi_codes(&self, text: &str) -> String {
        // Simple ANSI escape sequence removal
        let ansi_regex = Regex::new(r"\x1b\[[0-9;]*[a-zA-Z]").unwrap();
        ansi_regex.replace_all(text, "").to_string()
    }

    fn add_to_history(&mut self, command: Command) {
        // Add to history, maintaining max size
        if self.command_history.len() >= self.max_history_size {
            self.command_history.pop_front();
        }
        self.command_history.push_back(command);
    }

    pub fn get_command_history(&self, limit: Option<usize>) -> Vec<Command> {
        let limit = limit.unwrap_or(100);
        self.command_history
            .iter()
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    pub fn get_command_suggestions(&self, partial_command: &str) -> Vec<CommandSuggestion> {
        let mut suggestions = HashMap::new();
        
        // Analyze command history for suggestions
        for cmd in &self.command_history {
            if cmd.text.starts_with(partial_command) {
                let entry = suggestions.entry(cmd.text.clone()).or_insert(CommandSuggestion {
                    command: cmd.text.clone(),
                    description: format!("Previously used in {}", cmd.working_dir),
                    frequency: 0,
                    last_used: cmd.timestamp,
                });
                entry.frequency += 1;
                if cmd.timestamp > entry.last_used {
                    entry.last_used = cmd.timestamp;
                }
            }
        }

        // Add common commands based on shell type
        if partial_command.is_empty() || self.command_history.is_empty() {
            self.add_common_command_suggestions(partial_command, &mut suggestions);
        }

        // Sort by frequency and recency
        let mut result: Vec<CommandSuggestion> = suggestions.into_values().collect();
        result.sort_by(|a, b| {
            // Sort by frequency first, then by recency
            b.frequency.cmp(&a.frequency)
                .then(b.last_used.cmp(&a.last_used))
        });

        result.into_iter().take(10).collect() // Limit to 10 suggestions
    }

    fn add_common_command_suggestions(
        &self,
        partial_command: &str,
        suggestions: &mut HashMap<String, CommandSuggestion>,
    ) {
        let common_commands = match self.shell_type {
            ShellType::PowerShell => vec![
                ("Get-Process", "List running processes"),
                ("Get-ChildItem", "List directory contents"),
                ("Set-Location", "Change directory"),
                ("Get-Location", "Get current directory"),
                ("Clear-Host", "Clear the screen"),
                ("Get-Help", "Get help for commands"),
                ("ls", "List directory contents (alias)"),
                ("cd", "Change directory (alias)"),
                ("pwd", "Print working directory (alias)"),
                ("cls", "Clear screen (alias)"),
            ],
            ShellType::Bash | ShellType::Zsh => vec![
                ("ls", "List directory contents"),
                ("cd", "Change directory"),
                ("pwd", "Print working directory"),
                ("mkdir", "Create directory"),
                ("rmdir", "Remove directory"),
                ("rm", "Remove files"),
                ("cp", "Copy files"),
                ("mv", "Move files"),
                ("grep", "Search text"),
                ("find", "Find files"),
                ("cat", "Display file contents"),
                ("less", "View file contents"),
                ("vi", "Edit files"),
                ("nano", "Edit files"),
                ("chmod", "Change file permissions"),
                ("chown", "Change file ownership"),
                ("ps", "List processes"),
                ("kill", "Terminate processes"),
                ("top", "Display running processes"),
                ("df", "Show disk usage"),
                ("du", "Show directory usage"),
                ("free", "Show memory usage"),
                ("history", "Show command history"),
            ],
            ShellType::Fish => vec![
                ("ls", "List directory contents"),
                ("cd", "Change directory"),
                ("pwd", "Print working directory"),
                ("mkdir", "Create directory"),
                ("rm", "Remove files"),
                ("cp", "Copy files"),
                ("mv", "Move files"),
                ("grep", "Search text"),
                ("find", "Find files"),
                ("cat", "Display file contents"),
                ("funced", "Edit function"),
                ("funcsave", "Save function"),
                ("history", "Show command history"),
            ],
            ShellType::Cmd => vec![
                ("dir", "List directory contents"),
                ("cd", "Change directory"),
                ("mkdir", "Create directory"),
                ("rmdir", "Remove directory"),
                ("del", "Delete files"),
                ("copy", "Copy files"),
                ("move", "Move files"),
                ("type", "Display file contents"),
                ("find", "Search text in files"),
                ("cls", "Clear screen"),
            ],
            ShellType::Unknown => vec![],
        };

        for (cmd, desc) in common_commands {
            if cmd.starts_with(partial_command) {
                suggestions.entry(cmd.to_string()).or_insert(CommandSuggestion {
                    command: cmd.to_string(),
                    description: desc.to_string(),
                    frequency: 1,
                    last_used: 0,
                });
            }
        }
    }

    pub fn detect_shell_type(shell_path: &str) -> ShellType {
        let shell_name = std::path::Path::new(shell_path)
            .file_name()
            .and_then(|os_str| os_str.to_str())
            .unwrap_or("")
            .to_lowercase();

        match shell_name.as_str() {
            "powershell.exe" | "pwsh.exe" | "powershell" | "pwsh" => ShellType::PowerShell,
            "bash.exe" | "bash" => ShellType::Bash,
            "zsh.exe" | "zsh" => ShellType::Zsh,
            "fish.exe" | "fish" => ShellType::Fish,
            "cmd.exe" | "cmd" => ShellType::Cmd,
            _ => ShellType::Unknown,
        }
    }

    pub fn get_current_prompt(&self) -> Option<&PromptInfo> {
        self.current_prompt.as_ref()
    }

    pub fn is_at_prompt(&self) -> bool {
        self.current_prompt.is_some() && self.current_command.is_none()
    }

    pub fn get_working_directory(&self) -> &str {
        &self.working_dir
    }

    pub fn set_working_directory(&mut self, working_dir: String) {
        self.working_dir = working_dir;
    }

    pub fn search_history(&self, query: &str) -> Vec<Command> {
        self.command_history
            .iter()
            .filter(|cmd| cmd.text.contains(query))
            .rev()
            .take(50)
            .cloned()
            .collect()
    }

    pub fn get_recent_commands(&self, limit: usize) -> Vec<Command> {
        self.command_history
            .iter()
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    pub fn complete_command(&self, partial: &str) -> Vec<String> {
        let suggestions = self.get_command_suggestions(partial);
        suggestions.into_iter().map(|s| s.command).collect()
    }

    // Hook for handling command completion from shell
    pub fn handle_tab_completion(&self, current_line: &str, cursor_pos: usize) -> Vec<String> {
        // Extract the word at cursor position
        let words: Vec<&str> = current_line[..cursor_pos].split_whitespace().collect();
        
        if words.is_empty() {
            // Complete command names
            self.complete_command("")
        } else if words.len() == 1 {
            // Complete command names
            self.complete_command(words[0])
        } else {
            // Complete file/directory names (simplified)
            let last_word = words.last().map(|&s| s).unwrap_or("");
            self.complete_filesystem(last_word)
        }
    }

    fn complete_filesystem(&self, _partial: &str) -> Vec<String> {
        // Basic filesystem completion - in a real implementation this would
        // interact with the filesystem and shell completion mechanisms
        vec![]
    }
}

// Helper struct for managing shell hooks across all terminal sessions
pub struct ShellHooksManager {
    hooks: HashMap<String, ShellHooks>,
}

impl ShellHooksManager {
    pub fn new() -> Self {
        ShellHooksManager {
            hooks: HashMap::new(),
        }
    }

    pub fn create_session_hooks(
        &mut self,
        session_id: String,
        shell_path: &str,
        working_dir: String,
    ) {
        let shell_type = ShellHooks::detect_shell_type(shell_path);
        let hooks = ShellHooks::new(session_id.clone(), shell_type, working_dir);
        self.hooks.insert(session_id, hooks);
    }

    pub fn process_output(&mut self, session_id: &str, data: &str) {
        if let Some(hooks) = self.hooks.get_mut(session_id) {
            hooks.process_output(data);
        }
    }

    pub fn get_command_history(&self, session_id: &str, limit: Option<usize>) -> Option<Vec<Command>> {
        self.hooks.get(session_id).map(|hooks| hooks.get_command_history(limit))
    }

    pub fn get_command_suggestions(
        &self,
        session_id: &str,
        partial_command: &str,
    ) -> Option<Vec<CommandSuggestion>> {
        self.hooks
            .get(session_id)
            .map(|hooks| hooks.get_command_suggestions(partial_command))
    }

    pub fn handle_tab_completion(
        &self,
        session_id: &str,
        current_line: &str,
        cursor_pos: usize,
    ) -> Option<Vec<String>> {
        self.hooks
            .get(session_id)
            .map(|hooks| hooks.handle_tab_completion(current_line, cursor_pos))
    }

    pub fn is_at_prompt(&self, session_id: &str) -> bool {
        self.hooks
            .get(session_id)
            .map(|hooks| hooks.is_at_prompt())
            .unwrap_or(false)
    }

    pub fn get_current_prompt(&self, session_id: &str) -> Option<&PromptInfo> {
        self.hooks
            .get(session_id)
            .and_then(|hooks| hooks.get_current_prompt())
    }

    pub fn remove_session(&mut self, session_id: &str) {
        self.hooks.remove(session_id);
    }

    pub fn search_history(&self, session_id: &str, query: &str) -> Option<Vec<Command>> {
        self.hooks
            .get(session_id)
            .map(|hooks| hooks.search_history(query))
    }
}
