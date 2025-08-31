use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::net::{TcpListener, TcpStream};
use tokio::process::Command;
use tokio::sync::mpsc;
use tokio::time::{interval, timeout};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshConnection {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: Option<String>,
    pub private_key_path: Option<String>,
    pub identity_file: Option<String>,
    pub connection_timeout: u64,
    pub keepalive_interval: u64,
    pub compression: bool,
    pub forward_agent: bool,
    pub forward_x11: bool,
    pub proxy_jump: Option<String>,
    pub tags: Vec<String>,
    pub last_connected: Option<u64>,
    pub connection_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SshConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Failed,
    Timeout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveSshSession {
    pub connection_id: String,
    pub session_id: String,
    pub terminal_id: Option<String>,
    pub status: SshConnectionStatus,
    pub connected_at: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub last_activity: u64,
    pub local_port_forwards: Vec<PortForward>,
    pub remote_port_forwards: Vec<PortForward>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortForward {
    pub id: String,
    pub local_port: u16,
    pub remote_host: String,
    pub remote_port: u16,
    pub is_active: bool,
    pub created_at: u64,
    pub bytes_transferred: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub mac_address: String,
    pub ip_addresses: Vec<IpAddr>,
    pub subnet_mask: Option<String>,
    pub gateway: Option<IpAddr>,
    pub dns_servers: Vec<IpAddr>,
    pub is_up: bool,
    pub is_loopback: bool,
    pub is_wireless: bool,
    pub speed: Option<u64>, // in Mbps
    pub mtu: u32,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub rx_packets: u64,
    pub tx_packets: u64,
    pub rx_errors: u64,
    pub tx_errors: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConnection {
    pub local_address: SocketAddr,
    pub remote_address: Option<SocketAddr>,
    pub protocol: NetworkProtocol,
    pub state: ConnectionState,
    pub process_id: Option<u32>,
    pub process_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NetworkProtocol {
    Tcp,
    Udp,
    Icmp,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConnectionState {
    Listen,
    Established,
    SynSent,
    SynReceived,
    FinWait1,
    FinWait2,
    CloseWait,
    Closing,
    LastAck,
    TimeWait,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortScanResult {
    pub host: String,
    pub port: u16,
    pub is_open: bool,
    pub service: Option<String>,
    pub response_time: Option<Duration>,
    pub banner: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostDiscoveryResult {
    pub ip_address: IpAddr,
    pub hostname: Option<String>,
    pub mac_address: Option<String>,
    pub vendor: Option<String>,
    pub is_reachable: bool,
    pub response_time: Option<Duration>,
    pub open_ports: Vec<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    pub interfaces: Vec<NetworkInterface>,
    pub connections: Vec<NetworkConnection>,
    pub total_rx_bytes: u64,
    pub total_tx_bytes: u64,
    pub packets_per_second: f64,
    pub connections_count: usize,
    pub listening_ports: Vec<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMonitorConfig {
    pub interface_monitoring: bool,
    pub connection_monitoring: bool,
    pub port_scan_detection: bool,
    pub bandwidth_monitoring: bool,
    pub update_interval: u64, // seconds
    pub alert_thresholds: NetworkAlertThresholds,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkAlertThresholds {
    pub high_bandwidth_threshold: u64, // bytes per second
    pub suspicious_connection_count: usize,
    pub failed_connection_threshold: usize,
    pub port_scan_detection_threshold: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkAlert {
    pub alert_type: NetworkAlertType,
    pub severity: AlertSeverity,
    pub message: String,
    pub details: HashMap<String, String>,
    pub timestamp: u64,
    pub acknowledged: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NetworkAlertType {
    HighBandwidth,
    SuspiciousConnections,
    PortScanDetected,
    InterfaceDown,
    ConnectionFailed,
    UnauthorizedAccess,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

pub struct NetworkManager {
    ssh_connections: Arc<Mutex<HashMap<String, SshConnection>>>,
    active_sessions: Arc<Mutex<HashMap<String, ActiveSshSession>>>,
    port_forwards: Arc<Mutex<HashMap<String, PortForward>>>,
    network_interfaces: Arc<Mutex<Vec<NetworkInterface>>>,
    network_connections: Arc<Mutex<Vec<NetworkConnection>>>,
    monitoring_config: Arc<Mutex<NetworkMonitorConfig>>,
    alerts: Arc<Mutex<Vec<NetworkAlert>>>,
    monitoring_enabled: Arc<Mutex<bool>>,
}

impl NetworkManager {
    pub fn new() -> Self {
        let default_config = NetworkMonitorConfig {
            interface_monitoring: true,
            connection_monitoring: true,
            port_scan_detection: true,
            bandwidth_monitoring: true,
            update_interval: 5,
            alert_thresholds: NetworkAlertThresholds {
                high_bandwidth_threshold: 100 * 1024 * 1024, // 100 MB/s
                suspicious_connection_count: 50,
                failed_connection_threshold: 10,
                port_scan_detection_threshold: 20,
            },
        };

        Self {
            ssh_connections: Arc::new(Mutex::new(HashMap::new())),
            active_sessions: Arc::new(Mutex::new(HashMap::new())),
            port_forwards: Arc::new(Mutex::new(HashMap::new())),
            network_interfaces: Arc::new(Mutex::new(Vec::new())),
            network_connections: Arc::new(Mutex::new(Vec::new())),
            monitoring_config: Arc::new(Mutex::new(default_config)),
            alerts: Arc::new(Mutex::new(Vec::new())),
            monitoring_enabled: Arc::new(Mutex::new(false)),
        }
    }

    // SSH Connection Management
    pub fn add_ssh_connection(&self, connection: SshConnection) -> Result<String, String> {
        let mut connections = self.ssh_connections.lock().unwrap();
        let connection_id = connection.id.clone();
        connections.insert(connection_id.clone(), connection);
        Ok(connection_id)
    }

    pub fn get_ssh_connections(&self) -> Vec<SshConnection> {
        let connections = self.ssh_connections.lock().unwrap();
        connections.values().cloned().collect()
    }

    pub fn get_ssh_connection(&self, connection_id: &str) -> Option<SshConnection> {
        let connections = self.ssh_connections.lock().unwrap();
        connections.get(connection_id).cloned()
    }

    pub fn update_ssh_connection(&self, connection_id: &str, updated_connection: SshConnection) -> Result<(), String> {
        let mut connections = self.ssh_connections.lock().unwrap();
        if !connections.contains_key(connection_id) {
            return Err(format!("SSH connection {} not found", connection_id));
        }
        connections.insert(connection_id.to_string(), updated_connection);
        Ok(())
    }

    pub fn remove_ssh_connection(&self, connection_id: &str) -> Result<(), String> {
        let mut connections = self.ssh_connections.lock().unwrap();
        if !connections.contains_key(connection_id) {
            return Err(format!("SSH connection {} not found", connection_id));
        }
        connections.remove(connection_id);
        Ok(())
    }

    pub async fn connect_ssh(&self, connection_id: &str, terminal_id: Option<String>) -> Result<String, String> {
        let connection = self.get_ssh_connection(connection_id)
            .ok_or_else(|| format!("SSH connection {} not found", connection_id))?;

        let session_id = format!("{}-{}", connection_id, SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs());

        // Build SSH command
        let connect_timeout = format!("ConnectTimeout={}", connection.connection_timeout);
        let keepalive_interval = format!("ServerAliveInterval={}", connection.keepalive_interval);
        let port_str = connection.port.to_string();
        let user_host = format!("{}@{}", connection.username, connection.host);
        
        let mut ssh_args = vec![
            "-o", "StrictHostKeyChecking=no",
            "-o", &connect_timeout,
            "-o", &keepalive_interval,
        ];

        if connection.compression {
            ssh_args.push("-C");
        }

        if connection.forward_agent {
            ssh_args.push("-A");
        }

        if connection.forward_x11 {
            ssh_args.push("-X");
        }

        if let Some(ref identity_file) = connection.identity_file {
            ssh_args.extend_from_slice(&["-i", identity_file]);
        }

        if let Some(ref proxy_jump) = connection.proxy_jump {
            ssh_args.extend_from_slice(&["-J", proxy_jump]);
        }

        ssh_args.push("-p");
        ssh_args.push(&port_str);
        ssh_args.push(&user_host);

        // Start SSH process
        let mut ssh_command = Command::new("ssh");
        ssh_command.args(&ssh_args);

        match ssh_command.spawn() {
            Ok(_child) => {
                let session = ActiveSshSession {
                    connection_id: connection_id.to_string(),
                    session_id: session_id.clone(),
                    terminal_id,
                    status: SshConnectionStatus::Connected,
                    connected_at: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    bytes_sent: 0,
                    bytes_received: 0,
                    last_activity: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    local_port_forwards: Vec::new(),
                    remote_port_forwards: Vec::new(),
                };

                {
                    let mut sessions = self.active_sessions.lock().unwrap();
                    sessions.insert(session_id.clone(), session);
                }

                // Update connection stats
                {
                    let mut connections = self.ssh_connections.lock().unwrap();
                    if let Some(conn) = connections.get_mut(connection_id) {
                        conn.last_connected = Some(SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs());
                        conn.connection_count += 1;
                    }
                }

                Ok(session_id)
            }
            Err(e) => Err(format!("Failed to start SSH connection: {}", e)),
        }
    }

    pub fn disconnect_ssh(&self, session_id: &str) -> Result<(), String> {
        let mut sessions = self.active_sessions.lock().unwrap();
        if let Some(session) = sessions.get_mut(session_id) {
            session.status = SshConnectionStatus::Disconnected;
            sessions.remove(session_id);
            Ok(())
        } else {
            Err(format!("SSH session {} not found", session_id))
        }
    }

    pub fn get_active_ssh_sessions(&self) -> Vec<ActiveSshSession> {
        let sessions = self.active_sessions.lock().unwrap();
        sessions.values().cloned().collect()
    }

    // Port Forwarding
    pub async fn create_port_forward(
        &self,
        session_id: &str,
        local_port: u16,
        remote_host: String,
        remote_port: u16,
    ) -> Result<String, String> {
        let forward_id = format!("pf-{}-{}-{}", session_id, local_port, remote_port);
        
        let port_forward = PortForward {
            id: forward_id.clone(),
            local_port,
            remote_host,
            remote_port,
            is_active: true,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            bytes_transferred: 0,
        };

        {
            let mut forwards = self.port_forwards.lock().unwrap();
            forwards.insert(forward_id.clone(), port_forward.clone());
        }

        // Update session
        {
            let mut sessions = self.active_sessions.lock().unwrap();
            if let Some(session) = sessions.get_mut(session_id) {
                session.local_port_forwards.push(port_forward);
            }
        }

        Ok(forward_id)
    }

    pub fn remove_port_forward(&self, forward_id: &str) -> Result<(), String> {
        let mut forwards = self.port_forwards.lock().unwrap();
        if forwards.remove(forward_id).is_some() {
            Ok(())
        } else {
            Err(format!("Port forward {} not found", forward_id))
        }
    }

    pub fn get_port_forwards(&self) -> Vec<PortForward> {
        let forwards = self.port_forwards.lock().unwrap();
        forwards.values().cloned().collect()
    }

    // Network Monitoring
    pub async fn start_network_monitoring(&self) -> Result<mpsc::UnboundedReceiver<NetworkAlert>, String> {
        let (tx, rx) = mpsc::unbounded_channel();

        {
            let mut enabled = self.monitoring_enabled.lock().unwrap();
            *enabled = true;
        }

        // Start monitoring tasks
        let interfaces = self.network_interfaces.clone();
        let connections = self.network_connections.clone();
        let config = self.monitoring_config.clone();
        let enabled = self.monitoring_enabled.clone();
        let alert_tx = tx.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(
                config.lock().unwrap().update_interval
            ));

            while *enabled.lock().unwrap() {
                interval.tick().await;

                // Update network interfaces
                if let Ok(ifaces) = Self::get_network_interfaces().await {
                    let mut interfaces_guard = interfaces.lock().unwrap();
                    *interfaces_guard = ifaces;
                }

                // Update network connections
                if let Ok(conns) = Self::get_network_connections().await {
                    let mut connections_guard = connections.lock().unwrap();
                    *connections_guard = conns;
                }

                // Check for alerts
                // This is a simplified implementation - real monitoring would be more complex
            }
        });

        Ok(rx)
    }

    pub fn stop_network_monitoring(&self) {
        let mut enabled = self.monitoring_enabled.lock().unwrap();
        *enabled = false;
    }

    #[cfg(unix)]
    async fn get_network_interfaces() -> Result<Vec<NetworkInterface>, String> {
        use std::process::Stdio;

        let output = Command::new("ip")
            .args(&["addr", "show"])
            .stdout(Stdio::piped())
            .output()
            .await
            .map_err(|e| format!("Failed to execute ip command: {}", e))?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut interfaces = Vec::new();

        // Simple parsing - in a real implementation you'd use proper network libraries
        for line in output_str.lines() {
            if line.contains(": ") && !line.starts_with(' ') {
                if let Some(interface_name) = line.split(':').nth(1) {
                    let name = interface_name.trim().to_string();
                    interfaces.push(NetworkInterface {
                        name: name.clone(),
                        display_name: name.clone(),
                        description: format!("Network interface {}", name),
                        mac_address: "00:00:00:00:00:00".to_string(),
                        ip_addresses: vec![IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))],
                        subnet_mask: Some("255.255.255.0".to_string()),
                        gateway: None,
                        dns_servers: Vec::new(),
                        is_up: true,
                        is_loopback: name == "lo",
                        is_wireless: name.starts_with("wl"),
                        speed: None,
                        mtu: 1500,
                        rx_bytes: 0,
                        tx_bytes: 0,
                        rx_packets: 0,
                        tx_packets: 0,
                        rx_errors: 0,
                        tx_errors: 0,
                    });
                }
            }
        }

        Ok(interfaces)
    }

    #[cfg(windows)]
    async fn get_network_interfaces() -> Result<Vec<NetworkInterface>, String> {
        // Windows implementation would use Windows API
        Ok(Vec::new())
    }

    #[cfg(unix)]
    async fn get_network_connections() -> Result<Vec<NetworkConnection>, String> {
        let output = Command::new("ss")
            .args(&["-tuln"])
            .output()
            .await
            .map_err(|e| format!("Failed to execute ss command: {}", e))?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut connections = Vec::new();

        for line in output_str.lines().skip(1) { // Skip header
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 5 {
                if let Ok(local_addr) = parts[4].parse::<SocketAddr>() {
                    let protocol = match parts[0] {
                        "tcp" => NetworkProtocol::Tcp,
                        "udp" => NetworkProtocol::Udp,
                        _ => continue,
                    };

                    let state = match parts[1] {
                        "LISTEN" => ConnectionState::Listen,
                        "ESTAB" => ConnectionState::Established,
                        _ => ConnectionState::Closed,
                    };

                    connections.push(NetworkConnection {
                        local_address: local_addr,
                        remote_address: None,
                        protocol,
                        state,
                        process_id: None,
                        process_name: None,
                    });
                }
            }
        }

        Ok(connections)
    }

    #[cfg(windows)]
    async fn get_network_connections() -> Result<Vec<NetworkConnection>, String> {
        // Windows implementation would use Windows API
        Ok(Vec::new())
    }

    pub fn get_network_stats(&self) -> NetworkStats {
        let interfaces = self.network_interfaces.lock().unwrap().clone();
        let connections = self.network_connections.lock().unwrap().clone();

        let total_rx_bytes = interfaces.iter().map(|i| i.rx_bytes).sum();
        let total_tx_bytes = interfaces.iter().map(|i| i.tx_bytes).sum();
        let connections_count = connections.len();

        let listening_ports: Vec<u16> = connections
            .iter()
            .filter(|c| c.state == ConnectionState::Listen)
            .map(|c| c.local_address.port())
            .collect();

        NetworkStats {
            interfaces,
            connections,
            total_rx_bytes,
            total_tx_bytes,
            packets_per_second: 0.0, // Would be calculated over time
            connections_count,
            listening_ports,
        }
    }

    // Port Scanning
    pub async fn scan_ports(&self, host: &str, ports: Vec<u16>) -> Vec<PortScanResult> {
        let mut results = Vec::new();

        for port in ports {
            let start_time = std::time::Instant::now();
            let socket_addr = format!("{}:{}", host, port);

            let is_open = match timeout(Duration::from_secs(3), TcpStream::connect(socket_addr)).await {
                Ok(Ok(_)) => true,
                Ok(Err(_)) | Err(_) => false,
            };

            let response_time = if is_open {
                Some(start_time.elapsed())
            } else {
                None
            };

            let service = self.get_service_name(port);

            results.push(PortScanResult {
                host: host.to_string(),
                port,
                is_open,
                service,
                response_time,
                banner: None, // Could be implemented to grab banners
            });
        }

        results
    }

    fn get_service_name(&self, port: u16) -> Option<String> {
        match port {
            21 => Some("FTP".to_string()),
            22 => Some("SSH".to_string()),
            23 => Some("Telnet".to_string()),
            25 => Some("SMTP".to_string()),
            53 => Some("DNS".to_string()),
            80 => Some("HTTP".to_string()),
            110 => Some("POP3".to_string()),
            143 => Some("IMAP".to_string()),
            443 => Some("HTTPS".to_string()),
            993 => Some("IMAPS".to_string()),
            995 => Some("POP3S".to_string()),
            3389 => Some("RDP".to_string()),
            5432 => Some("PostgreSQL".to_string()),
            3306 => Some("MySQL".to_string()),
            _ => None,
        }
    }

    // Host Discovery
    pub async fn discover_hosts(&self, network: &str) -> Vec<HostDiscoveryResult> {
        let mut results = Vec::new();

        // Simple ping-based discovery
        let network_base = network.trim_end_matches("/24");
        for i in 1..255 {
            let ip_str = format!("{}.{}", network_base, i);
            if let Ok(ip) = ip_str.parse::<IpAddr>() {
                let start_time = std::time::Instant::now();

                #[cfg(unix)]
                let ping_result = Command::new("ping")
                    .args(&["-c", "1", "-W", "1000", &ip_str])
                    .output()
                    .await;

                #[cfg(windows)]
                let ping_result = Command::new("ping")
                    .args(&["-n", "1", "-w", "1000", &ip_str])
                    .output()
                    .await;

                let is_reachable = ping_result
                    .map(|output| output.status.success())
                    .unwrap_or(false);

                if is_reachable {
                    let response_time = Some(start_time.elapsed());
                    
                    // Try to resolve hostname
                    let hostname = self.resolve_hostname(&ip).await;

                    results.push(HostDiscoveryResult {
                        ip_address: ip,
                        hostname,
                        mac_address: None, // Could be implemented with ARP lookup
                        vendor: None,
                        is_reachable: true,
                        response_time,
                        open_ports: Vec::new(), // Could scan common ports
                    });
                }
            }
        }

        results
    }

    async fn resolve_hostname(&self, ip: &IpAddr) -> Option<String> {
        // Simple hostname resolution - in real implementation you'd use proper DNS libraries
        let output = Command::new("nslookup")
            .arg(ip.to_string())
            .output()
            .await
            .ok()?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        for line in output_str.lines() {
            if line.contains("name =") {
                if let Some(hostname) = line.split("name =").nth(1) {
                    return Some(hostname.trim().trim_end_matches('.').to_string());
                }
            }
        }

        None
    }

    // Configuration
    pub fn get_monitoring_config(&self) -> NetworkMonitorConfig {
        let config = self.monitoring_config.lock().unwrap();
        config.clone()
    }

    pub fn update_monitoring_config(&self, new_config: NetworkMonitorConfig) {
        let mut config = self.monitoring_config.lock().unwrap();
        *config = new_config;
    }

    // Alerts
    pub fn get_network_alerts(&self) -> Vec<NetworkAlert> {
        let alerts = self.alerts.lock().unwrap();
        alerts.clone()
    }

    pub fn acknowledge_alert(&self, alert_index: usize) -> Result<(), String> {
        let mut alerts = self.alerts.lock().unwrap();
        if let Some(alert) = alerts.get_mut(alert_index) {
            alert.acknowledged = true;
            Ok(())
        } else {
            Err("Alert not found".to_string())
        }
    }

    pub fn clear_alerts(&self) {
        let mut alerts = self.alerts.lock().unwrap();
        alerts.clear();
    }

    // Utilities
    pub async fn test_connectivity(&self, host: &str, port: u16) -> Result<Duration, String> {
        let start_time = std::time::Instant::now();
        let socket_addr = format!("{}:{}", host, port);

        match timeout(Duration::from_secs(5), TcpStream::connect(socket_addr)).await {
            Ok(Ok(_)) => Ok(start_time.elapsed()),
            Ok(Err(e)) => Err(format!("Connection failed: {}", e)),
            Err(_) => Err("Connection timeout".to_string()),
        }
    }

    pub async fn get_external_ip(&self) -> Result<IpAddr, String> {
        // Simple external IP detection - in real implementation you'd use multiple services
        let output = Command::new("curl")
            .args(&["-s", "https://api.ipify.org"])
            .output()
            .await
            .map_err(|e| format!("Failed to get external IP: {}", e))?;

        let ip_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        ip_str.parse::<IpAddr>()
            .map_err(|e| format!("Failed to parse IP address: {}", e))
    }

    pub fn export_ssh_connections(&self) -> Result<String, String> {
        let connections = self.ssh_connections.lock().unwrap();
        serde_json::to_string_pretty(&*connections)
            .map_err(|e| format!("Failed to serialize connections: {}", e))
    }

    pub fn import_ssh_connections(&self, json_data: &str) -> Result<usize, String> {
        let connections: HashMap<String, SshConnection> = serde_json::from_str(json_data)
            .map_err(|e| format!("Failed to parse connections JSON: {}", e))?;

        let count = connections.len();
        
        {
            let mut conn_guard = self.ssh_connections.lock().unwrap();
            conn_guard.extend(connections);
        }

        Ok(count)
    }
}
