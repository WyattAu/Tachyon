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
pub mod ediscovery;
pub mod document;
pub mod e2e_encryption;
pub mod ecosystem;
pub mod files;
pub mod gdpr;
pub mod graph_api;
pub mod health;
pub mod hipaa;
pub mod import;
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

// eDiscovery exports
pub use ediscovery::{EdiscoveryExportRequest, EdiscoveryState, export_ediscovery};

/// Create a test router with all routes for integration testing
pub async fn create_router() -> Router {
    use crate::routes::activity::{ActivityState, create_activity_router};
    use crate::routes::billing::{BillingState, create_billing_router};
    use crate::routes::catalog::{CatalogState, create_catalog_router};
    use crate::routes::conflict::{ConflictState, create_conflict_router};
    use crate::routes::digest::{DigestState, create_digest_router};
    use crate::routes::document::{DocumentState, create_document_router};
    use crate::routes::import::{ImportState, create_import_router};
    use crate::routes::node::{NodeState, create_node_router};
    use crate::routes::notification::{NotificationState, create_notification_router};
    use crate::routes::onboarding::{OnboardingState, create_onboarding_router};
    use crate::routes::organization::{OrganizationState, create_organization_router};
    use crate::routes::plugin::{PluginState, create_plugin_router_with_state};
    use crate::routes::repository::{RepositoryState, create_repository_router};
    use crate::routes::review::{ReviewState, create_review_router};
    use crate::routes::role::{RoleState, create_role_router};
    use crate::routes::search::{SearchState, create_search_router};
    use crate::routes::seo::{SeoState, create_seo_router};
    use crate::routes::session::{SessionState, create_session_router};
    use crate::routes::space::{SpaceState, create_space_router};
    use crate::routes::ssg::{SsgState, create_ssg_router};
    use crate::routes::tags::{TagsState, create_tags_router};
    use crate::routes::team::{TeamState, create_team_router};
    use crate::routes::user::{UserState, create_user_router};
    use crate::routes::webhook::{WebhookState, create_webhook_router};
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
    let import_state = ImportState {
        pool: pool.clone(),
        last_import: std::sync::Arc::new(std::sync::Mutex::new(std::time::Instant::now())),
    };

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
    let ediscovery_state = crate::routes::ediscovery::EdiscoveryState {
        pool: pool.clone(),
    };
    let ediscovery_router =
        crate::routes::ediscovery::create_ediscovery_router().with_state(ediscovery_state);
    let conflict_router = create_conflict_router().with_state(conflict_state);
    let seo_router = create_seo_router().with_state(seo_state);
    let digest_router = create_digest_router().with_state(digest_state);
    let import_router = create_import_router().with_state(import_state);

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
        .merge(ediscovery_router)
        .merge(conflict_router)
        .merge(seo_router)
        .merge(digest_router)
        .merge(import_router);

    Router::new().nest("/api/v1", api_v1)
}

// Catalog exports
pub use catalog::{
    AddMemberRequest, ApiResponse, CatalogState, PaginationParams, ProjectFilters,
    create_catalog_router, create_component, create_project, delete_component, delete_project,
    get_catalog_stats, get_component, get_project, get_project_by_slug, list_project_components,
    list_project_members, list_projects, remove_project_member, update_project,
};

// Graph API exports
pub use graph_api::{
    GraphApiState, GraphEdgesResponse, GraphNodesResponse, create_graph_api_router,
};

// Document exports
pub use document::{
    DocumentQuery, DocumentState, UpdateDocumentRequest, create_document, create_document_router,
    delete_document, get_document, list_documents, search_documents, update_document,
};

// Node exports
pub use node::{
    CreateEdgeRequest, CreateNodeRequest, GraphQueryRequest, NodeQuery, NodeState,
    UpdateNodeRequest, create_edge, create_node, create_node_router, delete_edge, delete_node,
    get_graph_stats, get_node, get_node_edges, list_nodes, query_graph, update_node,
};

// Repository exports
pub use repository::{
    RepositoryState, clone_repository, commit, create_repository_router, delete_repository,
    get_repository, init_repository, list_repositories, push, status,
};

// Role exports
pub use role::{
    CreateRoleRequest, RoleResponse, RoleState, UpdateRoleRequest, create_role, create_role_router,
    delete_role, get_role, list_roles, seed_default_roles, update_role,
};

// Search exports
pub use search::{
    CreateSavedSearchBody, SavedSearchResponse, SearchQuery, SearchResultsResponse, SearchState,
    UpdateSavedSearchBody, create_saved_search, create_search_router, delete_saved_search,
    get_saved_search, global_search, list_saved_searches, search, update_saved_search,
};

// Session exports
pub use session::{
    SessionState, create_session, create_session_router, get_session, list_sessions,
    revoke_all_sessions, revoke_session, validate_session,
};

// Team exports
pub use team::{
    AddMemberRequest as TeamAddMemberRequest, CreateTeamRequest, TeamMemberResponse, TeamQuery,
    TeamResponse, TeamState, UpdateMemberRequest, UpdateTeamRequest, add_team_member, create_team,
    create_team_router, delete_team, get_team, get_team_by_slug, list_team_members, list_teams,
    remove_team_member, update_team, update_team_member,
};

