use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc;
use tokio::time::interval;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProcessState {
    Running,
    Stopped,
    Suspended,
    Zombie,
    Finished,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProcessType {
    Foreground,
    Background,
    Job,
    Daemon,
    Service,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub ppid: Option<u32>,
    pub command: String,
    pub args: Vec<String>,
    pub working_dir: String,
    pub state: ProcessState,
    pub process_type: ProcessType,
    pub start_time: u64,
    pub cpu_usage: f64,
    pub memory_usage: u64,
    pub user: String,
    pub priority: i32,
    pub exit_code: Option<i32>,
    pub environment: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobInfo {
    pub job_id: u32,
    pub process_group_id: u32,
    pub command: String,
    pub state: ProcessState,
    pub processes: Vec<ProcessInfo>,
    pub is_background: bool,
    pub start_time: u64,
    pub terminal_session: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessTree {
    pub root: ProcessInfo,
    pub children: Vec<ProcessTree>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessStats {
    pub total_processes: usize,
    pub running: usize,
    pub sleeping: usize,
    pub stopped: usize,
    pub zombie: usize,
    pub system_load: (f64, f64, f64), // 1min, 5min, 15min
    pub memory_usage: u64,
    pub cpu_usage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessFilter {
    pub name_pattern: Option<String>,
    pub user: Option<String>,
    pub state: Option<ProcessState>,
    pub process_type: Option<ProcessType>,
    pub min_cpu_usage: Option<f64>,
    pub min_memory_usage: Option<u64>,
    pub pid_range: Option<(u32, u32)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessAction {
    pub action_type: ProcessActionType,
    pub pid: u32,
    pub signal: Option<String>,
    pub priority: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProcessActionType {
    Kill,
    Stop,
    Continue,
    Terminate,
    Suspend,
    Resume,
    SetPriority,
    SendSignal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessEvent {
    pub event_type: ProcessEventType,
    pub pid: u32,
    pub timestamp: u64,
    pub details: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProcessEventType {
    Started,
    Stopped,
    Crashed,
    HighCpuUsage,
    HighMemoryUsage,
    StateChanged,
    Suspended,
    Resumed,
}

pub struct ProcessManager {
    processes: Arc<Mutex<HashMap<u32, ProcessInfo>>>,
    jobs: Arc<Mutex<HashMap<u32, JobInfo>>>,
    next_job_id: Arc<Mutex<u32>>,
    event_sender: Arc<Mutex<Option<mpsc::UnboundedSender<ProcessEvent>>>>,
    monitoring_enabled: Arc<Mutex<bool>>,
    update_interval: Duration,
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            processes: Arc::new(Mutex::new(HashMap::new())),
            jobs: Arc::new(Mutex::new(HashMap::new())),
            next_job_id: Arc::new(Mutex::new(1)),
            event_sender: Arc::new(Mutex::new(None)),
            monitoring_enabled: Arc::new(Mutex::new(false)),
            update_interval: Duration::from_secs(2),
        }
    }

    pub async fn start_monitoring(&self) -> Result<mpsc::UnboundedReceiver<ProcessEvent>, String> {
        let (tx, rx) = mpsc::unbounded_channel();
        
        {
            let mut sender = self.event_sender.lock().unwrap();
            *sender = Some(tx);
        }

        {
            let mut enabled = self.monitoring_enabled.lock().unwrap();
            *enabled = true;
        }

        // Start monitoring task
        let processes = self.processes.clone();
        let jobs = self.jobs.clone();
        let enabled = self.monitoring_enabled.clone();
        let sender = self.event_sender.clone();
        let update_interval = self.update_interval;

        tokio::spawn(async move {
            let mut interval = interval(update_interval);
            
            while *enabled.lock().unwrap() {
                interval.tick().await;
                
                if let Err(e) = Self::update_process_info(&processes, &jobs, &sender).await {
                    eprintln!("Error updating process info: {}", e);
                }
            }
        });

        Ok(rx)
    }

    pub fn stop_monitoring(&self) {
        let mut enabled = self.monitoring_enabled.lock().unwrap();
        *enabled = false;
    }

    async fn update_process_info(
        processes: &Arc<Mutex<HashMap<u32, ProcessInfo>>>,
        jobs: &Arc<Mutex<HashMap<u32, JobInfo>>>,
        sender: &Arc<Mutex<Option<mpsc::UnboundedSender<ProcessEvent>>>>,
    ) -> Result<(), String> {
        let system_processes = Self::get_system_processes()?;
        
        let mut processes_guard = processes.lock().unwrap();
        let mut new_events = Vec::new();
        
        for process in system_processes {
            let pid = process.pid;
            
            if let Some(existing) = processes_guard.get(&pid) {
                // Check for state changes
                if existing.state != process.state {
                    new_events.push(ProcessEvent {
                        event_type: ProcessEventType::StateChanged,
                        pid,
                        timestamp: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                        details: [
                            ("old_state".to_string(), format!("{:?}", existing.state)),
                            ("new_state".to_string(), format!("{:?}", process.state)),
                        ].into_iter().collect(),
                    });
                }

                // Check for high resource usage
                if process.cpu_usage > 80.0 {
                    new_events.push(ProcessEvent {
                        event_type: ProcessEventType::HighCpuUsage,
                        pid,
                        timestamp: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                        details: [("cpu_usage".to_string(), process.cpu_usage.to_string())]
                            .into_iter().collect(),
                    });
                }

                if process.memory_usage > 1024 * 1024 * 1024 { // 1GB
                    new_events.push(ProcessEvent {
                        event_type: ProcessEventType::HighMemoryUsage,
                        pid,
                        timestamp: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                        details: [("memory_usage".to_string(), process.memory_usage.to_string())]
                            .into_iter().collect(),
                    });
                }
            } else {
                // New process detected
                new_events.push(ProcessEvent {
                    event_type: ProcessEventType::Started,
                    pid,
                    timestamp: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    details: [("command".to_string(), process.command.clone())]
                        .into_iter().collect(),
                });
            }
            
            processes_guard.insert(pid, process);
        }

        // Send events
        if let Some(ref sender) = *sender.lock().unwrap() {
            for event in new_events {
                let _ = sender.send(event);
            }
        }

        Ok(())
    }

    #[cfg(unix)]
    fn get_system_processes() -> Result<Vec<ProcessInfo>, String> {
        use std::fs;
        
        let mut processes = Vec::new();
        
        if let Ok(entries) = fs::read_dir("/proc") {
            for entry in entries.flatten() {
                if let Ok(file_name) = entry.file_name().into_string() {
                    if let Ok(pid) = file_name.parse::<u32>() {
                        if let Ok(process) = Self::get_process_info(pid) {
                            processes.push(process);
                        }
                    }
                }
            }
        }
        
        Ok(processes)
    }

    #[cfg(windows)]
    fn get_system_processes() -> Result<Vec<ProcessInfo>, String> {
        // Windows implementation would use Windows API
        Ok(Vec::new())
    }

    #[cfg(unix)]
    fn get_process_info(pid: u32) -> Result<ProcessInfo, String> {
        use std::fs;
        
        let stat_path = format!("/proc/{}/stat", pid);
        let cmdline_path = format!("/proc/{}/cmdline", pid);
        let status_path = format!("/proc/{}/status", pid);
        
        let stat_content = fs::read_to_string(stat_path)
            .map_err(|e| format!("Failed to read stat: {}", e))?;
        let cmdline_content = fs::read_to_string(cmdline_path).unwrap_or_default();
        let status_content = fs::read_to_string(status_path).unwrap_or_default();
        
        let stat_parts: Vec<&str> = stat_content.split_whitespace().collect();
        if stat_parts.len() < 20 {
            return Err("Invalid stat format".to_string());
        }
        
        let command = stat_parts.get(1)
            .map(|s| s.trim_matches(|c| c == '(' || c == ')').to_string())
            .unwrap_or_default();
        
        let state = match stat_parts.get(2) {
            Some(&"R") => ProcessState::Running,
            Some(&"S") | Some(&"I") => ProcessState::Stopped,
            Some(&"T") => ProcessState::Suspended,
            Some(&"Z") => ProcessState::Zombie,
            _ => ProcessState::Running,
        };
        
        let ppid = stat_parts.get(3)
            .and_then(|s| s.parse::<u32>().ok());
        
        let priority = stat_parts.get(17)
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(0);
        
        // Parse command line arguments
        let args: Vec<String> = cmdline_content
            .split('\0')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();
        
        // Get user info from status
        let user = Self::extract_user_from_status(&status_content);
        
        Ok(ProcessInfo {
            pid,
            ppid,
            command,
            args,
            working_dir: format!("/proc/{}/cwd", pid),
            state,
            process_type: ProcessType::Foreground,
            start_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            cpu_usage: 0.0,
            memory_usage: 0,
            user,
            priority,
            exit_code: None,
            environment: HashMap::new(),
        })
    }

    fn extract_user_from_status(status_content: &str) -> String {
        for line in status_content.lines() {
            if line.starts_with("Uid:") {
                if let Some(uid_str) = line.split_whitespace().nth(1) {
                    if let Ok(uid) = uid_str.parse::<u32>() {
                        return Self::get_username_by_uid(uid).unwrap_or(uid.to_string());
                    }
                }
            }
        }
        "unknown".to_string()
    }

    #[cfg(unix)]
    fn get_username_by_uid(uid: u32) -> Option<String> {
        use std::ffi::CStr;
        use std::ptr;
        
        unsafe {
            let passwd = libc::getpwuid(uid);
            if !passwd.is_null() {
                let name_ptr = (*passwd).pw_name;
                if !name_ptr.is_null() {
                    let name_cstr = CStr::from_ptr(name_ptr);
                    return name_cstr.to_string_lossy().into_owned().into();
                }
            }
        }
        None
    }

    #[cfg(windows)]
    fn get_username_by_uid(_uid: u32) -> Option<String> {
        // Windows doesn't use UIDs in the same way as Unix
        // This would require Windows API calls to get user information
        None
    }

    pub fn get_processes(&self, filter: Option<ProcessFilter>) -> Vec<ProcessInfo> {
        let processes = self.processes.lock().unwrap();
        let mut result: Vec<ProcessInfo> = processes.values().cloned().collect();
        
        if let Some(filter) = filter {
            result = result.into_iter().filter(|proc| {
                if let Some(ref pattern) = filter.name_pattern {
                    if !proc.command.contains(pattern) {
                        return false;
                    }
                }
                
                if let Some(ref user) = filter.user {
                    if proc.user != *user {
                        return false;
                    }
                }
                
                if let Some(ref state) = filter.state {
                    if proc.state != *state {
                        return false;
                    }
                }
                
                if let Some(ref process_type) = filter.process_type {
                    if proc.process_type != *process_type {
                        return false;
                    }
                }
                
                if let Some(min_cpu) = filter.min_cpu_usage {
                    if proc.cpu_usage < min_cpu {
                        return false;
                    }
                }
                
                if let Some(min_memory) = filter.min_memory_usage {
                    if proc.memory_usage < min_memory {
                        return false;
                    }
                }
                
                if let Some((min_pid, max_pid)) = filter.pid_range {
                    if proc.pid < min_pid || proc.pid > max_pid {
                        return false;
                    }
                }
                
                true
            }).collect();
        }
        
        result.sort_by(|a, b| a.pid.cmp(&b.pid));
        result
    }

    pub fn get_process_tree(&self, root_pid: Option<u32>) -> Result<Vec<ProcessTree>, String> {
        let processes = self.processes.lock().unwrap();
        let mut trees = Vec::new();
        
        if let Some(root_pid) = root_pid {
            if let Some(root_process) = processes.get(&root_pid) {
                let tree = self.build_process_tree(root_process, &processes);
                trees.push(tree);
            }
        } else {
            // Build trees for all root processes (no parent or parent not in our list)
            for process in processes.values() {
                if process.ppid.is_none() || 
                   process.ppid.map_or(true, |ppid| !processes.contains_key(&ppid)) {
                    let tree = self.build_process_tree(process, &processes);
                    trees.push(tree);
                }
            }
        }
        
        Ok(trees)
    }

    fn build_process_tree(
        &self,
        root: &ProcessInfo,
        all_processes: &HashMap<u32, ProcessInfo>
    ) -> ProcessTree {
        let children: Vec<ProcessTree> = all_processes
            .values()
            .filter(|proc| proc.ppid == Some(root.pid))
            .map(|proc| self.build_process_tree(proc, all_processes))
            .collect();
        
        ProcessTree {
            root: root.clone(),
            children,
        }
    }

    pub fn get_process_stats(&self) -> ProcessStats {
        let processes = self.processes.lock().unwrap();
        
        let mut running = 0;
        let mut sleeping = 0;
        let mut stopped = 0;
        let mut zombie = 0;
        
        for process in processes.values() {
            match process.state {
                ProcessState::Running => running += 1,
                ProcessState::Stopped => sleeping += 1,
                ProcessState::Suspended => stopped += 1,
                ProcessState::Zombie => zombie += 1,
                _ => {}
            }
        }
        
        ProcessStats {
            total_processes: processes.len(),
            running,
            sleeping,
            stopped,
            zombie,
            system_load: Self::get_system_load(),
            memory_usage: Self::get_memory_usage(),
            cpu_usage: Self::get_cpu_usage(),
        }
    }

    #[cfg(unix)]
    fn get_system_load() -> (f64, f64, f64) {
        use std::fs;
        
        if let Ok(content) = fs::read_to_string("/proc/loadavg") {
            let parts: Vec<&str> = content.split_whitespace().collect();
            if parts.len() >= 3 {
                let load1 = parts[0].parse::<f64>().unwrap_or(0.0);
                let load5 = parts[1].parse::<f64>().unwrap_or(0.0);
                let load15 = parts[2].parse::<f64>().unwrap_or(0.0);
                return (load1, load5, load15);
            }
        }
        (0.0, 0.0, 0.0)
    }

    #[cfg(windows)]
    fn get_system_load() -> (f64, f64, f64) {
        (0.0, 0.0, 0.0)
    }

    #[cfg(unix)]
    fn get_memory_usage() -> u64 {
        use std::fs;
        
        if let Ok(content) = fs::read_to_string("/proc/meminfo") {
            for line in content.lines() {
                if line.starts_with("MemAvailable:") {
                    if let Some(kb_str) = line.split_whitespace().nth(1) {
                        if let Ok(kb) = kb_str.parse::<u64>() {
                            return kb * 1024; // Convert to bytes
                        }
                    }
                }
            }
        }
        0
    }

    #[cfg(windows)]
    fn get_memory_usage() -> u64 {
        0
    }

    #[cfg(unix)]
    fn get_cpu_usage() -> f64 {
        // Simplified CPU usage calculation
        // In a real implementation, you'd want to calculate this over time
        0.0
    }

    #[cfg(windows)]
    fn get_cpu_usage() -> f64 {
        0.0
    }

    pub async fn execute_process_action(&self, action: ProcessAction) -> Result<String, String> {
        match action.action_type {
            ProcessActionType::Kill => {
                self.send_signal(action.pid, "SIGKILL").await
            }
            ProcessActionType::Terminate => {
                self.send_signal(action.pid, "SIGTERM").await
            }
            ProcessActionType::Stop => {
                self.send_signal(action.pid, "SIGSTOP").await
            }
            ProcessActionType::Continue => {
                self.send_signal(action.pid, "SIGCONT").await
            }
            ProcessActionType::Suspend => {
                self.send_signal(action.pid, "SIGTSTP").await
            }
            ProcessActionType::Resume => {
                self.send_signal(action.pid, "SIGCONT").await
            }
            ProcessActionType::SendSignal => {
                let signal = action.signal.unwrap_or("SIGTERM".to_string());
                self.send_signal(action.pid, &signal).await
            }
            ProcessActionType::SetPriority => {
                let priority = action.priority.unwrap_or(0);
                self.set_process_priority(action.pid, priority).await
            }
        }
    }

    #[cfg(unix)]
    async fn send_signal(&self, pid: u32, signal: &str) -> Result<String, String> {
        let signal_num = match signal {
            "SIGKILL" => 9,
            "SIGTERM" => 15,
            "SIGSTOP" => 19,
            "SIGCONT" => 18,
            "SIGTSTP" => 20,
            "SIGINT" => 2,
            "SIGHUP" => 1,
            _ => return Err(format!("Unknown signal: {}", signal)),
        };
        
        unsafe {
            let result = libc::kill(pid as i32, signal_num);
            if result == 0 {
                Ok(format!("Signal {} sent to process {}", signal, pid))
            } else {
                Err(format!("Failed to send signal {} to process {}", signal, pid))
            }
        }
    }

    #[cfg(windows)]
    async fn send_signal(&self, pid: u32, signal: &str) -> Result<String, String> {
        // Windows implementation would use Windows API
        Err("Signal sending not implemented on Windows".to_string())
    }

    #[cfg(unix)]
    async fn set_process_priority(&self, pid: u32, priority: i32) -> Result<String, String> {
        unsafe {
            let result = libc::setpriority(libc::PRIO_PROCESS, pid, priority);
            if result == 0 {
                Ok(format!("Priority set to {} for process {}", priority, pid))
            } else {
                Err(format!("Failed to set priority for process {}", pid))
            }
        }
    }

    #[cfg(windows)]
    async fn set_process_priority(&self, pid: u32, priority: i32) -> Result<String, String> {
        Err("Priority setting not implemented on Windows".to_string())
    }

    pub async fn create_job(&self, command: String, args: Vec<String>, is_background: bool, terminal_session: Option<String>) -> Result<u32, String> {
        let job_id = {
            let mut next_id = self.next_job_id.lock().unwrap();
            let id = *next_id;
            *next_id += 1;
            id
        };

        let mut cmd = Command::new(&command);
        cmd.args(&args);
        
        if is_background {
            cmd.stdout(Stdio::null());
            cmd.stderr(Stdio::null());
            cmd.stdin(Stdio::null());
        }

        let mut child = cmd.spawn()
            .map_err(|e| format!("Failed to spawn process: {}", e))?;

        let pid = child.id();
        
        let process_info = ProcessInfo {
            pid,
            ppid: Some(std::process::id()),
            command: command.clone(),
            args,
            working_dir: std::env::current_dir()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            state: ProcessState::Running,
            process_type: if is_background { ProcessType::Background } else { ProcessType::Job },
            start_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            cpu_usage: 0.0,
            memory_usage: 0,
            user: "current".to_string(),
            priority: 0,
            exit_code: None,
            environment: std::env::vars().collect(),
        };

        let job_info = JobInfo {
            job_id,
            process_group_id: pid,
            command,
            state: ProcessState::Running,
            processes: vec![process_info.clone()],
            is_background,
            start_time: process_info.start_time,
            terminal_session,
        };

        {
            let mut processes = self.processes.lock().unwrap();
            processes.insert(pid, process_info);
        }

        {
            let mut jobs = self.jobs.lock().unwrap();
            jobs.insert(job_id, job_info);
        }

        // Monitor the job in the background
        let jobs_clone = self.jobs.clone();
        let processes_clone = self.processes.clone();
        tokio::spawn(async move {
            let exit_status = child.wait();
            
            let mut jobs = jobs_clone.lock().unwrap();
            let mut processes = processes_clone.lock().unwrap();
            
            if let Some(job) = jobs.get_mut(&job_id) {
                if let Ok(status) = exit_status {
                    job.state = if status.success() { 
                        ProcessState::Finished 
                    } else { 
                        ProcessState::Failed 
                    };
                    
                    if let Some(process) = processes.get_mut(&pid) {
                        process.state = job.state.clone();
                        process.exit_code = status.code();
                    }
                }
            }
        });

        Ok(job_id)
    }

    pub fn get_jobs(&self) -> Vec<JobInfo> {
        let jobs = self.jobs.lock().unwrap();
        jobs.values().cloned().collect()
    }

    pub fn get_job(&self, job_id: u32) -> Option<JobInfo> {
        let jobs = self.jobs.lock().unwrap();
        jobs.get(&job_id).cloned()
    }

    pub async fn kill_job(&self, job_id: u32) -> Result<String, String> {
        let job = {
            let jobs = self.jobs.lock().unwrap();
            jobs.get(&job_id).cloned()
        };

        if let Some(job) = job {
            for process in &job.processes {
                let _ = self.send_signal(process.pid, "SIGKILL").await;
            }
            Ok(format!("Job {} killed", job_id))
        } else {
            Err(format!("Job {} not found", job_id))
        }
    }
}
