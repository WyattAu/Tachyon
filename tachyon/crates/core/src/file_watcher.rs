use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};
use walkdir::WalkDir;

use crate::{ErrorResult, TachyonError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FileChangeKind {
    Created,
    Modified,
    Deleted,
}

#[derive(Debug, Clone)]
pub struct FileChangeEvent {
    pub path: PathBuf,
    pub kind: FileChangeKind,
}

#[derive(Debug, Clone)]
pub struct FileWatcherConfig {
    pub watch_path: PathBuf,
    pub watch_extensions: Vec<String>,
    pub debounce_ms: u64,
    pub recursive: bool,
}

impl Default for FileWatcherConfig {
    fn default() -> Self {
        Self {
            watch_path: PathBuf::new(),
            watch_extensions: vec![".md".to_string(), ".markdown".to_string()],
            debounce_ms: 500,
            recursive: true,
        }
    }
}

#[derive(Debug)]
pub struct FileWatcher {
    watcher: Option<RecommendedWatcher>,
    config: FileWatcherConfig,
}

impl FileWatcher {
    pub fn new(config: FileWatcherConfig) -> ErrorResult<Self> {
        if !config.watch_path.exists() {
            return Err(TachyonError::validation(
                "INVALID_WATCH_PATH",
                format!("Watch path does not exist: {}", config.watch_path.display()),
            ));
        }
        if !config.watch_path.is_dir() {
            return Err(TachyonError::validation(
                "INVALID_WATCH_PATH",
                format!(
                    "Watch path is not a directory: {}",
                    config.watch_path.display()
                ),
            ));
        }
        Ok(Self {
            watcher: None,
            config,
        })
    }

    pub fn start(&mut self, tx: mpsc::Sender<FileChangeEvent>) -> ErrorResult<()> {
        let (notify_tx, notify_rx) = mpsc::channel::<notify::Result<Event>>();
        let debounce_ms = self.config.debounce_ms;
        let watch_extensions = self.config.watch_extensions.clone();
        let watch_path = self.config.watch_path.clone();

        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            if let Err(send_err) = notify_tx.send(res) {
                error!("Failed to send notify event to channel: {}", send_err);
            }
        })
        .map_err(|e| {
            TachyonError::internal(
                "WATCHER_INIT_FAILED",
                format!("Failed to create watcher: {}", e),
            )
        })?;

        let mode = if self.config.recursive {
            RecursiveMode::Recursive
        } else {
            RecursiveMode::NonRecursive
        };
        watcher.watch(&self.config.watch_path, mode).map_err(|e| {
            TachyonError::internal("WATCH_FAILED", format!("Failed to watch directory: {}", e))
        })?;

        self.watcher = Some(watcher);

        thread::spawn(move || {
            info!(
                "File watcher started, monitoring: {} (debounce: {}ms)",
                watch_path.display(),
                debounce_ms
            );
            debounce_and_forward(notify_rx, tx, debounce_ms, &watch_extensions, &watch_path);
        });

        Ok(())
    }

    pub fn stop(&mut self) {
        if let Some(mut watcher) = self.watcher.take() {
            if let Err(e) = watcher.unwatch(&self.config.watch_path) {
                warn!("Error stopping watcher: {}", e);
            }
            info!(
                "File watcher stopped for: {}",
                self.config.watch_path.display()
            );
        }
    }

    pub fn scan_initial(&self) -> Vec<FileChangeEvent> {
        let mut events = Vec::new();
        let entries = WalkDir::new(&self.config.watch_path)
            .follow_links(false)
            .into_iter();

        for entry in entries {
            match entry {
                Ok(e) => {
                    let path = e.path();
                    if !path.is_file() {
                        continue;
                    }
                    if !is_acceptable_path(path, &self.config.watch_extensions) {
                        continue;
                    }
                    events.push(FileChangeEvent {
                        path: path.to_path_buf(),
                        kind: FileChangeKind::Created,
                    });
                }
                Err(e) => {
                    warn!("Error scanning directory entry: {}", e);
                }
            }
        }

        info!(
            "Initial scan found {} markdown files in {}",
            events.len(),
            self.config.watch_path.display()
        );
        events
    }

    pub fn is_watched_extension(path: &Path, extensions: &[String]) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| {
                extensions
                    .iter()
                    .any(|watched| watched == ext || watched == &format!(".{}", ext))
            })
            .unwrap_or(false)
    }
}

fn is_acceptable_path(path: &Path, watch_extensions: &[String]) -> bool {
    if !FileWatcher::is_watched_extension(path, watch_extensions) {
        return false;
    }

    if path
        .file_name()
        .is_none_or(|name| name.to_str().is_some_and(|s| s.starts_with('.')))
    {
        return false;
    }

    for component in path.components() {
        if let std::path::Component::Normal(name) = component {
            if name.to_str().is_some_and(|s| s.starts_with('.')) {
                return false;
            }
        }
    }

    true
}

