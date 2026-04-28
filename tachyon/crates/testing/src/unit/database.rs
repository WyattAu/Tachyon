//! Unit tests for database types
//!
//! Tests for DatabaseConfig, SessionRecord validation, RolePermissionMapping,
//! DocumentMetadata serialization, and PolicyRecord parsing.

use chrono::{Duration, Utc};
use tachyon_database::types::*;

#[allow(dead_code)]
fn make_session_record(status: &str, expires_at: chrono::DateTime<Utc>) -> SessionRecord {
    SessionRecord {
        id: "sess-1".to_string(),
        user_id: "user-1".to_string(),
        session_type: "web".to_string(),
        status: status.to_string(),
        token_value: "token-abc".to_string(),
        token_type: "jwt".to_string(),
        ip_address: Some("127.0.0.1".to_string()),
        user_agent: Some("test-agent".to_string()),
        device_info: None,
        created_at: Utc::now() - Duration::hours(1),
        expires_at,
        last_activity: Utc::now(),
    }
}

#[test]
fn test_session_record_is_expired_true() {
    let record = make_session_record("Active", Utc::now() - Duration::hours(1));
    assert!(record.is_expired());
}

#[test]
fn test_session_record_is_expired_false() {
    let record = make_session_record("Active", Utc::now() + Duration::hours(1));
    assert!(!record.is_expired());
}

#[test]
fn test_session_record_is_valid_active_not_expired() {
    let record = make_session_record("Active", Utc::now() + Duration::hours(1));
    assert!(record.is_valid());
}

#[test]
fn test_session_record_invalid_when_expired() {
    let record = make_session_record("Active", Utc::now() - Duration::seconds(1));
    assert!(!record.is_valid());
}

#[test]
fn test_session_record_invalid_when_revoked() {
    let record = make_session_record("Revoked", Utc::now() + Duration::hours(1));
    assert!(!record.is_valid());
}

#[test]
fn test_session_record_invalid_when_status_not_active() {
    let record = make_session_record("Expired", Utc::now() + Duration::hours(1));
    assert!(!record.is_valid());
}

#[test]
fn test_session_record_fields() {
    let record = make_session_record("Active", Utc::now() + Duration::hours(1));
    assert_eq!(record.id, "sess-1");
    assert_eq!(record.user_id, "user-1");
    assert_eq!(record.session_type, "web");
    assert_eq!(record.token_value, "token-abc");
    assert_eq!(record.ip_address.as_deref(), Some("127.0.0.1"));
    assert!(record.device_info.is_none());
}

#[test]
fn test_database_config_defaults() {
    let config = DatabaseConfig::new();
    assert_eq!(config.max_connections, 10);
    assert_eq!(config.min_connections, 0);
    assert_eq!(config.connection_timeout, 30);
    assert!(config.enable_extensions);
    assert!(!config.enable_query_logging);
    assert_eq!(config.schema, "public");
}

#[test]
fn test_database_config_builder() {
    let config = DatabaseConfig::new()
        .with_max_connections(20)
        .with_min_connections(5)
        .with_connection_timeout(60)
        .with_extensions(false)
        .with_query_logging(true)
        .with_schema("tenant1");

    assert_eq!(config.max_connections, 20);
    assert_eq!(config.min_connections, 5);
    assert_eq!(config.connection_timeout, 60);
    assert!(!config.enable_extensions);
    assert!(config.enable_query_logging);
    assert_eq!(config.schema, "tenant1");
}

#[test]
fn test_database_config_default_trait() {
    let config = DatabaseConfig::default();
    assert_eq!(config.max_connections, 10);
}

