use chrono::Utc;
use serde_json;
use std::path::PathBuf;
use tachyon_core::{ErrorCategory, TachyonError};
use tachyon_desktop_app::*;
use tempfile::TempDir;

// ============================================================================
// 1. Command Registration — Type Verification (compile-time via direct calls)
// ============================================================================

#[test]
fn test_desktop_state_type_is_constructible_and_serializable() {
    let state = DesktopState::new("https://api.example.com");
    let json = serde_json::to_string(&state).expect("DesktopState must serialize for IPC");
    assert!(json.contains("https://api.example.com"));
    assert!(json.contains("\"auth_token\":null"));
}

#[test]
fn test_connection_status_all_variants_are_serializable() {
    let variants = vec![
        (ConnectionStatus::Disconnected, "Disconnected"),
        (ConnectionStatus::Connecting, "Connecting"),
        (ConnectionStatus::Connected, "Connected"),
        (ConnectionStatus::Error, "Error"),
    ];
    for (status, expected_display) in variants {
        let json = serde_json::to_string(&status).expect("ConnectionStatus must serialize");
        assert!(
            json.contains(expected_display),
            "Missing {expected_display} in {json}"
        );
        assert_eq!(format!("{status}"), expected_display);
    }
}

#[test]
fn test_desktop_event_all_variants_serialize_to_tagged_json() {
    use tachyon_desktop_app::NotificationLevel;

    let events = vec![
        DesktopEvent::connection_status_changed(ConnectionStatus::Connected),
        DesktopEvent::auth_status_changed(true, Some("user-1".into())),
        DesktopEvent::document_updated("doc-42", 7),
        DesktopEvent::file_changed("/notes.md", tachyon_desktop_app::FileChangeKind::Modified),
        DesktopEvent::notification(NotificationLevel::Warning, "Heads up", "Disk low"),
    ];
    for event in &events {
        let json = serde_json::to_string(event).expect("DesktopEvent must serialize for IPC");
        let v: serde_json::Value = serde_json::from_str(&json).expect("Must deserialize back");
        assert!(
            v.get("type").is_some(),
            "DesktopEvent JSON must have 'type' tag"
        );
        assert!(
            v.get("data").is_some(),
            "DesktopEvent JSON must have 'data' field"
        );
    }
}

// ============================================================================
// 2. Request / Response Serialization
// ============================================================================

#[test]
fn test_desktop_state_json_roundtrip_preserves_all_fields() {
    let state = DesktopState {
        server_url: "https://tachyon.example.com".into(),
        auth_token: Some("tok_abc123".into()),
        user_id: Some("usr_42".into()),
        repository_path: Some(PathBuf::from("/home/user/vault")),
        auto_sync_enabled: false,
        ..Default::default()
    };

    let json = serde_json::to_string(&state).unwrap();
    let deserialized: DesktopState = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.server_url, "https://tachyon.example.com");
    assert_eq!(deserialized.auth_token, Some("tok_abc123".into()));
    assert_eq!(deserialized.user_id, Some("usr_42".into()));
    assert_eq!(
        deserialized.repository_path,
        Some(PathBuf::from("/home/user/vault"))
    );
    assert!(!deserialized.auto_sync_enabled);
}

#[test]
fn test_vault_entry_json_roundtrip_with_all_fields() {
    let entry = VaultEntry {
        name: "README.md".into(),
        path: "/vault/README.md".into(),
        is_directory: false,
        extension: Some("md".into()),
        size: 2048,
        modified: Some("2025-06-15T08:30:00Z".into()),
    };

    let json = serde_json::to_string(&entry).unwrap();
    let deserialized: VaultEntry = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.name, "README.md");
    assert_eq!(deserialized.extension, Some("md".into()));
    assert_eq!(deserialized.size, 2048);
    assert!(!deserialized.is_directory);
    assert_eq!(deserialized.modified, Some("2025-06-15T08:30:00Z".into()));
}

