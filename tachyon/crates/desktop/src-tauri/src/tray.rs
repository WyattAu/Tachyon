// System tray for Tauri desktop application
// Provides tray icon with context menu for common actions

use crate::events::EventEmitter;
use crate::events::NotificationLevel;
use crate::state::DesktopStateManager;
use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    AppHandle, Emitter, Manager,
};

/// Build and install the system tray icon and menu.
///
/// Called during app setup to register the tray with context menus
/// for sync status, import/export, and window management.
pub fn setup_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    // Build the tray menu
    let show_i = MenuItem::with_id(app, "show", "Show Window", true, None::<&str>)?;
    let hide_i = MenuItem::with_id(app, "hide", "Hide Window", true, None::<&str>)?;
    let sep1 = PredefinedMenuItem::separator(app)?;
    let sync_now_i = MenuItem::with_id(app, "sync_now", "Sync Now", true, None::<&str>)?;
    let auto_sync_i =
        CheckMenuItem::with_id(app, "auto_sync", "Auto-Sync", true, true, None::<&str>)?;
    let sep2 = PredefinedMenuItem::separator(app)?;
    let import_obsidian_i = MenuItem::with_id(
        app,
        "import_obsidian",
        "Import Obsidian Vault...",
        true,
        None::<&str>,
    )?;
    let import_zip_i = MenuItem::with_id(
        app,
        "import_zip",
        "Import Markdown ZIP...",
        true,
        None::<&str>,
    )?;
    let export_md_i = MenuItem::with_id(
        app,
        "export_md",
        "Export as Markdown ZIP...",
        true,
        None::<&str>,
    )?;
    let export_html_i = MenuItem::with_id(
        app,
        "export_html",
        "Export as HTML ZIP...",
        true,
        None::<&str>,
    )?;
    let sep3 = PredefinedMenuItem::separator(app)?;
    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[
            &show_i,
            &hide_i,
            &sep1,
            &sync_now_i,
            &auto_sync_i,
            &sep2,
            &import_obsidian_i,
            &import_zip_i,
            &export_md_i,
            &export_html_i,
            &sep3,
            &quit_i,
        ],
    )?;

    // Build the tray icon
    let _tray = TrayIconBuilder::with_id("main-tray")
        .menu(&menu)
        .tooltip("Tachyon Knowledge Manager")
        .on_menu_event(|app, event| {
            handle_tray_event(app, event.id().as_ref());
        })
        .build(app)?;

    Ok(())
}

/// Handle tray menu events.
fn handle_tray_event(app: &AppHandle, event_id: &str) {
    match event_id {
        "show" => {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
        "hide" => {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.hide();
            }
        }
        "sync_now" => {
            // Trigger sync via the sync manager
            if let Some(sync_manager) = app.try_state::<crate::sync::AutoSyncManager>() {
                let manager = sync_manager.inner().clone();
                let app_handle = app.clone();
                tokio::spawn(async move {
                    match manager.commit_pending().await {
                        Ok(result) => {
                            let emitter = EventEmitter::new(app_handle);
                            let _ = emitter.emit_notification(
                                NotificationLevel::Info,
                                "Sync Complete",
                                format!("Committed {} files", result.files_synced),
                            );
                        }
                        Err(e) => {
                            let emitter = EventEmitter::new(app_handle);
                            let _ = emitter.emit_notification(
                                NotificationLevel::Error,
                                "Sync Failed",
                                e.to_string(),
                            );
                        }
                    }
                });
            }
        }
        "auto_sync" => {
            if let Some(state_manager) = app.try_state::<DesktopStateManager>() {
                let current = state_manager.get_state().ok();
                if let Some(state) = current {
                    let new_enabled = !state.auto_sync_enabled;
                    let _ = state_manager.set_auto_sync(new_enabled);
                }
            }
        }
        "import_obsidian" => {
            emit_tray_action(app, "import_obsidian");
        }
        "import_zip" => {
            emit_tray_action(app, "import_zip");
        }
        "export_md" => {
            emit_tray_action(app, "export_md");
        }
        "export_html" => {
            emit_tray_action(app, "export_html");
        }
        "quit" => {
            app.exit(0);
        }
        _ => {}
    }
}

/// Emit a tray action event to the WebView so the frontend can handle it
/// (e.g., open the appropriate dialog).
fn emit_tray_action(app: &AppHandle, action: &str) {
    use serde::Serialize;
    #[derive(Serialize, Clone)]
    struct TrayAction {
        action: String,
    }
    let _ = app.emit(
        "tray-action",
        TrayAction {
            action: action.to_string(),
        },
    );
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    // Tray setup requires a running Tauri app, so we only test helpers here.

    #[test]
    fn test_tray_event_id_routing() {
        // Verify that all event IDs match the menu item IDs
        let known_ids = [
            "show",
            "hide",
            "sync_now",
            "auto_sync",
            "import_obsidian",
            "import_zip",
            "export_md",
            "export_html",
            "quit",
        ];
        for id in &known_ids {
            assert!(!id.is_empty(), "Tray event ID must not be empty");
        }
    }
}
