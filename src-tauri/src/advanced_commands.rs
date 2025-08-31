use crate::process_manager::{ProcessManager, ProcessFilter, ProcessAction};
use crate::theme_manager::ThemeManager;
use crate::network_manager::NetworkManager;
use crate::dev_tools::DevToolsManager;
use crate::accessibility::{AccessibilityManager, I18nManager};
use crate::advanced_terminal::AdvancedTerminalManager;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::path::PathBuf;
use tauri::State;

// Process Management Commands
#[tauri::command]
pub async fn start_process_monitoring(
    process_manager: State<'_, Arc<Mutex<ProcessManager>>>,
) -> Result<(), String> {
    let manager = process_manager.lock().await;
    let _ = manager.start_monitoring().await?;
    Ok(())
}

#[tauri::command]
pub async fn stop_process_monitoring(
    process_manager: State<'_, Arc<Mutex<ProcessManager>>>,
) -> Result<(), String> {
    let manager = process_manager.lock().await;
    manager.stop_monitoring();
    Ok(())
}

#[tauri::command]
pub async fn get_processes(
    process_manager: State<'_, Arc<Mutex<ProcessManager>>>,
    filter: Option<ProcessFilter>,
) -> Result<Vec<crate::process_manager::ProcessInfo>, String> {
    let manager = process_manager.lock().await;
    Ok(manager.get_processes(filter))
}

#[tauri::command]
pub async fn execute_process_action(
    process_manager: State<'_, Arc<Mutex<ProcessManager>>>,
    action: ProcessAction,
) -> Result<String, String> {
    let manager = process_manager.lock().await;
    manager.execute_process_action(action).await
}

#[tauri::command]
pub async fn create_job(
    process_manager: State<'_, Arc<Mutex<ProcessManager>>>,
    command: String,
    args: Vec<String>,
    is_background: bool,
    terminal_session: Option<String>,
) -> Result<u32, String> {
    let manager = process_manager.lock().await;
    manager.create_job(command, args, is_background, terminal_session).await
}

#[tauri::command]
pub async fn get_jobs(
    process_manager: State<'_, Arc<Mutex<ProcessManager>>>,
) -> Result<Vec<crate::process_manager::JobInfo>, String> {
    let manager = process_manager.lock().await;
    Ok(manager.get_jobs())
}

#[tauri::command]
pub async fn kill_job(
    process_manager: State<'_, Arc<Mutex<ProcessManager>>>,
    job_id: u32,
) -> Result<String, String> {
    let manager = process_manager.lock().await;
    manager.kill_job(job_id).await
}

// Theme Management Commands
#[tauri::command]
pub async fn get_all_themes(
    theme_manager: State<'_, Arc<Mutex<ThemeManager>>>,
) -> Result<Vec<crate::theme_manager::Theme>, String> {
    let manager = theme_manager.lock().await;
    Ok(manager.get_all_themes())
}

#[tauri::command]
pub async fn get_current_theme(
    theme_manager: State<'_, Arc<Mutex<ThemeManager>>>,
) -> Result<Option<crate::theme_manager::Theme>, String> {
    let manager = theme_manager.lock().await;
    Ok(manager.get_current_theme())
}

#[tauri::command]
pub async fn set_current_theme(
    theme_manager: State<'_, Arc<Mutex<ThemeManager>>>,
    theme_id: String,
) -> Result<(), String> {
    let manager = theme_manager.lock().await;
    manager.set_current_theme(theme_id)
}

#[tauri::command]
pub async fn add_theme(
    theme_manager: State<'_, Arc<Mutex<ThemeManager>>>,
    theme: crate::theme_manager::Theme,
) -> Result<String, String> {
    let manager = theme_manager.lock().await;
    manager.add_theme(theme)
}

#[tauri::command]
pub async fn get_css_variables(
    theme_manager: State<'_, Arc<Mutex<ThemeManager>>>,
    theme_id: String,
) -> Result<String, String> {
    let manager = theme_manager.lock().await;
    manager.get_css_variables(&theme_id)
}

