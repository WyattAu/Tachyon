// Filesystem operations bridge
// Provides vault browsing, markdown read/write, and file watching via tachyon-core

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Arc};
use tauri::AppHandle;
use tokio::sync::RwLock;

use tachyon_core::{FileChangeEvent, FileChangeKind, FileWatcher, FileWatcherConfig};

use crate::events::{EventEmitter, FileChangeKind as EventFileChangeKind};

/// An entry in a vault directory listing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultEntry {
    /// File or directory name
    pub name: String,
    /// Full path
    pub path: String,
    /// Whether this is a directory
    pub is_directory: bool,
    /// File extension (empty for directories)
    pub extension: Option<String>,
    /// File size in bytes (0 for directories)
    pub size: u64,
    /// Last modified timestamp (ISO 8601)
    pub modified: Option<String>,
}

/// Result of reading a markdown file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkdownFile {
    /// File path
    pub path: String,
    /// Raw markdown content
    pub content: String,
    /// File name
    pub filename: String,
}

/// Handle to a running file watcher task.
pub struct FileWatchHandle {
    abort_handle: tokio::task::AbortHandle,
}

impl FileWatchHandle {
    pub fn abort(&self) {
        self.abort_handle.abort();
    }
}

/// List the contents of a vault directory.
///
/// Returns markdown files and subdirectories, filtering out hidden files
/// and common non-document directories (.git, .obsidian, node_modules, etc.).
pub fn list_vault_entries(dir: &Path) -> Result<Vec<VaultEntry>, String> {
    if !dir.exists() {
        return Err(format!("Directory does not exist: {}", dir.display()));
    }
    if !dir.is_dir() {
        return Err(format!("Path is not a directory: {}", dir.display()));
    }

    let mut entries = Vec::new();

    let read_dir =
        std::fs::read_dir(dir).map_err(|e| format!("Failed to read directory: {}", e))?;

    for entry in read_dir.flatten() {
        let path = entry.path();
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        // Skip hidden files/directories
        if name.starts_with('.') {
            continue;
        }

        // Skip common non-document directories
        if path.is_dir() {
            let dir_name = name.to_lowercase();
            if matches!(
                dir_name.as_str(),
                "node_modules" | "target" | "dist" | "build" | "__pycache__" | ".git" | ".obsidian"
            ) {
                continue;
            }
        }

        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        let is_directory = path.is_dir();
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase());

        let modified = metadata.modified().ok().map(|t| {
            chrono::DateTime::<chrono::Utc>::from(t)
                .format("%Y-%m-%dT%H:%M:%SZ")
                .to_string()
        });

        entries.push(VaultEntry {
            name,
            path: path.to_string_lossy().to_string(),
            is_directory,
            extension,
            size: if is_directory { 0 } else { metadata.len() },
            modified,
        });
    }

    // Sort: directories first, then alphabetically
    entries.sort_by(|a, b| match (a.is_directory, b.is_directory) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.cmp(&b.name),
    });

    Ok(entries)
}

/// Read a markdown file from disk.
pub fn read_markdown_file(path: &Path) -> Result<MarkdownFile, String> {
    if !path.exists() {
        return Err(format!("File does not exist: {}", path.display()));
    }
    if !path.is_file() {
        return Err(format!("Path is not a file: {}", path.display()));
    }

    let content =
        std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;

    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    Ok(MarkdownFile {
        path: path.to_string_lossy().to_string(),
        content,
        filename,
    })
}

/// Write a markdown file to disk. Creates parent directories if needed.
pub fn write_markdown_file(path: &Path, content: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directories: {}", e))?;
    }

    std::fs::write(path, content).map_err(|e| format!("Failed to write file: {}", e))?;

    Ok(())
}

/// Start a file watcher using tachyon-core's FileWatcher (notify-based).
///
/// Spawns a background task that receives file change events and emits
/// them to the Tauri WebView.
pub fn start_file_watch(
    watch_path: PathBuf,
    app_handle: AppHandle,
    watch_handle: Arc<RwLock<Option<FileWatchHandle>>>,
) -> Result<(), String> {
    let config = FileWatcherConfig {
        watch_path: watch_path.clone(),
        watch_extensions: vec![".md".to_string(), ".markdown".to_string()],
        debounce_ms: 500,
        recursive: true,
    };

    let mut watcher = FileWatcher::new(config).map_err(|e| e.to_string())?;

    let (tx, rx) = mpsc::channel::<FileChangeEvent>();

    watcher.start(tx).map_err(|e| e.to_string())?;

    // Scan initial files
    let initial_files = watcher.scan_initial();
    let emitter = EventEmitter::new(app_handle.clone());
    for event in initial_files {
        let _ = emitter.emit_file_changed(
            event.path.to_string_lossy().to_string(),
            core_to_event_kind(event.kind),
        );
    }

    // Spawn a task to receive file change events and forward to WebView
    let handle = tokio::spawn(async move {
        let emitter = EventEmitter::new(app_handle);
        loop {
            match rx.recv_timeout(std::time::Duration::from_secs(30)) {
                Ok(event) => {
                    let kind = core_to_event_kind(event.kind);
                    let _ =
                        emitter.emit_file_changed(event.path.to_string_lossy().to_string(), kind);
                }
                Err(mpsc::RecvTimeoutError::Timeout) => continue,
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    tracing::info!("File watcher channel disconnected, stopping");
                    break;
                }
            }
        }
    });

    let abort_handle = handle.abort_handle();

    // Store the handle for later stop
    let rt = tokio::runtime::Handle::current();
    rt.block_on(async {
        let mut guard = watch_handle.write().await;
        *guard = Some(FileWatchHandle { abort_handle });
    });

    tracing::info!("File watcher started for: {}", watch_path.display());
    Ok(())
}

