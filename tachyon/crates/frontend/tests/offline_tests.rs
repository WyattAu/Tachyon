use tachyon_frontend::offline::{OfflineDocument, OnlineStatus, PendingChange, compute_checksum};
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_offline_document_creation() {
    let doc = OfflineDocument {
        id: "test-1".to_string(),
        title: "Test Document".to_string(),
        content: "Hello world".to_string(),
        updated_at: "2024-01-01T00:00:00Z".to_string(),
        version: 1,
        checksum: compute_checksum("Hello world"),
    };

    assert_eq!(doc.id, "test-1");
    assert_eq!(doc.title, "Test Document");
    assert_eq!(doc.content, "Hello world");
    assert_eq!(doc.version, 1);
}

#[wasm_bindgen_test]
fn test_pending_change_creation() {
    let change = PendingChange {
        id: "change-1".to_string(),
        document_id: "doc-1".to_string(),
        operation: "insert".to_string(),
        payload: r#"{"position":0,"text":"new"}"#.to_string(),
        created_at: "2024-01-01T00:00:00Z".to_string(),
        retry_count: 0,
        version: 1,
        checksum: compute_checksum("test"),
    };

    assert_eq!(change.id, "change-1");
    assert_eq!(change.document_id, "doc-1");
    assert_eq!(change.operation, "insert");
    assert_eq!(change.retry_count, 0);
}

#[wasm_bindgen_test]
fn test_online_status_equality() {
    assert_eq!(OnlineStatus::Online, OnlineStatus::Online);
    assert_eq!(OnlineStatus::Offline, OnlineStatus::Offline);
    assert_eq!(OnlineStatus::Syncing, OnlineStatus::Syncing);
    assert_ne!(OnlineStatus::Online, OnlineStatus::Offline);
    assert_ne!(OnlineStatus::Online, OnlineStatus::Syncing);
    assert_ne!(OnlineStatus::Offline, OnlineStatus::Syncing);
}

#[wasm_bindgen_test]
fn test_compute_checksum_deterministic() {
    let checksum1 = compute_checksum("hello world");
    let checksum2 = compute_checksum("hello world");
    assert_eq!(checksum1, checksum2);
}

#[wasm_bindgen_test]
fn test_compute_checksum_different_inputs() {
    let checksum1 = compute_checksum("hello world");
    let checksum2 = compute_checksum("hello world!");
    assert_ne!(checksum1, checksum2);
}

#[wasm_bindgen_test]
fn test_compute_checksum_empty() {
    let checksum = compute_checksum("");
    assert!(!checksum.is_empty());
}

#[wasm_bindgen_test]
fn test_offline_document_serialization() {
    let doc = OfflineDocument {
        id: "test-1".to_string(),
        title: "Test".to_string(),
        content: "Content".to_string(),
        updated_at: "2024-01-01".to_string(),
        version: 1,
        checksum: compute_checksum("Content"),
    };

    let json = serde_json::to_string(&doc).unwrap();
    let deserialized: OfflineDocument = serde_json::from_str(&json).unwrap();

    assert_eq!(doc.id, deserialized.id);
    assert_eq!(doc.title, deserialized.title);
    assert_eq!(doc.content, deserialized.content);
    assert_eq!(doc.version, deserialized.version);
    assert_eq!(doc.checksum, deserialized.checksum);
}

#[wasm_bindgen_test]
fn test_pending_change_serialization() {
    let change = PendingChange {
        id: "change-1".to_string(),
        document_id: "doc-1".to_string(),
        operation: "insert".to_string(),
        payload: "test".to_string(),
        created_at: "2024-01-01".to_string(),
        retry_count: 0,
        version: 1,
        checksum: compute_checksum("test"),
    };

    let json = serde_json::to_string(&change).unwrap();
    let deserialized: PendingChange = serde_json::from_str(&json).unwrap();

    assert_eq!(change.id, deserialized.id);
    assert_eq!(change.document_id, deserialized.document_id);
    assert_eq!(change.operation, deserialized.operation);
    assert_eq!(change.retry_count, deserialized.retry_count);
}
