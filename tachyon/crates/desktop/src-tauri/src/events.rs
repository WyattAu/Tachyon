// Event handling for Tauri
// Provides event emission from Rust backend to WebView

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use tachyon_core::{ErrorResult, TachyonError};
use crate::state::ConnectionStatus;

/// Event types emitted from Rust backend to WebView
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum DesktopEvent {
    /// Connection status changed
    ConnectionStatusChanged {
        status: ConnectionStatus,
    },
    /// Authentication status changed
    AuthStatusChanged {
        authenticated: bool,
        user_id: Option<String>,
    },
    /// Sync status changed
    SyncStatusChanged {
        status: SyncStatus,
        message: Option<String>,
    },
    /// Document updated
    DocumentUpdated {
        document_id: String,
        version: u64,
    },
    /// Node updated
    NodeUpdated {
        node_id: String,
        version: u64,
    },
    /// Repository status changed
    RepositoryStatusChanged {
        repository_id: Option<String>,
        status: RepositoryStatus,
    },
    /// File changed in local repository
    FileChanged {
        path: String,
        kind: FileChangeKind,
    },
    /// Error occurred
    Error {
        category: String,
        code: String,
        message: String,
    },
    /// Notification
    Notification {
        level: NotificationLevel,
        title: String,
        message: String,
    },
}

/// Sync status for synchronization operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncStatus {
    /// Not syncing
    Idle,
    /// Sync in progress
    Syncing,
    /// Sync completed successfully
    Success,
    /// Sync failed
    Failed,
}

/// Repository status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RepositoryStatus {
    /// No repository
    None,
    /// Repository initializing
    Initializing,
    /// Repository ready
    Ready,
    /// Repository error
    Error,
}

/// File change kind
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileChangeKind {
    /// File created
    Created,
    /// File modified
    Modified,
    /// File deleted
    Deleted,
}

/// Notification level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationLevel {
    /// Info notification
    Info,
    /// Warning notification
    Warning,
    /// Error notification
    Error,
}

impl DesktopEvent {
    /// Create a connection status changed event
    ///
    /// # Arguments
    /// * `status` - Connection status
    pub fn connection_status_changed(status: ConnectionStatus) -> Self {
        Self::ConnectionStatusChanged { status }
    }

    /// Create an authentication status changed event
    ///
    /// # Arguments
    /// * `authenticated` - Whether the user is authenticated
    /// * `user_id` - User ID if authenticated
    pub fn auth_status_changed(authenticated: bool, user_id: Option<String>) -> Self {
        Self::AuthStatusChanged {
            authenticated,
            user_id,
        }
    }

    /// Create a sync status changed event
    ///
    /// # Arguments
    /// * `status` - Sync status
    /// * `message` - Optional status message
    pub fn sync_status_changed(status: SyncStatus, message: Option<String>) -> Self {
        Self::SyncStatusChanged { status, message }
    }

    /// Create a document updated event
    ///
    /// # Arguments
    /// * `document_id` - Document ID
    /// * `version` - Document version
    pub fn document_updated(document_id: impl Into<String>, version: u64) -> Self {
        Self::DocumentUpdated {
            document_id: document_id.into(),
            version,
        }
    }

    /// Create a node updated event
    ///
    /// # Arguments
    /// * `node_id` - Node ID
    /// * `version` - Node version
    pub fn node_updated(node_id: impl Into<String>, version: u64) -> Self {
        Self::NodeUpdated {
            node_id: node_id.into(),
            version,
        }
    }

    /// Create a repository status changed event
    ///
    /// # Arguments
    /// * `repository_id` - Repository ID if available
    /// * `status` - Repository status
    pub fn repository_status_changed(repository_id: Option<String>, status: RepositoryStatus) -> Self {
        Self::RepositoryStatusChanged {
            repository_id,
            status,
        }
    }

    /// Create a file changed event
    ///
    /// # Arguments
    /// * `path` - File path
    /// * `kind` - Change kind
    pub fn file_changed(path: impl Into<String>, kind: FileChangeKind) -> Self {
        Self::FileChanged {
            path: path.into(),
            kind,
        }
    }

    /// Create an error event
    ///
    /// # Arguments
    /// * `error` - Tachyon error
    pub fn error(error: &TachyonError) -> Self {
        Self::Error {
            category: error.category.to_string(),
            code: error.code.clone(),
            message: error.message.clone(),
        }
    }

    /// Create a notification event
    ///
    /// # Arguments
    /// * `level` - Notification level
    /// * `title` - Notification title
    /// * `message` - Notification message
    pub fn notification(level: NotificationLevel, title: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Notification {
            level,
            title: title.into(),
            message: message.into(),
        }
    }
}

/// Event emitter for desktop events
pub struct EventEmitter {
    app_handle: AppHandle,
}

impl EventEmitter {
    /// Create a new event emitter
    ///
    /// # Arguments
    /// * `app_handle` - Tauri app handle
    pub fn new(app_handle: AppHandle) -> Self {
        Self { app_handle }
    }

