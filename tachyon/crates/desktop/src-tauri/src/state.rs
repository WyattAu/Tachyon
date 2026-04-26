// Desktop state management
// Provides shared state for the Tauri application

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tachyon_core::{ErrorResult, TachyonError};

use crate::filesystem::FileWatchHandle;
use tokio::sync::RwLock;

/// Desktop application state
/// Manages the shared state across the Tauri application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopState {
    /// Server URL for API communication
    pub server_url: String,
    /// Authentication token
    pub auth_token: Option<String>,
    /// Current user ID
    pub user_id: Option<String>,
    /// Local repository path
    pub repository_path: Option<PathBuf>,
    /// Session ID
    pub session_id: Option<String>,
    /// Last sync timestamp
    pub last_sync: Option<DateTime<Utc>>,
    /// Connection status
    pub connection_status: ConnectionStatus,
    /// Auto-sync enabled
    pub auto_sync_enabled: bool,
    /// Local SQLite database path (for offline-first mode)
    pub database_path: Option<PathBuf>,
}

/// Connection status for the desktop client
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Default)]
pub enum ConnectionStatus {
    /// Not connected
    #[default]
    Disconnected,
    /// Currently connecting
    Connecting,
    /// Connected and online
    Connected,
    /// Connection error
    Error,
}


impl std::fmt::Display for ConnectionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Disconnected => write!(f, "Disconnected"),
            Self::Connecting => write!(f, "Connecting"),
            Self::Connected => write!(f, "Connected"),
            Self::Error => write!(f, "Error"),
        }
    }
}

impl Default for DesktopState {
    fn default() -> Self {
        Self {
            server_url: String::from("http://localhost:8080"),
            auth_token: None,
            user_id: None,
            repository_path: None,
            session_id: None,
            last_sync: None,
            connection_status: ConnectionStatus::default(),
            auto_sync_enabled: true,
            database_path: None,
        }
    }
}

impl DesktopState {
    /// Create a new desktop state
    ///
    /// # Arguments
    /// * `server_url` - Server URL for API communication
    pub fn new(server_url: impl Into<String>) -> Self {
        Self {
            server_url: server_url.into(),
            ..Default::default()
        }
    }

    /// Set the authentication token
    ///
    /// # Arguments
    /// * `token` - Authentication token
    pub fn set_auth_token(&mut self, token: Option<String>) {
        self.auth_token = token;
    }

    /// Set the current user ID
    ///
    /// # Arguments
    /// * `user_id` - User ID
    pub fn set_user_id(&mut self, user_id: Option<String>) {
        self.user_id = user_id;
    }

    /// Set the repository path
    ///
    /// # Arguments
    /// * `path` - Repository path
    pub fn set_repository_path(&mut self, path: Option<PathBuf>) {
        self.repository_path = path;
    }

    /// Set the session ID
    ///
    /// # Arguments
    /// * `session_id` - Session ID
    pub fn set_session_id(&mut self, session_id: Option<String>) {
        self.session_id = session_id;
    }

    /// Set the connection status
    ///
    /// # Arguments
    /// * `status` - Connection status
    pub fn set_connection_status(&mut self, status: ConnectionStatus) {
        self.connection_status = status;
    }

    /// Update the last sync timestamp
    pub fn update_last_sync(&mut self) {
        self.last_sync = Some(Utc::now());
    }

    /// Enable or disable auto-sync
    ///
    /// # Arguments
    /// * `enabled` - Auto-sync enabled flag
    pub fn set_auto_sync(&mut self, enabled: bool) {
        self.auto_sync_enabled = enabled;
    }

    /// Check if the client is authenticated
    pub fn is_authenticated(&self) -> bool {
        self.auth_token.is_some() && self.user_id.is_some()
    }

    /// Check if the client is connected to the server
    pub fn is_connected(&self) -> bool {
        self.connection_status == ConnectionStatus::Connected
    }

    /// Check if a repository is configured
    pub fn has_repository(&self) -> bool {
        self.repository_path.is_some()
    }
}

/// Thread-safe desktop state manager
#[derive(Clone)]
pub struct DesktopStateManager {
    state: Arc<Mutex<DesktopState>>,
}

impl DesktopStateManager {
    /// Create a new desktop state manager
    ///
    /// # Arguments
    /// * `state` - Initial desktop state
    pub fn new(state: DesktopState) -> Self {
        Self {
            state: Arc::new(Mutex::new(state)),
        }
    }

    /// Get a copy of the current state
    pub fn get_state(&self) -> ErrorResult<DesktopState> {
        self.state.lock().map(|guard| guard.clone()).map_err(|e| {
            TachyonError::internal(
                "STATE_LOCK_ERROR",
                format!("Failed to acquire state lock: {}", e),
            )
        })
    }

    /// Update the state with a function
    ///
    /// # Arguments
    /// * `f` - Function to update the state
    pub fn update_state<F>(&self, f: F) -> ErrorResult<()>
    where
        F: FnOnce(&mut DesktopState),
    {
        self.state
            .lock()
            .map(|mut guard| f(&mut guard))
            .map_err(|e| {
                TachyonError::internal(
                    "STATE_LOCK_ERROR",
                    format!("Failed to acquire state lock: {}", e),
                )
            })
    }

    /// Set the server URL
    ///
    /// # Arguments
    /// * `url` - Server URL
    pub fn set_server_url(&self, url: impl Into<String>) -> ErrorResult<()> {
        self.update_state(|state| state.server_url = url.into())
    }

    /// Set the authentication token
    ///
    /// # Arguments
    /// * `token` - Authentication token
    pub fn set_auth_token(&self, token: Option<String>) -> ErrorResult<()> {
        self.update_state(|state| state.set_auth_token(token))
    }