fn map_event_kind(kind: &EventKind) -> Option<FileChangeKind> {
    match kind {
        EventKind::Create(_) => Some(FileChangeKind::Created),
        EventKind::Modify(_) => Some(FileChangeKind::Modified),
        EventKind::Remove(_) => Some(FileChangeKind::Deleted),
        _ => None,
    }
}

fn debounce_and_forward(
    rx: mpsc::Receiver<notify::Result<Event>>,
    tx: mpsc::Sender<FileChangeEvent>,
    debounce_ms: u64,
    watch_extensions: &[String],
    _watch_path: &Path,
) {
    let debounce = Duration::from_millis(debounce_ms);
    let mut pending: HashMap<PathBuf, FileChangeKind> = HashMap::new();
    let mut deadline: Option<Instant> = None;

    loop {
        let recv_timeout = deadline.map(|d| d.saturating_duration_since(Instant::now()));
        let timeout = recv_timeout.unwrap_or(Duration::from_secs(30));

        match rx.recv_timeout(timeout) {
            Ok(Ok(event)) => {
                let file_change_kind = match map_event_kind(&event.kind) {
                    Some(k) => k,
                    None => continue,
                };

                for path in event.paths {
                    if !is_acceptable_path(&path, watch_extensions) {
                        continue;
                    }

                    pending.insert(path, file_change_kind);
                }

                if deadline.is_none() {
                    deadline = Some(Instant::now() + debounce);
                }
            }
            Ok(Err(e)) => {
                warn!("File watch error: {}", e);
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if let Some(dl) = deadline {
                    if Instant::now() >= dl {
                        flush_pending(&mut pending, &tx);
                        deadline = None;
                    }
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                debug!("Notify channel disconnected, flushing remaining events");
                flush_pending(&mut pending, &tx);
                break;
            }
        }
    }
}

fn flush_pending(
    pending: &mut HashMap<PathBuf, FileChangeKind>,
    tx: &mpsc::Sender<FileChangeEvent>,
) {
    if pending.is_empty() {
        return;
    }

    let count = pending.len();
    for (path, kind) in pending.drain() {
        debug!("Emitting file change: {:?} -> {:?}", path, kind);
        if tx.send(FileChangeEvent { path, kind }).is_err() {
            warn!("Receiver dropped, unable to send file change event");
            return;
        }
    }
    debug!("Flushed {} file change events", count);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = FileWatcherConfig::default();
        assert_eq!(config.watch_extensions, vec![".md", ".markdown"]);
        assert_eq!(config.debounce_ms, 500);
        assert!(config.recursive);
        assert!(config.watch_path.as_os_str().is_empty());
    }

    #[test]
    fn test_is_watched_extension_md() {
        let path = Path::new("/some/dir/notes.md");
        assert!(FileWatcher::is_watched_extension(
            path,
            &[".md".to_string()]
        ));
    }

    #[test]
    fn test_is_watched_extension_markdown() {
        let path = Path::new("/some/dir/notes.markdown");
        assert!(FileWatcher::is_watched_extension(
            path,
            &[".markdown".to_string()]
        ));
    }

    #[test]
    fn test_is_watched_extension_with_dot_prefix() {
        let path = Path::new("/some/dir/notes.md");
        assert!(FileWatcher::is_watched_extension(
            path,
            &[".md".to_string(), ".markdown".to_string()]
        ));
    }

    #[test]
    fn test_is_watched_extension_without_dot_prefix() {
        let path = Path::new("/some/dir/notes.md");
        assert!(FileWatcher::is_watched_extension(path, &["md".to_string()]));
    }

    #[test]
    fn test_is_watched_extension_unmatched() {
        let path = Path::new("/some/dir/image.png");
        assert!(!FileWatcher::is_watched_extension(
            path,
            &[".md".to_string()]
        ));
    }

    #[test]
    fn test_is_watched_extension_no_extension() {
        let path = Path::new("/some/dir/Makefile");
        assert!(!FileWatcher::is_watched_extension(
            path,
            &[".md".to_string()]
        ));
    }

    #[test]
    fn test_is_watched_extension_hidden_file() {
        let path = Path::new("/some/dir/.hidden.md");
        assert!(FileWatcher::is_watched_extension(
            path,
            &[".md".to_string()]
        ));
    }

    #[test]
    fn test_is_acceptable_path_normal_md_file() {
        let path = Path::new("/vault/docs/notes.md");
        let exts = vec![".md".to_string()];
        assert!(is_acceptable_path(path, &exts));
    }

    #[test]
    fn test_is_acceptable_path_hidden_file() {
        let path = Path::new("/vault/docs/.hidden.md");
        let exts = vec![".md".to_string()];
        assert!(!is_acceptable_path(path, &exts));
    }

    #[test]
    fn test_is_acceptable_path_hidden_directory() {
        let path = Path::new("/vault/.git/config");
        let exts = vec!["*".to_string()];
        assert!(!is_acceptable_path(path, &exts));
    }

    #[test]
    fn test_is_acceptable_path_nested_hidden_directory() {
        let path = Path::new("/vault/docs/.tachyon/metadata.json");
        let exts = vec!["json".to_string()];
        assert!(!is_acceptable_path(path, &exts));
    }

    #[test]
    fn test_is_acceptable_path_wrong_extension() {
        let path = Path::new("/vault/docs/image.png");
        let exts = vec![".md".to_string()];
        assert!(!is_acceptable_path(path, &exts));
    }

    #[test]
    fn test_is_acceptable_path_markdown_extension() {
        let path = Path::new("/vault/docs/notes.markdown");
        let exts = vec![".md".to_string(), ".markdown".to_string()];
        assert!(is_acceptable_path(path, &exts));
    }

    #[test]
    fn test_map_event_kind() {
        assert_eq!(
            map_event_kind(&EventKind::Create(notify::event::CreateKind::File)),
            Some(FileChangeKind::Created)
        );
        assert_eq!(
            map_event_kind(&EventKind::Modify(notify::event::ModifyKind::Any)),
            Some(FileChangeKind::Modified)
        );
        assert_eq!(
            map_event_kind(&EventKind::Remove(notify::event::RemoveKind::File)),
            Some(FileChangeKind::Deleted)
        );
        assert_eq!(
            map_event_kind(&EventKind::Access(notify::event::AccessKind::Any)),
            None
        );
    }

    #[test]
    fn test_new_with_nonexistent_path() {
        let config = FileWatcherConfig {
            watch_path: PathBuf::from("/nonexistent/path"),
            ..Default::default()
        };
        let result = FileWatcher::new(config);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, "INVALID_WATCH_PATH");
    }

    #[test]
    fn test_new_with_file_instead_of_directory() {
        let tmp = std::env::temp_dir().join("tachyon_test_file_watcher");
        std::fs::write(&tmp, "test").unwrap();
        let config = FileWatcherConfig {
            watch_path: tmp.clone(),
            ..Default::default()
        };
        let result = FileWatcher::new(config);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, "INVALID_WATCH_PATH");
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_new_with_valid_directory() {
        let dir = std::env::temp_dir().join("tachyon_test_watcher_dir");
        let _ = std::fs::create_dir_all(&dir);
        let config = FileWatcherConfig {
            watch_path: dir.clone(),
            ..Default::default()
        };
        let result = FileWatcher::new(config);
        assert!(result.is_ok());
        let _ = std::fs::remove_dir(&dir);
    }

    #[test]
    fn test_scan_initial_finds_md_files() {
        let dir = std::env::temp_dir().join("tachyon_test_scan");
        let _ = std::fs::create_dir_all(&dir);
        std::fs::write(dir.join("notes.md"), "# Hello").unwrap();
        std::fs::write(dir.join("image.png"), "binary").unwrap();

        let config = FileWatcherConfig {
            watch_path: dir.clone(),
            ..Default::default()
        };
        let watcher = FileWatcher::new(config).unwrap();
        let events = watcher.scan_initial();

        assert_eq!(events.len(), 1);
        assert!(events[0].path.ends_with("notes.md"));
        assert_eq!(events[0].kind, FileChangeKind::Created);

        let _ = std::fs::remove_file(dir.join("notes.md"));
        let _ = std::fs::remove_file(dir.join("image.png"));
        let _ = std::fs::remove_dir(&dir);
    }

    #[test]
    fn test_scan_initial_skips_hidden_files() {
        let dir = std::env::temp_dir().join("tachyon_test_scan_hidden");
        let _ = std::fs::create_dir_all(dir.join(".hidden"));
        std::fs::write(dir.join("notes.md"), "# Hello").unwrap();
        std::fs::write(dir.join(".hidden").join("draft.md"), "# Draft").unwrap();

        let config = FileWatcherConfig {
            watch_path: dir.clone(),
            ..Default::default()
        };
        let watcher = FileWatcher::new(config).unwrap();
        let events = watcher.scan_initial();

        assert_eq!(events.len(), 1);
        assert!(events[0].path.ends_with("notes.md"));

        let _ = std::fs::remove_file(dir.join("notes.md"));
        let _ = std::fs::remove_file(dir.join(".hidden").join("draft.md"));
        let _ = std::fs::remove_dir(dir.join(".hidden"));
        let _ = std::fs::remove_dir(&dir);
    }

    #[test]
    fn test_scan_initial_recursive() {
        let dir = std::env::temp_dir().join("tachyon_test_scan_recursive");
        let subdir = dir.join("sub");
        let _ = std::fs::create_dir_all(&subdir);
        std::fs::write(dir.join("root.md"), "# Root").unwrap();
        std::fs::write(subdir.join("nested.md"), "# Nested").unwrap();

        let config = FileWatcherConfig {
            watch_path: dir.clone(),
            recursive: true,
            ..Default::default()
        };
        let watcher = FileWatcher::new(config).unwrap();
        let events = watcher.scan_initial();

        assert_eq!(events.len(), 2);

        let _ = std::fs::remove_file(dir.join("root.md"));
        let _ = std::fs::remove_file(subdir.join("nested.md"));
        let _ = std::fs::remove_dir(&subdir);
        let _ = std::fs::remove_dir(&dir);
    }
}
