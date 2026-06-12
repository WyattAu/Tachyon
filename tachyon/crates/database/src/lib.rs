//! Tachyon Database — PostgreSQL persistence layer with migrations and repositories.

// Tachyon Database - PostgreSQL Database Layer
// Provides metadata storage, session persistence, and RBAC mapping tables

pub mod activity;
pub mod attachment;
pub mod billing;
pub mod canvas;
pub mod catalog;
pub mod comment;
pub mod crdt;
pub mod document_branch;
pub mod document_review;
pub mod document_version;
pub mod error;
pub mod flashcard;
pub mod graph;
pub mod magic_link;
pub mod migrations;
pub mod notification;
pub mod onboarding;
pub mod organization;
pub mod password_reset;
pub mod permissions;
pub mod plugin;
pub mod presence;
pub mod rbac;
pub mod refresh_token;
pub mod repository;
pub mod saved_search;
pub mod schema;
pub mod search;
pub mod session;
pub mod sms_otp;
pub mod space;
pub mod team;
pub mod template;
pub mod types;
pub mod usage;
pub mod user;
pub mod user_preferences;
pub mod webhook;

// Re-export common types for convenience
pub use activity::{ActivityEvent, ActivityRepository, CreateActivityEvent};
pub use attachment::{Attachment, AttachmentRepository, CreateAttachmentRequest};
pub use billing::{
    CreateInvoiceRequest, CreateSubscriptionRequest, Invoice, InvoiceRepository,
    NotificationPreference, NotificationPreferenceRepository, Subscription, SubscriptionRepository,
    UpdateInvoiceRequest, UpdateSubscriptionRequest, UpsertNotificationPrefRequest,
};
pub use canvas::{
    Canvas, CanvasEdge, CanvasNode, CanvasRepository, CreateCanvasEdgeRequest,
    CreateCanvasNodeRequest, CreateCanvasRequest, UpdateCanvasEdgeRequest, UpdateCanvasNodeRequest,
    UpdateCanvasRequest,
};
pub use catalog::CatalogRepository;
pub use catalog::{CatalogStats, CreateComponentRequest, CreateProjectRequest};
pub use comment::{
    Comment, CommentRepository, CreateCommentRequest as CreateDocumentCommentRequest,
    UpdateCommentRequest as UpdateDocumentCommentRequest,
};
pub use document_branch::{
    CreateBranchRow, DocumentBranchRepository, DocumentBranchRow, UpdateBranchRow,
};
pub use document_review::{
    CreateCommentRequest, CreateReviewRequest, DocumentReview, DocumentReviewRepository,
    ReviewComment, ReviewStatus, UpdateReviewRequest,
};
pub use document_version::{CreateVersionRequest, DocumentVersion, DocumentVersionRepository};
pub use error::{DatabaseError, DatabaseResult};
pub use flashcard::{
    CardState, CreateFlashcardRequest, Flashcard, FlashcardRepository, FlashcardSrsState, Rating,
    ReviewFlashcardRequest, ReviewLog, UpdateFlashcardRequest,
};
pub use graph::{GraphDiff, GraphRepository};
pub use magic_link::{MagicLinkRepository, MagicLinkToken};
pub use notification::{CreateNotification, Notification, NotificationRepository};
pub use onboarding::{OnboardingRepository, OnboardingStatus, OnboardingStep};
pub use organization::{
    AddOrganizationMemberRequest, CreateOrganizationRequest, Organization, OrganizationMember,
    OrganizationRepository, UpdateOrganizationMemberRequest, UpdateOrganizationRequest,
};
pub use password_reset::{EmailVerificationToken, PasswordResetRepository, PasswordResetToken};
pub use permissions::{DefaultRoles, Permission, ResourcePermission, Role};
pub use plugin::{CreatePluginRequest, Plugin, PluginRepository, UpdatePluginRequest};
pub use presence::{
    PRESENCE_TTL_SECS, Presence, PresenceRepository, UpdatePresenceRequest, UpsertPresenceRequest,
};
pub use rbac::{
    AuditLogRepository, PolicyRepository, RolePermissionRepository, UserRoleRepository,
};
pub use refresh_token::{RefreshToken, RefreshTokenRepository};
pub use repository::{DocumentRepository, RepositoryRepository};
pub use saved_search::{
    CreateSavedSearchRequest, SavedSearch, SavedSearchRepository, UpdateSavedSearchRequest,
};
pub use schema::DatabasePool;
pub use search::{
    FacetCount, GlobalSearchResponse, ProjectSearchResult, SearchFacets, SearchFilters,
    SearchHighlight, SearchRepository, SearchResponse, SearchResult,
};
pub use session::SessionRepository;
pub use sms_otp::{SmsOtpRepository, SmsOtpToken};
pub use space::{
    AddSpaceMemberRequest, CreateSpaceRequest, Space, SpaceMember, SpaceRepository,
    UpdateSpaceMemberRequest, UpdateSpaceRequest,
};
pub use team::{RoleRecord, RoleRepository, Team, TeamMember, TeamRepository};
pub use template::{
    CreateTemplateRequest, DocumentTemplate, TemplateRepository, UpdateTemplateRequest,
};
pub use types::*;
pub use types::{Component, Project, ProjectMember};
pub use usage::{UsageRecord, UsageRepository, UsageSummary};
pub use user::{UserRecord, UserRepository};
pub use user_preferences::UserPreferencesRepository;
pub use webhook::{CreateWebhook, Webhook, WebhookRepository};

// Re-export tachyon-core types
pub use tachyon_core::id::*;

/// Database library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Get the configured database backend type.
pub fn database_backend() -> &'static str {
    #[cfg(feature = "sqlite")]
    {
        "sqlite"
    }
    #[cfg(not(feature = "sqlite"))]
    {
        "postgresql"
    }
}

#[cfg(feature = "sqlite")]
pub mod sqlite {
    //! SQLite backend support (experimental).
    pub type Pool = sqlx::SqlitePool;
    pub type QueryResult = sqlx::sqlite::SqliteQueryResult;
}

/// Initialize the database with the given connection URL
///
/// # Arguments
/// * `database_url` - PostgreSQL database connection URL (e.g., "postgres://user:pass@localhost/db")
///
/// # Returns
/// Result containing the initialized DatabasePool or error
///
/// # Errors
/// Returns error if database connection or initialization fails
pub async fn init(database_url: &str) -> DatabaseResult<DatabasePool> {
    DatabasePool::new(database_url).await
}

/// Initialize the database with the given connection URL and run migrations
///
/// # Arguments
/// * `database_url` - PostgreSQL database connection URL (e.g., "postgres://user:pass@localhost/db")
///
/// # Returns
/// Result containing the initialized DatabasePool or error
///
/// # Errors
/// Returns error if database connection, migration, or initialization fails
pub async fn init_with_migrations(database_url: &str) -> DatabaseResult<DatabasePool> {
    let pool = DatabasePool::new(database_url).await?;
    migrations::run_migrations(&pool).await?;
    Ok(pool)
}