    /// Set the current user ID
    ///
    /// # Arguments
    /// * `user_id` - User ID
    pub fn set_user_id(&self, user_id: Option<String>) -> ErrorResult<()> {
        self.update_state(|state| state.set_user_id(user_id))
    }

    /// Set the repository path
    ///
    /// # Arguments
    /// * `path` - Repository path
    pub fn set_repository_path(&self, path: Option<PathBuf>) -> ErrorResult<()> {
        self.update_state(|state| state.set_repository_path(path))
    }

    /// Set the session ID
    ///
    /// # Arguments
    /// * `session_id` - Session ID
    pub fn set_session_id(&self, session_id: Option<String>) -> ErrorResult<()> {
        self.update_state(|state| state.set_session_id(session_id))
    }

    /// Set the connection status
    ///
    /// # Arguments
    /// * `status` - Connection status
    pub fn set_connection_status(&self, status: ConnectionStatus) -> ErrorResult<()> {
        self.update_state(|state| state.set_connection_status(status))
    }

    /// Update the last sync timestamp
    pub fn update_last_sync(&self) -> ErrorResult<()> {
        self.update_state(|state| state.update_last_sync())
    }

    /// Enable or disable auto-sync
    ///
    /// # Arguments
    /// * `enabled` - Auto-sync enabled flag
    pub fn set_auto_sync(&self, enabled: bool) -> ErrorResult<()> {
        self.update_state(|state| state.set_auto_sync(enabled))
    }

    /// Set the local database path
    ///
    /// # Arguments
    /// * `path` - Path to the SQLite database file
    pub fn set_database_path(&self, path: Option<PathBuf>) -> ErrorResult<()> {
        self.update_state(|state| state.database_path = path)
    }

    /// Check if the client is authenticated
    pub fn is_authenticated(&self) -> ErrorResult<bool> {
        self.get_state().map(|state| state.is_authenticated())
    }

    /// Check if the client is connected to the server
    pub fn is_connected(&self) -> ErrorResult<bool> {
        self.get_state().map(|state| state.is_connected())
    }

    /// Check if a repository is configured
    pub fn has_repository(&self) -> ErrorResult<bool> {
        self.get_state().map(|state| state.has_repository())
    }
}

impl Default for DesktopStateManager {
    fn default() -> Self {
        Self::new(DesktopState::default())
    }
}

/// Sync status for the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SyncStatus {
    #[default]
    Idle,
    Syncing,
    Success,
    Failed,
}

/// Shared Tauri state for filesystem watching and local database.
///
/// This state is managed by Tauri and shared across commands.
/// Unlike `DesktopStateManager` which uses `Mutex`, this uses
/// `tokio::RwLock` for async-safe access from Tauri commands.
pub struct DesktopAppState {
    /// Handle to the active file watcher (if running)
    pub file_watcher: Arc<RwLock<Option<FileWatchHandle>>>,
    /// Path to the local SQLite database
    pub local_db_path: Arc<RwLock<Option<PathBuf>>>,
    /// Current sync status
    pub sync_status: Arc<RwLock<SyncStatus>>,
}

impl DesktopAppState {
    pub fn new() -> Self {
        Self {
            file_watcher: Arc::new(RwLock::new(None)),
            local_db_path: Arc::new(RwLock::new(None)),
            sync_status: Arc::new(RwLock::new(SyncStatus::default())),
        }
    }
}

impl Default for DesktopAppState {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_desktop_state_default() {
        let state = DesktopState::default();
        assert_eq!(state.server_url, "http://localhost:8080");
        assert!(state.auth_token.is_none());
        assert!(state.user_id.is_none());
        assert!(state.repository_path.is_none());
        assert_eq!(state.connection_status, ConnectionStatus::Disconnected);
        assert!(state.auto_sync_enabled);
    }

    #[test]
    fn test_desktop_state_new() {
        let state = DesktopState::new("https://api.tachyon.io");
        assert_eq!(state.server_url, "https://api.tachyon.io");
    }

    #[test]
    fn test_set_auth_token() {
        let mut state = DesktopState::default();
        state.set_auth_token(Some("test-token".to_string()));
        assert_eq!(state.auth_token, Some("test-token".to_string()));
    }

    #[test]
    fn test_is_authenticated() {
        let mut state = DesktopState::default();
        assert!(!state.is_authenticated());

        state.set_auth_token(Some("test-token".to_string()));
        assert!(!state.is_authenticated());

        state.set_user_id(Some("user-123".to_string()));
        assert!(state.is_authenticated());
    }

    #[test]
    fn test_connection_status() {
        let mut state = DesktopState::default();
        assert_eq!(state.connection_status, ConnectionStatus::Disconnected);
        assert!(!state.is_connected());

        state.set_connection_status(ConnectionStatus::Connected);
        assert_eq!(state.connection_status, ConnectionStatus::Connected);
        assert!(state.is_connected());
    }

    #[test]
    fn test_desktop_state_manager() {
        let manager = DesktopStateManager::new(DesktopState::default());

        manager.set_server_url("https://api.tachyon.io").unwrap();
        let state = manager.get_state().unwrap();
        assert_eq!(state.server_url, "https://api.tachyon.io");
    }

    #[test]
    fn test_desktop_state_manager_update() {
        let manager = DesktopStateManager::new(DesktopState::default());

        manager
            .set_auth_token(Some("test-token".to_string()))
            .unwrap();
        manager.set_user_id(Some("user-123".to_string())).unwrap();

        assert!(manager.is_authenticated().unwrap());
    }

    #[test]
    fn test_update_last_sync() {
        let mut state = DesktopState::default();
        assert!(state.last_sync.is_none());

        state.update_last_sync();
        assert!(state.last_sync.is_some());
    }
}
