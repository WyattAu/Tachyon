//! Factory functions for creating test data
//!
//! Provides factory methods to generate test entities with randomized or deterministic data.

use chrono::Duration;
use tachyon_core::{
    generate_document_id, generate_repository_id, generate_session_id, generate_user_id,
    types::document::{Document, DocumentContent},
    types::repository::{Repository, RepositoryType},
    types::session::{Session, SessionType, TokenType},
    types::user::{User, UserAction, UserRole, UserType},
};
use tachyon_rbac::{
    types::{Action, AuthContext, Effect, Resource, Subject},
    Permission, Policy, PolicyRule, PolicyType,
};
use tachyon_search::SearchDocument;

pub fn create_test_user() -> User {
    User::new(
        generate_user_id(),
        "testuser".to_string(),
        "Test User".to_string(),
        UserRole::Reader,
    )
    .with_email("test@example.com".to_string())
}

pub fn create_test_user_with_role(role: UserRole) -> User {
    User::new(
        generate_user_id(),
        "testuser".to_string(),
        "Test User".to_string(),
        role,
    )
    .with_email("test@example.com".to_string())
}

pub fn create_test_document() -> Document {
    Document::new(
        generate_document_id(),
        "Test Document".to_string(),
        generate_user_id(),
        DocumentContent::markdown("# Test\n\nTest content.".to_string()),
    )
}

pub fn create_test_document_with_content(title: &str, content: &str) -> Document {
    Document::new(
        generate_document_id(),
        title.to_string(),
        generate_user_id(),
        DocumentContent::markdown(content.to_string()),
    )
}

pub fn create_test_space() -> tachyon_core::types::node::Node {
    tachyon_core::types::node::Node::new(
        generate_node_id(),
        tachyon_core::types::node::NodeType::Concept,
        "Test Space".to_string(),
        generate_user_id(),
    )
    .with_content("Test space content".to_string())
    .with_visibility(tachyon_core::types::node::NodeVisibility::Private)
}

pub fn create_test_repository() -> Repository {
    Repository::new(
        generate_repository_id(),
        "Test Repository".to_string(),
        RepositoryType::Personal,
        generate_user_id(),
    )
}

pub fn create_test_session() -> Session {
    Session::new(
        generate_session_id(),
        generate_user_id(),
        SessionType::Web,
        "test-token".to_string(),
        TokenType::Bearer,
        Duration::hours(24),
    )
}

pub fn create_test_search_document() -> SearchDocument {
    SearchDocument::new(
        generate_document_id(),
        "Test Search Document".to_string(),
        "Test content for search indexing".to_string(),
        generate_user_id(),
    )
}

pub fn create_test_search_document_with_tags(tags: Vec<&str>) -> SearchDocument {
    SearchDocument::new(
        generate_document_id(),
        "Tagged Document".to_string(),
        "Content with tags".to_string(),
        generate_user_id(),
    )
    .with_tags(tags.into_iter().map(String::from).collect())
}

pub fn create_test_search_request(query: &str) -> tachyon_search::SearchRequest {
    tachyon_search::SearchRequest::new(query)
}

pub fn create_test_subject() -> Subject {
    Subject::new("user", &generate_user_id().as_str())
}

pub fn create_test_resource(resource_type: &str, resource_id: &str) -> Resource {
    Resource::new(resource_type, resource_id)
}

pub fn create_test_auth_context() -> AuthContext {
    AuthContext::new(generate_user_id(), generate_session_id())
        .with_role("reader")
        .with_attribute("ip", "127.0.0.1")
}

pub fn create_test_policy(name: &str) -> Policy {
    Policy::new(name, name, PolicyType::Rbac).add_rule(PolicyRule::new(
        &format!("{}_allow_read", name),
        "user:*",
        "document:*",
        "read",
        Effect::Allow,
    ))
}

pub fn create_test_permission(id: &str, resource: &str, action: &str) -> Permission {
    Permission::new(id, resource, action, Effect::Allow)
}

pub fn create_test_access_request(
    subject: Subject,
    resource: Resource,
    action: Action,
) -> tachyon_rbac::types::AccessRequest {
    tachyon_rbac::types::AccessRequest::new(subject, resource, action, create_test_auth_context())
}

pub fn create_test_user_action(action: UserAction) -> User {
    let mut user = create_test_user();
    user.permissions.grant(action);
    user
}

pub fn create_test_service_user() -> User {
    User::new(
        generate_user_id(),
        "service-user".to_string(),
        "Service Account".to_string(),
        UserRole::Reader,
    )
    .with_user_type(UserType::Service)
}

pub fn create_test_admin_user() -> User {
    create_test_user_with_role(UserRole::Admin)
}

fn generate_node_id() -> tachyon_core::NodeId {
    tachyon_core::generate_node_id()
}
