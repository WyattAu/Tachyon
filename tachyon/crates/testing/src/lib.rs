//! Tachyon Testing Infrastructure
//!
//! This crate provides comprehensive testing infrastructure for the Tachyon project,
//! including unit tests, integration tests, fuzzing tests, and performance benchmarks.
//!
//! ## Modules
//!
//! - [`unit`] - Unit test modules for individual components
//! - [`integration`] - Integration test modules for end-to-end workflows
//! - [`fuzz`] - Fuzzing test modules for security and robustness
//! - [`benchmarks`] - Performance benchmark modules

#![allow(clippy::duplicate_mod, unused_imports)]

pub mod benchmarks;
pub mod common;
pub mod fuzz;
pub mod integration;
pub mod unit;

pub use mockall;
pub use proptest;
pub use serial_test;
pub use tokio_test;
pub use wiremock;

pub mod config {
    pub const DEFAULT_TEST_TIMEOUT: u64 = 30;
    pub const DB_TEST_PORT: u16 = 5432;
    pub const SHORT_FUZZ_ITERATIONS: usize = 10_000;
    pub const LONG_FUZZ_ITERATIONS: usize = 1_000_000;
    pub const BENCHMARK_SAMPLE_SIZE: usize = 100;
    pub const BENCHMARK_WARMUP_MS: u64 = 1000;
    pub const BENCHMARK_MEASUREMENT_MS: u64 = 5000;
}

pub type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

pub trait TestSetup {
    fn setup() -> Self;
    fn teardown(self);
}

pub struct TestApp {
    pub router: axum::Router,
    pub pool: Option<tachyon_database::DatabasePool>,
}

impl TestApp {
    pub async fn new() -> Self {
        let database_url = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
            "postgres://tachyon:tachyon@localhost:5432/tachyon_test".to_string()
        });

        let pool = tachyon_database::init_with_migrations(&database_url)
            .await
            .ok();

        let router = tachyon_server::routes::create_router().await;

        TestApp { router, pool }
    }

    pub fn pool(&self) -> Option<&tachyon_database::DatabasePool> {
        self.pool.as_ref()
    }

    pub async fn cleanup(&self) {
        if let Some(pool) = &self.pool {
            let _ = pool
                .execute(
                    "TRUNCATE \
                    review_comments, \
                    document_reviews, \
                    document_comments, \
                    document_presence, \
                    knowledge_graph_edges, \
                    knowledge_graph_nodes, \
                    document_versions, \
                    attachments, \
                    saved_searches, \
                    components, \
                    project_members, \
                    user_roles, \
                    team_members, \
                    space_members, \
                    organization_members, \
                    refresh_tokens, \
                    sessions, \
                    api_keys, \
                    connected_accounts, \
                    user_preferences, \
                    password_reset_tokens, \
                    email_verification_tokens, \
                    notifications, \
                    activity_events, \
                    webhooks, \
                    plugins, \
                    templates, \
                    documents, \
                    repositories, \
                    projects, \
                    teams, \
                    spaces, \
                    organizations, \
                    subscriptions, \
                    invoices, \
                    notification_preferences, \
                    payments, \
                    audit_log, \
                    search_index, \
                    roles, \
                    users \
                    CASCADE",
                )
                .await;
        }
    }
}

pub struct MockDataGenerator;

impl MockDataGenerator {
    pub fn user() -> serde_json::Value {
        let id = uuid::Uuid::new_v4();
        serde_json::json!({
            "id": id,
            "username": format!("testuser_{}", &id.to_string()[..8]),
            "email": format!("test_{}@example.com", &id.to_string()[..8]),
            "display_name": "Test User",
            "password_hash": "hashed_password_placeholder"
        })
    }

    pub fn document() -> serde_json::Value {
        let id = uuid::Uuid::new_v4();
        serde_json::json!({
            "id": id,
            "title": format!("TEST_Document_{}", &id.to_string()[..8]),
            "slug": format!("test-doc-{}", &id.to_string()[..8]),
            "author_id": uuid::Uuid::new_v4(),
            "description": "Test document for integration testing",
            "tags": "[\"test\", \"integration\"]",
            "visibility": "private",
            "status": "draft",
            "content_type": "markdown"
        })
    }

    pub fn project() -> serde_json::Value {
        let id = uuid::Uuid::new_v4();
        serde_json::json!({
            "id": id,
            "name": format!("TEST_Project_{}", &id.to_string()[..8]),
            "slug": format!("test-project-{}", &id.to_string()[..8]),
            "description": "Test project for integration testing",
            "project_type": "service",
            "owner_id": uuid::Uuid::new_v4(),
            "lifecycle": "experimental",
            "visibility": "internal",
            "status": "active"
        })
    }

    pub fn session(user_id: &str) -> serde_json::Value {
        let id = uuid::Uuid::new_v4();
        serde_json::json!({
            "id": id,
            "user_id": user_id,
            "session_type": "web",
            "status": "Active",
            "token_value": format!("test-token-{}", id),
            "token_type": "bearer"
        })
    }
}

pub mod db_helpers {
    use tachyon_database::DatabasePool;

    pub async fn setup_test_pool() -> Option<DatabasePool> {
        let database_url = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
            "postgres://tachyon:tachyon@localhost:5432/tachyon_test".to_string()
        });

        tachyon_database::init_with_migrations(&database_url)
            .await
            .ok()
    }

    pub async fn cleanup_test_data(pool: &DatabasePool) {
        let _ = pool
            .execute(
                "TRUNCATE \
                review_comments, \
                document_reviews, \
                document_comments, \
                document_presence, \
                knowledge_graph_edges, \
                knowledge_graph_nodes, \
                document_versions, \
                attachments, \
                saved_searches, \
                components, \
                project_members, \
                user_roles, \
                team_members, \
                space_members, \
                organization_members, \
                refresh_tokens, \
                sessions, \
                api_keys, \
                connected_accounts, \
                user_preferences, \
                password_reset_tokens, \
                email_verification_tokens, \
                notifications, \
                activity_events, \
                webhooks, \
                plugins, \
                templates, \
                documents, \
                repositories, \
                projects, \
                teams, \
                spaces, \
                organizations, \
                subscriptions, \
                invoices, \
                notification_preferences, \
                payments, \
                audit_log, \
                search_index, \
                roles, \
                users \
                CASCADE",
            )
            .await;
    }

    pub async fn teardown(pool: &DatabasePool) {
        cleanup_test_data(pool).await;
    }

    pub fn is_db_available() -> bool {
        std::env::var("TEST_DATABASE_URL").is_ok()
    }
}

pub mod assertions {
    use serde_json::Value;

    pub fn assert_json_field(json: &Value, field: &str, expected: &str) {
        let actual = json
            .get(field)
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| panic!("Field '{}' not found or not a string in JSON", field));
        assert_eq!(actual, expected, "Field '{}' mismatch", field);
    }

    pub fn assert_json_has_field(json: &Value, field: &str) {
        assert!(
            json.get(field).is_some(),
            "Expected field '{}' to exist in JSON: {}",
            field,
            json
        );
    }

    pub fn assert_status_in_range(status: u16, min: u16, max: u16) {
        assert!(
            status >= min && status <= max,
            "Status {} not in range [{}, {}]",
            status,
            min,
            max
        );
    }

    pub fn assert_success_or_accepted(status: u16) {
        assert!(
            status == 200 || status == 201 || status == 202 || status == 204,
            "Expected success status, got {}",
            status
        );
    }

    pub fn assert_client_error(status: u16) {
        assert!(
            (400..500).contains(&status),
            "Expected client error status (4xx), got {}",
            status
        );
    }
}