#[tauri::command]
pub async fn export_theme(
    theme_manager: State<'_, Arc<Mutex<ThemeManager>>>,
    theme_id: String,
) -> Result<String, String> {
    let manager = theme_manager.lock().await;
    manager.export_theme(&theme_id)
}

#[tauri::command]
pub async fn import_theme(
    theme_manager: State<'_, Arc<Mutex<ThemeManager>>>,
    json_data: String,
) -> Result<String, String> {
    let manager = theme_manager.lock().await;
    manager.import_theme(&json_data)
}

// Network Management Commands
#[tauri::command]
pub async fn add_ssh_connection(
    network_manager: State<'_, Arc<Mutex<NetworkManager>>>,
    connection: crate::network_manager::SshConnection,
) -> Result<String, String> {
    let manager = network_manager.lock().await;
    manager.add_ssh_connection(connection)
}

#[tauri::command]
pub async fn get_ssh_connections(
    network_manager: State<'_, Arc<Mutex<NetworkManager>>>,
) -> Result<Vec<crate::network_manager::SshConnection>, String> {
    let manager = network_manager.lock().await;
    Ok(manager.get_ssh_connections())
}

#[tauri::command]
pub async fn connect_ssh(
    network_manager: State<'_, Arc<Mutex<NetworkManager>>>,
    connection_id: String,
    terminal_id: Option<String>,
) -> Result<String, String> {
    let manager = network_manager.lock().await;
    manager.connect_ssh(&connection_id, terminal_id).await
}

#[tauri::command]
pub async fn disconnect_ssh(
    network_manager: State<'_, Arc<Mutex<NetworkManager>>>,
    session_id: String,
) -> Result<(), String> {
    let manager = network_manager.lock().await;
    manager.disconnect_ssh(&session_id)
}

#[tauri::command]
pub async fn scan_ports(
    network_manager: State<'_, Arc<Mutex<NetworkManager>>>,
    host: String,
    ports: Vec<u16>,
) -> Result<Vec<crate::network_manager::PortScanResult>, String> {
    let manager = network_manager.lock().await;
    Ok(manager.scan_ports(&host, ports).await)
}

#[tauri::command]
pub async fn get_network_stats(
    network_manager: State<'_, Arc<Mutex<NetworkManager>>>,
) -> Result<crate::network_manager::NetworkStats, String> {
    let manager = network_manager.lock().await;
    Ok(manager.get_network_stats())
}

// Developer Tools Commands
#[tauri::command]
pub async fn discover_git_repositories(
    dev_tools_manager: State<'_, Arc<Mutex<DevToolsManager>>>,
    base_path: PathBuf,
) -> Result<Vec<String>, String> {
    let manager = dev_tools_manager.lock().await;
    manager.discover_git_repositories(&base_path).await
}

#[tauri::command]
pub async fn load_git_repository(
    dev_tools_manager: State<'_, Arc<Mutex<DevToolsManager>>>,
    path: PathBuf,
) -> Result<crate::dev_tools::GitRepository, String> {
    let manager = dev_tools_manager.lock().await;
    manager.load_git_repository(&path).await
}

#[tauri::command]
pub async fn git_commit(
    dev_tools_manager: State<'_, Arc<Mutex<DevToolsManager>>>,
    repo_name: String,
    message: String,
    files: Vec<String>,
) -> Result<String, String> {
    let manager = dev_tools_manager.lock().await;
    manager.git_commit(&repo_name, &message, files).await
}

#[tauri::command]
pub async fn git_push(
    dev_tools_manager: State<'_, Arc<Mutex<DevToolsManager>>>,
    repo_name: String,
    remote: String,
    branch: String,
) -> Result<String, String> {
    let manager = dev_tools_manager.lock().await;
    manager.git_push(&repo_name, &remote, &branch).await
}

#[tauri::command]
pub async fn git_pull(
    dev_tools_manager: State<'_, Arc<Mutex<DevToolsManager>>>,
    repo_name: String,
) -> Result<String, String> {
    let manager = dev_tools_manager.lock().await;
    manager.git_pull(&repo_name).await
}

