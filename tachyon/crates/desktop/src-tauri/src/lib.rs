// Tauri Desktop Application Library
// IPC bridge between WebView and Tauri backend

// Module declarations
mod commands;
mod events;
mod file_dialog;
mod filesystem;
mod import_export;
mod state;
mod sync;
#[cfg(feature = "tray-icon")]
mod tray;

// Re-export public API
pub use events::{
    DesktopEvent, EventEmitter, FileChangeKind, NotificationLevel, RepositoryStatus,
    SyncStatus as EventSyncStatus,
};
pub use file_dialog::{
    FileContent, FileDialogManager, FileDialogOptions, FileDialogResult, FileWriteResult,
};
pub use filesystem::{FileWatchHandle, MarkdownFile, VaultEntry};
pub use state::{ConnectionStatus, DesktopAppState, DesktopState, DesktopStateManager, SyncStatus};
pub use sync::{AutoSyncManager, CommitQueueEntry, SyncConfig, SyncResult};

use std::sync::{Arc, Mutex};
use tauri::Manager;

/// Embedded server state shared between Tauri and the frontend.
#[derive(Default)]
struct EmbeddedServerState {
    /// Port the embedded server is listening on (0 if not started).
    port: u16,
    /// Whether the server has started successfully.
    started: bool,
}

/// Workaround for WebKitGTK DMA-BUF renderer failures on NVIDIA + Wayland.
///
/// On NVIDIA GPUs with KWin/Sway compositors, WebKitGTK's DMA-BUF renderer
/// fails with "Failed to create GBM buffer of size NxM: Invalid argument"
/// because the NVIDIA EGL/GBM implementation doesn't support the specific
/// buffer modifiers WebKit requests. Setting WEBKIT_DISABLE_DMABUF_RENDERER=1
/// forces WebKit to fall back to shared-memory (shm) rendering which works
/// on all GPU/compositor combinations.
///
/// This only sets the env var if it's not already explicitly configured,
/// so users can override with WEBKIT_DISABLE_DMABUF_RENDERER=0 if needed.
fn fix_webkit_dmabuf_on_nvidia() {
    if std::env::var("WEBKIT_DISABLE_DMABUF_RENDERER").is_err() {
        // Detect NVIDIA GPU via the DRM subsystem
        let has_nvidia = std::fs::read_dir("/dev/dri/by-path")
            .map(|entries| {
                entries.flatten().any(|e| {
                    e.file_name()
                        .to_string_lossy()
                        .contains("nvidia")
                })
            })
            .unwrap_or(false);

        // Also check the classic /proc/driver/nvidia path
        let has_nvidia_proc =
            std::path::Path::new("/proc/driver/nvidia").exists();

        if has_nvidia || has_nvidia_proc {
            // SAFETY: This runs before any threads are spawned (single-threaded
            // init phase of the Tauri app). set_var is unsafe in Rust 2024 to
            // prevent data races with concurrent getenv, but we're still in the
            // main thread's sequential setup code.
            unsafe {
                std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
            }
            tracing::info!(
                "NVIDIA GPU detected — set WEBKIT_DISABLE_DMABUF_RENDERER=1 \
                 to avoid DMA-BUF GBM buffer failures on Wayland"
            );
        }
    }
}

/// Run the Tauri application
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    fix_webkit_dmabuf_on_nvidia();

    let embedded_server = Arc::new(Mutex::new(EmbeddedServerState::default()));

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
            // Embedded server
            commands::get_embedded_server_port,
            commands::start_embedded_server,
            commands::stop_embedded_server,
            // Local storage (offline-first)
            commands::get_local_db_stats,
            commands::init_local_database,
            commands::get_local_tags,
            commands::search_local_documents,
            // Sync queue (offline→online reconciliation)
            commands::sync_enqueue,
            commands::sync_queue_summary,
            commands::sync_queue_pending,
            commands::sync_mark_synced,
            commands::sync_mark_failed,
            commands::sync_purge_synced,
            // Offline/online detection
            commands::set_connection_status,
            commands::is_online,
            // Local-first authentication
            commands::authenticate_offline,
            // Filesystem (v2 — vault browsing, file watching)
            commands::read_vault,
            commands::read_markdown_file,
            commands::write_markdown_file,
            commands::list_vault_files,
            commands::watch_directory,
            commands::stop_directory_watch,
            commands::is_directory_watched,
            commands::get_app_data_dir,
            commands::open_path,
        ])
        .manage(embedded_server)
        .setup(move |app| {
            // Initialize state manager
            let state_manager = DesktopStateManager::new(DesktopState::default());
            let sync_manager = AutoSyncManager::new(SyncConfig::default());
            let app_state = DesktopAppState::new();

            app.manage(state_manager);
            app.manage(sync_manager);
            app.manage(app_state);

            // Set up system tray (only when tray-icon feature is enabled)
            #[cfg(feature = "tray-icon")]
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
        assert_eq!(
            format!("{}", ConnectionStatus::Disconnected),
            "Disconnected"
        );
        assert_eq!(format!("{}", ConnectionStatus::Connecting), "Connecting");
        assert_eq!(format!("{}", ConnectionStatus::Error), "Error");
    }
}
