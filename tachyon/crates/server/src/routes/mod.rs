// API route handlers
// Defines all HTTP endpoints for the Tachyon server

pub mod activity;
pub mod ai_routes;
pub mod billing;
pub mod catalog;
pub mod collaboration;
pub mod comments;
pub mod compliance;
pub mod conflict;
pub mod digest;
pub mod document;
pub mod e2e_encryption;
pub mod ecosystem;
pub mod files;
pub mod gdpr;
pub mod health;
pub mod hipaa;
pub mod landing;
pub mod magic_link;
pub mod metrics;
pub mod mfa;
pub mod node;
pub mod notification;
pub mod oauth2;
pub mod onboarding;
pub mod organization;
pub mod password_reset;
pub mod plugin;
pub mod push;
pub mod repository;
pub mod review;
pub mod role;
pub mod search;
pub mod seo;
pub mod session;
pub mod siem;
pub mod signup;
pub mod sms_otp;
pub mod space;
pub mod ssg;
pub mod swagger;
pub mod tags;
pub mod team;
pub mod template_marketplace;
pub mod user;
pub mod v2;
pub mod webhook;

use crate::config::GuestConfig;
use axum::Router;

/// Create a test router with all routes for integration testing
pub async fn create_router() -> Router {
    use crate::routes::activity::{create_activity_router, ActivityState};
    use crate::routes::billing::{create_billing_router, BillingState};
    use crate::routes::catalog::{create_catalog_router, CatalogState};
    use crate::routes::conflict::{create_conflict_router, ConflictState};
    use crate::routes::digest::{create_digest_router, DigestState};
    use crate::routes::document::{create_document_router, DocumentState};
    use crate::routes::node::{create_node_router, NodeState};
    use crate::routes::notification::{create_notification_router, NotificationState};
    use crate::routes::onboarding::{create_onboarding_router, OnboardingState};
    use crate::routes::organization::{create_organization_router, OrganizationState};
    use crate::routes::plugin::{create_plugin_router_with_state, PluginState};
    use crate::routes::repository::{create_repository_router, RepositoryState};
    use crate::routes::review::{create_review_router, ReviewState};
    use crate::routes::role::{create_role_router, RoleState};
    use crate::routes::search::{create_search_router, SearchState};
    use crate::routes::seo::{create_seo_router, SeoState};
    use crate::routes::session::{create_session_router, SessionState};
    use crate::routes::space::{create_space_router, SpaceState};
    use crate::routes::ssg::{create_ssg_router, SsgState};
    use crate::routes::tags::{create_tags_router, TagsState};
    use crate::routes::team::{create_team_router, TeamState};
    use crate::routes::user::{create_user_router, UserState};
    use crate::routes::webhook::{create_webhook_router, WebhookState};
    use crate::websocket::ConnectionManager;
    use tachyon_database::init_with_migrations;

    // Use test database URL or default
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://tachyon:tachyon@localhost:5433/tachyon_test".to_string());

    // Initialize database pool
    let pool = init_with_migrations(&database_url)
        .await
        .expect("Failed to connect to test database");

    let guest_config = GuestConfig {
        guest_login_enabled: true,
        public_notes_enabled: true,
        guest_user_id: "00000000-0000-0000-0000-000000000000".to_string(),
    };

    let document_state = DocumentState::with_guest_config(
        pool.clone(),
        guest_config.clone(),
        reqwest::Client::new(),
    );
    let user_state = UserState::with_guest_config(
        pool.clone(),
        vec!["test_secret_key_that_is_at_least_32_ch".to_string()],
        3600,
        "tachyon".to_string(),
        "tachyon".to_string(),
        guest_config,
    );
    let session_state = SessionState::new(pool.clone(), 3600);
    let repository_state = RepositoryState::new(pool.clone());
    let node_state = NodeState::new(pool.clone());
    let catalog_state = CatalogState::new(pool.clone());
    let review_state = ReviewState::new(pool.clone(), reqwest::Client::new());
    let activity_state = ActivityState::new(pool.clone());
    let notification_state = NotificationState::new(pool.clone());
    let tags_state = TagsState { pool: pool.clone() };
    let webhook_state = WebhookState {
        pool: pool.clone(),
        audit_logger: crate::audit::AuditLogger::disabled(),
    };
    let plugin_runtime = tachyon_plugin_runtime::PluginRuntime::new(
        std::env::temp_dir().join("tachyon-test-plugins"),
    );
    let plugin_state = PluginState {
        pool: pool.clone(),
        runtime: plugin_runtime,
        audit_logger: crate::audit::AuditLogger::disabled(),
    };
    let space_state = SpaceState {
        pool: pool.clone(),
        audit_logger: crate::audit::AuditLogger::disabled(),
    };
    let team_state = TeamState::new(pool.clone());
    let role_state = RoleState::new(pool.clone());
    let search_state = SearchState::new(pool.clone());
    let billing_state = BillingState::new(pool.clone(), None); // No TrueLayer in tests
    let organization_state = OrganizationState {
        pool: pool.clone(),
        audit_logger: crate::audit::AuditLogger::disabled(),
    };
    let ssg_state = SsgState::new(pool.clone());
    let onboarding_state = OnboardingState { pool: pool.clone() };
    let conflict_state = ConflictState { pool: pool.clone() };
    let seo_state = SeoState {
        pool: pool.clone(),
        site_config: crate::config::SiteConfig::default(),
    };
    let digest_state = DigestState { pool: pool.clone() };
    let _connection_manager = ConnectionManager::new();

    let document_router = create_document_router().with_state(document_state);
    let user_router = create_user_router().with_state(user_state);
    let session_router = create_session_router().with_state(session_state);
    let repository_router = create_repository_router().with_state(repository_state);
    let node_router = create_node_router().with_state(node_state);
    let catalog_router = create_catalog_router().with_state(catalog_state);
    let review_router = create_review_router().with_state(review_state);
    let activity_router = create_activity_router().with_state(activity_state);
    let notification_router = create_notification_router().with_state(notification_state);
    let tags_router = create_tags_router().with_state(tags_state);
    let webhook_router = create_webhook_router().with_state(webhook_state);
    let plugin_router = create_plugin_router_with_state(plugin_state);
    let space_router = create_space_router().with_state(space_state);
    let team_router = create_team_router().with_state(team_state);
    let role_router = create_role_router().with_state(role_state);
    let search_router = create_search_router().with_state(search_state);
    let billing_router = create_billing_router().with_state(billing_state);
    let organization_router = create_organization_router().with_state(organization_state);
    let ssg_router = create_ssg_router().with_state(ssg_state);
    let onboarding_router = create_onboarding_router().with_state(onboarding_state);
    let conflict_router = create_conflict_router().with_state(conflict_state);
    let seo_router = create_seo_router().with_state(seo_state);
    let digest_router = create_digest_router().with_state(digest_state);

    let api_v1 = Router::new()
        .merge(document_router)
        .merge(user_router)
        .merge(session_router)
        .merge(repository_router)
        .merge(node_router)
        .merge(catalog_router)
        .merge(review_router)
        .merge(activity_router)
        .merge(notification_router)
        .merge(tags_router)
        .merge(webhook_router)
        .merge(plugin_router)
        .merge(space_router)
        .merge(team_router)
        .merge(role_router)
        .merge(search_router)
        .merge(billing_router)
        .merge(organization_router)
        .merge(ssg_router)
        .merge(onboarding_router)
        .merge(conflict_router)
        .merge(seo_router)
        .merge(digest_router);

    Router::new().nest("/api/v1", api_v1)
}