#[tauri::command]
pub async fn run_build(
    dev_tools_manager: State<'_, Arc<Mutex<DevToolsManager>>>,
    config_name: String,
) -> Result<String, String> {
    let manager = dev_tools_manager.lock().await;
    manager.run_build(&config_name).await
}

#[tauri::command]
pub async fn run_tests(
    dev_tools_manager: State<'_, Arc<Mutex<DevToolsManager>>>,
    config_name: String,
) -> Result<Vec<crate::dev_tools::TestResult>, String> {
    let manager = dev_tools_manager.lock().await;
    manager.run_tests(&config_name).await
}

// Accessibility Commands
#[tauri::command]
pub async fn get_accessibility_config(
    accessibility_manager: State<'_, Arc<Mutex<AccessibilityManager>>>,
) -> Result<crate::accessibility::AccessibilityConfig, String> {
    let manager = accessibility_manager.lock().await;
    Ok(manager.get_config())
}

#[tauri::command]
pub async fn update_accessibility_config(
    accessibility_manager: State<'_, Arc<Mutex<AccessibilityManager>>>,
    config: crate::accessibility::AccessibilityConfig,
) -> Result<(), String> {
    let manager = accessibility_manager.lock().await;
    manager.update_config(config);
    Ok(())
}

#[tauri::command]
pub async fn toggle_high_contrast(
    accessibility_manager: State<'_, Arc<Mutex<AccessibilityManager>>>,
) -> Result<bool, String> {
    let manager = accessibility_manager.lock().await;
    Ok(manager.toggle_high_contrast())
}

#[tauri::command]
pub async fn set_magnification(
    accessibility_manager: State<'_, Arc<Mutex<AccessibilityManager>>>,
    level: f32,
) -> Result<(), String> {
    let manager = accessibility_manager.lock().await;
    manager.set_magnification(level);
    Ok(())
}

#[tauri::command]
pub async fn announce(
    accessibility_manager: State<'_, Arc<Mutex<AccessibilityManager>>>,
    message: String,
    priority: crate::accessibility::AnnouncementPriority,
    interrupt: bool,
) -> Result<(), String> {
    let manager = accessibility_manager.lock().await;
    manager.announce(&message, priority, interrupt);
    Ok(())
}

#[tauri::command]
pub async fn get_keyboard_shortcuts(
    accessibility_manager: State<'_, Arc<Mutex<AccessibilityManager>>>,
    context: Option<crate::accessibility::ShortcutContext>,
) -> Result<Vec<crate::accessibility::KeyboardShortcut>, String> {
    let manager = accessibility_manager.lock().await;
    Ok(manager.get_shortcuts(context))
}

// Internationalization Commands
#[tauri::command]
pub async fn get_i18n_config(
    i18n_manager: State<'_, Arc<Mutex<I18nManager>>>,
) -> Result<crate::accessibility::I18nConfig, String> {
    let manager = i18n_manager.lock().await;
    Ok(manager.get_config())
}

#[tauri::command]
pub async fn set_locale(
    i18n_manager: State<'_, Arc<Mutex<I18nManager>>>,
    locale: String,
) -> Result<(), String> {
    let manager = i18n_manager.lock().await;
    manager.set_locale(&locale)
}

#[tauri::command]
pub async fn translate(
    i18n_manager: State<'_, Arc<Mutex<I18nManager>>>,
    key: String,
    interpolations: Option<HashMap<String, String>>,
) -> Result<String, String> {
    let manager = i18n_manager.lock().await;
    Ok(manager.translate(&key, interpolations))
}

#[tauri::command]
pub async fn format_currency(
    i18n_manager: State<'_, Arc<Mutex<I18nManager>>>,
    amount: f64,
) -> Result<String, String> {
    let manager = i18n_manager.lock().await;
    Ok(manager.format_currency(amount))
}

// Advanced Terminal Commands
#[tauri::command]
pub async fn create_terminal_session(
    terminal_manager: State<'_, Arc<Mutex<AdvancedTerminalManager>>>,
    name: Option<String>,
    template_id: Option<String>,
) -> Result<String, String> {
    let manager = terminal_manager.lock().await;
    manager.create_session(name, template_id)
}