#[test]
fn test_markdown_file_json_roundtrip_with_unicode_content() {
    let file = MarkdownFile {
        path: "/vault/日本語.md".into(),
        content: "# こんにちは\n\nHello 世界 🌍".into(),
        filename: "日本語.md".into(),
    };

    let json = serde_json::to_string(&file).unwrap();
    let deserialized: MarkdownFile = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.content, "# こんにちは\n\nHello 世界 🌍");
    assert_eq!(deserialized.filename, "日本語.md");
}

#[test]
fn test_sync_config_json_roundtrip_with_custom_values() {
    let config = SyncConfig {
        auto_sync_enabled: false,
        sync_interval_seconds: 60,
        max_queue_size: 500,
        commit_message_template: "docs: update {filename}".into(),
    };

    let json = serde_json::to_string(&config).unwrap();
    let deserialized: SyncConfig = serde_json::from_str(&json).unwrap();

    assert!(!deserialized.auto_sync_enabled);
    assert_eq!(deserialized.sync_interval_seconds, 60);
    assert_eq!(deserialized.max_queue_size, 500);
    assert_eq!(
        deserialized.commit_message_template,
        "docs: update {filename}"
    );
}

#[test]
fn test_file_dialog_types_serialize_correctly() {
    let result = FileDialogResult {
        paths: vec!["/home/user/doc.md".into(), "/home/user/notes.md".into()],
        canceled: false,
    };
    let json = serde_json::to_string(&result).unwrap();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["paths"].as_array().unwrap().len(), 2);
    assert_eq!(v["canceled"], false);

    let write_result = FileWriteResult {
        path: "/tmp/out.md".into(),
        bytes_written: 1024,
    };
    let json = serde_json::to_string(&write_result).unwrap();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["bytes_written"], 1024);
}

// ============================================================================
// 3. Error Propagation
// ============================================================================

#[test]
fn test_tachyon_error_full_ipc_serialization() {
    let error = TachyonError::validation("FIELD_MISSING", "Required field 'title' is empty")
        .with_context("POST /api/v1/documents")
        .with_source("serde validation");

    let json = serde_json::to_string(&error).unwrap();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert_eq!(v["category"], "Validation");
    assert_eq!(v["code"], "FIELD_MISSING");
    assert!(v["message"].as_str().unwrap().contains("title"));
    assert_eq!(v["context"], "POST /api/v1/documents");
    assert_eq!(v["source"], "serde validation");
}

#[test]
fn test_tachyon_error_roundtrip_preserves_semantics() {
    let original = TachyonError::git("PUSH_ERROR", "remote rejected: permission denied")
        .with_context("auto-sync on repo /vault");

    let json = serde_json::to_string(&original).unwrap();
    let restored: TachyonError = serde_json::from_str(&json).unwrap();

    assert_eq!(restored.category, ErrorCategory::Git);
    assert_eq!(restored.code, "PUSH_ERROR");
    assert_eq!(restored.message, "remote rejected: permission denied");
    assert_eq!(restored.context, original.context);
}

#[test]
fn test_all_error_categories_are_ipc_safe() {
    let categories = [
        (ErrorCategory::Storage, "Storage"),
        (ErrorCategory::Authentication, "Authentication"),
        (ErrorCategory::Authorization, "Authorization"),
        (ErrorCategory::Rendering, "Rendering"),
        (ErrorCategory::Git, "Git"),
        (ErrorCategory::Validation, "Validation"),
        (ErrorCategory::Network, "Network"),
        (ErrorCategory::Configuration, "Configuration"),
        (ErrorCategory::Internal, "Internal"),
    ];
    for (cat, serialized_name) in categories {
        let error = TachyonError::new(cat, "TEST", "test message");
        let json = serde_json::to_string(&error).unwrap();
        assert!(
            json.contains(serialized_name),
            "Category {serialized_name} missing from JSON: {json}"
        );
    }
}