/// Stop the file watcher if running.
pub async fn stop_file_watch(
    watch_handle: Arc<RwLock<Option<FileWatchHandle>>>,
) -> Result<(), String> {
    let mut guard = watch_handle.write().await;
    if let Some(handle) = guard.take() {
        handle.abort();
        tracing::info!("File watcher stopped");
    }
    Ok(())
}

/// Check if the file watcher is currently running.
pub async fn is_file_watch_active(watch_handle: Arc<RwLock<Option<FileWatchHandle>>>) -> bool {
    watch_handle.read().await.is_some()
}

/// Recursively list all markdown files in a vault directory.
pub fn list_vault_markdown_files(dir: &Path) -> Result<Vec<VaultEntry>, String> {
    if !dir.exists() {
        return Err(format!("Directory does not exist: {}", dir.display()));
    }

    let mut entries = Vec::new();

    for entry in walkdir::WalkDir::new(dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        // Skip hidden files and directories
        let is_hidden = path.components().any(|c| {
            if let std::path::Component::Normal(name) = c {
                name.to_str().is_some_and(|s| s.starts_with('.'))
            } else {
                false
            }
        });
        if is_hidden {
            continue;
        }

        if !path.is_file() {
            continue;
        }

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase());

        match ext.as_deref() {
            Some("md") | Some("markdown") => {}
            _ => continue,
        }

        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        let metadata = match std::fs::metadata(path) {
            Ok(m) => m,
            Err(_) => continue,
        };

        let modified = metadata.modified().ok().map(|t| {
            chrono::DateTime::<chrono::Utc>::from(t)
                .format("%Y-%m-%dT%H:%M:%SZ")
                .to_string()
        });

        entries.push(VaultEntry {
            name,
            path: path.to_string_lossy().to_string(),
            is_directory: false,
            extension: ext,
            size: metadata.len(),
            modified,
        });
    }

    entries.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(entries)
}

fn core_to_event_kind(kind: FileChangeKind) -> EventFileChangeKind {
    match kind {
        FileChangeKind::Created => EventFileChangeKind::Created,
        FileChangeKind::Modified => EventFileChangeKind::Modified,
        FileChangeKind::Deleted => EventFileChangeKind::Deleted,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vault_entry_serialization() {
        let entry = VaultEntry {
            name: "notes.md".to_string(),
            path: "/vault/notes.md".to_string(),
            is_directory: false,
            extension: Some("md".to_string()),
            size: 42,
            modified: Some("2024-01-15T10:30:00Z".to_string()),
        };
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("notes.md"));
        assert!(json.contains("is_directory"));
    }

    #[test]
    fn test_markdown_file() {
        let file = MarkdownFile {
            path: "/vault/test.md".to_string(),
            content: "# Hello".to_string(),
            filename: "test.md".to_string(),
        };
        assert_eq!(file.filename, "test.md");
    }

    #[test]
    fn test_list_vault_entries_nonexistent() {
        let result = list_vault_entries(Path::new("/nonexistent/path"));
        assert!(result.is_err());
    }

    #[test]
    fn test_read_markdown_file_nonexistent() {
        let result = read_markdown_file(Path::new("/nonexistent/file.md"));
        assert!(result.is_err());
    }

    #[test]
    fn test_write_and_read_markdown_file() {
        let dir = std::env::temp_dir().join("tachyon_test_md_rw");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("test.md");

        write_markdown_file(&path, "# Test\n\nHello world").unwrap();
        let file = read_markdown_file(&path).unwrap();
        assert_eq!(file.content, "# Test\n\nHello world");
        assert_eq!(file.filename, "test.md");

        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_dir(&dir);
    }

    #[test]
    fn test_list_vault_entries_filters_hidden() {
        let dir = std::env::temp_dir().join("tachyon_test_vault_list");
        let _ = std::fs::create_dir_all(&dir);
        std::fs::write(dir.join("visible.md"), "# Visible").unwrap();
        std::fs::write(dir.join(".hidden.md"), "# Hidden").unwrap();

        let entries = list_vault_entries(&dir).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "visible.md");

        let _ = std::fs::remove_file(dir.join("visible.md"));
        let _ = std::fs::remove_file(dir.join(".hidden.md"));
        let _ = std::fs::remove_dir(&dir);
    }

    #[test]
    fn test_core_to_event_kind() {
        assert_eq!(
            core_to_event_kind(FileChangeKind::Created),
            EventFileChangeKind::Created
        );
        assert_eq!(
            core_to_event_kind(FileChangeKind::Modified),
            EventFileChangeKind::Modified
        );
        assert_eq!(
            core_to_event_kind(FileChangeKind::Deleted),
            EventFileChangeKind::Deleted
        );
    }
}