// Catalog exports
pub use catalog::{
    create_catalog_router, create_component, create_project, delete_component, delete_project,
    get_catalog_stats, get_component, get_project, get_project_by_slug, list_project_components,
    list_project_members, list_projects, remove_project_member, update_project, AddMemberRequest,
    ApiResponse, CatalogState, PaginationParams, ProjectFilters,
};

// Document exports
pub use document::{
    create_document, create_document_router, delete_document, get_document, list_documents,
    search_documents, update_document, DocumentQuery, DocumentState, UpdateDocumentRequest,
};

// Node exports
pub use node::{
    create_edge, create_node, create_node_router, delete_edge, delete_node, get_graph_stats,
    get_node, get_node_edges, list_nodes, query_graph, update_node, CreateEdgeRequest,
    CreateNodeRequest, GraphQueryRequest, NodeQuery, NodeState, UpdateNodeRequest,
};

// Repository exports
pub use repository::{
    clone_repository, commit, create_repository_router, delete_repository, get_repository,
    init_repository, list_repositories, push, status, RepositoryState,
};

// Role exports
pub use role::{
    create_role, create_role_router, delete_role, get_role, list_roles, seed_default_roles,
    update_role, CreateRoleRequest, RoleResponse, RoleState, UpdateRoleRequest,
};

// Search exports
pub use search::{
    create_saved_search, create_search_router, delete_saved_search, get_saved_search,
    global_search, list_saved_searches, search, update_saved_search, CreateSavedSearchBody,
    SavedSearchResponse, SearchQuery, SearchResultsResponse, SearchState, UpdateSavedSearchBody,
};

// Session exports
pub use session::{
    create_session, create_session_router, get_session, list_sessions, revoke_all_sessions,
    revoke_session, validate_session, SessionState,
};

// Team exports
pub use team::{
    add_team_member, create_team, create_team_router, delete_team, get_team, get_team_by_slug,
    list_team_members, list_teams, remove_team_member, update_team, update_team_member,
    AddMemberRequest as TeamAddMemberRequest, CreateTeamRequest, TeamMemberResponse, TeamQuery,
    TeamResponse, TeamState, UpdateMemberRequest, UpdateTeamRequest,
};

