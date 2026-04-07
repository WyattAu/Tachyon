// Tauri command handlers
// Implements IPC commands that can be invoked from the WebView

use serde::{Deserialize, Serialize};
use tauri::{State, AppHandle};
use tachyon_core::TachyonError;

use crate::state::{DesktopStateManager, DesktopState, ConnectionStatus};
use crate::events::EventEmitter;
use crate::file_dialog::{FileDialogManager, FileDialogOptions, FileContent, FileWriteResult};
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

/// Server configuration request
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfigRequest {
    /// Server URL
    pub server_url: String,
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
    state.get_state()
        .map_err(|e| e.to_string())
}

/// Set server URL
#[tauri::command]
pub async fn set_server_url(
    url: String,
    state: State<'_, DesktopStateManager>,
    app: AppHandle,
) -> Result<(), String> {
    state.set_server_url(url.clone())
        .map_err(|e| e.to_string())?;

    // Emit connection status changed event
    let emitter = EventEmitter::new(app);
    emitter.emit_connection_status_changed(ConnectionStatus::Connecting)
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
    let server_url = state.get_state()
        .map(|s| s.server_url.clone())
        .unwrap_or_else(|_| "http://localhost:8080".to_string());

    // Attempt to authenticate with the server
    match authenticate_with_server(&server_url, &request).await {
        Ok((token, user_id)) => {
            // Update state with authentication info
            state.set_auth_token(Some(token.clone()))
                .map_err(|e| e.to_string())?;
            state.set_user_id(Some(user_id.clone()))
                .map_err(|e| e.to_string())?;
            state.set_connection_status(ConnectionStatus::Connected)
                .map_err(|e| e.to_string())?;

            // Emit authentication status changed event
            let emitter = EventEmitter::new(app);
            emitter.emit_auth_status_changed(true, Some(user_id.clone()))
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
        access_token: Option<String>,  // Alternative field names
        id: Option<String>,
    }
    
    let body: AuthResponseBody = response.json::<AuthResponseBody>().await
        .map_err(|e| format!("Failed to parse authentication response: {}", e))?;
    
    // Extract token (check multiple possible field names)
    let token = body.token
        .or(body.access_token)
        .ok_or_else(|| "No token in authentication response".to_string())?;
    
    // Extract user ID (check multiple possible field names)
    let user_id = body.user_id
        .or(body.id)
        .unwrap_or_else(|| format!("user_{}", uuid::Uuid::new_v4()));
    
    Ok((token, user_id))
}

/// Authenticate in local-first mode (offline)
///
/// Creates a local session without requiring server connectivity.
/// Useful for offline operation and personal use cases.
///
/// # Arguments
/// * `username` - Local username
///
/// # Returns
/// Result containing (token, user_id) or error message
#[allow(dead_code)]
fn authenticate_local_first(username: &str) -> Result<(String, String), String> {
    // Generate a deterministic user ID based on username
    use sha2::{Digest, Sha256};
    
    let mut hasher = Sha256::new();
    hasher.update(username.as_bytes());
    hasher.update(b"tachyon-local-user");
    let hash = hasher.finalize();
    let user_id = format!("local_{}", hex::encode(&hash[..16]));
    
    // Generate a local session token
    let token = format!("local_{}", uuid::Uuid::new_v4());
    
    tracing::info!("Created local-first session for user: {}", username);
    
    Ok((token, user_id))
}

/// Authenticate with server or fall back to local-first mode
///
/// # Arguments
/// * `server_url` - Server base URL  
/// * `request` - Authentication request
/// * `prefer_local` - Whether to prefer local-first mode
///
/// # Returns
/// Result containing (token, user_id, is_local) or error message
#[allow(dead_code)]
pub async fn authenticate_with_fallback(
    server_url: &str,
    request: &AuthRequest,
    prefer_local: bool,
) -> Result<(String, String, bool), String> {
    // If local-first mode is preferred, skip server authentication
    if prefer_local {
        let (token, user_id) = authenticate_local_first(&request.username)?;
        return Ok((token, user_id, true));
    }
    
    // Try server authentication first
    match authenticate_with_server(server_url, request).await {
        Ok((token, user_id)) => Ok((token, user_id, false)),
        Err(e) => {
            tracing::warn!("Server authentication failed, falling back to local-first: {}", e);
            
            // Fall back to local-first mode
            let (token, user_id) = authenticate_local_first(&request.username)?;
            Ok((token, user_id, true))
        }
    }
}

/// Logout
#[tauri::command]
pub async fn logout(
    state: State<'_, DesktopStateManager>,
    app: AppHandle,
) -> Result<(), String> {
    state.set_auth_token(None)
        .map_err(|e| e.to_string())?;
    state.set_user_id(None)
        .map_err(|e| e.to_string())?;
    state.set_connection_status(ConnectionStatus::Disconnected)
        .map_err(|e| e.to_string())?;

    // Emit authentication status changed event
    let emitter = EventEmitter::new(app);
    emitter.emit_auth_status_changed(false, None)
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
    manager.open_file_dialog(options).await
        .map_err(|e| e.to_string())
}

/// Save file dialog
#[tauri::command]
pub async fn save_file_dialog(
    options: FileDialogOptions,
    app: AppHandle,
) -> Result<crate::file_dialog::FileDialogResult, String> {
    let manager = FileDialogManager::new(app);
    manager.save_file_dialog(options).await
        .map_err(|e| e.to_string())
}

/// Read file content
#[tauri::command]
pub async fn read_file(
    path: String,
    app: AppHandle,
) -> Result<FileContent, String> {
    let manager = FileDialogManager::new(app);
    manager.read_file(path).await
        .map_err(|e| e.to_string())
}

/// Write file content
#[tauri::command]
pub async fn write_file(
    path: String,
    content: String,
    app: AppHandle,
) -> Result<FileWriteResult, String> {
    let manager = FileDialogManager::new(app);
    manager.write_file(path, content).await
        .map_err(|e| e.to_string())
}

/// Check if file exists
#[tauri::command]
pub async fn file_exists(
    path: String,
    app: AppHandle,
) -> Result<bool, String> {
    let manager = FileDialogManager::new(app);
    manager.file_exists(path).await
        .map_err(|e| e.to_string())
}

/// Delete file
#[tauri::command]
pub async fn delete_file(
    path: String,
    app: AppHandle,
) -> Result<(), String> {
    let manager = FileDialogManager::new(app);
    manager.delete_file(path).await
        .map_err(|e| e.to_string())
}

/// Create directory
#[tauri::command]
pub async fn create_directory(
    path: String,
    app: AppHandle,
) -> Result<(), String> {
    let manager = FileDialogManager::new(app);
    manager.create_directory(path).await
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
    state.set_repository_path(Some(path))
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
    let repo_path = state.get_state()
        .map(|s| s.repository_path.clone())
        .map_err(|e| e.to_string())?;

    let path = repo_path
        .ok_or_else(|| "Repository path not configured. Call set_repository_path first.".to_string())?;

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
        git2::Repository::init(&path_for_thread)
            .map_err(|e| TachyonError::git("INIT_ERROR", format!("Failed to initialize repository: {}", e)))?;
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
    let result = sync_manager.commit_pending().await
        .map_err(|e| e.to_string())?;

    // Emit sync status changed event
    let emitter = EventEmitter::new(app);
    use crate::events::SyncStatus as EventSyncStatus;
    let event_status = match result.status {
        crate::sync::SyncStatus::Success => EventSyncStatus::Success,
        crate::sync::SyncStatus::Failed => EventSyncStatus::Failed,
        _ => EventSyncStatus::Idle,
    };
    emitter.emit_sync_status_changed(event_status, result.error.clone())
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

    let result = sync_manager.push_to_remote(remote_name, branch_name).await
        .map_err(|e| e.to_string())?;

    // Emit sync status changed event
    let emitter = EventEmitter::new(app);
    use crate::events::SyncStatus as EventSyncStatus;
    emitter.emit_sync_status_changed(EventSyncStatus::Success, result.error.clone())
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

    let result = sync_manager.pull_from_remote(remote_name, branch_name).await
        .map_err(|e| e.to_string())?;

    // Emit sync status changed event
    let emitter = EventEmitter::new(app);
    use crate::events::SyncStatus as EventSyncStatus;
    emitter.emit_sync_status_changed(EventSyncStatus::Success, result.error.clone())
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
pub async fn get_queue_size(
    sync_manager: State<'_, AutoSyncManager>,
) -> Result<usize, String> {
    Ok(sync_manager.get_queue_size().await)
}

/// Clear commit queue
#[tauri::command]
pub async fn clear_queue(
    sync_manager: State<'_, AutoSyncManager>,
) -> Result<(), String> {
    sync_manager.clear_queue().await;
    Ok(())
}

/// Enable auto-sync
#[tauri::command]
pub async fn enable_auto_sync(
    state: State<'_, DesktopStateManager>,
) -> Result<(), String> {
    state.set_auto_sync(true)
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Disable auto-sync
#[tauri::command]
pub async fn disable_auto_sync(
    state: State<'_, DesktopStateManager>,
) -> Result<(), String> {
    state.set_auto_sync(false)
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Queue file change for commit
#[tauri::command]
pub async fn queue_file_change(
    path: String,
    sync_manager: State<'_, AutoSyncManager>,
) -> Result<(), String> {
    sync_manager.queue_file_change(path).await
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
pub async fn is_authenticated(
    state: State<'_, DesktopStateManager>,
) -> Result<bool, String> {
    state.is_authenticated()
        .map_err(|e| e.to_string())
}

/// Check connection status
#[tauri::command]
pub async fn is_connected(
    state: State<'_, DesktopStateManager>,
) -> Result<bool, String> {
    state.is_connected()
        .map_err(|e| e.to_string())
}

/// Check if repository is configured
#[tauri::command]
pub async fn has_repository(
    state: State<'_, DesktopStateManager>,
) -> Result<bool, String> {
    state.has_repository()
        .map_err(|e| e.to_string())
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
    fn test_server_config_request() {
        let request = ServerConfigRequest {
            server_url: "https://api.tachyon.io".to_string(),
        };

        assert_eq!(request.server_url, "https://api.tachyon.io");
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