#[tauri::command]
pub async fn get_terminal_session(
    terminal_manager: State<'_, Arc<Mutex<AdvancedTerminalManager>>>,
    session_id: String,
) -> Result<Option<crate::advanced_terminal::TerminalSession>, String> {
    let manager = terminal_manager.lock().await;
    Ok(manager.get_session(&session_id))
}

#[tauri::command]
pub async fn get_all_terminal_sessions(
    terminal_manager: State<'_, Arc<Mutex<AdvancedTerminalManager>>>,
) -> Result<Vec<crate::advanced_terminal::TerminalSession>, String> {
    let manager = terminal_manager.lock().await;
    Ok(manager.get_all_sessions())
}

#[tauri::command]
pub async fn split_pane(
    terminal_manager: State<'_, Arc<Mutex<AdvancedTerminalManager>>>,
    session_id: String,
    pane_id: String,
    split_type: crate::advanced_terminal::SplitType,
    ratio: f32,
) -> Result<String, String> {
    let manager = terminal_manager.lock().await;
    manager.split_pane(&session_id, &pane_id, split_type, ratio)
}

#[tauri::command]
pub async fn close_pane(
    terminal_manager: State<'_, Arc<Mutex<AdvancedTerminalManager>>>,
    session_id: String,
    pane_id: String,
) -> Result<(), String> {
    let manager = terminal_manager.lock().await;
    manager.close_pane(&session_id, &pane_id)
}

#[tauri::command]
pub async fn create_terminal_tab(
    terminal_manager: State<'_, Arc<Mutex<AdvancedTerminalManager>>>,
    session_id: String,
    title: Option<String>,
) -> Result<String, String> {
    let manager = terminal_manager.lock().await;
    manager.create_tab(&session_id, title)
}

#[tauri::command]
pub async fn close_terminal_tab(
    terminal_manager: State<'_, Arc<Mutex<AdvancedTerminalManager>>>,
    session_id: String,
    tab_index: usize,
) -> Result<(), String> {
    let manager = terminal_manager.lock().await;
    manager.close_tab(&session_id, tab_index)
}

#[tauri::command]
pub async fn switch_terminal_tab(
    terminal_manager: State<'_, Arc<Mutex<AdvancedTerminalManager>>>,
    session_id: String,
    tab_index: usize,
) -> Result<(), String> {
    let manager = terminal_manager.lock().await;
    manager.switch_tab(&session_id, tab_index)
}

#[tauri::command]
pub async fn create_session_snapshot(
    terminal_manager: State<'_, Arc<Mutex<AdvancedTerminalManager>>>,
    session_id: String,
    name: Option<String>,
    notes: Option<String>,
) -> Result<String, String> {
    let manager = terminal_manager.lock().await;
    manager.create_snapshot(&session_id, name, notes)
}

#[tauri::command]
pub async fn restore_session(
    terminal_manager: State<'_, Arc<Mutex<AdvancedTerminalManager>>>,
    snapshot_id: String,
) -> Result<String, String> {
    let manager = terminal_manager.lock().await;
    manager.restore_session(&snapshot_id)
}

#[tauri::command]
pub async fn get_session_templates(
    terminal_manager: State<'_, Arc<Mutex<AdvancedTerminalManager>>>,
) -> Result<Vec<crate::advanced_terminal::SessionTemplate>, String> {
    let manager = terminal_manager.lock().await;
    Ok(manager.get_templates())
}

#[tauri::command]
pub async fn export_session(
    terminal_manager: State<'_, Arc<Mutex<AdvancedTerminalManager>>>,
    session_id: String,
) -> Result<String, String> {
    let manager = terminal_manager.lock().await;
    manager.export_session(&session_id)
}

#[tauri::command]
pub async fn import_session(
    terminal_manager: State<'_, Arc<Mutex<AdvancedTerminalManager>>>,
    json_data: String,
) -> Result<String, String> {
    let manager = terminal_manager.lock().await;
    manager.import_session(&json_data)
}
