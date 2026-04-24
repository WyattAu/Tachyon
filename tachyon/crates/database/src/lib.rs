// Tachyon Database - PostgreSQL Database Layer
// Provides metadata storage, session persistence, and RBAC mapping tables

pub mod activity;
pub mod attachment;
pub mod billing;
pub mod catalog;
pub mod comment;
pub mod document_review;
pub mod document_version;
pub mod error;
pub mod graph;
pub mod migrations;
pub mod notification;
pub mod onboarding;
pub mod organization;
pub mod password_reset;
pub mod permissions;
pub mod plugin;
pub mod presence;
pub mod rbac;
pub mod repository;
pub mod saved_search;
pub mod schema;
pub mod search;
pub mod session;
pub mod space;
pub mod team;
pub mod template;
pub mod types;
pub mod user;
pub mod user_preferences;
pub mod webhook;

// Re-export common types for convenience
pub use activity::{ActivityEvent, ActivityRepository, CreateActivityEvent};
pub use attachment::{Attachment, AttachmentRepository, CreateAttachmentRequest};
pub use catalog::CatalogRepository;
pub use catalog::{CatalogStats, CreateComponentRequest, CreateProjectRequest};
pub use document_review::{
    CreateCommentRequest, CreateReviewRequest, DocumentReview, DocumentReviewRepository,
    ReviewComment, ReviewStatus, UpdateReviewRequest,
};
pub use document_version::{CreateVersionRequest, DocumentVersion, DocumentVersionRepository};
pub use error::{DatabaseError, DatabaseResult};
pub use notification::{CreateNotification, Notification, NotificationRepository};
pub use organization::{
    Organization, OrganizationMember, OrganizationRepository,
    CreateOrganizationRequest, UpdateOrganizationRequest,
    AddOrganizationMemberRequest, UpdateOrganizationMemberRequest,
};
pub use permissions::{DefaultRoles, Permission, ResourcePermission, Role};
pub use template::{CreateTemplateRequest, DocumentTemplate, TemplateRepository, UpdateTemplateRequest};
pub use types::{Component, Project, ProjectMember};
pub use rbac::{
    AuditLogRepository, PolicyRepository, RolePermissionRepository, UserRoleRepository,
};
pub use repository::{DocumentRepository, RepositoryRepository};
pub use saved_search::{CreateSavedSearchRequest, SavedSearch, SavedSearchRepository, UpdateSavedSearchRequest};
pub use schema::DatabasePool;
pub use search::{
    FacetCount, GlobalSearchResponse, ProjectSearchResult, SearchFacets, SearchFilters,
    SearchRepository, SearchResponse, SearchResult, SearchHighlight,
};
pub use session::SessionRepository;
pub use team::{RoleRecord, RoleRepository, Team, TeamMember, TeamRepository};
pub use graph::{GraphDiff, GraphRepository};
pub use user::{UserRecord, UserRepository};
pub use user_preferences::UserPreferencesRepository;
pub use webhook::{CreateWebhook, Webhook, WebhookRepository};
pub use plugin::{CreatePluginRequest, Plugin, PluginRepository, UpdatePluginRequest};
pub use comment::{
    Comment, CommentRepository,
    CreateCommentRequest as CreateDocumentCommentRequest,
    UpdateCommentRequest as UpdateDocumentCommentRequest,
};
pub use billing::{
    Subscription, SubscriptionRepository, CreateSubscriptionRequest, UpdateSubscriptionRequest,
    Invoice, InvoiceRepository, CreateInvoiceRequest, UpdateInvoiceRequest,
    NotificationPreference, NotificationPreferenceRepository, UpsertNotificationPrefRequest,
};
pub use password_reset::{
    PasswordResetRepository, PasswordResetToken, EmailVerificationToken,
};
pub use onboarding::{OnboardingRepository, OnboardingStatus, OnboardingStep};
pub use presence::{Presence, PresenceRepository, UpsertPresenceRequest, UpdatePresenceRequest, PRESENCE_TTL_SECS};
pub use space::{
    AddSpaceMemberRequest, CreateSpaceRequest, Space, SpaceMember, SpaceRepository,
    UpdateSpaceMemberRequest, UpdateSpaceRequest,
};
pub use types::*;

// Re-export tachyon-core types
pub use tachyon_core::id::*;

/// Database library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

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
