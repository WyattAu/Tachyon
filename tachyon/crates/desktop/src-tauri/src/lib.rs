// Tauri Desktop Application Library
// IPC bridge between WebView and Tauri backend

// Module declarations
mod state;
mod events;
mod file_dialog;
mod sync;
mod commands;

// Re-export public API
pub use state::{DesktopState, DesktopStateManager, ConnectionStatus};
pub use events::{DesktopEvent, EventEmitter, SyncStatus, RepositoryStatus, FileChangeKind, NotificationLevel};
pub use file_dialog::{FileDialogManager, FileDialogOptions, FileDialogResult, FileContent, FileWriteResult};
pub use sync::{AutoSyncManager, SyncConfig, SyncResult, CommitQueueEntry};

use tauri::Manager;

/// Run the Tauri application
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_log::Builder::default().build())
        .setup(|app| {
            // Initialize state manager
            let state_manager = DesktopStateManager::new(DesktopState::default());
            let sync_manager = AutoSyncManager::new(SyncConfig::default());
            
            app.manage(state_manager);
            app.manage(sync_manager);
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_status_display() {
        assert_eq!(format!("{}", ConnectionStatus::Connected), "Connected");
        assert_eq!(format!("{}", ConnectionStatus::Disconnected), "Disconnected");
        assert_eq!(format!("{}", ConnectionStatus::Connecting), "Connecting");
        assert_eq!(format!("{}", ConnectionStatus::Error), "Error");
    }
}