#[test]
fn test_role_permission_mapping_parse_conditions() {
    let mapping = RolePermissionMapping {
        id: 1,
        role: "admin".to_string(),
        permission: "read".to_string(),
        resource_type: Some("document".to_string()),
        conditions: Some(r#"{"env": "production"}"#.to_string()),
        created_at: Utc::now(),
    };

    let conditions = mapping.parse_conditions().unwrap();
    assert_eq!(conditions.unwrap().get("env").unwrap(), "production");
}

#[test]
fn test_role_permission_mapping_parse_conditions_none() {
    let mapping = RolePermissionMapping {
        id: 1,
        role: "admin".to_string(),
        permission: "read".to_string(),
        resource_type: None,
        conditions: None,
        created_at: Utc::now(),
    };

    let conditions = mapping.parse_conditions().unwrap();
    assert!(conditions.is_none());
}

#[test]
fn test_role_permission_mapping_serialize_conditions() {
    let val = Some(serde_json::json!({"key": "value"}));
    let result = RolePermissionMapping::serialize_conditions(&val).unwrap();
    assert!(result.is_some());
    let parsed: serde_json::Value = serde_json::from_str(result.as_ref().unwrap()).unwrap();
    assert_eq!(parsed.get("key").unwrap(), "value");
}

#[test]
fn test_policy_record_parse_rules() {
    let record = PolicyRecord {
        id: 1,
        name: "Test Policy".to_string(),
        description: None,
        rules: r#"[{"action": "read"}, {"action": "write"}]"#.to_string(),
        effect: "allow".to_string(),
        policy_type: Some("rbac".to_string()),
        created_by: None,
        enabled: true,
        priority: 10,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let rules = record.parse_rules().unwrap();
    assert_eq!(rules.len(), 2);
}

#[test]
fn test_policy_record_serialize_rules() {
    let rules = vec![serde_json::json!({"action": "read"})];
    let result = PolicyRecord::serialize_rules(&rules).unwrap();
    let parsed: Vec<serde_json::Value> = serde_json::from_str(&result).unwrap();
    assert_eq!(parsed.len(), 1);
}

#[test]
fn test_document_metadata_parse_tags() {
    let meta = DocumentMetadata {
        id: "doc-1".to_string(),
        title: "Test".to_string(),
        slug: Some("test".to_string()),
        author_id: "user-1".to_string(),
        description: None,
        tags: r#"["rust","web"]"#.to_string(),
        frontmatter: None,
        project_id: None,
        visibility: "Private".to_string(),
        status: "Draft".to_string(),
        content_type: "Markdown".to_string(),
        word_count: 10,
        character_count: 50,
        read_count: 0,
        edit_count: 0,
        content: None,
        html: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        published_at: None,
        content_hash: None,
        conflict_detected: None,
    };

    let tags = meta.parse_tags().unwrap();
    assert_eq!(tags, vec!["rust", "web"]);
}

#[test]
fn test_document_metadata_serialize_tags() {
    let tags = vec!["tag1".to_string(), "tag2".to_string()];
    let result = DocumentMetadata::serialize_tags(&tags).unwrap();
    let parsed: Vec<String> = serde_json::from_str(&result).unwrap();
    assert_eq!(parsed, tags);
}

#[test]
fn test_document_metadata_parse_frontmatter() {
    let meta = DocumentMetadata {
        id: "doc-1".to_string(),
        title: "Test".to_string(),
        slug: Some("test".to_string()),
        author_id: "user-1".to_string(),
        description: None,
        tags: "[]".to_string(),
        frontmatter: Some(r#"{"custom": "value"}"#.to_string()),
        project_id: None,
        visibility: "Private".to_string(),
        status: "Draft".to_string(),
        content_type: "Markdown".to_string(),
        word_count: 0,
        character_count: 0,
        read_count: 0,
        edit_count: 0,
        content: None,
        html: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        published_at: None,
        content_hash: None,
        conflict_detected: None,
    };

    let fm = meta.parse_frontmatter().unwrap();
    assert_eq!(fm.get("custom").unwrap(), "value");
}

#[test]
fn test_document_metadata_parse_frontmatter_none() {
    let meta = DocumentMetadata {
        id: "doc-1".to_string(),
        title: "Test".to_string(),
        slug: Some("test".to_string()),
        author_id: "user-1".to_string(),
        description: None,
        tags: "[]".to_string(),
        frontmatter: None,
        project_id: None,
        visibility: "Private".to_string(),
        status: "Draft".to_string(),
        content_type: "Markdown".to_string(),
        word_count: 0,
        character_count: 0,
        read_count: 0,
        edit_count: 0,
        content: None,
        html: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        published_at: None,
        content_hash: None,
        conflict_detected: None,
    };

    let fm = meta.parse_frontmatter().unwrap();
    assert!(fm.is_empty());
}

#[test]
fn test_session_record_serde() {
    let record = make_session_record("Active", Utc::now() + Duration::hours(1));
    let json = serde_json::to_string(&record).unwrap();
    let de: SessionRecord = serde_json::from_str(&json).unwrap();
    assert_eq!(record.id, de.id);
    assert_eq!(record.status, de.status);
}

#[test]
fn test_database_config_debug() {
    let config = DatabaseConfig::new();
    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("max_connections"));
}