// User exports
pub use user::{
    auth_status, authenticate, create_user, create_user_router, delete_user, get_me, get_user,
    list_users, logout, update_me, update_user, AuthenticateRequest, AuthenticateResponse,
    CreateUserRequest, UpdateProfileRequest, UpdateUserRequest, UserListResponse, UserQuery,
    UserResponse, UserState,
};

// SEO exports
pub use seo::{create_seo_router, SeoState};

// Review exports
pub use review::{
    create_comment, create_review, create_review_router, get_review_status, list_comments,
    list_reviews, update_review, CommentResponse, ReviewResponse, ReviewState,
    ReviewStatusResponse,
};

// Activity exports
pub use activity::{
    create_activity, create_activity_router, list_activity, list_activity_cursor,
    ActivityCursorPage, ActivityListResponse, ActivityState, ListActivityQuery,
};

// Notification exports
pub use notification::{
    create_notification_router, list_notifications, list_notifications_cursor, mark_all_read,
    mark_notification_read, unread_count, ListNotificationsQuery, MarkAllReadResponse,
    MarkReadResponse, NotificationCursorPage, NotificationListResponse, NotificationState,
    UnreadCountResponse,
};

// Conflict exports
pub use conflict::{
    create_conflict_router, get_conflict_info, resolve_conflict, ConflictInfo, ConflictState,
    MergeResultInfo, ResolveConflictRequest,
};

// Tags exports
pub use tags::{create_tags_router, list_tags, TagInfo, TagsResponse, TagsState};

// Webhook exports
pub use webhook::{
    create_webhook, create_webhook_router, delete_webhook, list_webhooks, CreateWebhookBody,
    WebhookResponse, WebhookState,
};

// Plugin exports
pub use plugin::{
    create_plugin, create_plugin_router_with_state, delete_plugin, get_plugin, list_plugins,
    list_plugins_cursor, update_plugin, CreatePluginBody, PluginCursorPage, PluginResponse,
    PluginState, UpdatePluginBody,
};

// Space exports
pub use space::{
    create_space_router, list_spaces_cursor, AddMemberBody, CreateSpaceBody, SpaceCursorPage,
    SpaceMemberResponse, SpaceQuery, SpaceResponse, SpaceState, UpdateMemberBody, UpdateSpaceBody,
};

// Billing exports
pub use billing::{
    cancel_subscription, create_billing_router, create_subscription, get_subscription, get_usage,
    list_invoices, list_plans, BillingState, InvoicesResponse, Plan, PlanDetails, PlanInfo,
    PlansResponse, SubscriptionResponse, UsageMetrics, UsageResponse,
};

// Organization exports
pub use organization::{
    add_member, create_organization, create_organization_router, delete_organization,
    get_organization, list_members, list_organizations, list_organizations_cursor, remove_member,
    update_member, update_organization, AddMemberBody as OrgAddMemberBody, CreateOrganizationBody,
    OrganizationCursorPage, OrganizationMemberResponse, OrganizationQuery, OrganizationResponse,
    OrganizationState, UpdateMemberBody as OrgUpdateMemberBody, UpdateOrganizationBody,
};

// Password reset exports
pub use password_reset::{
    create_password_reset_router, EmailVerifyConfirm, EmailVerifyRequest,
    MessageResponse as PasswordResetMessageResponse, PasswordResetConfirm, PasswordResetRequest,
    PasswordResetState,
};

// Magic link exports
pub use magic_link::{
    create_magic_link_router, MagicLinkMessageResponse, MagicLinkRequest, MagicLinkState,
    MagicLinkVerify,
};

// Files exports
pub use files::{create_files_router, FilesState, UploadResponse};

// SSG exports
pub use ssg::{
    build_site, create_ssg_router, download_site, SsgBuildRequest, SsgBuildResponse,
    SsgBuildResultWrapper, SsgNavLink, SsgState,
};

// Onboarding exports
pub use onboarding::{
    create_onboarding_router, CompleteStepRequest, CompleteStepResponse, OnboardingState,
    OnboardingStatusResponse, SampleContentResponse, SuggestionsResponse, TemplateSuggestion,
};

// Comment exports
pub use comments::{create_comment_router, CommentState};

// SMS OTP exports
pub use sms_otp::{
    create_sms_otp_router, SmsOtpMessageResponse, SmsOtpRequest, SmsOtpRouteState, SmsOtpVerify,
};

// E2E Encryption exports
pub use e2e_encryption::{
    register_encryption_key, E2eState, EncryptionKeyMeta, EncryptionStatus, RegisterKeyRequest,
};

// Compliance (SOC 2) exports
pub use compliance::{generate_soc2_checklist, ComplianceChecklist, ComplianceItem, Soc2Category};

// GDPR exports
pub use gdpr::{
    generate_data_export, generate_deletion_confirmation, GdprActivityEntry, GdprDataExport,
    GdprDeletionResult, GdprDocumentsSummary, GdprPersonalData,
};

// HIPAA exports
pub use hipaa::{hipaa_compliance_status, HipaaAuditEntry, HipaaComplianceStatus};
