//! Unit tests for core types
//!
//! Tests for DocumentId, UserId, SessionId, RepositoryId, NodeId, EdgeId, and TagId types.

#[allow(unused_imports)]
use tachyon_core::{
    generate_document_id, generate_edge_id, generate_node_id, generate_repository_id,
    generate_session_id, generate_tag_id, generate_user_id, Id, IdParseError,
};

#[test]
fn test_id_new_generates_unique_ids() {
    let id1 = Id::new();
    let id2 = Id::new();
    assert_ne!(id1, id2);
    assert!(!id1.is_nil());
    assert!(!id2.is_nil());
}

#[test]
fn test_id_validates_v7() {
    let id = Id::new();
    assert!(id.validate_v7());
}

#[test]
fn test_nil_id() {
    let nil = Id::from_uuid(uuid::Uuid::nil());
    assert!(nil.is_nil());
    assert!(!nil.validate_v7());
}

#[test]
fn test_id_from_and_to_str() {
    let id = Id::new();
    let s = id.as_str();
    let parsed = Id::parse_str(&s).expect("parse should succeed");
    assert_eq!(id, parsed);
}

#[test]
fn test_id_from_str_trait() {
    let id = Id::new();
    let s = id.as_str();
    let parsed: Id = s.parse().expect("FromStr should succeed");
    assert_eq!(id, parsed);
}

#[test]
fn test_id_parse_invalid() {
    let err = Id::parse_str("not-a-uuid").unwrap_err();
    assert!(matches!(err, IdParseError::InvalidFormat(_)));
}

#[test]
fn test_id_display() {
    let id = Id::new();
    let display = format!("{}", id);
    let parsed: Id = display.parse().expect("should round-trip");
    assert_eq!(id, parsed);
}

#[test]
fn test_id_clone_and_copy() {
    let id = Id::new();
    let copied = id;
    assert_eq!(id, copied);
    let cloned = id;
    assert_eq!(id, cloned);
}

#[test]
fn test_id_ordering() {
    let id1 = Id::new();
    let id2 = Id::new();
    assert_ne!(id1.cmp(&id2), std::cmp::Ordering::Equal);
}

#[test]
fn test_id_serde_roundtrip() {
    let id = Id::new();
    let json = serde_json::to_string(&id).expect("serialize should succeed");
    let de: Id = serde_json::from_str(&json).expect("deserialize should succeed");
    assert_eq!(id, de);
}

#[test]
fn test_type_aliases_are_distinct() {
    let doc_id: tachyon_core::DocumentId = generate_document_id();
    let user_id: tachyon_core::UserId = generate_user_id();
    let session_id: tachyon_core::SessionId = generate_session_id();
    let repo_id: tachyon_core::RepositoryId = generate_repository_id();
    let node_id: tachyon_core::NodeId = generate_node_id();
    let edge_id: tachyon_core::EdgeId = generate_edge_id();
    let tag_id: tachyon_core::TagId = generate_tag_id();

    assert_ne!(doc_id, user_id);
    assert_ne!(session_id, repo_id);
    assert_ne!(node_id, edge_id);
    assert_ne!(tag_id, doc_id);
    assert!(doc_id.validate_v7());
    assert!(user_id.validate_v7());
    assert!(session_id.validate_v7());
    assert!(repo_id.validate_v7());
    assert!(node_id.validate_v7());
    assert!(edge_id.validate_v7());
    assert!(tag_id.validate_v7());
}

#[test]
fn test_document_id_generate() {
    let id = generate_document_id();
    assert!(!id.is_nil());
    assert!(id.validate_v7());
}

#[test]
fn test_user_id_generate() {
    let id = generate_user_id();
    assert!(!id.is_nil());
}

#[test]
fn test_session_id_generate() {
    let id = generate_session_id();
    assert!(!id.is_nil());
}

#[test]
fn test_repository_id_generate() {
    let id = generate_repository_id();
    assert!(!id.is_nil());
}

#[test]
fn test_node_id_generate() {
    let id = generate_node_id();
    assert!(!id.is_nil());
}

#[test]
fn test_edge_id_generate() {
    let id = generate_edge_id();
    assert!(!id.is_nil());
}

#[test]
fn test_tag_id_generate() {
    let id = generate_tag_id();
    assert!(!id.is_nil());
}

#[test]
fn test_id_default() {
    let id = Id::default();
    assert!(!id.is_nil());
    assert!(id.validate_v7());
}
