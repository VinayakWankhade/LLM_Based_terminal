use crate::terminal::{TerminalManager, TerminalGrid};
use crate::pty::TerminalSize;
use crate::shell_hooks::{Command, CommandSuggestion, PromptInfo};
use crate::search::{ScrollMatch, ContextLine};
use crate::ai::{AiClient, AiRequest};
use crate::workflows;
use crate::settings::{Settings, load_settings, save_settings};
use crate::plugins;
use crate::telemetry;
use tauri::State;
use std::sync::Arc;
use tokio::sync::Mutex;

pub type TerminalManagerState = Arc<Mutex<TerminalManager>>;

#[tauri::command]
pub async fn create_terminal(
    cols: u16,
    rows: u16,
    shell: Option<String>,
    working_dir: Option<String>,
    terminal_manager: State<'_, TerminalManagerState>,
) -> Result<String, String> {
    let size = TerminalSize {
        cols,
        rows,
        pixel_width: 0, // Will be calculated on frontend
        pixel_height: 0,
    };

    terminal_manager
        .lock()
        .await
        .create_terminal(size, shell, working_dir)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn write_to_terminal(
    terminal_id: String,
    data: String,
    terminal_manager: State<'_, TerminalManagerState>,
) -> Result<(), String> {
    terminal_manager
        .lock()
        .await
        .write_to_terminal(&terminal_id, &data)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn resize_terminal(
    terminal_id: String,
    cols: u16,
    rows: u16,
    terminal_manager: State<'_, TerminalManagerState>,
) -> Result<(), String> {
    let size = TerminalSize {
        cols,
        rows,
        pixel_width: 0,
        pixel_height: 0,
    };

    terminal_manager
        .lock()
        .await
        .resize_terminal(&terminal_id, size)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn close_terminal(
    terminal_id: String,
    terminal_manager: State<'_, TerminalManagerState>,
) -> Result<(), String> {
    terminal_manager
        .lock()
        .await
        .close_terminal(&terminal_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_terminal_state(
    terminal_id: String,
    terminal_manager: State<'_, TerminalManagerState>,
) -> Result<Option<TerminalGrid>, String> {
    Ok(terminal_manager
        .lock()
        .await
        .get_terminal_state(&terminal_id))
}

// Shell integration commands
#[tauri::command]
pub async fn get_command_history(
    terminal_id: String,
    limit: Option<usize>,
    terminal_manager: State<'_, TerminalManagerState>,
) -> Result<Vec<Command>, String> {
    Ok(terminal_manager
        .lock()
        .await
        .get_command_history(&terminal_id, limit)
        .unwrap_or_default())
}

#[tauri::command]
pub async fn get_scrollback_context(
    terminal_id: String,
    line_index: usize,
    before: Option<usize>,
    after: Option<usize>,
    terminal_manager: State<'_, TerminalManagerState>,
) -> Result<Vec<ContextLine>, String> {
    Ok(terminal_manager
        .lock()
        .await
        .get_scrollback_context(&terminal_id, line_index, before.unwrap_or(3), after.unwrap_or(3))
        .unwrap_or_default())
}

#[tauri::command]
pub async fn get_command_suggestions(
    terminal_id: String,
    partial_command: String,
    terminal_manager: State<'_, TerminalManagerState>,
) -> Result<Vec<CommandSuggestion>, String> {
    Ok(terminal_manager
        .lock()
        .await
        .get_command_suggestions(&terminal_id, &partial_command)
        .unwrap_or_default())
}

#[tauri::command]
pub async fn handle_tab_completion(
    terminal_id: String,
    current_line: String,
    cursor_pos: usize,
    terminal_manager: State<'_, TerminalManagerState>,
) -> Result<Vec<String>, String> {
    Ok(terminal_manager
        .lock()
        .await
        .handle_tab_completion(&terminal_id, &current_line, cursor_pos)
        .unwrap_or_default())
}

#[tauri::command]
pub async fn is_at_prompt(
    terminal_id: String,
    terminal_manager: State<'_, TerminalManagerState>,
) -> Result<bool, String> {
    Ok(terminal_manager
        .lock()
        .await
        .is_at_prompt(&terminal_id))
}

#[tauri::command]
pub async fn get_current_prompt(
    terminal_id: String,
    terminal_manager: State<'_, TerminalManagerState>,
) -> Result<Option<PromptInfo>, String> {
    Ok(terminal_manager
        .lock()
        .await
        .get_current_prompt(&terminal_id))
}

#[tauri::command]
pub async fn search_history(
    terminal_id: String,
    query: String,
    terminal_manager: State<'_, TerminalManagerState>,
) -> Result<Vec<Command>, String> {
    Ok(terminal_manager
        .lock()
        .await
        .search_history(&terminal_id, &query)
        .unwrap_or_default())
}

#[tauri::command]
pub async fn search_scrollback(
    terminal_id: String,
    query: String,
    case_sensitive: Option<bool>,
    use_regex: Option<bool>,
    limit: Option<usize>,
    terminal_manager: State<'_, TerminalManagerState>,
) -> Result<Vec<ScrollMatch>, String> {
    Ok(terminal_manager
        .lock()
        .await
        .search_scrollback(&terminal_id, &query, case_sensitive.unwrap_or(false), use_regex.unwrap_or(false), limit.unwrap_or(200))
        .unwrap_or_default())
}

// Settings endpoints
#[tauri::command]
pub async fn get_settings() -> Result<Settings, String> { load_settings() }
#[tauri::command]
pub async fn save_user_settings(settings: Settings) -> Result<(), String> { save_settings(&settings) }

// Plugins
#[tauri::command]
pub async fn list_plugins() -> Result<Vec<plugins::PluginManifest>, String> { Ok(plugins::list_plugins()) }

// Telemetry
#[tauri::command]
pub async fn record_event(kind: String, data: serde_json::Value) { telemetry::record(&kind, data); }

// Workflow endpoints
#[tauri::command]
pub async fn list_workflows() -> Result<Vec<workflows::Workflow>, String> {
    workflows::load_all()
}

#[tauri::command]
pub async fn save_workflow(workflow: workflows::Workflow) -> Result<workflows::Workflow, String> {
    workflows::upsert(workflow)
}

#[tauri::command]
pub async fn delete_workflow(id: String) -> Result<(), String> {
    workflows::delete(&id)
}

#[tauri::command]
pub async fn preview_workflow_command(workflow_id: String, values: std::collections::HashMap<String, String>) -> Result<String, String> {
    let wf = workflows::get(&workflow_id)?;
    Ok(workflows::render_command(&wf.command, &values))
}

#[tauri::command]
pub async fn run_workflow(terminal_id: String, workflow_id: String, values: std::collections::HashMap<String, String>, terminal_manager: State<'_, TerminalManagerState>) -> Result<(), String> {
    let wf = workflows::get(&workflow_id)?;
    let cmd = workflows::render_command(&wf.command, &values) + "\r";
    terminal_manager.lock().await.write_to_terminal(&terminal_id, &cmd).map_err(|e| e.to_string())
}

// AI endpoints
#[tauri::command]
pub async fn ai_generate_command(
    terminal_id: Option<String>,
    user_input: String,
    terminal_manager: State<'_, TerminalManagerState>,
) -> Result<String, String> {
    let ctx = if let Some(id) = &terminal_id {
        terminal_manager.lock().await.gather_context(id).unwrap_or_else(|| crate::ai::AiContext { working_dir: None, prompt: None, recent_commands: vec![], tail_output: vec![] })
    } else {
        crate::ai::AiContext { working_dir: None, prompt: None, recent_commands: vec![], tail_output: vec![] }
    };
    let client = AiClient::from_env();
    let req = AiRequest { task: "generate_command".into(), user_input, context: ctx };
    client.generate(req).await.map(|r| r.text).map_err(|e| e)
}

#[tauri::command]
pub async fn ai_explain_error(
    terminal_id: Option<String>,
    error_text: Option<String>,
    terminal_manager: State<'_, TerminalManagerState>,
) -> Result<String, String> {
    let ctx = if let Some(id) = &terminal_id { terminal_manager.lock().await.gather_context(id).unwrap_or_else(|| crate::ai::AiContext { working_dir: None, prompt: None, recent_commands: vec![], tail_output: vec![] }) } else { crate::ai::AiContext { working_dir: None, prompt: None, recent_commands: vec![], tail_output: vec![] } };
    // If no error text provided, try to synthesize from tail
    let text = error_text.unwrap_or_else(|| ctx.tail_output.join("\n"));
    let client = AiClient::from_env();
    let req = AiRequest { task: "explain_error".into(), user_input: text, context: ctx };
    client.generate(req).await.map(|r| r.text).map_err(|e| e)
}

#[tauri::command]
pub async fn ai_suggest_next(
    terminal_id: String,
    terminal_manager: State<'_, TerminalManagerState>,
) -> Result<String, String> {
    let ctx = terminal_manager.lock().await.gather_context(&terminal_id).unwrap_or_else(|| crate::ai::AiContext { working_dir: None, prompt: None, recent_commands: vec![], tail_output: vec![] });
    let client = AiClient::from_env();
    let req = AiRequest { task: "suggest_next".into(), user_input: String::new(), context: ctx };
    client.generate(req).await.map(|r| r.text).map_err(|e| e)
}
