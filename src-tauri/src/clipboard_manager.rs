use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use tauri::State;
use std::sync::{Arc, Mutex};
use arboard::Clipboard;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextSelection {
    pub id: String,
    pub session_id: String,
    pub start_line: usize,
    pub start_col: usize,
    pub end_line: usize,
    pub end_col: usize,
    pub text: String,
    pub selection_type: SelectionType,
    pub created_at: DateTime<Utc>,
    pub metadata: SelectionMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SelectionType {
    Character,  // Normal text selection
    Word,       // Word-based selection
    Line,       // Line-based selection
    Block,      // Block/rectangular selection
    Stream,     // Streaming selection
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionMetadata {
    pub source: String,
    pub command_context: Option<String>,
    pub file_path: Option<String>,
    pub is_sensitive: bool,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardEntry {
    pub id: String,
    pub content: String,
    pub content_type: ClipboardContentType,
    pub timestamp: DateTime<Utc>,
    pub source: ClipboardSource,
    pub size_bytes: usize,
    pub preview: String,
    pub metadata: ClipboardMetadata,
    pub favorite: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ClipboardContentType {
    PlainText,
    RichText,
    Html,
    Image,
    File,
    Command,
    Output,
    Code,
    Url,
    Path,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ClipboardSource {
    Terminal,
    System,
    Manual,
    Command,
    File,
    Selection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardMetadata {
    pub language: Option<String>,
    pub file_extension: Option<String>,
    pub mime_type: Option<String>,
    pub encoding: String,
    pub line_count: usize,
    pub char_count: usize,
    pub word_count: usize,
    pub is_multiline: bool,
    pub contains_ansi: bool,
    pub security_level: SecurityLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SecurityLevel {
    Public,     // Safe to share
    Internal,   // Internal use only
    Private,    // Contains sensitive data
    Secret,     // Contains secrets/passwords
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiSelection {
    pub id: String,
    pub session_id: String,
    pub selections: Vec<TextSelection>,
    pub combined_text: String,
    pub created_at: DateTime<Utc>,
    pub selection_mode: MultiSelectionMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MultiSelectionMode {
    Sequential,  // Selections in order
    Block,       // Block selections
    Scattered,   // Non-contiguous selections
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardHistory {
    pub entries: Vec<ClipboardEntry>,
    pub max_entries: usize,
    pub max_size_mb: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardFilter {
    pub content_types: Vec<ClipboardContentType>,
    pub sources: Vec<ClipboardSource>,
    pub date_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    pub search_query: Option<String>,
    pub security_levels: Vec<SecurityLevel>,
    pub tags: Vec<String>,
    pub favorites_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardSync {
    pub enabled: bool,
    pub sync_to_system: bool,
    pub sync_from_system: bool,
    pub auto_detect_content_type: bool,
    pub preserve_formatting: bool,
    pub sanitize_content: bool,
}

pub type ClipboardManager = Arc<Mutex<ClipboardState>>;

pub struct ClipboardState {
    pub selections: HashMap<String, TextSelection>,
    pub multi_selections: HashMap<String, MultiSelection>,
    pub clipboard_history: ClipboardHistory,
    pub system_clipboard: Option<Clipboard>,
    pub sync_settings: ClipboardSync,
    pub content_filters: Vec<String>, // Regex patterns for content filtering
}

impl ClipboardState {
    pub fn new() -> Self {
        let system_clipboard = Clipboard::new().ok();
        
        Self {
            selections: HashMap::new(),
            multi_selections: HashMap::new(),
            clipboard_history: ClipboardHistory {
                entries: Vec::new(),
                max_entries: 1000,
                max_size_mb: 100,
            },
            system_clipboard,
            sync_settings: ClipboardSync {
                enabled: true,
                sync_to_system: true,
                sync_from_system: true,
                auto_detect_content_type: true,
                preserve_formatting: true,
                sanitize_content: true,
            },
            content_filters: vec![
                r"password\s*[:=]\s*\S+".to_string(),
                r"api[_-]?key\s*[:=]\s*\S+".to_string(),
                r"secret\s*[:=]\s*\S+".to_string(),
                r"token\s*[:=]\s*\S+".to_string(),
            ],
        }
    }

    pub fn create_selection(
        &mut self,
        session_id: String,
        start_line: usize,
        start_col: usize,
        end_line: usize,
        end_col: usize,
        text: String,
        selection_type: SelectionType,
        metadata: SelectionMetadata,
    ) -> String {
        let selection_id = uuid::Uuid::new_v4().to_string();
        let selection = TextSelection {
            id: selection_id.clone(),
            session_id,
            start_line,
            start_col,
            end_line,
            end_col,
            text,
            selection_type,
            created_at: Utc::now(),
            metadata,
        };

        self.selections.insert(selection_id.clone(), selection);
        selection_id
    }

    pub fn add_to_clipboard(&mut self, content: String, content_type: ClipboardContentType, source: ClipboardSource) -> Result<String, String> {
        let entry_id = uuid::Uuid::new_v4().to_string();
        let sanitized_content = if self.sync_settings.sanitize_content {
            self.sanitize_content(&content)
        } else {
            content.clone()
        };

        let metadata = self.analyze_content(&sanitized_content, &content_type);
        let preview = self.generate_preview(&sanitized_content, 100);

        let entry = ClipboardEntry {
            id: entry_id.clone(),
            content: sanitized_content.clone(),
            content_type,
            timestamp: Utc::now(),
            source,
            size_bytes: sanitized_content.len(),
            preview,
            metadata,
            favorite: false,
        };

        // Check size limits
        let total_size: usize = self.clipboard_history.entries.iter().map(|e| e.size_bytes).sum();
        if total_size + entry.size_bytes > self.clipboard_history.max_size_mb * 1024 * 1024 {
            self.cleanup_old_entries();
        }

        // Add to history
        self.clipboard_history.entries.insert(0, entry);
        if self.clipboard_history.entries.len() > self.clipboard_history.max_entries {
            self.clipboard_history.entries.truncate(self.clipboard_history.max_entries);
        }

        // Sync to system clipboard if enabled
        if self.sync_settings.sync_to_system && self.sync_settings.enabled {
            if let Some(clipboard) = &mut self.system_clipboard {
                let _ = clipboard.set_text(&sanitized_content);
            }
        }

        Ok(entry_id)
    }

    pub fn get_from_clipboard(&mut self) -> Result<Option<String>, String> {
        if !self.sync_settings.sync_from_system || !self.sync_settings.enabled {
            return Ok(self.clipboard_history.entries.first().map(|e| e.content.clone()));
        }

        if let Some(clipboard) = &mut self.system_clipboard {
            match clipboard.get_text() {
                Ok(content) => {
                    // Check if this is new content
                    if let Some(last_entry) = self.clipboard_history.entries.first() {
                        if last_entry.content != content {
                            let _ = self.add_to_clipboard(content.clone(), ClipboardContentType::PlainText, ClipboardSource::System);
                        }
                    } else {
                        let _ = self.add_to_clipboard(content.clone(), ClipboardContentType::PlainText, ClipboardSource::System);
                    }
                    Ok(Some(content))
                }
                Err(_) => Ok(None),
            }
        } else {
            Ok(self.clipboard_history.entries.first().map(|e| e.content.clone()))
        }
    }

    pub fn search_clipboard(&self, filter: &ClipboardFilter) -> Vec<ClipboardEntry> {
        let mut results: Vec<ClipboardEntry> = self.clipboard_history.entries
            .iter()
            .filter(|entry| {
                // Filter by content type
                if !filter.content_types.is_empty() && !filter.content_types.contains(&entry.content_type) {
                    return false;
                }

                // Filter by source
                if !filter.sources.is_empty() && !filter.sources.contains(&entry.source) {
                    return false;
                }

                // Filter by security level
                if !filter.security_levels.is_empty() && !filter.security_levels.contains(&entry.metadata.security_level) {
                    return false;
                }

                // Filter by date range
                if let Some((start, end)) = &filter.date_range {
                    if entry.timestamp < *start || entry.timestamp > *end {
                        return false;
                    }
                }

                // Filter by search query
                if let Some(query) = &filter.search_query {
                    let query_lower = query.to_lowercase();
                    if !entry.content.to_lowercase().contains(&query_lower) &&
                       !entry.preview.to_lowercase().contains(&query_lower) {
                        return false;
                    }
                }

                // Filter favorites only
                if filter.favorites_only && !entry.favorite {
                    return false;
                }

                true
            })
            .cloned()
            .collect();

        results.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        results
    }

    pub fn create_multi_selection(
        &mut self,
        session_id: String,
        selection_ids: Vec<String>,
        mode: MultiSelectionMode,
    ) -> Result<String, String> {
        let multi_id = uuid::Uuid::new_v4().to_string();
        let mut selections = Vec::new();
        let mut combined_text = String::new();

        for selection_id in selection_ids {
            if let Some(selection) = self.selections.get(&selection_id) {
                selections.push(selection.clone());
                if !combined_text.is_empty() {
                    combined_text.push('\n');
                }
                combined_text.push_str(&selection.text);
            } else {
                return Err(format!("Selection not found: {}", selection_id));
            }
        }

        let multi_selection = MultiSelection {
            id: multi_id.clone(),
            session_id,
            selections,
            combined_text,
            created_at: Utc::now(),
            selection_mode: mode,
        };

        self.multi_selections.insert(multi_id.clone(), multi_selection);
        Ok(multi_id)
    }

    fn sanitize_content(&self, content: &str) -> String {
        let mut sanitized = content.to_string();
        
        // Apply content filters to redact sensitive information
        for pattern in &self.content_filters {
            if let Ok(regex) = regex::Regex::new(pattern) {
                sanitized = regex.replace_all(&sanitized, "[REDACTED]").to_string();
            }
        }

        sanitized
    }

    fn analyze_content(&self, content: &str, content_type: &ClipboardContentType) -> ClipboardMetadata {
        let lines: Vec<&str> = content.lines().collect();
        let line_count = lines.len();
        let char_count = content.chars().count();
        let word_count = content.split_whitespace().count();
        let is_multiline = line_count > 1;
        let contains_ansi = content.contains('\x1b');

        // Detect security level
        let security_level = if self.content_filters.iter().any(|pattern| {
            regex::Regex::new(pattern).map(|r| r.is_match(content)).unwrap_or(false)
        }) {
            SecurityLevel::Secret
        } else if content.to_lowercase().contains("password") || 
                  content.to_lowercase().contains("secret") ||
                  content.to_lowercase().contains("token") {
            SecurityLevel::Private
        } else {
            SecurityLevel::Public
        };

        // Detect language/syntax
        let language = match content_type {
            ClipboardContentType::Code => self.detect_language(content),
            _ => None,
        };

        ClipboardMetadata {
            language,
            file_extension: None,
            mime_type: self.detect_mime_type(content_type),
            encoding: "utf-8".to_string(),
            line_count,
            char_count,
            word_count,
            is_multiline,
            contains_ansi,
            security_level,
        }
    }

    fn detect_language(&self, content: &str) -> Option<String> {
        // Simple language detection based on common patterns
        if content.contains("#!/bin/bash") || content.contains("#!/bin/sh") {
            Some("bash".to_string())
        } else if content.contains("#!/usr/bin/env python") || content.contains("import ") {
            Some("python".to_string())
        } else if content.contains("function ") && content.contains("return") {
            Some("javascript".to_string())
        } else if content.contains("pub fn ") || content.contains("fn main()") {
            Some("rust".to_string())
        } else if content.contains("#include") || content.contains("int main(") {
            Some("c".to_string())
        } else {
            None
        }
    }

    fn detect_mime_type(&self, content_type: &ClipboardContentType) -> Option<String> {
        match content_type {
            ClipboardContentType::PlainText => Some("text/plain".to_string()),
            ClipboardContentType::Html => Some("text/html".to_string()),
            ClipboardContentType::Code => Some("text/plain".to_string()),
            ClipboardContentType::Url => Some("text/uri-list".to_string()),
            _ => None,
        }
    }

    fn generate_preview(&self, content: &str, max_length: usize) -> String {
        let cleaned = content.replace('\n', " ").replace('\r', " ");
        if cleaned.len() <= max_length {
            cleaned
        } else {
            format!("{}...", &cleaned[..max_length])
        }
    }

    fn cleanup_old_entries(&mut self) {
        // Remove non-favorite entries older than 30 days
        let cutoff = Utc::now() - chrono::Duration::days(30);
        self.clipboard_history.entries.retain(|entry| {
            entry.favorite || entry.timestamp > cutoff
        });
    }

    pub fn toggle_favorite(&mut self, entry_id: &str) -> Result<bool, String> {
        if let Some(entry) = self.clipboard_history.entries.iter_mut().find(|e| e.id == entry_id) {
            entry.favorite = !entry.favorite;
            Ok(entry.favorite)
        } else {
            Err("Entry not found".to_string())
        }
    }

    pub fn delete_entry(&mut self, entry_id: &str) -> Result<(), String> {
        let initial_len = self.clipboard_history.entries.len();
        self.clipboard_history.entries.retain(|e| e.id != entry_id);
        
        if self.clipboard_history.entries.len() < initial_len {
            Ok(())
        } else {
            Err("Entry not found".to_string())
        }
    }

    pub fn clear_clipboard(&mut self, keep_favorites: bool) {
        if keep_favorites {
            self.clipboard_history.entries.retain(|e| e.favorite);
        } else {
            self.clipboard_history.entries.clear();
        }
    }
}

// Tauri commands
#[tauri::command]
pub async fn create_text_selection(
    session_id: String,
    start_line: usize,
    start_col: usize,
    end_line: usize,
    end_col: usize,
    text: String,
    selection_type: SelectionType,
    metadata: SelectionMetadata,
    clipboard_manager: State<'_, ClipboardManager>,
) -> Result<String, String> {
    let mut manager = clipboard_manager.lock().map_err(|e| e.to_string())?;
    Ok(manager.create_selection(session_id, start_line, start_col, end_line, end_col, text, selection_type, metadata))
}

#[tauri::command]
pub async fn copy_to_clipboard(
    content: String,
    content_type: ClipboardContentType,
    source: ClipboardSource,
    clipboard_manager: State<'_, ClipboardManager>,
) -> Result<String, String> {
    let mut manager = clipboard_manager.lock().map_err(|e| e.to_string())?;
    manager.add_to_clipboard(content, content_type, source)
}

#[tauri::command]
pub async fn paste_from_clipboard(
    clipboard_manager: State<'_, ClipboardManager>,
) -> Result<Option<String>, String> {
    let mut manager = clipboard_manager.lock().map_err(|e| e.to_string())?;
    manager.get_from_clipboard()
}

#[tauri::command]
pub async fn search_clipboard_history(
    filter: ClipboardFilter,
    clipboard_manager: State<'_, ClipboardManager>,
) -> Result<Vec<ClipboardEntry>, String> {
    let manager = clipboard_manager.lock().map_err(|e| e.to_string())?;
    Ok(manager.search_clipboard(&filter))
}

#[tauri::command]
pub async fn get_clipboard_history(
    limit: Option<usize>,
    clipboard_manager: State<'_, ClipboardManager>,
) -> Result<Vec<ClipboardEntry>, String> {
    let manager = clipboard_manager.lock().map_err(|e| e.to_string())?;
    let mut entries = manager.clipboard_history.entries.clone();
    if let Some(limit) = limit {
        entries.truncate(limit);
    }
    Ok(entries)
}

#[tauri::command]
pub async fn create_multi_selection(
    session_id: String,
    selection_ids: Vec<String>,
    mode: MultiSelectionMode,
    clipboard_manager: State<'_, ClipboardManager>,
) -> Result<String, String> {
    let mut manager = clipboard_manager.lock().map_err(|e| e.to_string())?;
    manager.create_multi_selection(session_id, selection_ids, mode)
}

#[tauri::command]
pub async fn get_multi_selections(
    session_id: String,
    clipboard_manager: State<'_, ClipboardManager>,
) -> Result<Vec<MultiSelection>, String> {
    let manager = clipboard_manager.lock().map_err(|e| e.to_string())?;
    Ok(manager.multi_selections.values()
        .filter(|ms| ms.session_id == session_id)
        .cloned()
        .collect())
}

#[tauri::command]
pub async fn toggle_clipboard_favorite(
    entry_id: String,
    clipboard_manager: State<'_, ClipboardManager>,
) -> Result<bool, String> {
    let mut manager = clipboard_manager.lock().map_err(|e| e.to_string())?;
    manager.toggle_favorite(&entry_id)
}

#[tauri::command]
pub async fn delete_clipboard_entry(
    entry_id: String,
    clipboard_manager: State<'_, ClipboardManager>,
) -> Result<(), String> {
    let mut manager = clipboard_manager.lock().map_err(|e| e.to_string())?;
    manager.delete_entry(&entry_id)
}

#[tauri::command]
pub async fn clear_clipboard_history(
    keep_favorites: bool,
    clipboard_manager: State<'_, ClipboardManager>,
) -> Result<(), String> {
    let mut manager = clipboard_manager.lock().map_err(|e| e.to_string())?;
    manager.clear_clipboard(keep_favorites);
    Ok(())
}

#[tauri::command]
pub async fn get_selection_by_id(
    selection_id: String,
    clipboard_manager: State<'_, ClipboardManager>,
) -> Result<Option<TextSelection>, String> {
    let manager = clipboard_manager.lock().map_err(|e| e.to_string())?;
    Ok(manager.selections.get(&selection_id).cloned())
}

#[tauri::command]
pub async fn copy_selection_to_clipboard(
    selection_id: String,
    clipboard_manager: State<'_, ClipboardManager>,
) -> Result<String, String> {
    let mut manager = clipboard_manager.lock().map_err(|e| e.to_string())?;
    
    if let Some(selection) = manager.selections.get(&selection_id) {
        let content = selection.text.clone();
        manager.add_to_clipboard(content, ClipboardContentType::PlainText, ClipboardSource::Selection)
    } else {
        Err("Selection not found".to_string())
    }
}
