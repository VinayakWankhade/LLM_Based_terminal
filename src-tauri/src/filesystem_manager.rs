use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs::{self, Metadata};
use std::time::SystemTime;
use chrono::{DateTime, Utc};
use tauri::State;
use std::sync::{Arc, Mutex};
use notify::{RecursiveMode, Event, EventKind};
use tokio::sync::broadcast;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSystemEntry {
    pub path: String,
    pub name: String,
    pub file_type: EntryType,
    pub size: u64,
    pub permissions: FilePermissions,
    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
    pub accessed: DateTime<Utc>,
    pub is_hidden: bool,
    pub is_symlink: bool,
    pub symlink_target: Option<String>,
    pub mime_type: Option<String>,
    pub extension: Option<String>,
    pub metadata: FileMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EntryType {
    File,
    Directory,
    Symlink,
    Pipe,
    Socket,
    BlockDevice,
    CharDevice,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilePermissions {
    pub readable: bool,
    pub writable: bool,
    pub executable: bool,
    pub owner: String,
    pub group: String,
    pub mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub line_count: Option<usize>,
    pub encoding: Option<String>,
    pub language: Option<String>,
    pub is_binary: bool,
    pub is_executable: bool,
    pub is_archive: bool,
    pub is_image: bool,
    pub is_video: bool,
    pub is_audio: bool,
    pub checksum: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryListing {
    pub path: String,
    pub entries: Vec<FileSystemEntry>,
    pub total_size: u64,
    pub total_count: usize,
    pub directory_count: usize,
    pub file_count: usize,
    pub hidden_count: usize,
    pub sort_by: SortBy,
    pub sort_order: SortOrder,
    pub show_hidden: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortBy {
    Name,
    Size,
    Modified,
    Created,
    Type,
    Extension,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortOrder {
    Ascending,
    Descending,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileOperation {
    pub id: String,
    pub operation_type: OperationType,
    pub source: Vec<String>,
    pub destination: Option<String>,
    pub status: OperationStatus,
    pub progress: f64,
    pub bytes_processed: u64,
    pub total_bytes: u64,
    pub files_processed: usize,
    pub total_files: usize,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
    pub can_resume: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationType {
    Copy,
    Move,
    Delete,
    Archive,
    Extract,
    Compress,
    Encrypt,
    Decrypt,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationStatus {
    Pending,
    Running,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileWatcher {
    pub id: String,
    pub path: String,
    pub recursive: bool,
    pub events: Vec<WatchEventType>,
    pub created_at: DateTime<Utc>,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WatchEventType {
    Created,
    Modified,
    Deleted,
    Moved,
    All,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileWatchEvent {
    pub watcher_id: String,
    pub event_type: WatchEventType,
    pub path: String,
    pub old_path: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathCompletion {
    pub path: String,
    pub display: String,
    pub entry_type: EntryType,
    pub is_accessible: bool,
    pub priority: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub path: String,
    pub entry: FileSystemEntry,
    pub score: f64,
    pub matches: Vec<SearchMatch>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchMatch {
    pub match_type: MatchType,
    pub text: String,
    pub line_number: Option<usize>,
    pub column_start: Option<usize>,
    pub column_end: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MatchType {
    FileName,
    FilePath,
    FileContent,
    FileType,
    FileSize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub pattern: String,
    pub search_type: SearchType,
    pub file_types: Vec<String>,
    pub size_range: Option<(u64, u64)>,
    pub date_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    pub include_hidden: bool,
    pub case_sensitive: bool,
    pub use_regex: bool,
    pub max_results: usize,
    pub max_depth: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SearchType {
    Name,
    Content,
    Both,
}

pub type FileSystemManager = Arc<Mutex<FileSystemState>>;

pub struct FileSystemState {
    pub operations: HashMap<String, FileOperation>,
    pub watchers: HashMap<String, FileWatcher>,
    pub watch_tx: Option<broadcast::Sender<FileWatchEvent>>,
    pub recent_paths: Vec<String>,
    pub bookmarks: Vec<PathBookmark>,
    pub quick_access: Vec<QuickAccessEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathBookmark {
    pub name: String,
    pub path: String,
    pub icon: Option<String>,
    pub created_at: DateTime<Utc>,
    pub access_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickAccessEntry {
    pub name: String,
    pub path: String,
    pub entry_type: EntryType,
    pub last_accessed: DateTime<Utc>,
    pub access_count: u64,
}

impl FileSystemState {
    pub fn new() -> Self {
        let (watch_tx, _) = broadcast::channel(1000);
        
        Self {
            operations: HashMap::new(),
            watchers: HashMap::new(),
            watch_tx: Some(watch_tx),
            recent_paths: Vec::new(),
            bookmarks: Vec::new(),
            quick_access: Vec::new(),
        }
    }

    pub fn list_directory(
        &mut self,
        path: &str,
        sort_by: SortBy,
        sort_order: SortOrder,
        show_hidden: bool,
    ) -> Result<DirectoryListing, String> {
        let path_buf = PathBuf::from(path);
        
        if !path_buf.exists() {
            return Err("Directory does not exist".to_string());
        }
        
        if !path_buf.is_dir() {
            return Err("Path is not a directory".to_string());
        }

        let mut entries = Vec::new();
        let mut total_size = 0u64;
        let mut directory_count = 0usize;
        let mut file_count = 0usize;
        let mut hidden_count = 0usize;

        match fs::read_dir(&path_buf) {
            Ok(dir_entries) => {
                for entry in dir_entries {
                    if let Ok(entry) = entry {
                        let entry_path = entry.path();
                        let name = entry.file_name().to_string_lossy().to_string();
                        
                        let is_hidden = name.starts_with('.');
                        if is_hidden {
                            hidden_count += 1;
                            if !show_hidden {
                                continue;
                            }
                        }

                        if let Ok(fs_entry) = self.create_filesystem_entry(&entry_path) {
                            total_size += fs_entry.size;
                            match fs_entry.file_type {
                                EntryType::Directory => directory_count += 1,
                                EntryType::File => file_count += 1,
                                _ => {}
                            }
                            entries.push(fs_entry);
                        }
                    }
                }
            }
            Err(e) => return Err(format!("Failed to read directory: {}", e)),
        }

        // Sort entries
        self.sort_entries(&mut entries, &sort_by, &sort_order);

        // Add to recent paths
        self.add_recent_path(path.to_string());

        Ok(DirectoryListing {
            path: path.to_string(),
            total_count: entries.len(),
            entries,
            total_size,
            directory_count,
            file_count,
            hidden_count,
            sort_by,
            sort_order,
            show_hidden,
        })
    }

    pub fn get_file_info(&self, path: &str) -> Result<FileSystemEntry, String> {
        let path_buf = PathBuf::from(path);
        
        if !path_buf.exists() {
            return Err("File does not exist".to_string());
        }

        self.create_filesystem_entry(&path_buf)
    }

    pub fn create_file_operation(
        &mut self,
        operation_type: OperationType,
        source: Vec<String>,
        destination: Option<String>,
    ) -> String {
        let operation_id = uuid::Uuid::new_v4().to_string();
        
        // Calculate total bytes and files
        let (total_bytes, total_files) = self.calculate_operation_size(&source);

        let operation = FileOperation {
            id: operation_id.clone(),
            operation_type,
            source,
            destination,
            status: OperationStatus::Pending,
            progress: 0.0,
            bytes_processed: 0,
            total_bytes,
            files_processed: 0,
            total_files,
            started_at: Utc::now(),
            completed_at: None,
            error: None,
            can_resume: false,
        };

        self.operations.insert(operation_id.clone(), operation);
        operation_id
    }

    pub fn start_file_operation(&mut self, operation_id: &str) -> Result<(), String> {
        if let Some(operation) = self.operations.get_mut(operation_id) {
            operation.status = OperationStatus::Running;
            operation.started_at = Utc::now();
            // In a real implementation, this would spawn an async task
            Ok(())
        } else {
            Err("Operation not found".to_string())
        }
    }

    pub fn create_watcher(
        &mut self,
        path: String,
        recursive: bool,
        events: Vec<WatchEventType>,
    ) -> Result<String, String> {
        let watcher_id = uuid::Uuid::new_v4().to_string();
        
        let watcher = FileWatcher {
            id: watcher_id.clone(),
            path: path.clone(),
            recursive,
            events,
            created_at: Utc::now(),
            active: true,
        };

        self.watchers.insert(watcher_id.clone(), watcher);
        
        // In a real implementation, this would create an actual file watcher
        // using the notify crate and send events to the broadcast channel
        
        Ok(watcher_id)
    }

    pub fn get_path_completions(&self, partial_path: &str, limit: usize) -> Vec<PathCompletion> {
        let mut completions = Vec::new();
        
        let path_buf = PathBuf::from(partial_path);
        let (directory, prefix) = if partial_path.ends_with('/') || partial_path.ends_with('\\') {
            (path_buf, String::new())
        } else {
            let directory = path_buf.parent().unwrap_or(Path::new(".")).to_path_buf();
            let prefix = path_buf.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            (directory, prefix)
        };

        if let Ok(entries) = fs::read_dir(&directory) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                
                if name.starts_with(&prefix) {
                    let full_path = entry.path();
                    let is_dir = full_path.is_dir();
                    let is_accessible = self.is_accessible(&full_path);
                    
                    let display = if is_dir {
                        format!("{}/", name)
                    } else {
                        name.clone()
                    };

                    completions.push(PathCompletion {
                        path: full_path.to_string_lossy().to_string(),
                        display,
                        entry_type: if is_dir { EntryType::Directory } else { EntryType::File },
                        is_accessible,
                        priority: if is_dir { 100 } else { 50 },
                    });

                    if completions.len() >= limit {
                        break;
                    }
                }
            }
        }

        // Sort by priority and name
        completions.sort_by(|a, b| {
            b.priority.cmp(&a.priority)
                .then_with(|| a.display.cmp(&b.display))
        });

        completions
    }

    pub fn search_files(&self, query: &SearchQuery, base_path: &str) -> Vec<SearchResult> {
        let mut results = Vec::new();
        
        if let Ok(entries) = fs::read_dir(base_path) {
            for entry in entries.flatten() {
                if results.len() >= query.max_results {
                    break;
                }

                let path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();
                
                // Skip hidden files if not requested
                if !query.include_hidden && name.starts_with('.') {
                    continue;
                }

                if let Ok(fs_entry) = self.create_filesystem_entry(&path) {
                    let mut matches = Vec::new();
                    let mut score = 0.0;

                    // Check file name match
                    if matches!(query.search_type, SearchType::Name | SearchType::Both) {
                        if self.matches_pattern(&name, &query.pattern, query.case_sensitive, query.use_regex) {
                            matches.push(SearchMatch {
                                match_type: MatchType::FileName,
                                text: name.clone(),
                                line_number: None,
                                column_start: None,
                                column_end: None,
                            });
                            score += 10.0;
                        }
                    }

                    // Check file content match (for text files)
                    if matches!(query.search_type, SearchType::Content | SearchType::Both) 
                        && fs_entry.file_type == EntryType::File 
                        && !fs_entry.metadata.is_binary {
                        if let Ok(content) = fs::read_to_string(&path) {
                            for (line_num, line) in content.lines().enumerate() {
                                if self.matches_pattern(line, &query.pattern, query.case_sensitive, query.use_regex) {
                                    matches.push(SearchMatch {
                                        match_type: MatchType::FileContent,
                                        text: line.to_string(),
                                        line_number: Some(line_num + 1),
                                        column_start: None,
                                        column_end: None,
                                    });
                                    score += 5.0;
                                    
                                    if matches.len() >= 10 {
                                        break;
                                    }
                                }
                            }
                        }
                    }

                    if !matches.is_empty() {
                        results.push(SearchResult {
                            path: path.to_string_lossy().to_string(),
                            entry: fs_entry,
                            score,
                            matches,
                        });
                    }
                }

                // Recurse into subdirectories
                if path.is_dir() && query.max_depth.map_or(true, |d| d > 0) {
                    let sub_query = SearchQuery {
                        max_depth: query.max_depth.map(|d| d - 1),
                        ..query.clone()
                    };
                    
                    let sub_results = self.search_files(&sub_query, &path.to_string_lossy());
                    results.extend(sub_results);
                }
            }
        }

        // Sort by score descending
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(query.max_results);
        results
    }

    fn create_filesystem_entry(&self, path: &Path) -> Result<FileSystemEntry, String> {
        let metadata = fs::metadata(path).map_err(|e| e.to_string())?;
        let name = path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        let file_type = self.get_entry_type(&metadata);
        let permissions = self.get_permissions(&metadata);
        let is_hidden = name.starts_with('.');
        let extension = path.extension().map(|e| e.to_string_lossy().to_string());
        let mime_type = self.detect_mime_type(&extension);

        let created = metadata.created()
            .map(|t| DateTime::from(t))
            .unwrap_or_else(|_| Utc::now());
        
        let modified = metadata.modified()
            .map(|t| DateTime::from(t))
            .unwrap_or_else(|_| Utc::now());
        
        let accessed = metadata.accessed()
            .map(|t| DateTime::from(t))
            .unwrap_or_else(|_| Utc::now());

        // Handle symlinks
        let (is_symlink, symlink_target) = if path.is_symlink() {
            let target = fs::read_link(path)
                .map(|p| p.to_string_lossy().to_string())
                .ok();
            (true, target)
        } else {
            (false, None)
        };

        let file_metadata = self.analyze_file_metadata(path, &file_type, &extension);

        Ok(FileSystemEntry {
            path: path.to_string_lossy().to_string(),
            name,
            file_type,
            size: metadata.len(),
            permissions,
            created,
            modified,
            accessed,
            is_hidden,
            is_symlink,
            symlink_target,
            mime_type,
            extension,
            metadata: file_metadata,
        })
    }

    fn get_entry_type(&self, metadata: &Metadata) -> EntryType {
        if metadata.is_dir() {
            EntryType::Directory
        } else if metadata.is_file() {
            EntryType::File
        } else {
            EntryType::Unknown
        }
    }

    fn get_permissions(&self, metadata: &Metadata) -> FilePermissions {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = metadata.permissions().mode();
            FilePermissions {
                readable: mode & 0o400 != 0,
                writable: mode & 0o200 != 0,
                executable: mode & 0o100 != 0,
                owner: "unknown".to_string(),
                group: "unknown".to_string(),
                mode: format!("{:o}", mode & 0o777),
            }
        }
        #[cfg(not(unix))]
        {
            FilePermissions {
                readable: !metadata.permissions().readonly(),
                writable: !metadata.permissions().readonly(),
                executable: false,
                owner: "unknown".to_string(),
                group: "unknown".to_string(),
                mode: "unknown".to_string(),
            }
        }
    }

    fn detect_mime_type(&self, extension: &Option<String>) -> Option<String> {
        if let Some(ext) = extension {
            match ext.to_lowercase().as_str() {
                "txt" | "md" | "rst" => Some("text/plain".to_string()),
                "html" | "htm" => Some("text/html".to_string()),
                "css" => Some("text/css".to_string()),
                "js" => Some("text/javascript".to_string()),
                "json" => Some("application/json".to_string()),
                "xml" => Some("application/xml".to_string()),
                "pdf" => Some("application/pdf".to_string()),
                "jpg" | "jpeg" => Some("image/jpeg".to_string()),
                "png" => Some("image/png".to_string()),
                "gif" => Some("image/gif".to_string()),
                "mp3" => Some("audio/mpeg".to_string()),
                "mp4" => Some("video/mp4".to_string()),
                "zip" => Some("application/zip".to_string()),
                "tar" => Some("application/tar".to_string()),
                "gz" => Some("application/gzip".to_string()),
                _ => None,
            }
        } else {
            None
        }
    }

    fn analyze_file_metadata(&self, path: &Path, entry_type: &EntryType, extension: &Option<String>) -> FileMetadata {
        if *entry_type != EntryType::File {
            return FileMetadata {
                line_count: None,
                encoding: None,
                language: None,
                is_binary: false,
                is_executable: false,
                is_archive: false,
                is_image: false,
                is_video: false,
                is_audio: false,
                checksum: None,
            };
        }

        let is_archive = if let Some(ext) = extension {
            matches!(ext.to_lowercase().as_str(), "zip" | "tar" | "gz" | "7z" | "rar")
        } else {
            false
        };

        let is_image = if let Some(ext) = extension {
            matches!(ext.to_lowercase().as_str(), "jpg" | "jpeg" | "png" | "gif" | "bmp" | "svg")
        } else {
            false
        };

        let is_video = if let Some(ext) = extension {
            matches!(ext.to_lowercase().as_str(), "mp4" | "avi" | "mov" | "mkv" | "wmv")
        } else {
            false
        };

        let is_audio = if let Some(ext) = extension {
            matches!(ext.to_lowercase().as_str(), "mp3" | "wav" | "flac" | "ogg" | "m4a")
        } else {
            false
        };

        let language = self.detect_language(extension);
        
        // Try to read file to detect if binary and count lines
        let (is_binary, line_count, encoding) = if let Ok(bytes) = fs::read(path) {
            let is_binary = bytes.iter().take(1024).any(|&b| b == 0);
            
            if !is_binary {
                if let Ok(content) = String::from_utf8(bytes) {
                    let lines = content.lines().count();
                    (false, Some(lines), Some("utf-8".to_string()))
                } else {
                    (true, None, None)
                }
            } else {
                (true, None, None)
            }
        } else {
            (false, None, None)
        };

        FileMetadata {
            line_count,
            encoding,
            language,
            is_binary,
            is_executable: self.is_executable(path),
            is_archive,
            is_image,
            is_video,
            is_audio,
            checksum: None,
        }
    }

    fn detect_language(&self, extension: &Option<String>) -> Option<String> {
        if let Some(ext) = extension {
            match ext.to_lowercase().as_str() {
                "rs" => Some("rust".to_string()),
                "js" | "mjs" => Some("javascript".to_string()),
                "ts" => Some("typescript".to_string()),
                "py" => Some("python".to_string()),
                "java" => Some("java".to_string()),
                "c" => Some("c".to_string()),
                "cpp" | "cc" | "cxx" => Some("cpp".to_string()),
                "h" | "hpp" => Some("c".to_string()),
                "go" => Some("go".to_string()),
                "rb" => Some("ruby".to_string()),
                "php" => Some("php".to_string()),
                "sh" | "bash" => Some("bash".to_string()),
                "ps1" => Some("powershell".to_string()),
                "html" | "htm" => Some("html".to_string()),
                "css" => Some("css".to_string()),
                "scss" | "sass" => Some("scss".to_string()),
                "json" => Some("json".to_string()),
                "yaml" | "yml" => Some("yaml".to_string()),
                "toml" => Some("toml".to_string()),
                "xml" => Some("xml".to_string()),
                "md" => Some("markdown".to_string()),
                _ => None,
            }
        } else {
            None
        }
    }

    fn is_executable(&self, path: &Path) -> bool {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = fs::metadata(path) {
                metadata.permissions().mode() & 0o111 != 0
            } else {
                false
            }
        }
        #[cfg(not(unix))]
        {
            if let Some(ext) = path.extension() {
                matches!(ext.to_string_lossy().to_lowercase().as_str(), "exe" | "com" | "bat" | "cmd")
            } else {
                false
            }
        }
    }

    fn is_accessible(&self, path: &Path) -> bool {
        path.exists() && fs::metadata(path).is_ok()
    }

    fn sort_entries(&self, entries: &mut Vec<FileSystemEntry>, sort_by: &SortBy, sort_order: &SortOrder) {
        entries.sort_by(|a, b| {
            let cmp = match sort_by {
                SortBy::Name => a.name.cmp(&b.name),
                SortBy::Size => a.size.cmp(&b.size),
                SortBy::Modified => a.modified.cmp(&b.modified),
                SortBy::Created => a.created.cmp(&b.created),
                SortBy::Type => a.file_type.to_string().cmp(&b.file_type.to_string()),
                SortBy::Extension => a.extension.cmp(&b.extension),
            };

            match sort_order {
                SortOrder::Ascending => cmp,
                SortOrder::Descending => cmp.reverse(),
            }
        });
    }

    fn calculate_operation_size(&self, paths: &[String]) -> (u64, usize) {
        let mut total_bytes = 0u64;
        let mut total_files = 0usize;

        for path in paths {
            if let Ok(metadata) = fs::metadata(path) {
                if metadata.is_file() {
                    total_bytes += metadata.len();
                    total_files += 1;
                } else if metadata.is_dir() {
                    // Would need to recursively calculate directory size
                    total_files += 1;
                }
            }
        }

        (total_bytes, total_files)
    }

    fn matches_pattern(&self, text: &str, pattern: &str, case_sensitive: bool, use_regex: bool) -> bool {
        if use_regex {
            if let Ok(regex) = regex::Regex::new(pattern) {
                regex.is_match(text)
            } else {
                false
            }
        } else {
            if case_sensitive {
                text.contains(pattern)
            } else {
                text.to_lowercase().contains(&pattern.to_lowercase())
            }
        }
    }

    fn add_recent_path(&mut self, path: String) {
        if let Some(pos) = self.recent_paths.iter().position(|p| p == &path) {
            self.recent_paths.remove(pos);
        }
        self.recent_paths.insert(0, path);
        self.recent_paths.truncate(50); // Keep last 50
    }
}

// Implementation for EntryType Display trait for sorting
impl std::fmt::Display for EntryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntryType::Directory => write!(f, "directory"),
            EntryType::File => write!(f, "file"),
            EntryType::Symlink => write!(f, "symlink"),
            EntryType::Pipe => write!(f, "pipe"),
            EntryType::Socket => write!(f, "socket"),
            EntryType::BlockDevice => write!(f, "block_device"),
            EntryType::CharDevice => write!(f, "char_device"),
            EntryType::Unknown => write!(f, "unknown"),
        }
    }
}

// Tauri commands
#[tauri::command]
pub async fn list_directory(
    path: String,
    sort_by: SortBy,
    sort_order: SortOrder,
    show_hidden: bool,
    fs_manager: State<'_, FileSystemManager>,
) -> Result<DirectoryListing, String> {
    let mut manager = fs_manager.lock().map_err(|e| e.to_string())?;
    manager.list_directory(&path, sort_by, sort_order, show_hidden)
}

#[tauri::command]
pub async fn get_file_info(
    path: String,
    fs_manager: State<'_, FileSystemManager>,
) -> Result<FileSystemEntry, String> {
    let manager = fs_manager.lock().map_err(|e| e.to_string())?;
    manager.get_file_info(&path)
}

#[tauri::command]
pub async fn get_path_completions(
    partial_path: String,
    limit: usize,
    fs_manager: State<'_, FileSystemManager>,
) -> Result<Vec<PathCompletion>, String> {
    let manager = fs_manager.lock().map_err(|e| e.to_string())?;
    Ok(manager.get_path_completions(&partial_path, limit))
}

#[tauri::command]
pub async fn search_files(
    query: SearchQuery,
    base_path: String,
    fs_manager: State<'_, FileSystemManager>,
) -> Result<Vec<SearchResult>, String> {
    let manager = fs_manager.lock().map_err(|e| e.to_string())?;
    Ok(manager.search_files(&query, &base_path))
}

#[tauri::command]
pub async fn create_file_operation(
    operation_type: OperationType,
    source: Vec<String>,
    destination: Option<String>,
    fs_manager: State<'_, FileSystemManager>,
) -> Result<String, String> {
    let mut manager = fs_manager.lock().map_err(|e| e.to_string())?;
    Ok(manager.create_file_operation(operation_type, source, destination))
}

#[tauri::command]
pub async fn start_file_operation(
    operation_id: String,
    fs_manager: State<'_, FileSystemManager>,
) -> Result<(), String> {
    let mut manager = fs_manager.lock().map_err(|e| e.to_string())?;
    manager.start_file_operation(&operation_id)
}

#[tauri::command]
pub async fn get_file_operations(
    fs_manager: State<'_, FileSystemManager>,
) -> Result<Vec<FileOperation>, String> {
    let manager = fs_manager.lock().map_err(|e| e.to_string())?;
    Ok(manager.operations.values().cloned().collect())
}

#[tauri::command]
pub async fn create_file_watcher(
    path: String,
    recursive: bool,
    events: Vec<WatchEventType>,
    fs_manager: State<'_, FileSystemManager>,
) -> Result<String, String> {
    let mut manager = fs_manager.lock().map_err(|e| e.to_string())?;
    manager.create_watcher(path, recursive, events)
}

#[tauri::command]
pub async fn get_recent_paths(
    fs_manager: State<'_, FileSystemManager>,
) -> Result<Vec<String>, String> {
    let manager = fs_manager.lock().map_err(|e| e.to_string())?;
    Ok(manager.recent_paths.clone())
}

#[tauri::command]
pub async fn add_path_bookmark(
    name: String,
    path: String,
    icon: Option<String>,
    fs_manager: State<'_, FileSystemManager>,
) -> Result<(), String> {
    let mut manager = fs_manager.lock().map_err(|e| e.to_string())?;
    let bookmark = PathBookmark {
        name,
        path,
        icon,
        created_at: Utc::now(),
        access_count: 0,
    };
    manager.bookmarks.push(bookmark);
    Ok(())
}

#[tauri::command]
pub async fn get_path_bookmarks(
    fs_manager: State<'_, FileSystemManager>,
) -> Result<Vec<PathBookmark>, String> {
    let manager = fs_manager.lock().map_err(|e| e.to_string())?;
    Ok(manager.bookmarks.clone())
}