#[test]
fn test_tachyon_error_not_found_serializes_for_filesystem_commands() {
    let error = TachyonError::not_found("File: /vault/missing.md");
    let json = serde_json::to_string(&error).unwrap();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert_eq!(v["code"], "NOT_FOUND");
    assert!(v["message"].as_str().unwrap().contains("missing.md"));
}

// ============================================================================
// 4. State Management
// ============================================================================

#[test]
fn test_state_manager_default_values_match_ipc_contract() {
    let manager = DesktopStateManager::new(DesktopState::default());
    let state = manager.get_state().unwrap();

    assert_eq!(state.server_url, "http://localhost:8080");
    assert!(state.auth_token.is_none());
    assert!(state.user_id.is_none());
    assert!(state.repository_path.is_none());
    assert_eq!(state.connection_status, ConnectionStatus::Disconnected);
    assert!(state.auto_sync_enabled);
    assert!(!manager.is_authenticated().unwrap());
    assert!(!manager.is_connected().unwrap());
    assert!(!manager.has_repository().unwrap());
}

#[test]
fn test_state_manager_server_url_update_reflects_in_state() {
    let manager = DesktopStateManager::new(DesktopState::default());
    manager
        .set_server_url("https://tachyon.example.com")
        .unwrap();

    let state = manager.get_state().unwrap();
    assert_eq!(state.server_url, "https://tachyon.example.com");

    manager.set_server_url("http://localhost:3000").unwrap();
    let state = manager.get_state().unwrap();
    assert_eq!(state.server_url, "http://localhost:3000");
}

#[test]
fn test_state_manager_auth_flow_integration() {
    let manager = DesktopStateManager::new(DesktopState::default());

    assert!(!manager.is_authenticated().unwrap());

    manager
        .set_auth_token(Some("jwt_token_abc".into()))
        .unwrap();
    assert!(!manager.is_authenticated().unwrap());

    manager.set_user_id(Some("usr_42".into())).unwrap();
    assert!(manager.is_authenticated().unwrap());

    manager.set_auth_token(None).unwrap();
    assert!(!manager.is_authenticated().unwrap());
}

#[test]
fn test_state_manager_connection_status_lifecycle() {
    let manager = DesktopStateManager::new(DesktopState::default());

    manager
        .set_connection_status(ConnectionStatus::Connecting)
        .unwrap();
    assert!(!manager.is_connected().unwrap());

    manager
        .set_connection_status(ConnectionStatus::Connected)
        .unwrap();
    assert!(manager.is_connected().unwrap());

    manager
        .set_connection_status(ConnectionStatus::Error)
        .unwrap();
    assert!(!manager.is_connected().unwrap());

    manager
        .set_connection_status(ConnectionStatus::Disconnected)
        .unwrap();
    assert!(!manager.is_connected().unwrap());
}

#[test]
fn test_state_manager_repository_path_and_has_repository() {
    let manager = DesktopStateManager::new(DesktopState::default());
    assert!(!manager.has_repository().unwrap());

    manager
        .set_repository_path(Some(PathBuf::from("/home/user/vault")))
        .unwrap();
    assert!(manager.has_repository().unwrap());

    let state = manager.get_state().unwrap();
    assert_eq!(
        state.repository_path,
        Some(PathBuf::from("/home/user/vault"))
    );
}

#[test]
fn test_state_manager_auto_sync_toggle() {
    let manager = DesktopStateManager::new(DesktopState::default());

    let state = manager.get_state().unwrap();
    assert!(state.auto_sync_enabled);

    manager.set_auto_sync(false).unwrap();
    assert!(!manager.get_state().unwrap().auto_sync_enabled);

    manager.set_auto_sync(true).unwrap();
    assert!(manager.get_state().unwrap().auto_sync_enabled);
}

