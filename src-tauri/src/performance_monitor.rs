use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub terminal_id: String,
    pub timestamp: u64,
    pub cpu_usage: f64,
    pub memory_usage: u64,
    pub output_rate: f64,        // bytes per second
    pub input_rate: f64,         // bytes per second
    pub render_time_ms: f64,
    pub latency_ms: f64,
    pub scrollback_size: u64,
    pub active_processes: u32,
    pub bandwidth_in: u64,
    pub bandwidth_out: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandPerformance {
    pub command: String,
    pub start_time: u64,
    pub end_time: Option<u64>,
    pub duration_ms: Option<u64>,
    pub exit_code: Option<i32>,
    pub memory_peak: u64,
    pub cpu_peak: f64,
    pub output_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemResources {
    pub total_memory: u64,
    pub available_memory: u64,
    pub cpu_count: u32,
    pub cpu_usage: f64,
    pub disk_usage: HashMap<String, DiskUsage>,
    pub network_interfaces: Vec<NetworkInterface>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskUsage {
    pub total: u64,
    pub available: u64,
    pub used: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    pub name: String,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAlert {
    pub id: String,
    pub terminal_id: String,
    pub alert_type: AlertType,
    pub message: String,
    pub timestamp: u64,
    pub threshold: f64,
    pub current_value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertType {
    HighCpuUsage,
    HighMemoryUsage,
    HighLatency,
    SlowCommand,
    LargeOutput,
    HighBandwidth,
}

pub struct PerformanceMonitor {
    metrics_history: Arc<Mutex<HashMap<String, VecDeque<PerformanceMetrics>>>>,
    command_history: Arc<Mutex<HashMap<String, VecDeque<CommandPerformance>>>>,
    active_commands: Arc<Mutex<HashMap<String, CommandPerformance>>>,
    alerts: Arc<Mutex<VecDeque<PerformanceAlert>>>,
    alert_sender: mpsc::UnboundedSender<PerformanceAlert>,
    monitoring_enabled: Arc<Mutex<bool>>,
    thresholds: Arc<Mutex<PerformanceThresholds>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceThresholds {
    pub cpu_threshold: f64,
    pub memory_threshold: u64,
    pub latency_threshold: f64,
    pub command_timeout: u64,
    pub output_size_threshold: u64,
    pub bandwidth_threshold: u64,
}

impl Default for PerformanceThresholds {
    fn default() -> Self {
        Self {
            cpu_threshold: 80.0,           // 80% CPU usage
            memory_threshold: 1024 * 1024 * 1024, // 1GB memory usage
            latency_threshold: 100.0,      // 100ms latency
            command_timeout: 30000,        // 30 seconds
            output_size_threshold: 10 * 1024 * 1024, // 10MB output
            bandwidth_threshold: 100 * 1024 * 1024, // 100MB/s bandwidth
        }
    }
}

impl PerformanceMonitor {
    pub fn new() -> (Self, mpsc::UnboundedReceiver<PerformanceAlert>) {
        let (alert_sender, alert_receiver) = mpsc::unbounded_channel();
        
        let monitor = PerformanceMonitor {
            metrics_history: Arc::new(Mutex::new(HashMap::new())),
            command_history: Arc::new(Mutex::new(HashMap::new())),
            active_commands: Arc::new(Mutex::new(HashMap::new())),
            alerts: Arc::new(Mutex::new(VecDeque::new())),
            alert_sender,
            monitoring_enabled: Arc::new(Mutex::new(true)),
            thresholds: Arc::new(Mutex::new(PerformanceThresholds::default())),
        };

        (monitor, alert_receiver)
    }

    pub fn start_monitoring(&self, terminal_id: String) {
        if !*self.monitoring_enabled.lock().unwrap() {
            return;
        }

        let metrics_history = self.metrics_history.clone();
        let alert_sender = self.alert_sender.clone();
        let thresholds = self.thresholds.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(1));
            
            loop {
                interval.tick().await;
                
                if let Ok(metrics) = Self::collect_metrics(&terminal_id).await {
                    // Store metrics
                    {
                        let mut history = metrics_history.lock().unwrap();
                        let terminal_history = history.entry(terminal_id.clone()).or_insert_with(VecDeque::new);
                        terminal_history.push_back(metrics.clone());
                        
                        // Keep only last 3600 entries (1 hour at 1 second intervals)
                        if terminal_history.len() > 3600 {
                            terminal_history.pop_front();
                        }
                    }
                    
                    // Check thresholds and generate alerts
                    Self::check_thresholds(&metrics, &thresholds, &alert_sender);
                }
            }
        });
    }

    pub fn start_command_monitoring(&self, terminal_id: String, command: String) -> String {
        let command_id = Uuid::new_v4().to_string();
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;

        let command_perf = CommandPerformance {
            command: command.clone(),
            start_time: now,
            end_time: None,
            duration_ms: None,
            exit_code: None,
            memory_peak: 0,
            cpu_peak: 0.0,
            output_size: 0,
        };

        self.active_commands.lock().unwrap().insert(command_id.clone(), command_perf);

        // Monitor command resources
        let active_commands = self.active_commands.clone();
        let alert_sender = self.alert_sender.clone();
        let thresholds = self.thresholds.clone();
        let cmd_id = command_id.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(100));
            
            while active_commands.lock().unwrap().contains_key(&cmd_id) {
                interval.tick().await;
                
                // Collect command-specific metrics (simplified)
                if let Ok(resources) = Self::get_process_resources(&command).await {
                    let mut commands = active_commands.lock().unwrap();
                    if let Some(cmd_perf) = commands.get_mut(&cmd_id) {
                        cmd_perf.memory_peak = cmd_perf.memory_peak.max(resources.memory);
                        cmd_perf.cpu_peak = cmd_perf.cpu_peak.max(resources.cpu);
                        
                        // Check for slow command alerts
                        let elapsed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64 - cmd_perf.start_time;
                        let threshold_ms = thresholds.lock().unwrap().command_timeout;
                        
                        if elapsed > threshold_ms {
                            let alert = PerformanceAlert {
                                id: Uuid::new_v4().to_string(),
                                terminal_id: terminal_id.clone(),
                                alert_type: AlertType::SlowCommand,
                                message: format!("Command '{}' has been running for {} seconds", command, elapsed / 1000),
                                timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64,
                                threshold: threshold_ms as f64,
                                current_value: elapsed as f64,
                            };
                            let _ = alert_sender.send(alert);
                        }
                    }
                }
                
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        });

        command_id
    }

    pub fn end_command_monitoring(&self, command_id: &str, exit_code: Option<i32>, output_size: u64) {
        let mut active_commands = self.active_commands.lock().unwrap();
        
        if let Some(mut command_perf) = active_commands.remove(command_id) {
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
            command_perf.end_time = Some(now);
            command_perf.duration_ms = Some(now - command_perf.start_time);
            command_perf.exit_code = exit_code;
            command_perf.output_size = output_size;

            // Store in command history
            let terminal_id = "default".to_string(); // This should be properly tracked
            let mut command_history = self.command_history.lock().unwrap();
            let history = command_history.entry(terminal_id).or_insert_with(VecDeque::new);
            history.push_back(command_perf);
            
            // Keep only last 1000 commands
            if history.len() > 1000 {
                history.pop_front();
            }
        }
    }

    pub fn get_metrics_history(&self, terminal_id: &str, duration_seconds: Option<u64>) -> Vec<PerformanceMetrics> {
        let history = self.metrics_history.lock().unwrap();
        
        if let Some(terminal_history) = history.get(terminal_id) {
            let limit = duration_seconds.unwrap_or(3600); // Default to 1 hour
            let entries_to_take = (limit as usize).min(terminal_history.len());
            
            terminal_history.iter()
                .rev()
                .take(entries_to_take)
                .cloned()
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn get_command_history(&self, terminal_id: &str) -> Vec<CommandPerformance> {
        let history = self.command_history.lock().unwrap();
        
        if let Some(terminal_history) = history.get(terminal_id) {
            terminal_history.iter().cloned().collect()
        } else {
            Vec::new()
        }
    }

    pub fn get_recent_alerts(&self, limit: Option<usize>) -> Vec<PerformanceAlert> {
        let alerts = self.alerts.lock().unwrap();
        let take_count = limit.unwrap_or(50).min(alerts.len());
        
        alerts.iter()
            .rev()
            .take(take_count)
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }

    pub fn set_thresholds(&self, thresholds: PerformanceThresholds) {
        *self.thresholds.lock().unwrap() = thresholds;
    }

    pub fn toggle_monitoring(&self, enabled: bool) {
        *self.monitoring_enabled.lock().unwrap() = enabled;
    }

    async fn collect_metrics(terminal_id: &str) -> Result<PerformanceMetrics, String> {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
        
        // Collect system metrics (simplified implementation)
        let system_info = Self::get_system_info().await?;
        
        Ok(PerformanceMetrics {
            terminal_id: terminal_id.to_string(),
            timestamp,
            cpu_usage: system_info.cpu_usage,
            memory_usage: system_info.total_memory - system_info.available_memory,
            output_rate: 0.0, // This would be calculated from actual output
            input_rate: 0.0,  // This would be calculated from actual input
            render_time_ms: 0.0, // This would be measured during rendering
            latency_ms: 0.0,  // This would be measured for input->output latency
            scrollback_size: 0, // This would come from terminal state
            active_processes: 1, // This would count active processes
            bandwidth_in: 0,
            bandwidth_out: 0,
        })
    }

    async fn get_system_info() -> Result<SystemResources, String> {
        // Simplified system info collection
        // In a real implementation, this would use system APIs or libraries like sysinfo
        
        Ok(SystemResources {
            total_memory: 8 * 1024 * 1024 * 1024, // 8GB
            available_memory: 4 * 1024 * 1024 * 1024, // 4GB
            cpu_count: num_cpus::get() as u32,
            cpu_usage: 25.0, // Placeholder
            disk_usage: HashMap::new(),
            network_interfaces: Vec::new(),
        })
    }

    async fn get_process_resources(_command: &str) -> Result<ProcessResources, String> {
        // Simplified process resource collection
        Ok(ProcessResources {
            memory: 100 * 1024 * 1024, // 100MB
            cpu: 10.0, // 10% CPU
        })
    }

    fn check_thresholds(
        metrics: &PerformanceMetrics,
        thresholds: &Arc<Mutex<PerformanceThresholds>>,
        alert_sender: &mpsc::UnboundedSender<PerformanceAlert>,
    ) {
        let thresholds = thresholds.lock().unwrap();
        
        // Check CPU usage
        if metrics.cpu_usage > thresholds.cpu_threshold {
            let alert = PerformanceAlert {
                id: Uuid::new_v4().to_string(),
                terminal_id: metrics.terminal_id.clone(),
                alert_type: AlertType::HighCpuUsage,
                message: format!("High CPU usage: {:.1}%", metrics.cpu_usage),
                timestamp: metrics.timestamp,
                threshold: thresholds.cpu_threshold,
                current_value: metrics.cpu_usage,
            };
            let _ = alert_sender.send(alert);
        }
        
        // Check memory usage
        if metrics.memory_usage > thresholds.memory_threshold {
            let alert = PerformanceAlert {
                id: Uuid::new_v4().to_string(),
                terminal_id: metrics.terminal_id.clone(),
                alert_type: AlertType::HighMemoryUsage,
                message: format!("High memory usage: {} MB", metrics.memory_usage / (1024 * 1024)),
                timestamp: metrics.timestamp,
                threshold: thresholds.memory_threshold as f64,
                current_value: metrics.memory_usage as f64,
            };
            let _ = alert_sender.send(alert);
        }
        
        // Check latency
        if metrics.latency_ms > thresholds.latency_threshold {
            let alert = PerformanceAlert {
                id: Uuid::new_v4().to_string(),
                terminal_id: metrics.terminal_id.clone(),
                alert_type: AlertType::HighLatency,
                message: format!("High latency: {:.1}ms", metrics.latency_ms),
                timestamp: metrics.timestamp,
                threshold: thresholds.latency_threshold,
                current_value: metrics.latency_ms,
            };
            let _ = alert_sender.send(alert);
        }
    }
}

struct ProcessResources {
    memory: u64,
    cpu: f64,
}

// Tauri commands for performance monitoring
#[tauri::command]
pub async fn get_performance_metrics(terminal_id: String, duration_seconds: Option<u64>) -> Result<Vec<PerformanceMetrics>, String> {
    // This would access the global performance monitor instance
    // For now, return empty metrics
    Ok(vec![])
}

#[tauri::command]
pub async fn get_command_performance_history(terminal_id: String) -> Result<Vec<CommandPerformance>, String> {
    // This would access the global performance monitor instance
    Ok(vec![])
}

#[tauri::command]
pub async fn get_system_resources() -> Result<SystemResources, String> {
    PerformanceMonitor::get_system_info().await
}

#[tauri::command]
pub async fn get_performance_alerts(limit: Option<usize>) -> Result<Vec<PerformanceAlert>, String> {
    // This would access the global performance monitor instance
    Ok(vec![])
}

#[tauri::command]
pub async fn set_performance_thresholds(thresholds: PerformanceThresholds) -> Result<(), String> {
    // This would access the global performance monitor instance
    Ok(())
}

#[tauri::command]
pub async fn toggle_performance_monitoring(enabled: bool) -> Result<(), String> {
    // This would access the global performance monitor instance
    Ok(())
}
