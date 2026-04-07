// File dialog integration for Tauri
// Provides native file dialogs and file operations

use serde::{Deserialize, Serialize};
use std::path::Path;
use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;
use tachyon_core::{ErrorResult, TachyonError};

/// File dialog options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDialogOptions {
    /// Default file name for save dialog
    pub default_name: Option<String>,
    /// Allowed file extensions
    pub extensions: Vec<String>,
    /// Allow multiple file selection
    pub multiple: bool,
    /// Allow directory selection
    pub directory: bool,
}

impl Default for FileDialogOptions {
    fn default() -> Self {
        Self {
            default_name: None,
            extensions: vec![],
            multiple: false,
            directory: false,
        }
    }
}

/// File dialog result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDialogResult {
    /// Selected file paths
    pub paths: Vec<String>,
    /// Canceled flag
    pub canceled: bool,
}

/// File content read result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileContent {
    /// File path
    pub path: String,
    /// File content
    pub content: String,
    /// File encoding
    pub encoding: String,
}

/// File write result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileWriteResult {
    /// File path
    pub path: String,
    /// Bytes written
    pub bytes_written: u64,
}

/// File dialog manager
pub struct FileDialogManager {
    app_handle: AppHandle,
}

impl FileDialogManager {
    /// Create a new file dialog manager
    pub fn new(app_handle: AppHandle) -> Self {
        Self { app_handle }
    }

    /// Open a file dialog for selecting files
    pub async fn open_file_dialog(&self, options: FileDialogOptions) -> ErrorResult<FileDialogResult> {
        use tauri_plugin_dialog::DialogExt;

        // FileDialogBuilder methods consume self and return Self — must chain
        let mut builder = self.app_handle.dialog().file();

        if let Some(name) = &options.default_name {
            builder = builder.set_file_name(name);
        }

        if !options.extensions.is_empty() {
            let ext_refs: Vec<&str> = options.extensions.iter().map(|s| s.as_str()).collect();
            builder = builder.add_filter("Files", &ext_refs);
        }

        let (sender, receiver) = tokio::sync::oneshot::channel::<FileDialogResult>();

        if options.directory {
            builder.pick_folder(move |path: Option<tauri_plugin_dialog::FilePath>| {
                let result = match path {
                    Some(p) => FileDialogResult {
                        paths: vec![p.to_string()],
                        canceled: false,
                    },
                    None => FileDialogResult { paths: vec![], canceled: true },
                };
                let _ = sender.send(result);
            });
        } else if options.multiple {
            builder.pick_files(move |paths: Option<Vec<tauri_plugin_dialog::FilePath>>| {
                let result = match paths {
                    Some(ps) => FileDialogResult {
                        paths: ps.iter().map(|p| p.to_string()).collect(),
                        canceled: false,
                    },
                    None => FileDialogResult { paths: vec![], canceled: true },
                };
                let _ = sender.send(result);
            });
        } else {
            builder.pick_file(move |path: Option<tauri_plugin_dialog::FilePath>| {
                let result = match path {
                    Some(p) => FileDialogResult {
                        paths: vec![p.to_string()],
                        canceled: false,
                    },
                    None => FileDialogResult { paths: vec![], canceled: true },
                };
                let _ = sender.send(result);
            });
        }

        receiver.await.map_err(|_| {
            TachyonError::internal("DIALOG_ERROR", "File dialog channel closed")
        })
    }

    /// Open a save file dialog
    pub async fn save_file_dialog(&self, options: FileDialogOptions) -> ErrorResult<FileDialogResult> {
        use tauri_plugin_dialog::DialogExt;

        // FileDialogBuilder methods consume self and return Self — must chain
        let mut builder = self.app_handle.dialog().file();

        if let Some(name) = &options.default_name {
            builder = builder.set_file_name(name);
        }

        if !options.extensions.is_empty() {
            let ext_refs: Vec<&str> = options.extensions.iter().map(|s| s.as_str()).collect();
            builder = builder.add_filter("Files", &ext_refs);
        }

        let (sender, receiver) = tokio::sync::oneshot::channel::<FileDialogResult>();

        builder.save_file(move |path: Option<tauri_plugin_dialog::FilePath>| {
            let result = match path {
                Some(p) => FileDialogResult {
                    paths: vec![p.to_string()],
                    canceled: false,
                },
                None => FileDialogResult { paths: vec![], canceled: true },
            };
            let _ = sender.send(result);
        });

        receiver.await.map_err(|_| {
            TachyonError::internal("DIALOG_ERROR", "Save dialog channel closed")
        })
    }