#[test]
fn test_state_manager_last_sync_updates_on_call() {
    let manager = DesktopStateManager::new(DesktopState::default());

    let state = manager.get_state().unwrap();
    assert!(state.last_sync.is_none());

    manager.update_last_sync().unwrap();

    let state = manager.get_state().unwrap();
    assert!(state.last_sync.is_some());
    let sync_time = state.last_sync.unwrap();
    assert!(sync_time <= Utc::now());
}

#[test]
fn test_state_manager_serializable_for_get_state_command() {
    let state = DesktopState {
        server_url: "https://api.example.com".into(),
        auth_token: Some("tok".into()),
        user_id: Some("uid".into()),
        repository_path: Some(PathBuf::from("/vault")),
        ..Default::default()
    };

    let manager = DesktopStateManager::new(state);
    let retrieved = manager.get_state().unwrap();

    let json = serde_json::to_string(&retrieved).unwrap();
    assert!(json.contains("api.example.com"));
    assert!(json.contains("tok"));
    assert!(json.contains("uid"));
    assert!(json.contains("/vault"));
}

// ============================================================================
// 5. Sync Engine
// ============================================================================

#[tokio::test]
async fn test_auto_sync_queue_file_change_increments_size() {
    let manager = AutoSyncManager::new(SyncConfig::default());
    assert_eq!(manager.get_queue_size().await, 0);

    manager.queue_file_change("/vault/notes.md").await.unwrap();
    assert_eq!(manager.get_queue_size().await, 1);

    manager.queue_file_change("/vault/README.md").await.unwrap();
    assert_eq!(manager.get_queue_size().await, 2);
}

#[tokio::test]
async fn test_auto_sync_clear_queue_empties_pending_items() {
    let manager = AutoSyncManager::new(SyncConfig::default());

    manager.queue_file_change("/a.md").await.unwrap();
    manager.queue_file_change("/b.md").await.unwrap();
    assert_eq!(manager.get_queue_size().await, 2);

    manager.clear_queue().await;
    assert_eq!(manager.get_queue_size().await, 0);
}

#[tokio::test]
async fn test_auto_sync_sync_status_reflects_state() {
    let manager = AutoSyncManager::new(SyncConfig::default());
    let status = manager.get_sync_status().await;
    assert_eq!(serde_json::to_string(&status).unwrap(), "\"Idle\"");
}

#[test]
fn test_auto_sync_config_defaults_are_sensible() {
    let config = SyncConfig::default();
    assert!(config.auto_sync_enabled);
    assert_eq!(config.sync_interval_seconds, 30);
    assert_eq!(config.max_queue_size, 100);
    assert!(config.commit_message_template.contains("{filename}"));
}

#[test]
fn test_commit_queue_entry_json_roundtrip() {
    let entry = CommitQueueEntry {
        id: "cq-001".into(),
        path: "/vault/notes.md".into(),
        message: "Auto-sync: Update notes.md".into(),
        timestamp: Utc::now(),
        committed: false,
    };

    let json = serde_json::to_string(&entry).unwrap();
    let deserialized: CommitQueueEntry = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.id, "cq-001");
    assert_eq!(deserialized.path, "/vault/notes.md");
    assert!(!deserialized.committed);
    assert_eq!(deserialized.message, entry.message);
}

#[test]
fn test_sync_result_json_roundtrip_with_error() {
    let result_json = serde_json::json!({
        "status": "Failed",
        "commits_made": 1,
        "files_synced": 1,
        "error": "Permission denied on push"
    });
    let result: SyncResult = serde_json::from_value(result_json.clone()).unwrap();

    let json = serde_json::to_string(&result).unwrap();
    let deserialized: SyncResult = serde_json::from_str(&json).unwrap();

    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["status"], "Failed");
    assert_eq!(v["commits_made"], 1);
    assert_eq!(v["error"], "Permission denied on push");

    assert_eq!(
        serde_json::to_value(&deserialized).unwrap()["status"],
        result_json["status"]
    );
}

