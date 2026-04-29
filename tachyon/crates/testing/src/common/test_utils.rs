use chrono::{Duration, Utc};
use serde_json::json;
use tachyon_database::{
    init_with_migrations, CatalogRepository, CreateTemplateRequest, DatabasePool, DocumentMetadata,
    DocumentRepository, Project, SessionRecord, Team,
};

pub struct TestDatabase {
    pub pool: DatabasePool,
}

impl TestDatabase {
    pub async fn new() -> Self {
        let database_url = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
            "postgres://tachyon:tachyon@localhost:5432/tachyon_test".to_string()
        });

        let pool = init_with_migrations(&database_url)
            .await
            .expect("Failed to setup test database");

        Self { pool }
    }

    pub async fn cleanup(&self) {
        let cleanup_queries = vec![
            "DELETE FROM documents WHERE title LIKE 'TEST_%'",
            "DELETE FROM projects WHERE name LIKE 'TEST_%'",
            "DELETE FROM sessions WHERE id LIKE 'TEST_%'",
            "DELETE FROM teams WHERE name LIKE 'TEST_%'",
            "DELETE FROM templates WHERE name LIKE 'TEST_%'",
        ];

        for query in cleanup_queries {
            let _ = self.pool.execute(query).await;
        }
    }
}

pub struct TestDataFactory;

impl TestDataFactory {
    pub fn create_document() -> DocumentMetadata {
        let now = Utc::now();
        DocumentMetadata {
            id: uuid::Uuid::new_v4().to_string(),
            title: format!("TEST_Document_{}", now.timestamp()),
            slug: Some(format!("test-doc-{}", now.timestamp())),
            author_id: uuid::Uuid::new_v4().to_string(),
            description: Some("Test document description".to_string()),
            tags: "test".to_string(),
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
            created_at: now,
            updated_at: now,
            published_at: None,
            content_hash: None,
            conflict_detected: None,
        }
    }

    pub fn create_document_with_title(title: &str) -> DocumentMetadata {
        let mut doc = Self::create_document();
        doc.title = format!("TEST_{}", title);
        doc.slug = Some(format!("test-{}", title.to_lowercase().replace(' ', "-")));
        doc
    }

    pub fn create_project() -> Project {
        let now = Utc::now();
        Project {
            id: uuid::Uuid::new_v4().to_string(),
            name: format!("TEST_Project_{}", now.timestamp()),
            slug: format!("test-project-{}", now.timestamp()),
            description: Some("Test project".to_string()),
            project_type: "service".to_string(),
            owner_id: uuid::Uuid::new_v4().to_string(),
            organization_id: None,
            lifecycle: "development".to_string(),
            repository_url: None,
            docs_url: None,
            api_url: None,
            tags: vec!["test".to_string()],
            metadata: json!({}),
            language: Some("Rust".to_string()),
            framework: Some("Axum".to_string()),
            visibility: "Private".to_string(),
            status: "Active".to_string(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn create_session() -> SessionRecord {
        let now = Utc::now();
        SessionRecord {
            id: format!("TEST_Session_{}", now.timestamp()),
            user_id: uuid::Uuid::new_v4().to_string(),
            session_type: "web".to_string(),
            status: "Active".to_string(),
            token_value: uuid::Uuid::new_v4().to_string(),
            token_type: "bearer".to_string(),
            ip_address: Some("127.0.0.1".to_string()),
            user_agent: Some("Test Agent".to_string()),
            device_info: Some("Test Device".to_string()),
            created_at: now,
            expires_at: now + Duration::hours(24),
            last_activity: now,
        }
    }

    pub fn create_team() -> Team {
        let now = Utc::now();
        let owner_id = uuid::Uuid::new_v4().to_string();
        Team {
            id: uuid::Uuid::new_v4().to_string(),
            name: format!("TEST_Team_{}", now.timestamp()),
            slug: format!("test-team-{}", now.timestamp()),
            description: Some("Test team".to_string()),
            owner_id: owner_id.clone(),
            avatar_url: None,
            settings: json!({}),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn create_template() -> CreateTemplateRequest {
        CreateTemplateRequest {
            name: format!("TEST_Template_{}", Utc::now().timestamp()),
            description: Some("Test template".to_string()),
            content: "# Test Template\n\nContent here.".to_string(),
            category: Some("general".to_string()),
            tags: Some(vec!["test".to_string()]),
            created_by: uuid::Uuid::new_v4().to_string(),
        }
    }
}

pub struct TestFixtures;

impl TestFixtures {
    pub async fn create_test_documents(
        repo: &DocumentRepository,
        count: usize,
    ) -> Vec<DocumentMetadata> {
        let mut documents = Vec::new();

        for i in 0..count {
            let mut doc = TestDataFactory::create_document();
            doc.title = format!("TEST_Fixture_{}_{}", i, Utc::now().timestamp());
            doc.slug = Some(format!("test-fixture-{}-{}", i, Utc::now().timestamp()));

            repo.create(doc.clone())
                .await
                .expect("Failed to create test document");
            documents.push(doc);

            tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
        }

        documents
    }

    pub async fn create_test_projects(repo: &CatalogRepository, count: usize) -> Vec<Project> {
        let mut projects = Vec::new();

        for i in 0..count {
            let mut project = TestDataFactory::create_project();
            project.name = format!("TEST_Fixture_Project_{}_{}", i, Utc::now().timestamp());
            project.slug = format!("test-fixture-project-{}-{}", i, Utc::now().timestamp());

            repo.create_project(&project)
                .await
                .expect("Failed to create test project");
            projects.push(project);

            tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
        }

        projects
    }
}

#[macro_export]
macro_rules! assert_ok {
    ($expr:expr) => {
        match $expr {
            Ok(v) => v,
            Err(e) => panic!("assertion failed: expected Ok, got Err({:?})", e),
        }
    };
    ($expr:expr, $msg:expr) => {
        match $expr {
            Ok(v) => v,
            Err(e) => panic!("{}: expected Ok, got Err({:?})", $msg, e),
        }
    };
}

#[macro_export]
macro_rules! assert_err {
    ($expr:expr) => {
        match $expr {
            Err(e) => e,
            Ok(v) => panic!("assertion failed: expected Err, got Ok({:?})", v),
        }
    };
    ($expr:expr, $msg:expr) => {
        match $expr {
            Err(e) => e,
            Ok(v) => panic!("{}: expected Err, got Ok({:?})", $msg, v),
        }
    };
}

pub fn setup_test_env() {
    if std::env::var("TEST_DATABASE_URL").is_err() {
        unsafe {
            std::env::set_var(
                "TEST_DATABASE_URL",
                "postgres://tachyon:tachyon@localhost:5432/tachyon_test",
            );
        }
    }
}

pub async fn with_test_db<F, Fut>(f: F)
where
    F: FnOnce(TestDatabase) -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    setup_test_env();
    let db = TestDatabase::new().await;

    db.cleanup().await;

    f(db.clone()).await;

    db.cleanup().await;
}

impl Clone for TestDatabase {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
        }
    }
}