    /// Read file content
    pub async fn read_file(&self, path: impl AsRef<Path>) -> ErrorResult<FileContent> {
        let path_ref = path.as_ref();
        
        if !path_ref.exists() {
            return Err(TachyonError::not_found(format!("File: {}", path_ref.display())));
        }
        
        if !path_ref.is_file() {
            return Err(TachyonError::validation("NOT_A_FILE", format!("Path is not a file: {}", path_ref.display())));
        }

        let content = std::fs::read_to_string(path_ref)
            .map_err(|e| TachyonError::storage("READ_ERROR", format!("Failed to read file: {}", e)))?;

        Ok(FileContent {
            path: path_ref.to_string_lossy().to_string(),
            content,
            encoding: "utf-8".to_string(),
        })
    }

    /// Write file content
    pub async fn write_file(&self, path: impl AsRef<Path>, content: impl AsRef<str>) -> ErrorResult<FileWriteResult> {
        let path_ref = path.as_ref();
        let content_ref = content.as_ref();
        let bytes_written = content_ref.len() as u64;

        std::fs::write(path_ref, content_ref)
            .map_err(|e| TachyonError::storage("WRITE_ERROR", format!("Failed to write file: {}", e)))?;

        Ok(FileWriteResult {
            path: path_ref.to_string_lossy().to_string(),
            bytes_written,
        })
    }

    /// Check if file exists
    pub async fn file_exists(&self, path: impl AsRef<Path>) -> ErrorResult<bool> {
        Ok(path.as_ref().exists())
    }

    /// Delete a file
    pub async fn delete_file(&self, path: impl AsRef<Path>) -> ErrorResult<()> {
        let path_ref = path.as_ref();

        if !path_ref.exists() {
            return Err(TachyonError::not_found(format!("File: {}", path_ref.display())));
        }

        std::fs::remove_file(path_ref)
            .map_err(|e| TachyonError::storage("DELETE_ERROR", format!("Failed to delete file: {}", e)))?;

        Ok(())
    }

    /// Create a directory
    pub async fn create_directory(&self, path: impl AsRef<Path>) -> ErrorResult<()> {
        std::fs::create_dir_all(path.as_ref())
            .map_err(|e| TachyonError::storage("CREATE_DIR_ERROR", format!("Failed to create directory: {}", e)))?;

        Ok(())
    }

    /// Show an error message dialog
    pub fn show_error(&self, title: impl AsRef<str>, message: impl AsRef<str>) {
        let _ = self.app_handle
            .dialog()
            .message(message.as_ref())
            .title(title.as_ref())
            .kind(tauri_plugin_dialog::MessageDialogKind::Error)
            .show(|_| {});
    }

    /// Show a warning message dialog
    pub fn show_warning(&self, title: impl AsRef<str>, message: impl AsRef<str>) {
        let _ = self.app_handle
            .dialog()
            .message(message.as_ref())
            .title(title.as_ref())
            .kind(tauri_plugin_dialog::MessageDialogKind::Warning)
            .show(|_| {});
    }

    /// Show an info message dialog
    pub fn show_info(&self, title: impl AsRef<str>, message: impl AsRef<str>) {
        let _ = self.app_handle
            .dialog()
            .message(message.as_ref())
            .title(title.as_ref())
            .kind(tauri_plugin_dialog::MessageDialogKind::Info)
            .show(|_| {});
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_dialog_options_default() {
        let options = FileDialogOptions::default();
        assert!(options.default_name.is_none());
        assert!(options.extensions.is_empty());
        assert!(!options.multiple);
        assert!(!options.directory);
    }

    #[test]
    fn test_file_dialog_result() {
        let result = FileDialogResult {
            paths: vec!["/path/to/file.txt".to_string()],
            canceled: false,
        };

        assert_eq!(result.paths.len(), 1);
        assert_eq!(result.paths[0], "/path/to/file.txt");
        assert!(!result.canceled);
    }

    #[test]
    fn test_file_content() {
        let content = FileContent {
            path: "/path/to/file.txt".to_string(),
            content: "Hello, world!".to_string(),
            encoding: "utf-8".to_string(),
        };

        assert_eq!(content.path, "/path/to/file.txt");
        assert_eq!(content.content, "Hello, world!");
        assert_eq!(content.encoding, "utf-8");
    }

    #[test]
    fn test_file_write_result() {
        let result = FileWriteResult {
            path: "/path/to/file.txt".to_string(),
            bytes_written: 13,
        };

        assert_eq!(result.path, "/path/to/file.txt");
        assert_eq!(result.bytes_written, 13);
    }
}
