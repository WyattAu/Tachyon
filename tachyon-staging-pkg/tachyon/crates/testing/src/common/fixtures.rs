//! Test fixtures for test setup and teardown
//!
//! Provides reusable fixtures for creating test entities with sensible defaults.

use chrono::Duration;
use tachyon_core::{
    generate_document_id, generate_repository_id, generate_session_id, generate_user_id,
    types::document::{Document, DocumentContent},
    types::repository::{Repository, RepositoryConfig, RepositoryType},
    types::session::{Session, SessionType, TokenType},
    types::user::{User, UserRole},
};

pub struct SessionFixture {
    pub session: Session,
    pub user_id: tachyon_core::UserId,
    pub session_id: tachyon_core::SessionId,
}

impl SessionFixture {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let session_id = generate_session_id();
        let user_id = generate_user_id();
        let session = Session::new(
            session_id,
            user_id,
            SessionType::Web,
            "fixture-token".to_string(),
            TokenType::Bearer,
            Duration::hours(1),
        )
        .with_ip_address("127.0.0.1".to_string())
        .with_user_agent("TestAgent/1.0".to_string())
        .with_device_info("TestDevice".to_string());

        Self {
            session,
            user_id,
            session_id,
        }
    }

    pub fn expired() -> Self {
        let session_id = generate_session_id();
        let user_id = generate_user_id();
        let session = Session::new(
            session_id,
            user_id,
            SessionType::Api,
            "expired-token".to_string(),
            TokenType::Jwt,
            Duration::seconds(-1),
        );

        Self {
            session,
            user_id,
            session_id,
        }
    }
}

pub struct DocumentFixture {
    pub document: Document,
    pub document_id: tachyon_core::DocumentId,
    pub author_id: tachyon_core::UserId,
}

impl DocumentFixture {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let document_id = generate_document_id();
        let author_id = generate_user_id();
        let document = Document::new(
            document_id,
            "Test Document".to_string(),
            author_id,
            DocumentContent::markdown("# Test Document\n\nThis is test content.".to_string()),
        );

        Self {
            document,
            document_id,
            author_id,
        }
    }

    pub fn published() -> Self {
        let mut fixture = Self::new();
        fixture.document.publish().unwrap();
        fixture
    }

    pub fn with_tags(tags: Vec<&str>) -> Self {
        let mut fixture = Self::new();
        for tag in tags {
            fixture.document.metadata.add_tag(tag.to_string()).unwrap();
        }
        fixture
    }
}

pub struct RepositoryFixture {
    pub repository: Repository,
    pub repository_id: tachyon_core::RepositoryId,
    pub owner_id: tachyon_core::UserId,
}

impl RepositoryFixture {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let repository_id = generate_repository_id();
        let owner_id = generate_user_id();
        let repository = Repository::new(
            repository_id,
            "Test Repository".to_string(),
            RepositoryType::Personal,
            owner_id,
        );

        Self {
            repository,
            repository_id,
            owner_id,
        }
    }

    pub fn cloned() -> Self {
        let mut fixture = Self::new();
        fixture.repository = fixture
            .repository
            .with_status(tachyon_core::types::repository::RepositoryStatus::Cloned);
        fixture
    }

    pub fn with_config(config: RepositoryConfig) -> Self {
        let repository_id = generate_repository_id();
        let owner_id = generate_user_id();
        let mut repository = Repository::new(
            repository_id,
            "Configured Repo".to_string(),
            RepositoryType::Team,
            owner_id,
        );
        repository.metadata.config = config;

        Self {
            repository,
            repository_id,
            owner_id,
        }
    }
}

pub struct UserFixture {
    pub user: User,
    pub user_id: tachyon_core::UserId,
}

impl UserFixture {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let user_id = generate_user_id();
        let user = User::new(
            user_id,
            "testuser".to_string(),
            "Test User".to_string(),
            UserRole::Reader,
        )
        .with_email("test@example.com".to_string());

        Self { user, user_id }
    }

    pub fn admin() -> Self {
        let user_id = generate_user_id();
        let user = User::new(
            user_id,
            "admin".to_string(),
            "Admin User".to_string(),
            UserRole::Admin,
        )
        .with_email("admin@example.com".to_string());

        Self { user, user_id }
    }

    pub fn writer() -> Self {
        let user_id = generate_user_id();
        let user = User::new(
            user_id,
            "writer".to_string(),
            "Writer User".to_string(),
            UserRole::Writer,
        )
        .with_email("writer@example.com".to_string());

        Self { user, user_id }
    }

    pub fn inactive() -> Self {
        let mut fixture = Self::new();
        fixture.user.is_active = Some(false);
        fixture
    }
}
