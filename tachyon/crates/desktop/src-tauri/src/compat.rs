use tauri::{AppHandle, Emitter, Manager};

#[allow(dead_code)]
pub const TAURI_VERSION: u32 = 2;

#[allow(dead_code)]
pub fn emit_event<T: serde::Serialize + Clone>(
    app: &AppHandle,
    event_name: &str,
    payload: &T,
) -> Result<(), tauri::Error> {
    app.emit(event_name, payload)
}

#[allow(dead_code)]
pub fn get_main_window(app: &AppHandle) -> Option<tauri::WebviewWindow> {
    app.get_webview_window("main")
}

#[allow(dead_code)]
pub fn app_data_dir(app: &AppHandle) -> Result<std::path::PathBuf, tauri::Error> {
    app.path().app_data_dir()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tauri_version_macro() {
        assert_eq!(TAURI_VERSION, 2);
    }

    #[test]
    fn test_tauri_version_is_u32() {
        let _v: u32 = TAURI_VERSION;
    }
}