// User exports
pub use user::{
    AuthenticateRequest, AuthenticateResponse, CreateUserRequest, UpdateProfileRequest,
    UpdateUserRequest, UserListResponse, UserQuery, UserResponse, UserState, auth_status,
    authenticate, create_user, create_user_router, delete_user, get_me, get_user, list_users,
    logout, update_me, update_user,
};

// SEO exports
pub use seo::{SeoState, create_seo_router};

// Review exports
pub use review::{
    CommentResponse, ReviewResponse, ReviewState, ReviewStatusResponse, create_comment,
    create_review, create_review_router, get_review_status, list_comments, list_reviews,
    update_review,
};

// Activity exports
pub use activity::{
    ActivityCursorPage, ActivityListResponse, ActivityState, ListActivityQuery, create_activity,
    create_activity_router, list_activity, list_activity_cursor,
};

// Notification exports
pub use notification::{
    ListNotificationsQuery, MarkAllReadResponse, MarkReadResponse, NotificationCursorPage,
    NotificationListResponse, NotificationState, UnreadCountResponse, create_notification_router,
    list_notifications, list_notifications_cursor, mark_all_read, mark_notification_read,
    unread_count,
};

// Conflict exports
pub use conflict::{
    ConflictInfo, ConflictState, MergeResultInfo, ResolveConflictRequest, create_conflict_router,
    get_conflict_info, resolve_conflict,
};

// Tags exports
pub use tags::{TagInfo, TagsResponse, TagsState, create_tags_router, list_tags};

// Webhook exports
pub use webhook::{
    CreateWebhookBody, WebhookResponse, WebhookState, create_webhook, create_webhook_router,
    delete_webhook, list_webhooks,
};

// Plugin exports
pub use plugin::{
    CreatePluginBody, PluginCursorPage, PluginResponse, PluginState, UpdatePluginBody,
    create_plugin, create_plugin_router_with_state, delete_plugin, get_plugin, list_plugins,
    list_plugins_cursor, update_plugin,
};

// Space exports
pub use space::{
    AddMemberBody, CreateSpaceBody, SpaceCursorPage, SpaceMemberResponse, SpaceQuery,
    SpaceResponse, SpaceState, UpdateMemberBody, UpdateSpaceBody, create_space_router,
    list_spaces_cursor,
};

// Billing exports
pub use billing::{
    BillingState, InvoicesResponse, Plan, PlanDetails, PlanInfo, PlansResponse,
    SubscriptionResponse, UsageMetrics, UsageResponse, cancel_subscription, create_billing_router,
    create_subscription, get_subscription, get_usage, list_invoices, list_plans,
};

// Organization exports
pub use organization::{
    AddMemberBody as OrgAddMemberBody, CreateOrganizationBody, OrganizationCursorPage,
    OrganizationMemberResponse, OrganizationQuery, OrganizationResponse, OrganizationState,
    UpdateMemberBody as OrgUpdateMemberBody, UpdateOrganizationBody, add_member,
    create_organization, create_organization_router, delete_organization, get_organization,
    list_members, list_organizations, list_organizations_cursor, remove_member, update_member,
    update_organization,
};

// Password reset exports
pub use password_reset::{
    EmailVerifyConfirm, EmailVerifyRequest, MessageResponse as PasswordResetMessageResponse,
    PasswordResetConfirm, PasswordResetRequest, PasswordResetState, create_password_reset_router,
};

// Magic link exports
pub use magic_link::{
    MagicLinkMessageResponse, MagicLinkRequest, MagicLinkState, MagicLinkVerify,
    create_magic_link_router,
};

// Files exports
pub use files::{FilesState, UploadResponse, create_files_router};

// SSG exports
pub use ssg::{
    SsgBuildRequest, SsgBuildResponse, SsgBuildResultWrapper, SsgNavLink, SsgState, build_site,
    create_ssg_router, download_site,
};

// Onboarding exports
pub use onboarding::{
    CompleteStepRequest, CompleteStepResponse, OnboardingState, OnboardingStatusResponse,
    SampleContentResponse, SuggestionsResponse, TemplateSuggestion, create_onboarding_router,
};

// Comment exports
pub use comments::{CommentState, create_comment_router};

// SMS OTP exports
pub use sms_otp::{
    SmsOtpMessageResponse, SmsOtpRequest, SmsOtpRouteState, SmsOtpVerify, create_sms_otp_router,
};

// E2E Encryption exports
pub use e2e_encryption::{
    E2eState, EncryptionKeyMeta, EncryptionStatus, RegisterKeyRequest, register_encryption_key,
};

// Compliance (SOC 2) exports
pub use compliance::{ComplianceChecklist, ComplianceItem, Soc2Category, generate_soc2_checklist};

// GDPR exports
pub use gdpr::{
    GdprActivityEntry, GdprDataExport, GdprDeletionResult, GdprDocumentsSummary, GdprPersonalData,
    generate_data_export, generate_deletion_confirmation,
};

// HIPAA exports
pub use hipaa::{HipaaAuditEntry, HipaaComplianceStatus, hipaa_compliance_status};
