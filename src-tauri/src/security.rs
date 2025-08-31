use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use regex::Regex;
use sha2::{Sha256, Digest};
use base64::{Engine as _, engine::general_purpose};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicy {
    pub enable_audit_logging: bool,
    pub mask_sensitive_data: bool,
    pub blocked_commands: Vec<String>,
    pub allowed_directories: Vec<String>,
    pub require_confirmation: Vec<String>,  // Commands that require user confirmation
    pub secure_input_mode: bool,
    pub max_session_duration: Option<u64>, // in seconds
    pub auto_lock_timeout: Option<u64>,    // in seconds
    pub encryption_enabled: bool,
}

impl Default for SecurityPolicy {
    fn default() -> Self {
        Self {
            enable_audit_logging: true,
            mask_sensitive_data: true,
            blocked_commands: vec![
                "sudo rm -rf /".to_string(),
                "rm -rf /*".to_string(),
                "format c:".to_string(),
                "del /s /q c:\\*".to_string(),
            ],
            allowed_directories: vec![],
            require_confirmation: vec![
                "rm -rf".to_string(),
                "sudo rm".to_string(),
                "format".to_string(),
                "del /s".to_string(),
                "DROP TABLE".to_string(),
                "DROP DATABASE".to_string(),
            ],
            secure_input_mode: false,
            max_session_duration: Some(8 * 3600), // 8 hours
            auto_lock_timeout: Some(30 * 60),     // 30 minutes
            encryption_enabled: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: String,
    pub timestamp: u64,
    pub session_id: String,
    pub user: String,
    pub command: String,
    pub working_directory: String,
    pub exit_code: Option<i32>,
    pub duration_ms: Option<u64>,
    pub ip_address: Option<String>,
    pub event_type: AuditEventType,
    pub risk_level: RiskLevel,
    pub blocked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditEventType {
    CommandExecution,
    FileAccess,
    NetworkActivity,
    PrivilegeEscalation,
    SecurityViolation,
    SessionStart,
    SessionEnd,
    Authentication,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAlert {
    pub id: String,
    pub timestamp: u64,
    pub session_id: String,
    pub alert_type: SecurityAlertType,
    pub message: String,
    pub risk_level: RiskLevel,
    pub command: Option<String>,
    pub remediation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityAlertType {
    BlockedCommand,
    SuspiciousActivity,
    PrivilegeEscalation,
    UnauthorizedAccess,
    DataLeakage,
    MaliciousPattern,
    SessionTimeout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecureSession {
    pub id: String,
    pub start_time: u64,
    pub last_activity: u64,
    pub user: String,
    pub encrypted: bool,
    pub locked: bool,
    pub authentication_required: bool,
    pub risk_score: f64,
}

pub struct SecurityManager {
    policy: Arc<Mutex<SecurityPolicy>>,
    audit_logs: Arc<Mutex<Vec<AuditLogEntry>>>,
    security_alerts: Arc<Mutex<Vec<SecurityAlert>>>,
    secure_sessions: Arc<Mutex<HashMap<String, SecureSession>>>,
    sensitive_patterns: Arc<Mutex<Vec<Regex>>>,
    command_risk_scores: Arc<Mutex<HashMap<String, f64>>>,
    blocked_ips: Arc<Mutex<HashSet<String>>>,
    encryption_key: Arc<Mutex<Option<Vec<u8>>>>,
}

impl SecurityManager {
    pub fn new() -> Self {
        let manager = SecurityManager {
            policy: Arc::new(Mutex::new(SecurityPolicy::default())),
            audit_logs: Arc::new(Mutex::new(Vec::new())),
            security_alerts: Arc::new(Mutex::new(Vec::new())),
            secure_sessions: Arc::new(Mutex::new(HashMap::new())),
            sensitive_patterns: Arc::new(Mutex::new(Vec::new())),
            command_risk_scores: Arc::new(Mutex::new(HashMap::new())),
            blocked_ips: Arc::new(Mutex::new(HashSet::new())),
            encryption_key: Arc::new(Mutex::new(None)),
        };

        manager.initialize_patterns();
        manager.initialize_risk_scores();
        manager
    }

    fn initialize_patterns(&self) {
        let patterns = vec![
            // Password patterns
            r"password\s*[:=]\s*\S+",
            r"passwd\s+\S+",
            r"--password\s+\S+",
            
            // API keys and tokens
            r"api[_-]?key\s*[:=]\s*\S+",
            r"access[_-]?token\s*[:=]\s*\S+",
            r"bearer\s+\S+",
            
            // SSH keys
            r"-----BEGIN [A-Z]+ PRIVATE KEY-----",
            r"ssh-rsa\s+[A-Za-z0-9+/]+",
            
            // Database connection strings
            r"mongodb://[^/]+/",
            r"postgres://[^/]+/",
            r"mysql://[^/]+/",
            
            // Credit card numbers (simple pattern)
            r"\b\d{4}[\s-]?\d{4}[\s-]?\d{4}[\s-]?\d{4}\b",
            
            // Social security numbers
            r"\b\d{3}-\d{2}-\d{4}\b",
        ];

        let mut sensitive_patterns = self.sensitive_patterns.lock().unwrap();
        for pattern in patterns {
            if let Ok(regex) = Regex::new(pattern) {
                sensitive_patterns.push(regex);
            }
        }
    }

    fn initialize_risk_scores(&self) {
        let mut risk_scores = self.command_risk_scores.lock().unwrap();
        
        // High-risk commands
        for cmd in &["rm", "del", "format", "sudo", "su", "chmod", "chown"] {
            risk_scores.insert(cmd.to_string(), 0.8);
        }
        
        // Medium-risk commands
        for cmd in &["curl", "wget", "nc", "netcat", "ssh", "scp", "rsync"] {
            risk_scores.insert(cmd.to_string(), 0.6);
        }
        
        // Low-risk commands
        for cmd in &["ls", "dir", "cat", "type", "echo", "pwd"] {
            risk_scores.insert(cmd.to_string(), 0.2);
        }
    }

    pub fn create_secure_session(&self, user: String) -> String {
        let session_id = Uuid::new_v4().to_string();
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        
        let session = SecureSession {
            id: session_id.clone(),
            start_time: now,
            last_activity: now,
            user: user.clone(),
            encrypted: self.policy.lock().unwrap().encryption_enabled,
            locked: false,
            authentication_required: false,
            risk_score: 0.0,
        };
        
        self.secure_sessions.lock().unwrap().insert(session_id.clone(), session);
        
        // Log session start
        self.log_audit_event(AuditLogEntry {
            id: Uuid::new_v4().to_string(),
            timestamp: now * 1000,
            session_id: session_id.clone(),
            user,
            command: "SESSION_START".to_string(),
            working_directory: std::env::current_dir()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            exit_code: None,
            duration_ms: None,
            ip_address: None,
            event_type: AuditEventType::SessionStart,
            risk_level: RiskLevel::Low,
            blocked: false,
        });
        
        session_id
    }

    pub fn validate_command(&self, session_id: &str, command: &str) -> CommandValidationResult {
        let policy = self.policy.lock().unwrap();
        
        // Check if command is blocked
        for blocked_cmd in &policy.blocked_commands {
            if command.contains(blocked_cmd) {
                self.generate_security_alert(
                    session_id,
                    SecurityAlertType::BlockedCommand,
                    format!("Blocked command attempted: {}", command),
                    RiskLevel::High,
                    Some(command.to_string()),
                );
                return CommandValidationResult::Blocked(format!("Command blocked by security policy: {}", blocked_cmd));
            }
        }
        
        // Check if command requires confirmation
        for confirm_pattern in &policy.require_confirmation {
            if command.contains(confirm_pattern) {
                return CommandValidationResult::RequiresConfirmation(
                    format!("This command is potentially dangerous: {}. Are you sure you want to continue?", command)
                );
            }
        }
        
        // Calculate risk score
        let risk_score = self.calculate_command_risk(command);
        
        // Update session risk score
        if let Some(session) = self.secure_sessions.lock().unwrap().get_mut(session_id) {
            session.risk_score = (session.risk_score + risk_score) / 2.0;
            session.last_activity = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            
            // Generate alert for high-risk commands
            if risk_score > 0.7 {
                self.generate_security_alert(
                    session_id,
                    SecurityAlertType::SuspiciousActivity,
                    format!("High-risk command executed: {}", command),
                    RiskLevel::High,
                    Some(command.to_string()),
                );
            }
        }
        
        CommandValidationResult::Allowed
    }

    pub fn mask_sensitive_data(&self, input: &str) -> String {
        let policy = self.policy.lock().unwrap();
        if !policy.mask_sensitive_data {
            return input.to_string();
        }
        drop(policy);
        
        let mut masked = input.to_string();
        let patterns = self.sensitive_patterns.lock().unwrap();
        
        for pattern in patterns.iter() {
            masked = pattern.replace_all(&masked, "[MASKED]").to_string();
        }
        
        masked
    }

    pub fn log_audit_event(&self, event: AuditLogEntry) {
        let mut logs = self.audit_logs.lock().unwrap();
        logs.push(event.clone());
        
        // Keep only recent logs (last 10000 entries)
        if logs.len() > 10000 {
            logs.remove(0);
        }
        
        // Write to file if audit logging is enabled
        if self.policy.lock().unwrap().enable_audit_logging {
            self.write_audit_log_to_file(&event);
        }
    }

    pub fn generate_security_alert(&self, session_id: &str, alert_type: SecurityAlertType, message: String, risk_level: RiskLevel, command: Option<String>) {
        let alert = SecurityAlert {
            id: Uuid::new_v4().to_string(),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64,
            session_id: session_id.to_string(),
            alert_type: alert_type.clone(),
            message,
            risk_level,
            command,
            remediation: self.get_remediation_advice(&alert_type),
        };
        
        self.security_alerts.lock().unwrap().push(alert);
    }

    pub fn check_session_timeout(&self, session_id: &str) -> bool {
        let policy = self.policy.lock().unwrap();
        let timeout = policy.auto_lock_timeout;
        drop(policy);
        
        if let Some(timeout_secs) = timeout {
            if let Some(session) = self.secure_sessions.lock().unwrap().get(session_id) {
                let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                let inactive_time = now - session.last_activity;
                
                if inactive_time > timeout_secs {
                    self.generate_security_alert(
                        session_id,
                        SecurityAlertType::SessionTimeout,
                        "Session timed out due to inactivity".to_string(),
                        RiskLevel::Medium,
                        None,
                    );
                    return true;
                }
            }
        }
        
        false
    }

    pub fn lock_session(&self, session_id: &str) {
        if let Some(session) = self.secure_sessions.lock().unwrap().get_mut(session_id) {
            session.locked = true;
            session.authentication_required = true;
        }
    }

    pub fn unlock_session(&self, session_id: &str, _credentials: &str) -> bool {
        // In a real implementation, this would verify credentials
        if let Some(session) = self.secure_sessions.lock().unwrap().get_mut(session_id) {
            session.locked = false;
            session.authentication_required = false;
            session.last_activity = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            return true;
        }
        false
    }

    pub fn encrypt_data(&self, data: &str) -> Result<String, String> {
        let key = self.encryption_key.lock().unwrap();
        if key.is_none() {
            return Err("Encryption key not set".to_string());
        }
        
        // Simplified encryption (in production, use proper encryption)
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        let hash = hasher.finalize();
        
        Ok(general_purpose::STANDARD.encode(hash))
    }

    pub fn decrypt_data(&self, _encrypted_data: &str) -> Result<String, String> {
        // Simplified decryption placeholder
        Err("Decryption not implemented in this example".to_string())
    }

    pub fn get_audit_logs(&self, limit: Option<usize>, filter: Option<AuditLogFilter>) -> Vec<AuditLogEntry> {
        let logs = self.audit_logs.lock().unwrap();
        let mut filtered_logs: Vec<AuditLogEntry> = logs.iter().cloned().collect();
        
        if let Some(filter) = filter {
            filtered_logs = filtered_logs.into_iter()
                .filter(|log| self.matches_filter(log, &filter))
                .collect();
        }
        
        filtered_logs.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        
        if let Some(limit) = limit {
            filtered_logs.truncate(limit);
        }
        
        filtered_logs
    }

    pub fn get_security_alerts(&self, limit: Option<usize>) -> Vec<SecurityAlert> {
        let alerts = self.security_alerts.lock().unwrap();
        let mut sorted_alerts: Vec<SecurityAlert> = alerts.iter().cloned().collect();
        
        sorted_alerts.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        
        if let Some(limit) = limit {
            sorted_alerts.truncate(limit);
        }
        
        sorted_alerts
    }

    pub fn update_security_policy(&self, policy: SecurityPolicy) {
        *self.policy.lock().unwrap() = policy;
    }

    pub fn get_session_info(&self, session_id: &str) -> Option<SecureSession> {
        self.secure_sessions.lock().unwrap().get(session_id).cloned()
    }

    fn calculate_command_risk(&self, command: &str) -> f64 {
        let risk_scores = self.command_risk_scores.lock().unwrap();
        let words: Vec<&str> = command.split_whitespace().collect();
        
        if words.is_empty() {
            return 0.0;
        }
        
        let base_command = words[0];
        let base_risk = risk_scores.get(base_command).copied().unwrap_or(0.3);
        
        // Additional risk factors
        let mut risk_multiplier = 1.0;
        
        // Check for dangerous flags
        if command.contains("-rf") || command.contains("/s") || command.contains("--force") {
            risk_multiplier += 0.5;
        }
        
        // Check for wildcards
        if command.contains("*") || command.contains("?") {
            risk_multiplier += 0.2;
        }
        
        // Check for privilege escalation
        if command.contains("sudo") || command.contains("su ") {
            risk_multiplier += 0.3;
        }
        
        (base_risk * risk_multiplier).min(1.0)
    }

    fn get_remediation_advice(&self, alert_type: &SecurityAlertType) -> Option<String> {
        match alert_type {
            SecurityAlertType::BlockedCommand => Some("Review security policy and consider if this command should be allowed".to_string()),
            SecurityAlertType::SuspiciousActivity => Some("Investigate the command and user activity for potential security threats".to_string()),
            SecurityAlertType::PrivilegeEscalation => Some("Verify that privilege escalation is authorized and necessary".to_string()),
            SecurityAlertType::UnauthorizedAccess => Some("Check user permissions and session authentication".to_string()),
            SecurityAlertType::DataLeakage => Some("Review data access patterns and implement additional monitoring".to_string()),
            SecurityAlertType::MaliciousPattern => Some("Investigate for malware or unauthorized scripts".to_string()),
            SecurityAlertType::SessionTimeout => Some("Re-authenticate user before continuing session".to_string()),
        }
    }

    fn write_audit_log_to_file(&self, event: &AuditLogEntry) {
        // In a real implementation, this would write to a secure log file
        // For now, we'll just log to stdout in debug builds
        #[cfg(debug_assertions)]
        println!("AUDIT: {} - {} executed '{}' in {}", 
            event.timestamp, event.user, event.command, event.working_directory);
    }

    fn matches_filter(&self, log: &AuditLogEntry, filter: &AuditLogFilter) -> bool {
        if let Some(ref user) = filter.user {
            if !log.user.contains(user) {
                return false;
            }
        }
        
        if let Some(ref command) = filter.command {
            if !log.command.contains(command) {
                return false;
            }
        }
        
        if let Some(ref event_type) = filter.event_type {
            if std::mem::discriminant(&log.event_type) != std::mem::discriminant(event_type) {
                return false;
            }
        }
        
        if let Some(ref risk_level) = filter.risk_level {
            if std::mem::discriminant(&log.risk_level) != std::mem::discriminant(risk_level) {
                return false;
            }
        }
        
        if let Some(start_time) = filter.start_time {
            if log.timestamp < start_time {
                return false;
            }
        }
        
        if let Some(end_time) = filter.end_time {
            if log.timestamp > end_time {
                return false;
            }
        }
        
        true
    }
}

#[derive(Debug, Clone)]
pub struct AuditLogFilter {
    pub user: Option<String>,
    pub command: Option<String>,
    pub event_type: Option<AuditEventType>,
    pub risk_level: Option<RiskLevel>,
    pub start_time: Option<u64>,
    pub end_time: Option<u64>,
}

pub enum CommandValidationResult {
    Allowed,
    Blocked(String),
    RequiresConfirmation(String),
}

// Tauri commands for security features
#[tauri::command]
pub async fn validate_command(session_id: String, command: String) -> Result<String, String> {
    // This would access the global security manager instance
    // For now, return allowed
    Ok("allowed".to_string())
}

#[tauri::command]
pub async fn get_security_alerts(limit: Option<usize>) -> Result<Vec<SecurityAlert>, String> {
    // This would access the global security manager instance
    Ok(vec![])
}

#[tauri::command]
pub async fn get_audit_logs(limit: Option<usize>) -> Result<Vec<AuditLogEntry>, String> {
    // This would access the global security manager instance
    Ok(vec![])
}

#[tauri::command]
pub async fn update_security_policy(policy: SecurityPolicy) -> Result<(), String> {
    // This would access the global security manager instance
    Ok(())
}

#[tauri::command]
pub async fn lock_session(session_id: String) -> Result<(), String> {
    // This would access the global security manager instance
    Ok(())
}

#[tauri::command]
pub async fn unlock_session(session_id: String, credentials: String) -> Result<bool, String> {
    // This would access the global security manager instance
    Ok(true)
}

#[tauri::command]
pub async fn get_session_security_info(session_id: String) -> Result<Option<SecureSession>, String> {
    // This would access the global security manager instance
    Ok(None)
}
