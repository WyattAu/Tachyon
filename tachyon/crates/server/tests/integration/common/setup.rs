use std::env;

use tachyon_core::id::{generate_document_id, generate_repository_id, generate_user_id};
use tachyon_core::types::user::{User, UserRole};
use tachyon_database::space::CreateSpaceRequest;
use tachyon_database::{
    DatabasePool, DocumentMetadata, DocumentRepository, RepositoryMetadata, RepositoryRepository,
    SpaceRepository, UserRepository,
};

const DEFAULT_TEST_DB_URL: &str =
    "postgres://tachyon_test:tachyon_test@127.0.0.1:5433/tachyon_test";

pub fn test_db_url() -> String {
    env::var("DATABASE_URL")
        .or_else(|_| env::var("TEST_DATABASE_URL"))
        .unwrap_or_else(|_| DEFAULT_TEST_DB_URL.to_string())
}

pub async fn create_test_pool() -> DatabasePool {
    DatabasePool::new(&test_db_url()).await.expect(
        "Failed to connect to integration test database. Is PostgreSQL running on port 5433?",
    )
}

pub async fn setup_database(_pool: &DatabasePool) {
    tachyon_database::init_with_migrations(&test_db_url())
        .await
        .expect("Failed to run migrations for integration tests");
}

pub async fn teardown_database(pool: &DatabasePool) {
    // TRUNCATE ... CASCADE handles all FK dependencies automatically.
    // We only need to truncate the root tables; everything else cascades.
    let root_tables = ["users", "teams"];
    for table in &root_tables {
        let _ = pool
            .execute(&format!("TRUNCATE TABLE {} CASCADE", table))
            .await;
    }
}

pub async fn create_test_user(pool: &DatabasePool) -> User {
    let user_id = generate_user_id();
    let mut user = User::new(
        user_id,
        format!("testuser_{}", uuid::Uuid::new_v4().as_simple()),
        "Test User".to_string(),
        UserRole::Admin,
    );
    user.email = Some(format!(
        "test_{}@integration.test",
        uuid::Uuid::new_v4().as_simple()
    ));
    user.set_password("TestPassword123!")
        .expect("Failed to hash password");

    let repo = UserRepository::new(pool.clone());
    repo.create(&user)
        .await
        .expect("Failed to create test user")
}

pub async fn create_test_document(pool: &DatabasePool, author_id: &str) -> DocumentMetadata {
    let id = generate_document_id();
    let now = chrono::Utc::now();
    let meta = DocumentMetadata {
        id: id.as_str(),
        title: format!("Test Document {}", uuid::Uuid::new_v4().as_simple()),
        slug: Some(format!("test-doc-{}", uuid::Uuid::new_v4().as_simple())),
        author_id: author_id.to_string(),
        description: Some("Integration test document".to_string()),
        tags: serde_json::to_string(&vec!["test".to_string()]).unwrap(),
        frontmatter: None,
        project_id: None,
        visibility: "private".to_string(),
        status: "draft".to_string(),
        content_type: "markdown".to_string(),
        word_count: 10,
        character_count: 50,
        read_count: 0,
        edit_count: 0,
        content: Some("# Test\n\nHello world.".to_string()),
        html: Some("<h1>Test</h1><p>Hello world.</p>".to_string()),
        created_at: now,
        updated_at: now,
        published_at: None,
        content_hash: None,
        conflict_detected: None,
    };

    let repo = DocumentRepository::new(pool.clone());
    repo.create(meta.clone())
        .await
        .expect("Failed to create test document");
    meta
}

pub async fn create_test_repository(pool: &DatabasePool, owner_id: &str) -> RepositoryMetadata {
    let id = generate_repository_id();
    let now = chrono::Utc::now();
    let meta = RepositoryMetadata {
        id: id.as_str(),
        name: format!("Test Repo {}", uuid::Uuid::new_v4().as_simple()),
        slug: Some(format!("test-repo-{}", uuid::Uuid::new_v4().as_simple())),
        description: Some("Integration test repository".to_string()),
        repository_type: "local".to_string(),
        owner_id: owner_id.to_string(),
        visibility: "private".to_string(),
        status: "active".to_string(),
        default_branch: Some("main".to_string()),
        auto_sync: false,
        sync_interval_seconds: 300,
        file_watching_enabled: false,
        remote_url: None,
        last_commit_hash: None,
        current_branch: Some("main".to_string()),
        commits_ahead: Some(0),
        commits_behind: Some(0),
        document_count: 0,
        total_storage_bytes: 0,
        member_count: 1,
        local_path: None,
        created_at: now,
        updated_at: now,
    };

    let repo = RepositoryRepository::new(pool.clone());
    repo.create(meta.clone())
        .await
        .expect("Failed to create test repository");
    meta
}

pub async fn create_test_space(
    pool: &DatabasePool,
    owner_id: &str,
) -> tachyon_database::space::Space {
    let repo = SpaceRepository::new(pool.clone());
    let req = CreateSpaceRequest {
        name: format!("Test Space {}", uuid::Uuid::new_v4().as_simple()),
        description: Some("Integration test space".to_string()),
        icon: Some("folder".to_string()),
        color: Some("#3B82F6".to_string()),
        parent_id: None,
        visibility: Some("private".to_string()),
    };
    repo.create(owner_id, req)
        .await
        .expect("Failed to create test space")
}