    /// Emit an event to the WebView
    ///
    /// # Arguments
    /// * `event` - Event to emit
    pub fn emit(&self, event: DesktopEvent) -> ErrorResult<()> {
        let event_name = match &event {
            DesktopEvent::ConnectionStatusChanged { .. } => "connection-status-changed",
            DesktopEvent::AuthStatusChanged { .. } => "auth-status-changed",
            DesktopEvent::SyncStatusChanged { .. } => "sync-status-changed",
            DesktopEvent::DocumentUpdated { .. } => "document-updated",
            DesktopEvent::NodeUpdated { .. } => "node-updated",
            DesktopEvent::RepositoryStatusChanged { .. } => "repository-status-changed",
            DesktopEvent::FileChanged { .. } => "file-changed",
            DesktopEvent::Error { .. } => "error",
            DesktopEvent::Notification { .. } => "notification",
        };

        self.app_handle
            .emit(event_name, event)
            .map_err(|e| TachyonError::internal("EMIT_ERROR", format!("Failed to emit event: {}", e)))
    }

    /// Emit connection status changed event
    ///
    /// # Arguments
    /// * `status` - Connection status
    pub fn emit_connection_status_changed(&self, status: ConnectionStatus) -> ErrorResult<()> {
        self.emit(DesktopEvent::connection_status_changed(status))
    }

    /// Emit authentication status changed event
    ///
    /// # Arguments
    /// * `authenticated` - Whether the user is authenticated
    /// * `user_id` - User ID if authenticated
    pub fn emit_auth_status_changed(&self, authenticated: bool, user_id: Option<String>) -> ErrorResult<()> {
        self.emit(DesktopEvent::auth_status_changed(authenticated, user_id))
    }

    /// Emit sync status changed event
    ///
    /// # Arguments
    /// * `status` - Sync status
    /// * `message` - Optional status message
    pub fn emit_sync_status_changed(&self, status: SyncStatus, message: Option<String>) -> ErrorResult<()> {
        self.emit(DesktopEvent::sync_status_changed(status, message))
    }

    /// Emit document updated event
    ///
    /// # Arguments
    /// * `document_id` - Document ID
    /// * `version` - Document version
    pub fn emit_document_updated(&self, document_id: impl Into<String>, version: u64) -> ErrorResult<()> {
        self.emit(DesktopEvent::document_updated(document_id, version))
    }

    /// Emit node updated event
    ///
    /// # Arguments
    /// * `node_id` - Node ID
    /// * `version` - Node version
    pub fn emit_node_updated(&self, node_id: impl Into<String>, version: u64) -> ErrorResult<()> {
        self.emit(DesktopEvent::node_updated(node_id, version))
    }

    /// Emit repository status changed event
    ///
    /// # Arguments
    /// * `repository_id` - Repository ID if available
    /// * `status` - Repository status
    pub fn emit_repository_status_changed(&self, repository_id: Option<String>, status: RepositoryStatus) -> ErrorResult<()> {
        self.emit(DesktopEvent::repository_status_changed(repository_id, status))
    }

    /// Emit file changed event
    ///
    /// # Arguments
    /// * `path` - File path
    /// * `kind` - Change kind
    pub fn emit_file_changed(&self, path: impl Into<String>, kind: FileChangeKind) -> ErrorResult<()> {
        self.emit(DesktopEvent::file_changed(path, kind))
    }

    /// Emit error event
    ///
    /// # Arguments
    /// * `error` - Tachyon error
    pub fn emit_error(&self, error: &TachyonError) -> ErrorResult<()> {
        self.emit(DesktopEvent::error(error))
    }

    /// Emit notification event
    ///
    /// # Arguments
    /// * `level` - Notification level
    /// * `title` - Notification title
    /// * `message` - Notification message
    pub fn emit_notification(&self, level: NotificationLevel, title: impl Into<String>, message: impl Into<String>) -> ErrorResult<()> {
        self.emit(DesktopEvent::notification(level, title, message))
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_desktop_event_connection_status_changed() {
        let event = DesktopEvent::connection_status_changed(ConnectionStatus::Connected);
        match event {
            DesktopEvent::ConnectionStatusChanged { status } => {
                assert_eq!(status, ConnectionStatus::Connected);
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn test_desktop_event_auth_status_changed() {
        let event = DesktopEvent::auth_status_changed(true, Some("user-123".to_string()));
        match event {
            DesktopEvent::AuthStatusChanged { authenticated, user_id } => {
                assert!(authenticated);
                assert_eq!(user_id, Some("user-123".to_string()));
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn test_desktop_event_document_updated() {
        let event = DesktopEvent::document_updated("doc-123", 42);
        match event {
            DesktopEvent::DocumentUpdated { document_id, version } => {
                assert_eq!(document_id, "doc-123");
                assert_eq!(version, 42);
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn test_desktop_event_file_changed() {
        let event = DesktopEvent::file_changed("/path/to/file.md", FileChangeKind::Modified);
        match event {
            DesktopEvent::FileChanged { path, kind } => {
                assert_eq!(path, "/path/to/file.md");
                assert_eq!(kind, FileChangeKind::Modified);
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn test_desktop_event_notification() {
        let event = DesktopEvent::notification(NotificationLevel::Warning, "Warning", "File modified");
        match event {
            DesktopEvent::Notification { level, title, message } => {
                assert_eq!(level, NotificationLevel::Warning);
                assert_eq!(title, "Warning");
                assert_eq!(message, "File modified");
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn test_desktop_event_error() {
        let error = TachyonError::validation("TEST_ERROR", "Test error message");
        let event = DesktopEvent::error(&error);
        match event {
            DesktopEvent::Error { category, code, message } => {
                assert_eq!(category, "VALIDATION");
                assert_eq!(code, "TEST_ERROR");
                assert_eq!(message, "Test error message");
            }
            _ => panic!("Wrong event type"),
        }
    }
}
