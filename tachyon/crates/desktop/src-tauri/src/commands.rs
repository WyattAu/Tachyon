// Tauri command handlers
// Implements IPC commands that can be invoked from the WebView

use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tachyon_core::{DocumentStore, TachyonError};
use tauri::{AppHandle, State};

use crate::EmbeddedServerState;
use crate::events::EventEmitter;
use crate::file_dialog::{FileContent, FileDialogManager, FileDialogOptions, FileWriteResult};
use crate::filesystem;
use crate::state::{ConnectionStatus, DesktopAppState, DesktopState, DesktopStateManager};
use crate::sync::{AutoSyncManager, SyncResult};

/// Authentication request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthRequest {
    /// Username
    pub username: String,
    /// Password
    pub password: String,
}

/// Authentication response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    /// Success flag
    pub success: bool,
    /// Authentication token
    pub token: Option<String>,
    /// User ID
    pub user_id: Option<String>,
    /// Error message
    pub error: Option<String>,
}

/// Repository configuration request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryConfigRequest {
    /// Repository path
    pub repository_path: String,
}

/// Sync request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncRequest {
    /// Remote name
    pub remote_name: Option<String>,
    /// Branch name
    pub branch_name: Option<String>,
}

/// Get current state
#[tauri::command]
pub async fn get_state(state: State<'_, DesktopStateManager>) -> Result<DesktopState, String> {
    state.get_state().map_err(|e| e.to_string())
}