#[test]
fn test_auto_sync_manager_initialize_repo_on_disk() {
    let tmp = TempDir::new().unwrap();
    let repo_path = tmp.path().join("vault");

    let mut manager = AutoSyncManager::new(SyncConfig::default());
    manager.set_repository_path(repo_path.clone());
    manager.initialize_repository().unwrap();

    assert!(
        repo_path.join(".git").exists(),
        "initialize_repository should create .git directory"
    );
}

#[tokio::test]
async fn test_auto_sync_queue_evicts_oldest_when_full() {
    let config = SyncConfig {
        max_queue_size: 3,
        ..Default::default()
    };
    let manager = AutoSyncManager::new(config);

    manager.queue_file_change("a.md").await.unwrap();
    manager.queue_file_change("b.md").await.unwrap();
    manager.queue_file_change("c.md").await.unwrap();
    assert_eq!(manager.get_queue_size().await, 3);

    manager.queue_file_change("d.md").await.unwrap();
    assert_eq!(manager.get_queue_size().await, 3);
}

// ============================================================================
// 6. Filesystem Operations (type-level + tempdir patterns)
// ============================================================================

#[test]
fn test_filesystem_vault_entry_sorts_directories_first() {
    let entries = vec![
        VaultEntry {
            name: "z-last.md".into(),
            path: "/vault/z-last.md".into(),
            is_directory: false,
            extension: Some("md".into()),
            size: 10,
            modified: None,
        },
        VaultEntry {
            name: "subdir".into(),
            path: "/vault/subdir".into(),
            is_directory: true,
            extension: None,
            size: 0,
            modified: None,
        },
        VaultEntry {
            name: "a-first.md".into(),
            path: "/vault/a-first.md".into(),
            is_directory: false,
            extension: Some("md".into()),
            size: 5,
            modified: None,
        },
    ];

    let json = serde_json::to_string(&entries).unwrap();
    let deserialized: Vec<VaultEntry> = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.len(), 3);
}

#[test]
fn test_filesystem_markdown_file_type_with_special_chars() {
    let file = MarkdownFile {
        path: "/vault/path with spaces/file (1).md".into(),
        content: "Content with \"quotes\" and \\backslashes\\".into(),
        filename: "file (1).md".into(),
    };

    let json = serde_json::to_string(&file).unwrap();
    let deserialized: MarkdownFile = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.path, file.path);
    assert_eq!(deserialized.content, file.content);
}

#[test]
fn test_filesystem_file_content_type_for_ipc_read_response() {
    let content = FileContent {
        path: "/vault/notes.md".into(),
        content: "# Title\n\nBody with unicode: café".into(),
        encoding: "utf-8".into(),
    };

    let json = serde_json::to_string(&content).unwrap();
    let deserialized: FileContent = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.path, "/vault/notes.md");
    assert!(deserialized.content.contains("café"));
    assert_eq!(deserialized.encoding, "utf-8");
}

#[test]
fn test_filesystem_dialog_result_canceled_state() {
    let canceled = FileDialogResult {
        paths: vec![],
        canceled: true,
    };
    let selected = FileDialogResult {
        paths: vec!["/home/user/file.md".into()],
        canceled: false,
    };

    let json_canceled = serde_json::to_string(&canceled).unwrap();
    let json_selected = serde_json::to_string(&selected).unwrap();

    let v_canceled: serde_json::Value = serde_json::from_str(&json_canceled).unwrap();
    let v_selected: serde_json::Value = serde_json::from_str(&json_selected).unwrap();

    assert_eq!(v_canceled["canceled"], true);
    assert_eq!(v_canceled["paths"].as_array().unwrap().len(), 0);
    assert_eq!(v_selected["canceled"], false);
    assert_eq!(v_selected["paths"].as_array().unwrap().len(), 1);
}
