use chrono::Duration;
use chrono::Utc;
use serde_json::json;
use tachyon_database::{Component, DocumentMetadata, Project, SessionRecord};

pub fn create_test_document_metadata() -> DocumentMetadata {
    DocumentMetadata {
        id: uuid::Uuid::new_v4().to_string(),
        title: format!("Test Document {}", Utc::now().timestamp()),
        slug: format!("test-doc-{}", uuid::Uuid::new_v4()),
        author_id: uuid::Uuid::new_v4().to_string(),
        description: Some("Test document for unit testing".to_string()),
        tags: r#"["test", "fixture"]"#.to_string(),
        frontmatter: None,
        project_id: None,
        visibility: "private".to_string(),
        status: "draft".to_string(),
        content_type: "markdown".to_string(),
        word_count: 100,
        character_count: 500,
        read_count: 0,
        edit_count: 1,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        published_at: None,
    }
}

pub fn create_test_project() -> Project {
    let now = Utc::now();
    Project {
        id: uuid::Uuid::new_v4().to_string(),
        name: format!("Test Project {}", now.timestamp()),
        slug: format!("test-project-{}", uuid::Uuid::new_v4()),
        description: Some("Test project for unit testing".to_string()),
        project_type: "service".to_string(),
        owner_id: uuid::Uuid::new_v4().to_string(),
        organization_id: None,
        lifecycle: "experimental".to_string(),
        repository_url: Some("https://github.com/test/repo".to_string()),
        docs_url: None,
        api_url: None,
        tags: vec!["test".to_string(), "fixture".to_string()],
        metadata: json!({}),
        language: Some("rust".to_string()),
        framework: Some("axum".to_string()),
        visibility: "internal".to_string(),
        status: "active".to_string(),
        created_at: now,
        updated_at: now,
    }
}

pub fn create_test_component(project_id: &str) -> Component {
    let now = Utc::now();
    Component {
        id: uuid::Uuid::new_v4().to_string(),
        name: format!("Test Component {}", now.timestamp()),
        component_type: "service".to_string(),
        project_id: project_id.to_string(),
        owner_id: uuid::Uuid::new_v4().to_string(),
        system_id: None,
        repository_url: None,
        docs_url: None,
        api_spec_url: None,
        tags: vec!["test".to_string()],
        lifecycle: "experimental".to_string(),
        created_at: now,
        updated_at: now,
    }
}

pub fn create_test_session(user_id: &str) -> SessionRecord {
    let now = Utc::now();
    SessionRecord {
        id: uuid::Uuid::new_v4().to_string(),
        user_id: user_id.to_string(),
        session_type: "web".to_string(),
        status: "Active".to_string(),
        token_value: format!("test-token-{}", uuid::Uuid::new_v4()),
        token_type: "bearer".to_string(),
        ip_address: Some("127.0.0.1".to_string()),
        user_agent: Some("TestAgent/1.0".to_string()),
        device_info: Some("Test Device".to_string()),
        created_at: now,
        expires_at: now + Duration::hours(24),
        last_activity: now,
    }
}

pub fn create_expired_session(user_id: &str) -> SessionRecord {
    let mut session = create_test_session(user_id);
    session.expires_at = Utc::now() - Duration::hours(1);
    session
}

pub fn create_revoked_session(user_id: &str) -> SessionRecord {
    let mut session = create_test_session(user_id);
    session.status = "Revoked".to_string();
    session
}

pub fn sample_markdown_content() -> &'static str {
    r#"# Test Document

This is a **test** document with _markdown_ formatting.

## Features

- List item 1
- List item 2
- List item 3

### Code Block

```rust
fn main() {
    println!("Hello, World!");
}
```

## Links

[External Link](https://example.com)

## Table

| Column 1 | Column 2 |
|----------|----------|
| Value 1  | Value 2  |
"#
}

pub fn sample_frontmatter() -> serde_json::Value {
    json!({
        "title": "Test Document",
        "author": "Test Author",
        "date": "2024-01-01",
        "tags": ["test", "example"],
        "published": true
    })
}
