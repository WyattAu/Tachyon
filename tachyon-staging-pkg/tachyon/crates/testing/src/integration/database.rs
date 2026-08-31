//! Integration tests for database types and operations
//!
//! Tests database record validation, serialization, and type interactions
//! without requiring a running database server.

#[allow(unused_imports)]
use chrono::{Duration, Utc};
#[allow(unused_imports)]
use tachyon_database::types::*;

#[test]
fn test_session_record_lifecycle() {
    let now = Utc::now();
    let record = SessionRecord {
        id: "sess-lifecycle".to_string(),
        user_id: "user-1".to_string(),
        session_type: "web".to_string(),
        status: "Active".to_string(),
        token_value: "token-xyz".to_string(),
        token_type: "bearer".to_string(),
        ip_address: Some("10.0.0.1".to_string()),
        user_agent: Some("Mozilla/5.0".to_string()),
        device_info: Some("Desktop".to_string()),
        created_at: now,
        expires_at: now + Duration::hours(24),
        last_activity: now,
    };

    assert!(record.is_valid());
    assert!(!record.is_expired());

    let expired_record = SessionRecord {
        expires_at: now - Duration::hours(1),
        ..record.clone()
    };
    assert!(!expired_record.is_valid());
    assert!(expired_record.is_expired());

    let revoked_record = SessionRecord {
        status: "Revoked".to_string(),
        ..record
    };
    assert!(!revoked_record.is_valid());
}