/// Set server URL
#[tauri::command]
pub async fn set_server_url(
    url: String,
    state: State<'_, DesktopStateManager>,
    app: AppHandle,
) -> Result<(), String> {
    state
        .set_server_url(url.clone())
        .map_err(|e| e.to_string())?;

    // Emit connection status changed event
    let emitter = EventEmitter::new(app);
    emitter
        .emit_connection_status_changed(ConnectionStatus::Connecting)
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Authenticate with server
#[tauri::command]
pub async fn authenticate(
    request: AuthRequest,
    state: State<'_, DesktopStateManager>,
    app: AppHandle,
) -> Result<AuthResponse, String> {
    // Validate input
    if request.username.is_empty() {
        return Ok(AuthResponse {
            success: false,
            token: None,
            user_id: None,
            error: Some("Username cannot be empty".to_string()),
        });
    }

    if request.password.is_empty() {
        return Ok(AuthResponse {
            success: false,
            token: None,
            user_id: None,
            error: Some("Password cannot be empty".to_string()),
        });
    }

    // Get server URL from state
    let server_url = state
        .get_state()
        .map(|s| s.server_url.clone())
        .unwrap_or_else(|_| "http://localhost:8080".to_string());

    // Attempt to authenticate with the server
    match authenticate_with_server(&server_url, &request).await {
        Ok((token, user_id)) => {
            // Update state with authentication info
            state
                .set_auth_token(Some(token.clone()))
                .map_err(|e| e.to_string())?;
            state
                .set_user_id(Some(user_id.clone()))
                .map_err(|e| e.to_string())?;
            state
                .set_connection_status(ConnectionStatus::Connected)
                .map_err(|e| e.to_string())?;

            // Emit authentication status changed event
            let emitter = EventEmitter::new(app);
            emitter
                .emit_auth_status_changed(true, Some(user_id.clone()))
                .map_err(|e| e.to_string())?;

            Ok(AuthResponse {
                success: true,
                token: Some(token),
                user_id: Some(user_id),
                error: None,
            })
        }
        Err(e) => {
            // Update connection status to error
            let _ = state.set_connection_status(ConnectionStatus::Error);

            Ok(AuthResponse {
                success: false,
                token: None,
                user_id: None,
                error: Some(e),
            })
        }
    }
}

/// Authenticate with the remote server
///
/// # Arguments
/// * `server_url` - Server base URL
/// * `request` - Authentication request
///
/// # Returns
/// Result containing (token, user_id) or error message
async fn authenticate_with_server(
    server_url: &str,
    request: &AuthRequest,
) -> Result<(String, String), String> {
    use reqwest::Client;
    use serde_json::json;

    let client: Client = Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let auth_url = format!("{}/api/v1/auth/login", server_url.trim_end_matches('/'));

    let response = client
        .post(&auth_url)
        .json(&json!({
            "username": request.username,
            "password": request.password,
        }))
        .send()
        .await
        .map_err(|e| format!("Authentication request failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body: String = response.text().await.unwrap_or_default();
        return Err(format!("Authentication failed ({}): {}", status, body));
    }

    // Parse the response
    #[derive(serde::Deserialize)]
    struct AuthResponseBody {
        token: Option<String>,
        user_id: Option<String>,
        access_token: Option<String>, // Alternative field names
        id: Option<String>,
    }

    let body: AuthResponseBody = response
        .json::<AuthResponseBody>()
        .await
        .map_err(|e| format!("Failed to parse authentication response: {}", e))?;

    // Extract token (check multiple possible field names)
    let token = body
        .token
        .or(body.access_token)
        .ok_or_else(|| "No token in authentication response".to_string())?;

    // Extract user ID (check multiple possible field names)
    let user_id = body
        .user_id
        .or(body.id)
        .unwrap_or_else(|| format!("user_{}", uuid::Uuid::new_v4()));

    Ok((token, user_id))
}

/// Logout
#[tauri::command]
pub async fn logout(state: State<'_, DesktopStateManager>, app: AppHandle) -> Result<(), String> {
    state.set_auth_token(None).map_err(|e| e.to_string())?;
    state.set_user_id(None).map_err(|e| e.to_string())?;
    state
        .set_connection_status(ConnectionStatus::Disconnected)
        .map_err(|e| e.to_string())?;

    // Emit authentication status changed event
    let emitter = EventEmitter::new(app);
    emitter
        .emit_auth_status_changed(false, None)
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Open file dialog
#[tauri::command]
pub async fn open_file_dialog(
    options: FileDialogOptions,
    app: AppHandle,
) -> Result<crate::file_dialog::FileDialogResult, String> {
    let manager = FileDialogManager::new(app);
    manager
        .open_file_dialog(options)
        .await
        .map_err(|e| e.to_string())
}

/// Save file dialog
#[tauri::command]
pub async fn save_file_dialog(
    options: FileDialogOptions,
    app: AppHandle,
) -> Result<crate::file_dialog::FileDialogResult, String> {
    let manager = FileDialogManager::new(app);
    manager
        .save_file_dialog(options)
        .await
        .map_err(|e| e.to_string())
}

/// Read file content
#[tauri::command]
pub async fn read_file(path: String, app: AppHandle) -> Result<FileContent, String> {
    let manager = FileDialogManager::new(app);
    manager.read_file(path).await.map_err(|e| e.to_string())
}

/// Write file content
#[tauri::command]
pub async fn write_file(
    path: String,
    content: String,
    app: AppHandle,
) -> Result<FileWriteResult, String> {
    let manager = FileDialogManager::new(app);
    manager
        .write_file(path, content)
        .await
        .map_err(|e| e.to_string())
}

/// Check if file exists
#[tauri::command]
pub async fn file_exists(path: String, app: AppHandle) -> Result<bool, String> {
    let manager = FileDialogManager::new(app);
    manager.file_exists(path).await.map_err(|e| e.to_string())
}

/// Delete file
#[tauri::command]
pub async fn delete_file(path: String, app: AppHandle) -> Result<(), String> {
    let manager = FileDialogManager::new(app);
    manager.delete_file(path).await.map_err(|e| e.to_string())
}

/// Create directory
#[tauri::command]
pub async fn create_directory(path: String, app: AppHandle) -> Result<(), String> {
    let manager = FileDialogManager::new(app);
    manager
        .create_directory(path)
        .await
        .map_err(|e| e.to_string())
}

/// Set repository path
#[tauri::command]
pub async fn set_repository_path(
    request: RepositoryConfigRequest,
    state: State<'_, DesktopStateManager>,
) -> Result<(), String> {
    use std::path::PathBuf;
    let path = PathBuf::from(request.repository_path);
    state
        .set_repository_path(Some(path))
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Initialize repository
#[tauri::command]
pub async fn initialize_repository(
    state: State<'_, DesktopStateManager>,
    _sync_manager: State<'_, AutoSyncManager>,
) -> Result<(), String> {
    // Get the configured repository path from state
    let repo_path = state
        .get_state()
        .map(|s| s.repository_path.clone())
        .map_err(|e| e.to_string())?;

    let path = repo_path.ok_or_else(|| {
        "Repository path not configured. Call set_repository_path first.".to_string()
    })?;

    // Use spawn_blocking because git2::Repository is not Send.
    // initialize_repository() calls Repository::init(path) which creates
    // the repo and drops it immediately — safe to run off-thread.
    let path_for_thread = path.clone();
    tokio::task::spawn_blocking(move || {
        // Ensure parent directories exist
        if let Some(parent) = std::path::Path::new(&path_for_thread).parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        // git2::Repository::init is a quick filesystem operation; we don't
        // need the full AutoSyncManager for this — just init the repo.
        git2::Repository::init(&path_for_thread).map_err(|e| {
            TachyonError::git(
                "INIT_ERROR",
                format!("Failed to initialize repository: {}", e),
            )
        })?;
        Ok::<(), TachyonError>(())
    })
    .await
    .map_err(|e: tokio::task::JoinError| e.to_string())?
    .map_err(|e: TachyonError| e.to_string())
}

/// Commit pending changes
#[tauri::command]
pub async fn commit_pending(
    sync_manager: State<'_, AutoSyncManager>,
    app: AppHandle,
) -> Result<SyncResult, String> {
    let result = sync_manager
        .commit_pending()
        .await
        .map_err(|e| e.to_string())?;

    // Emit sync status changed event
    let emitter = EventEmitter::new(app);
    use crate::events::SyncStatus as EventSyncStatus;
    let event_status = match result.status {
        crate::sync::SyncStatus::Success => EventSyncStatus::Success,
        crate::sync::SyncStatus::Failed => EventSyncStatus::Failed,
        _ => EventSyncStatus::Idle,
    };
    emitter
        .emit_sync_status_changed(event_status, result.error.clone())
        .map_err(|e| e.to_string())?;

    Ok(result)
}

/// Push to remote
#[tauri::command]
pub async fn push_to_remote(
    request: SyncRequest,
    sync_manager: State<'_, AutoSyncManager>,
    app: AppHandle,
) -> Result<SyncResult, String> {
    let remote_name = request.remote_name.as_deref().unwrap_or("origin");
    let branch_name = request.branch_name.as_deref().unwrap_or("main");

    let result = sync_manager
        .push_to_remote(remote_name, branch_name)
        .await
        .map_err(|e| e.to_string())?;

    // Emit sync status changed event
    let emitter = EventEmitter::new(app);
    use crate::events::SyncStatus as EventSyncStatus;
    emitter
        .emit_sync_status_changed(EventSyncStatus::Success, result.error.clone())
        .map_err(|e| e.to_string())?;

    Ok(result)
}

/// Pull from remote
#[tauri::command]
pub async fn pull_from_remote(
    request: SyncRequest,
    sync_manager: State<'_, AutoSyncManager>,
    app: AppHandle,
) -> Result<SyncResult, String> {
    let remote_name = request.remote_name.as_deref().unwrap_or("origin");
    let branch_name = request.branch_name.as_deref().unwrap_or("main");

    let result = sync_manager
        .pull_from_remote(remote_name, branch_name)
        .await
        .map_err(|e| e.to_string())?;

    // Emit sync status changed event
    let emitter = EventEmitter::new(app);
    use crate::events::SyncStatus as EventSyncStatus;
    emitter
        .emit_sync_status_changed(EventSyncStatus::Success, result.error.clone())
        .map_err(|e| e.to_string())?;

    Ok(result)
}

/// Get sync status
#[tauri::command]
pub async fn get_sync_status(
    sync_manager: State<'_, AutoSyncManager>,
) -> Result<crate::sync::SyncStatus, String> {
    Ok(sync_manager.get_sync_status().await)
}

/// Get commit queue size
#[tauri::command]
pub async fn get_queue_size(sync_manager: State<'_, AutoSyncManager>) -> Result<usize, String> {
    Ok(sync_manager.get_queue_size().await)
}

/// Clear commit queue
#[tauri::command]
pub async fn clear_queue(sync_manager: State<'_, AutoSyncManager>) -> Result<(), String> {
    sync_manager.clear_queue().await;
    Ok(())
}

/// Enable auto-sync
#[tauri::command]
pub async fn enable_auto_sync(state: State<'_, DesktopStateManager>) -> Result<(), String> {
    state.set_auto_sync(true).map_err(|e| e.to_string())?;
    Ok(())
}

/// Disable auto-sync
#[tauri::command]
pub async fn disable_auto_sync(state: State<'_, DesktopStateManager>) -> Result<(), String> {
    state.set_auto_sync(false).map_err(|e| e.to_string())?;
    Ok(())
}

/// Queue file change for commit
#[tauri::command]
pub async fn queue_file_change(
    path: String,
    sync_manager: State<'_, AutoSyncManager>,
) -> Result<(), String> {
    sync_manager
        .queue_file_change(path)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Show error dialog
#[tauri::command]
pub async fn show_error_dialog(
    title: String,
    message: String,
    app: AppHandle,
) -> Result<(), String> {
    let manager = FileDialogManager::new(app);
    manager.show_error(title, message);
    Ok(())
}

/// Show warning dialog
#[tauri::command]
pub async fn show_warning_dialog(
    title: String,
    message: String,
    app: AppHandle,
) -> Result<(), String> {
    let manager = FileDialogManager::new(app);
    manager.show_warning(title, message);
    Ok(())
}

/// Show info dialog
#[tauri::command]
pub async fn show_info_dialog(
    title: String,
    message: String,
    app: AppHandle,
) -> Result<(), String> {
    let manager = FileDialogManager::new(app);
    manager.show_info(title, message);
    Ok(())
}

/// Check authentication status
#[tauri::command]
pub async fn is_authenticated(state: State<'_, DesktopStateManager>) -> Result<bool, String> {
    state.is_authenticated().map_err(|e| e.to_string())
}

/// Check connection status
#[tauri::command]
pub async fn is_connected(state: State<'_, DesktopStateManager>) -> Result<bool, String> {
    state.is_connected().map_err(|e| e.to_string())
}

/// Check if repository is configured
#[tauri::command]
pub async fn has_repository(state: State<'_, DesktopStateManager>) -> Result<bool, String> {
    state.has_repository().map_err(|e| e.to_string())
}

/// Start the file watcher for automatic change detection
#[tauri::command]
pub async fn start_file_watcher(
    interval_secs: Option<u64>,
    sync_manager: State<'_, AutoSyncManager>,
    app: AppHandle,
) -> Result<(), String> {
    sync_manager
        .start_file_watcher(interval_secs)
        .map_err(|e| e.to_string())?;

    let emitter = EventEmitter::new(app);
    emitter
        .emit_notification(
            crate::events::NotificationLevel::Info,
            "File Watcher Started",
            "Watching repository for changes",
        )
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Stop the file watcher
#[tauri::command]
pub async fn stop_file_watcher(
    sync_manager: State<'_, AutoSyncManager>,
    app: AppHandle,
) -> Result<(), String> {
    sync_manager
        .stop_file_watcher()
        .map_err(|e| e.to_string())?;

    let emitter = EventEmitter::new(app);
    emitter
        .emit_notification(
            crate::events::NotificationLevel::Info,
            "File Watcher Stopped",
            "No longer watching for file changes",
        )
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Check if the file watcher is running
#[tauri::command]
pub async fn is_file_watching(sync_manager: State<'_, AutoSyncManager>) -> Result<bool, String> {
    Ok(sync_manager.is_watching())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_request() {
        let request = AuthRequest {
            username: "test_user".to_string(),
            password: "test_pass".to_string(),
        };

        assert_eq!(request.username, "test_user");
        assert_eq!(request.password, "test_pass");
    }

    #[test]
    fn test_auth_response() {
        let response = AuthResponse {
            success: true,
            token: Some("test_token".to_string()),
            user_id: Some("user_123".to_string()),
            error: None,
        };

        assert!(response.success);
        assert_eq!(response.token, Some("test_token".to_string()));
        assert_eq!(response.user_id, Some("user_123".to_string()));
        assert!(response.error.is_none());
    }

    #[test]
    fn test_repository_config_request() {
        let request = RepositoryConfigRequest {
            repository_path: "/path/to/repo".to_string(),
        };

        assert_eq!(request.repository_path, "/path/to/repo");
    }

    #[test]
    fn test_sync_request() {
        let request = SyncRequest {
            remote_name: Some("origin".to_string()),
            branch_name: Some("main".to_string()),
        };

        assert_eq!(request.remote_name, Some("origin".to_string()));
        assert_eq!(request.branch_name, Some("main".to_string()));
    }
}

// ============================================================================
// Filesystem Commands (v2 — vault browsing, file watching, Obsidian import)
// ============================================================================

/// List entries in a vault directory.
///
/// Returns markdown files and subdirectories, filtering out hidden files
/// and common non-document directories.
#[tauri::command]
pub async fn read_vault(path: String) -> Result<Vec<filesystem::VaultEntry>, String> {
    let dir = std::path::PathBuf::from(path);
    tokio::task::spawn_blocking(move || filesystem::list_vault_entries(&dir))
        .await
        .map_err(|e| format!("Task join error: {}", e))?
}

/// Read a markdown file from disk.
#[tauri::command]
pub async fn read_markdown_file(path: String) -> Result<filesystem::MarkdownFile, String> {
    let p = std::path::PathBuf::from(path);
    tokio::task::spawn_blocking(move || filesystem::read_markdown_file(&p))
        .await
        .map_err(|e| format!("Task join error: {}", e))?
}

/// Write a markdown file to disk.
#[tauri::command]
pub async fn write_markdown_file(path: String, content: String) -> Result<(), String> {
    let p = std::path::PathBuf::from(path);
    tokio::task::spawn_blocking(move || filesystem::write_markdown_file(&p, &content))
        .await
        .map_err(|e| format!("Task join error: {}", e))?
}

/// List all markdown files recursively in a vault directory.
#[tauri::command]
pub async fn list_vault_files(path: String) -> Result<Vec<filesystem::VaultEntry>, String> {
    let dir = std::path::PathBuf::from(path);
    tokio::task::spawn_blocking(move || filesystem::list_vault_markdown_files(&dir))
        .await
        .map_err(|e| format!("Task join error: {}", e))?
}

/// Start a file watcher using tachyon-core's notify-based FileWatcher.
///
/// Watches the given directory for markdown file changes and emits
/// `file-changed` events to the WebView.
#[tauri::command]
pub async fn watch_directory(
    path: String,
    app: AppHandle,
    app_state: tauri::State<'_, DesktopAppState>,
) -> Result<(), String> {
    let watch_path = std::path::PathBuf::from(&path);

    if !watch_path.exists() {
        return Err(format!("Path does not exist: {}", path));
    }
    if !watch_path.is_dir() {
        return Err(format!("Path is not a directory: {}", path));
    }

    // Stop any existing watcher first
    filesystem::stop_file_watch(app_state.file_watcher.clone())
        .await
        .map_err(|e| e.to_string())?;

    filesystem::start_file_watch(watch_path, app, app_state.file_watcher.clone())
}

/// Stop the file watcher.
#[tauri::command]
pub async fn stop_directory_watch(
    app_state: tauri::State<'_, DesktopAppState>,
) -> Result<(), String> {
    filesystem::stop_file_watch(app_state.file_watcher.clone()).await
}

/// Check if a file watcher is currently active.
#[tauri::command]
pub async fn is_directory_watched(
    app_state: tauri::State<'_, DesktopAppState>,
) -> Result<bool, String> {
    Ok(filesystem::is_file_watch_active(app_state.file_watcher.clone()).await)
}

/// Get the application data directory path.
#[tauri::command]
pub async fn get_app_data_dir(app: AppHandle) -> Result<String, String> {
    use tauri::Manager;
    let path_resolver = app.path();
    path_resolver
        .app_data_dir()
        .map(|p| p.to_string_lossy().to_string())
        .map_err(|e| format!("Failed to get app data dir: {}", e))
}

/// Open a file or directory in the system file manager.
#[tauri::command]
pub async fn open_path(path: String) -> Result<(), String> {
    let p = std::path::Path::new(&path);
    if !p.exists() {
        return Err(format!("Path does not exist: {}", path));
    }

    open::that(p).map_err(|e| format!("Failed to open path: {}", e))
}

// ============================================================================
// Embedded Server Commands (v0.23.0)
// Only compiled when the "embedded-server" feature is enabled.
// This pulls in wasmtime, 10 tree-sitter grammars, sqlx×4, tantivy,
// async-graphql, ldap3, and many other heavy crates.
// ============================================================================

/// Get the port of the embedded server, if it's running.
///
/// Returns 0 if the server has not been started.
#[cfg(feature = "embedded-server")]
#[tauri::command]
pub fn get_embedded_server_port(state: tauri::State<'_, Arc<Mutex<EmbeddedServerState>>>) -> u16 {
    match state.lock() {
        Ok(s) => s.port,
        Err(e) => {
            eprintln!("Failed to lock embedded server state: {}", e);
            0
        }
    }
}

#[cfg(not(feature = "embedded-server"))]
#[tauri::command]
pub fn get_embedded_server_port(_state: tauri::State<'_, Arc<Mutex<EmbeddedServerState>>>) -> u16 {
    tracing::warn!("Embedded server not available — rebuild with --features embedded-server");
    0
}

/// Start the embedded Axum server on a random port.
///
/// This runs the full Tachyon server (API, WebSocket, renderer)
/// in-process alongside the Tauri application. The server binds to
/// 127.0.0.1:0 and the assigned port is stored in state.
///
/// Returns the port number on success, or 0 on failure.
#[cfg(feature = "embedded-server")]
#[tauri::command]
pub async fn start_embedded_server(
    state: tauri::State<'_, Arc<Mutex<EmbeddedServerState>>>,
    database_url: Option<String>,
) -> Result<u16, String> {
    // Prevent double-start
    {
        let s = state
            .lock()
            .map_err(|e| format!("Failed to lock embedded server state: {}", e))?;
        if s.started {
            return Ok(s.port);
        }
    }

    // Build server config from environment, with optional database URL override
    let mut config = tachyon_server::ServerConfig::from_env();

    // Override database URL if provided (for custom database paths)
    if let Some(url) = database_url {
        config.database_url = url;
    }

    // Force bind to localhost only for security
    config.host = "127.0.0.1".to_string();

    // Disable CSP in desktop mode: the webview manages its own CSP via
    // tauri.conf.json. The server's CSP would block inline styles (Tailwind
    // CSS, Leptos-generated classes) since WebKit enforces both HTTP header
    // CSP and meta-tag CSP (intersection).
    config.security.csp_enabled = false;

    // Start the server
    let app = match tachyon_server::build_server(&config).await {
        Ok(app) => app,
        Err(e) => {
            return Err(format!("Failed to build server: {}", e));
        }
    };

    // Bind to port 0 (OS assigns an available port)
    let listener = match tokio::net::TcpListener::bind("127.0.0.1:0").await {
        Ok(l) => l,
        Err(e) => {
            return Err(format!("Failed to bind to port: {}", e));
        }
    };

    let port = listener.local_addr().map(|a| a.port()).unwrap_or(0);

    // Update state
    {
        let mut s = state
            .lock()
            .map_err(|e| format!("Failed to lock embedded server state: {}", e))?;
        s.port = port;
        s.started = true;
    }

    tracing::info!("Embedded Tachyon server started on 127.0.0.1:{}", port);

    // Spawn the server in the background
    tauri::async_runtime::spawn(async move {
        match axum::serve(listener, app).await {
            Ok(()) => {
                tracing::info!("Embedded server shut down gracefully");
            }
            Err(e) => {
                tracing::error!("Embedded server error: {}", e);
            }
        }
    });

    Ok(port)
}

#[cfg(not(feature = "embedded-server"))]
#[tauri::command]
pub async fn start_embedded_server(
    _state: tauri::State<'_, Arc<Mutex<EmbeddedServerState>>>,
    _database_url: Option<String>,
) -> Result<u16, String> {
    Err("Embedded server not available — rebuild with --features embedded-server".to_string())
}

/// Stop the embedded server.
///
/// Note: This does not gracefully shut down the server — it just marks
/// the state as not started. The actual server task will terminate
/// when the Tauri app exits.
#[cfg(feature = "embedded-server")]
#[tauri::command]
pub fn stop_embedded_server(state: tauri::State<'_, Arc<Mutex<EmbeddedServerState>>>) -> bool {
    let mut s = match state.lock() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to lock embedded server state: {}", e);
            return false;
        }
    };
    let was_started = s.started;
    s.started = false;
    s.port = 0;
    was_started
}

#[cfg(not(feature = "embedded-server"))]
#[tauri::command]
pub fn stop_embedded_server(state: tauri::State<'_, Arc<Mutex<EmbeddedServerState>>>) -> bool {
    let mut s = match state.lock() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to lock embedded server state: {}", e);
            return false;
        }
    };
    let was_started = s.started;
    s.started = false;
    s.port = 0;
    was_started
}

// ============================================================================
// Local Storage Commands (SQLite for offline-first mode)
// ============================================================================

/// Local database statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalDbStats {
    pub is_available: bool,
    pub total_documents: usize,
    pub draft_count: usize,
    pub published_count: usize,
    pub archived_count: usize,
    pub total_word_count: usize,
    pub total_tags: usize,
    pub database_path: Option<String>,
}

/// Get local database statistics
#[tauri::command]
pub async fn get_local_db_stats(
    state: State<'_, DesktopStateManager>,
) -> Result<LocalDbStats, String> {
    let desktop_state = state.get_state().map_err(|e| e.to_string())?;
    let db_path = desktop_state.database_path.clone();

    let path = match db_path {
        Some(ref p) => p.clone(),
        None => {
            return Ok(LocalDbStats {
                is_available: false,
                total_documents: 0,
                draft_count: 0,
                published_count: 0,
                archived_count: 0,
                total_word_count: 0,
                total_tags: 0,
                database_path: None,
            });
        }
    };

    let store = tachyon_storage::SqliteStore::open(&path)
        .await
        .map_err(|e| format!("Failed to open local database: {}", e))?;

    let available = store.is_available().await.unwrap_or(false);

    let summary = store
        .get_list_summary()
        .await
        .unwrap_or(tachyon_core::types::storage::DocumentListSummary::default());

    Ok(LocalDbStats {
        is_available: available,
        total_documents: summary.total_documents,
        draft_count: summary.draft_count,
        published_count: summary.published_count,
        archived_count: summary.archived_count,
        total_word_count: summary.total_word_count,
        total_tags: summary.total_tags,
        database_path: Some(path.to_string_lossy().to_string()),
    })
}

/// Initialize local database at the given path
#[tauri::command]
pub async fn init_local_database(
    state: State<'_, DesktopStateManager>,
    path: String,
) -> Result<LocalDbStats, String> {
    // Update state with the new database path
    state
        .set_database_path(Some(std::path::PathBuf::from(&path)))
        .map_err(|e| e.to_string())?;

    // Open (or create) the database
    let store = tachyon_storage::SqliteStore::open(&path)
        .await
        .map_err(|e| format!("Failed to initialize local database: {}", e))?;

    // Verify it's available
    let available = store.is_available().await.unwrap_or(false);
    if !available {
        return Err("Database initialized but not available".to_string());
    }

    let summary = store
        .get_list_summary()
        .await
        .unwrap_or(tachyon_core::types::storage::DocumentListSummary::default());

    tracing::info!(
        "Local SQLite database initialized at: {} ({} documents)",
        path,
        summary.total_documents
    );

    Ok(LocalDbStats {
        is_available: true,
        total_documents: summary.total_documents,
        draft_count: summary.draft_count,
        published_count: summary.published_count,
        archived_count: summary.archived_count,
        total_word_count: summary.total_word_count,
        total_tags: summary.total_tags,
        database_path: Some(path),
    })
}

/// Get all tags from the local database
#[tauri::command]
pub async fn get_local_tags(state: State<'_, DesktopStateManager>) -> Result<Vec<String>, String> {
    let desktop_state = state.get_state().map_err(|e| e.to_string())?;

    let path = match &desktop_state.database_path {
        Some(p) => p.clone(),
        None => return Ok(vec![]),
    };

    let store = tachyon_storage::SqliteStore::open(&path)
        .await
        .map_err(|e| format!("Failed to open local database: {}", e))?;

    store
        .get_all_tags()
        .await
        .map_err(|e| format!("Failed to get tags: {}", e))
}

/// Search local documents
#[tauri::command]
pub async fn search_local_documents(
    state: State<'_, DesktopStateManager>,
    query: String,
    page: Option<usize>,
    page_size: Option<usize>,
) -> Result<serde_json::Value, String> {
    let desktop_state = state.get_state().map_err(|e| e.to_string())?;

    let path = match &desktop_state.database_path {
        Some(p) => p.clone(),
        None => {
            return Ok(serde_json::json!({
                "items": [],
                "total": 0,
                "page": 1,
                "page_size": 20,
            }));
        }
    };

    let store = tachyon_storage::SqliteStore::open(&path)
        .await
        .map_err(|e| format!("Failed to open local database: {}", e))?;

    let result = store
        .search_documents(&query, page.unwrap_or(1), page_size.unwrap_or(20))
        .await
        .map_err(|e| format!("Failed to search local documents: {}", e))?;

    serde_json::to_value(&result).map_err(|e| format!("Failed to serialize results: {}", e))
}

// ============================================================================
// Sync Queue Commands (offline→online reconciliation)
// ============================================================================

/// Enqueue a document mutation for later sync to the remote server.
///
/// The frontend should call this whenever it writes to the local SQLite
/// database while offline. When connectivity is restored, the entries are
/// replayed against the remote server.
#[tauri::command]
pub async fn sync_enqueue(
    state: State<'_, DesktopStateManager>,
    operation: String, // "create" | "update_content" | "update_metadata" | "delete" | "permanent_delete"
    document_id: String,
    payload: Option<String>, // JSON: full metadata+content for create, partial for updates
) -> Result<String, String> {
    let desktop_state = state.get_state().map_err(|e| e.to_string())?;
    let path = match &desktop_state.database_path {
        Some(p) => p.clone(),
        None => return Err("No local database configured".to_string()),
    };

    let queue = tachyon_storage::SyncQueue::open(&path)
        .await
        .map_err(|e| format!("Failed to open sync queue: {}", e))?;

    let op = match operation.as_str() {
        "create" => tachyon_storage::SyncOperation::Create,
        "update_content" => tachyon_storage::SyncOperation::UpdateContent,
        "update_metadata" => tachyon_storage::SyncOperation::UpdateMetadata,
        "delete" => tachyon_storage::SyncOperation::Delete,
        "permanent_delete" => tachyon_storage::SyncOperation::PermanentDelete,
        _ => return Err(format!("Unknown sync operation: {}", operation)),
    };

    let entry_id = queue
        .enqueue(op, &document_id, payload)
        .await
        .map_err(|e| format!("Failed to enqueue sync entry: {}", e))?;

    tracing::info!(
        "Enqueued sync entry {} ({} on {})",
        entry_id,
        operation,
        document_id
    );
    Ok(entry_id)
}

/// Get the sync queue summary (pending/in-flight/synced/failed counts).
#[tauri::command]
pub async fn sync_queue_summary(
    state: State<'_, DesktopStateManager>,
) -> Result<tachyon_storage::SyncQueueSummary, String> {
    let desktop_state = state.get_state().map_err(|e| e.to_string())?;
    let path = match &desktop_state.database_path {
        Some(p) => p.clone(),
        None => return Ok(tachyon_storage::SyncQueueSummary::default()),
    };

    let queue = tachyon_storage::SyncQueue::open(&path)
        .await
        .map_err(|e| format!("Failed to open sync queue: {}", e))?;

    queue
        .summary()
        .await
        .map_err(|e| format!("Failed to get sync queue summary: {}", e))
}

/// Get pending sync entries (oldest first).
#[tauri::command]
pub async fn sync_queue_pending(
    state: State<'_, DesktopStateManager>,
    limit: Option<usize>,
) -> Result<Vec<tachyon_storage::SyncQueueEntry>, String> {
    let desktop_state = state.get_state().map_err(|e| e.to_string())?;
    let path = match &desktop_state.database_path {
        Some(p) => p.clone(),
        None => return Ok(vec![]),
    };

    let queue = tachyon_storage::SyncQueue::open(&path)
        .await
        .map_err(|e| format!("Failed to open sync queue: {}", e))?;

    queue
        .pending_entries(limit.unwrap_or(50))
        .await
        .map_err(|e| format!("Failed to get pending entries: {}", e))
}

/// Mark a sync queue entry as successfully synced.
#[tauri::command]
pub async fn sync_mark_synced(
    state: State<'_, DesktopStateManager>,
    entry_id: String,
) -> Result<(), String> {
    let desktop_state = state.get_state().map_err(|e| e.to_string())?;
    let path = match &desktop_state.database_path {
        Some(p) => p.clone(),
        None => return Err("No local database configured".to_string()),
    };

    let queue = tachyon_storage::SyncQueue::open(&path)
        .await
        .map_err(|e| format!("Failed to open sync queue: {}", e))?;

    queue
        .mark_synced(&entry_id)
        .await
        .map_err(|e| format!("Failed to mark entry synced: {}", e))
}

/// Mark a sync queue entry as failed (will be retried).
#[tauri::command]
pub async fn sync_mark_failed(
    state: State<'_, DesktopStateManager>,
    entry_id: String,
    error: String,
) -> Result<(), String> {
    let desktop_state = state.get_state().map_err(|e| e.to_string())?;
    let path = match &desktop_state.database_path {
        Some(p) => p.clone(),
        None => return Err("No local database configured".to_string()),
    };

    let queue = tachyon_storage::SyncQueue::open(&path)
        .await
        .map_err(|e| format!("Failed to open sync queue: {}", e))?;

    queue
        .mark_failed(&entry_id, &error)
        .await
        .map_err(|e| format!("Failed to mark entry failed: {}", e))
}

/// Purge successfully synced entries from the queue.
#[tauri::command]
pub async fn sync_purge_synced(state: State<'_, DesktopStateManager>) -> Result<u64, String> {
    let desktop_state = state.get_state().map_err(|e| e.to_string())?;
    let path = match &desktop_state.database_path {
        Some(p) => p.clone(),
        None => return Ok(0),
    };

    let queue = tachyon_storage::SyncQueue::open(&path)
        .await
        .map_err(|e| format!("Failed to open sync queue: {}", e))?;

    queue
        .purge_synced()
        .await
        .map_err(|e| format!("Failed to purge synced entries: {}", e))
}

// ============================================================================
// Offline/Online Detection Commands
// ============================================================================

/// Update the connection status. Called by the frontend when it detects
/// a connectivity change via `navigator.onLine` or failed HTTP requests.
#[tauri::command]
pub async fn set_connection_status(
    state: State<'_, DesktopStateManager>,
    status: String, // "connected" | "disconnected" | "error"
) -> Result<bool, String> {
    let conn_status = match status.as_str() {
        "connected" => ConnectionStatus::Connected,
        "disconnected" => ConnectionStatus::Disconnected,
        "error" => ConnectionStatus::Error,
        _ => return Err(format!("Unknown connection status: {}", status)),
    };

    state
        .set_connection_status(conn_status)
        .map_err(|e| e.to_string())?;

    tracing::info!("Connection status changed to: {}", status);
    Ok(true)
}

/// Check if the app is currently online (has a connected status).
#[tauri::command]
pub async fn is_online(state: State<'_, DesktopStateManager>) -> Result<bool, String> {
    state.is_connected().map_err(|e| e.to_string())
}

// ============================================================================
// Local-First Authentication Commands
// ============================================================================

/// Authenticate in local-first (offline) mode.
///
/// Creates a local session without requiring server connectivity.
/// The user can work offline and sync later when the server is available.
#[tauri::command]
pub async fn authenticate_offline(
    username: String,
    state: State<'_, DesktopStateManager>,
    app: AppHandle,
) -> Result<AuthResponse, String> {
    if username.is_empty() {
        return Ok(AuthResponse {
            success: false,
            token: None,
            user_id: None,
            error: Some("Username cannot be empty".to_string()),
        });
    }

    // Generate deterministic local user identity
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(username.as_bytes());
    hasher.update(b"tachyon-local-user");
    let hash = hasher.finalize();
    let user_id = format!("local_{}", hex::encode(&hash[..16]));

    // Generate a local session token
    let token = format!("local_{}", uuid::Uuid::new_v4());

    // Update state with local auth
    state
        .set_auth_token(Some(token.clone()))
        .map_err(|e| e.to_string())?;
    state
        .set_user_id(Some(user_id.clone()))
        .map_err(|e| e.to_string())?;
    state
        .set_connection_status(ConnectionStatus::Disconnected)
        .map_err(|e| e.to_string())?;

    // Emit auth status changed event
    let emitter = EventEmitter::new(app);
    emitter
        .emit_auth_status_changed(true, Some(user_id.clone()))
        .map_err(|e| e.to_string())?;

    tracing::info!("Created local-first session for user: {}", username);

    Ok(AuthResponse {
        success: true,
        token: Some(token),
        user_id: Some(user_id),
        error: None,
    })
}

// ---------------------------------------------------------------------------
// Debug reporter — called via Tauri IPC from injected JS.
// Writes one JSON line per call to /tmp/tachyon-debug.jsonl.
// This is the only mechanism that works for WebView debugging when CSP
// blocks inline scripts (Tauri 2 injects nonces which make 'unsafe-inline'
// ineffective per CSP Level 3 spec). webview.eval() bypasses CSP entirely.
// ---------------------------------------------------------------------------

/// Append a JSON debug event to /tmp/tachyon-debug.jsonl.
///
/// Called from JS injected via `webview.eval()` in setup().
/// Supports both single entries (`{type, ...}`) and batched arrays (`[{...}, ...]`).
/// Each call appends one or more lines: `{"ts":"...","type":"...","data":{...}}`.
#[tauri::command]
pub fn debug_report(data: serde_json::Value) -> Result<(), String> {
    use std::io::Write;

    let entries: Vec<serde_json::Value> = match &data {
        serde_json::Value::String(s) => {
            // Batched: data is a JSON string like "[{...},{...}]"
            serde_json::from_str(s).unwrap_or_else(|_| vec![data.clone()])
        }
        other => {
            // Single entry: {type: ..., message: ...}
            vec![other.clone()]
        }
    };

    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/tachyon-debug.jsonl")
        .map_err(|e| e.to_string())?;

    for entry in entries {
        let ts = chrono::Utc::now().to_rfc3339();
        let mut line = serde_json::Map::new();
        line.insert("ts".into(), serde_json::Value::String(ts));
        if let Some(t) = entry.get("type") {
            line.insert("type".into(), t.clone());
        }
        if let Some(obj) = entry.as_object() {
            for (k, v) in obj {
                if k != "type" {
                    line.insert(k.clone(), v.clone());
                }
            }
        }
        let s = serde_json::to_string(&line).map_err(|e| e.to_string())?;
        writeln!(f, "{}", s).map_err(|e| e.to_string())?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// API proxy — makes HTTP requests from the Rust side, bypassing WebView CORS.
//
// The Tauri WebView loads from `tauri://localhost` (custom protocol) but the
// backend server runs on `http://localhost:8080`. WebKit treats `tauri://`
// as a non-HTTP origin and blocks cross-origin fetch to `http://localhost`.
// This command proxies all API requests through the native Rust reqwest client,
// which has no origin restrictions.
// ---------------------------------------------------------------------------

/// Response from the api_proxy command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse {
    pub status: u16,
    pub body: serde_json::Value,
    pub headers: std::collections::HashMap<String, String>,
}

/// Proxy an API request through the native HTTP client.
///
/// The WebView calls `invoke("api_proxy", { method, path, body?, headers? })`
/// instead of using `fetch()`. This bypasses CORS entirely because the request
/// originates from the Rust process, not the WebView.
#[tauri::command]
pub async fn api_proxy(
    method: String,
    path: String,
    body: Option<serde_json::Value>,
    headers: Option<std::collections::HashMap<String, String>>,
) -> Result<ApiResponse, String> {
    let base_url = std::env::var("TACHYON_API_URL")
        .unwrap_or_else(|_| "http://localhost:8080".to_string());
    let method_str = method.clone();
    let url = format!("{}{}", base_url.trim_end_matches('/'), path);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let method = match method.to_uppercase().as_str() {
        "GET" => reqwest::Method::GET,
        "POST" => reqwest::Method::POST,
        "PUT" => reqwest::Method::PUT,
        "DELETE" => reqwest::Method::DELETE,
        "PATCH" => reqwest::Method::PATCH,
        "HEAD" => reqwest::Method::HEAD,
        "OPTIONS" => reqwest::Method::OPTIONS,
        other => return Err(format!("Unsupported HTTP method: {}", other)),
    };

    let mut req = client.request(method, &url);

    // Apply custom headers
    if let Some(hdrs) = headers {
        for (key, value) in hdrs {
            // Skip content-length (reqwest sets it automatically)
            if key.to_lowercase() != "content-length" {
                if let Ok(header_name) = reqwest::header::HeaderName::from_bytes(key.as_bytes()) {
                    if let Ok(header_value) = value.parse::<reqwest::header::HeaderValue>() {
                        req = req.header(header_name, header_value);
                    }
                }
            }
        }
    }

    // Apply body for POST/PUT/PATCH
    if let Some(b) = body {
        req = req.json(&b);
    }

    let response = req.send().await.map_err(|e| format!("Request failed: {}", e))?;
    let status = response.status().as_u16();

    tracing::info!("[api_proxy] {} {} → HTTP {}", method_str, url, status);

    // Collect response headers
    let mut resp_headers = std::collections::HashMap::new();
    for (key, value) in response.headers() {
        if let Ok(v) = value.to_str() {
            resp_headers.insert(key.to_string(), v.to_string());
        }
    }

    // Parse response body as JSON (fallback to raw string)
    let body_text = response
        .text()
        .await
        .map_err(|e| format!("Failed to read response body: {}", e))?;
    let body_val = body_text
        .parse::<serde_json::Value>()
        .unwrap_or_else(|_| {
            tracing::warn!("[api_proxy] response body not JSON ({} bytes), first 200: {}", body_text.len(), &body_text[..body_text.len().min(200)]);
            serde_json::Value::String(body_text.clone())
        });

    Ok(ApiResponse {
        status,
        body: body_val,
        headers: resp_headers,
    })
}
