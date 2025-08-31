mod pty;
mod ansi;
mod terminal;
mod terminal_types;
mod commands;
mod shell_hooks;
mod search;
mod ai;
mod workflows;
mod settings;
mod telemetry;
mod plugins;
mod session_manager;
mod performance_monitor;
mod security;
mod execution_context;
mod shell_integration;
mod clipboard_manager;
mod filesystem_manager;
mod process_manager;
mod theme_manager;
mod network_manager;
mod dev_tools;
mod accessibility;
mod advanced_terminal;
mod advanced_commands;

use commands::*;
use advanced_commands::*;
use terminal::TerminalManager;
use session_manager::*;
use performance_monitor::*;
use security::*;
use execution_context::*;
use shell_integration::*;
use clipboard_manager::*;
use filesystem_manager::*;
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::{Manager, Emitter};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .setup(|app| {
      // Install panic hook to crash-log
      crate::telemetry::install_panic_hook();
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }

      // Initialize terminal manager
      let (terminal_manager, output_receiver) = TerminalManager::new();
      let terminal_manager_state = Arc::new(Mutex::new(terminal_manager));
      
      // Initialize additional managers
      let session_manager = Arc::new(Mutex::new(session_manager::SessionManager::new(terminal_manager_state.clone())));
      let (performance_monitor_instance, _alert_receiver) = performance_monitor::PerformanceMonitor::new();
      let performance_monitor = Arc::new(Mutex::new(performance_monitor_instance));
      let security_manager = Arc::new(Mutex::new(security::SecurityManager::new()));
      let execution_context_manager = Arc::new(Mutex::new(execution_context::ExecutionContextState::new()));
      let shell_integration_manager = Arc::new(Mutex::new(shell_integration::ShellIntegrationState::new()));
      let clipboard_manager = Arc::new(Mutex::new(clipboard_manager::ClipboardState::new()));
      let filesystem_manager = Arc::new(Mutex::new(filesystem_manager::FileSystemState::new()));
      let process_manager = Arc::new(Mutex::new(process_manager::ProcessManager::new()));
      let theme_manager = Arc::new(Mutex::new(theme_manager::ThemeManager::new("themes".to_string())));
      let network_manager = Arc::new(Mutex::new(network_manager::NetworkManager::new()));
      let dev_tools_manager = Arc::new(Mutex::new(dev_tools::DevToolsManager::new()));
      let accessibility_manager = Arc::new(Mutex::new(accessibility::AccessibilityManager::new()));
      let i18n_manager = Arc::new(Mutex::new(accessibility::I18nManager::new()));
      let advanced_terminal_manager = Arc::new(Mutex::new(advanced_terminal::AdvancedTerminalManager::new()));
      
      // Store managers in app state
      app.manage(terminal_manager_state.clone());
      app.manage(session_manager);
      app.manage(performance_monitor);
      app.manage(security_manager);
      app.manage(execution_context_manager);
      app.manage(shell_integration_manager);
      app.manage(clipboard_manager);
      app.manage(filesystem_manager);
      app.manage(process_manager);
      app.manage(theme_manager);
      app.manage(network_manager);
      app.manage(dev_tools_manager);
      app.manage(accessibility_manager);
      app.manage(i18n_manager);
      app.manage(advanced_terminal_manager);

      // Spawn task to handle terminal output using tauri async runtime
      let app_handle = app.handle().clone();
      let terminal_manager_clone = terminal_manager_state.clone();
      
      tauri::async_runtime::spawn(async move {
        let mut output_receiver = output_receiver;
        while let Some(output) = output_receiver.recv().await {
          // Emit terminal output to frontend
          let _ = app_handle.emit("terminal-output", &output);
          
          // Process output in terminal manager
          // For now, skip processing output since we need to handle async properly
          // TODO: Refactor output processing to be async-compatible
        }
      });

      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      // Core terminal endpoints
      create_terminal,
      write_to_terminal,
      resize_terminal,
      close_terminal,
      get_terminal_state,
      // Shell integration endpoints
      get_command_history,
      get_command_suggestions,
      handle_tab_completion,
      is_at_prompt,
      get_current_prompt,
      search_history,
      search_scrollback,
      get_scrollback_context,
      // AI endpoints
      ai_generate_command,
      ai_explain_error,
      ai_suggest_next,
      // Workflow endpoints
      list_workflows,
      save_workflow,
      delete_workflow,
      preview_workflow_command,
      run_workflow,
      // Session management endpoints
      create_session,
      list_sessions,
      attach_session,
      detach_session,
      kill_session,
      // Performance monitoring endpoints
      get_performance_metrics,
      get_command_performance_history,
      get_system_resources,
      get_performance_alerts,
      set_performance_thresholds,
      toggle_performance_monitoring,
      // Security endpoints
      validate_command,
      get_security_alerts,
      get_audit_logs,
      update_security_policy,
      lock_session,
      unlock_session,
      get_session_security_info,
      // Settings, plugins, telemetry
      get_settings,
      save_user_settings,
      list_plugins,
      record_event,
      // Execution context commands
      get_execution_context,
      create_execution_context,
      refresh_execution_context,
      update_selected_text,
      add_directory_bookmark,
      get_directory_bookmarks,
      update_current_directory,
      // Shell integration commands
      get_shell_completions,
      add_command_to_history,
      search_command_history,
      add_shell_alias,
      get_shell_aliases,
      get_git_status,
      create_shell_script,
      get_shell_scripts,
      generate_custom_prompt,
      // Clipboard management commands
      create_text_selection,
      copy_to_clipboard,
      paste_from_clipboard,
      search_clipboard_history,
      get_clipboard_history,
      create_multi_selection,
      get_multi_selections,
      toggle_clipboard_favorite,
      delete_clipboard_entry,
      clear_clipboard_history,
      get_selection_by_id,
      copy_selection_to_clipboard,
      // File system commands
      list_directory,
      get_file_info,
      get_path_completions,
      search_files,
      create_file_operation,
      start_file_operation,
      get_file_operations,
      create_file_watcher,
      get_recent_paths,
      add_path_bookmark,
      get_path_bookmarks,
      // Process management commands
      start_process_monitoring,
      stop_process_monitoring,
      get_processes,
      execute_process_action,
      create_job,
      get_jobs,
      kill_job,
      // Theme management commands
      get_all_themes,
      get_current_theme,
      set_current_theme,
      add_theme,
      get_css_variables,
      export_theme,
      import_theme,
      // Network management commands
      add_ssh_connection,
      get_ssh_connections,
      connect_ssh,
      disconnect_ssh,
      scan_ports,
      get_network_stats,
      // Developer tools commands
      discover_git_repositories,
      load_git_repository,
      git_commit,
      git_push,
      git_pull,
      run_build,
      run_tests,
      // Accessibility commands
      get_accessibility_config,
      update_accessibility_config,
      toggle_high_contrast,
      set_magnification,
      announce,
      get_keyboard_shortcuts,
      // Internationalization commands
      get_i18n_config,
      set_locale,
      translate,
      format_currency,
      // Advanced terminal commands
      create_terminal_session,
      get_terminal_session,
      get_all_terminal_sessions,
      split_pane,
      close_pane,
      create_terminal_tab,
      close_terminal_tab,
      switch_terminal_tab,
      create_session_snapshot,
      restore_session,
      get_session_templates,
      export_session,
      import_session
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
