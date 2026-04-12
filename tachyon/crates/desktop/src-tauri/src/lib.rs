// Tauri Desktop Application Library
// IPC bridge between WebView and Tauri backend

// Module declarations
mod state;
mod events;
mod file_dialog;
mod sync;
mod commands;
mod import_export;
mod tray;

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
        .plugin(tauri_plugin_notification::init())
        .invoke_handler(tauri::generate_handler![
            // State & Auth
            commands::get_state,
            commands::set_server_url,
            commands::authenticate,
            commands::logout,
            commands::is_authenticated,
            commands::is_connected,
            commands::has_repository,
            // File dialogs
            commands::open_file_dialog,
            commands::save_file_dialog,
            // File I/O
            commands::read_file,
            commands::write_file,
            commands::file_exists,
            commands::delete_file,
            commands::create_directory,
            // Repository & Git sync
            commands::set_repository_path,
            commands::initialize_repository,
            commands::commit_pending,
            commands::push_to_remote,
            commands::pull_from_remote,
            commands::get_sync_status,
            commands::get_queue_size,
            commands::clear_queue,
            commands::queue_file_change,
            commands::enable_auto_sync,
            commands::disable_auto_sync,
            // File watcher
            commands::start_file_watcher,
            commands::stop_file_watcher,
            commands::is_file_watching,
            // Dialogs
            commands::show_error_dialog,
            commands::show_warning_dialog,
            commands::show_info_dialog,
            // Import/Export
            import_export::import_obsidian_vault,
            import_export::import_markdown_zip,
            import_export::export_markdown_zip,
            import_export::export_html,
        ])
        .setup(|app| {
            // Initialize state manager
            let state_manager = DesktopStateManager::new(DesktopState::default());
            let sync_manager = AutoSyncManager::new(SyncConfig::default());
            
            app.manage(state_manager);
            app.manage(sync_manager);
            
            // Set up system tray
            if let Err(e) = tray::setup_tray(app) {
                tracing::warn!("Failed to set up system tray: {}", e);
                // Non-fatal: tray is optional
            }
            
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