#[test]
fn test_document_metadata_serialization_roundtrip() {
    let meta = DocumentMetadata {
        id: "doc-1".to_string(),
        title: "Integration Test Doc".to_string(),
        slug: Some("integration-test-doc".to_string()),
        author_id: "user-1".to_string(),
        description: Some("A test document".to_string()),
        tags: r#"["test","integration"]"#.to_string(),
        frontmatter: Some(r#"{"key": "value"}"#.to_string()),
        project_id: Some("proj-1".to_string()),
        visibility: "Private".to_string(),
        status: "Published".to_string(),
        content_type: "Markdown".to_string(),
        word_count: 100,
        character_count: 500,
        read_count: 42,
        edit_count: 5,
        content: Some("# Hello\n\nWorld".to_string()),
        html: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        published_at: Some(Utc::now()),
        content_hash: Some("abc123".to_string()),
        conflict_detected: Some(false),
    };

    let json = serde_json::to_string(&meta).unwrap();
    let de: DocumentMetadata = serde_json::from_str(&json).unwrap();
    assert_eq!(meta.id, de.id);
    assert_eq!(meta.title, de.title);
    assert_eq!(meta.visibility, de.visibility);
    assert_eq!(meta.status, de.status);
    assert_eq!(meta.content_hash, de.content_hash);
    assert_eq!(meta.conflict_detected, de.conflict_detected);
}

#[test]
fn test_policy_record_with_rules_roundtrip() {
    let record = PolicyRecord {
        id: 1,
        name: "Document Policy".to_string(),
        description: Some("Controls document access".to_string()),
        rules: r#"[{"effect":"allow","resource":"document:*","action":"read"}]"#.to_string(),
        effect: "allow".to_string(),
        policy_type: Some("rbac".to_string()),
        created_by: Some("admin".to_string()),
        enabled: true,
        priority: 10,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let json = serde_json::to_string(&record).unwrap();
    let de: PolicyRecord = serde_json::from_str(&json).unwrap();
    assert_eq!(record.name, de.name);
    assert_eq!(record.priority, de.priority);
    assert_eq!(record.enabled, de.enabled);
    assert_eq!(record.effect, de.effect);
}

#[test]
fn test_repository_metadata_fields() {
    let meta = RepositoryMetadata {
        id: "repo-1".to_string(),
        name: "Test Repo".to_string(),
        slug: Some("test-repo".to_string()),
        description: Some("Test repo desc".to_string()),
        repository_type: "team".to_string(),
        owner_id: "user-1".to_string(),
        visibility: "private".to_string(),
        status: "cloned".to_string(),
        default_branch: Some("main".to_string()),
        auto_sync: true,
        sync_interval_seconds: 300,
        file_watching_enabled: true,
        remote_url: Some("https://github.com/test/repo".to_string()),
        last_commit_hash: Some("abc123".to_string()),
        current_branch: Some("develop".to_string()),
        commits_ahead: Some(3),
        commits_behind: Some(1),
        document_count: 10,
        total_storage_bytes: 1024,
        member_count: 5,
        local_path: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let json = serde_json::to_string(&meta).unwrap();
    let de: RepositoryMetadata = serde_json::from_str(&json).unwrap();
    assert_eq!(meta.name, de.name);
    assert_eq!(meta.commits_ahead, de.commits_ahead);
    assert_eq!(meta.total_storage_bytes, de.total_storage_bytes);
}

#[test]
fn test_graph_node_fields() {
    let node = GraphNode {
        id: "node-1".to_string(),
        node_type: "concept".to_string(),
        name: "Test Concept".to_string(),
        slug: Some("test-concept".to_string()),
        description: Some("A concept".to_string()),
        content: Some("Content here".to_string()),
        visibility: "private".to_string(),
        weight: 0.75,
        properties: serde_json::json!({"custom": "field"}),
        project_id: None,
        document_id: Some("doc-1".to_string()),
        created_by: Some("user-1".to_string()),
        is_active: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        deactivated_at: None,
    };

    let json = serde_json::to_string(&node).unwrap();
    let de: GraphNode = serde_json::from_str(&json).unwrap();
    assert_eq!(node.name, de.name);
    assert_eq!(node.node_type, de.node_type);
    assert_eq!(node.weight, de.weight);
}

#[test]
fn test_graph_edge_fields() {
    let edge = GraphEdge {
        id: "edge-1".to_string(),
        source_id: "node-1".to_string(),
        target_id: "node-2".to_string(),
        edge_type: "references".to_string(),
        label: Some("ref label".to_string()),
        description: Some("A reference".to_string()),
        weight: 0.5,
        confidence: Some(0.9),
        properties: serde_json::json!({}),
        project_id: None,
        created_by: Some("user-1".to_string()),
        is_active: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        deactivated_at: None,
    };

    let json = serde_json::to_string(&edge).unwrap();
    let de: GraphEdge = serde_json::from_str(&json).unwrap();
    assert_eq!(edge.source_id, de.source_id);
    assert_eq!(edge.target_id, de.target_id);
    assert_eq!(edge.confidence, de.confidence);
}

#[test]
fn test_user_role_mapping() {
    let mapping = UserRoleMapping {
        id: 1,
        user_id: "user-1".to_string(),
        role: "admin".to_string(),
        assigned_by: Some("owner-1".to_string()),
        assigned_at: Utc::now(),
        expires_at: Some(Utc::now() + Duration::days(30)),
    };

    let json = serde_json::to_string(&mapping).unwrap();
    let de: UserRoleMapping = serde_json::from_str(&json).unwrap();
    assert_eq!(mapping.user_id, de.user_id);
    assert_eq!(mapping.role, de.role);
    assert!(de.expires_at.is_some());
}

#[test]
fn test_project_fields() {
    let project = Project {
        id: "proj-1".to_string(),
        name: "Test Project".to_string(),
        slug: "test-project".to_string(),
        description: Some("A project".to_string()),
        project_type: "service".to_string(),
        owner_id: "user-1".to_string(),
        organization_id: None,
        lifecycle: "production".to_string(),
        repository_url: Some("https://github.com/test/repo".to_string()),
        docs_url: None,
        api_url: None,
        tags: vec!["rust".to_string(), "web".to_string()],
        metadata: serde_json::json!({"key": "value"}),
        language: Some("Rust".to_string()),
        framework: Some("Axum".to_string()),
        visibility: "private".to_string(),
        status: "active".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let json = serde_json::to_string(&project).unwrap();
    let de: Project = serde_json::from_str(&json).unwrap();
    assert_eq!(project.name, de.name);
    assert_eq!(project.tags, de.tags);
    assert_eq!(project.language, de.language);
}

#[test]
fn test_database_config_consistency() {
    let config = DatabaseConfig::new();
    assert!(config.max_connections > config.min_connections);
    assert!(config.connection_timeout > 0);
    assert!(!config.schema.is_empty());
}

#[test]
fn test_search_index_fields() {
    let index = SearchIndex {
        id: 1,
        document_id: "doc-1".to_string(),
        content_type: "title".to_string(),
        content: "Searchable title text".to_string(),
        weight: 2.0,
        indexed_at: Utc::now(),
    };

    assert!(index.weight > 0.0);
    assert!(!index.content.is_empty());
}
