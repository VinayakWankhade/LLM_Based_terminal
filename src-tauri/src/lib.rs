mod pty;
mod ansi;
mod terminal;
mod commands;
mod shell_hooks;
mod search;
mod ai;
mod workflows;
mod settings;
mod telemetry;
mod plugins;

use commands::*;
use terminal::TerminalManager;
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
      
      // Store the terminal manager in app state
      app.manage(terminal_manager_state.clone());

      // Spawn task to handle terminal output using tauri async runtime
      let app_handle = app.handle().clone();
      let terminal_manager_clone = terminal_manager_state.clone();
      
      tauri::async_runtime::spawn(async move {
        let mut output_receiver = output_receiver;
        while let Some(output) = output_receiver.recv().await {
          // Emit terminal output to frontend
          let _ = app_handle.emit("terminal-output", &output);
          
          // Process output in terminal manager
          terminal_manager_clone.lock().await.process_output(output);
        }
      });

      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      create_terminal,
      write_to_terminal,
      resize_terminal,
      close_terminal,
      get_terminal_state,
      get_command_history,
      get_command_suggestions,
      handle_tab_completion,
      is_at_prompt,
      get_current_prompt,
      search_history,
      search_scrollback,
      get_scrollback_context,
      ai_generate_command,
      ai_explain_error,
      ai_suggest_next,
      list_workflows,
      save_workflow,
      delete_workflow,
      preview_workflow_command,
      run_workflow,
      // settings, plugins, telemetry
      get_settings,
      save_user_settings,
      list_plugins,
      record_event
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
