// API route handlers
// Defines all HTTP endpoints for the Tachyon server

pub mod activity;
pub mod billing;
pub mod files;
pub mod catalog;
pub mod collaboration;
pub mod conflict;
pub mod document;
pub mod ecosystem;
pub mod node;
pub mod notification;
pub mod oauth2;
pub mod onboarding;
pub mod organization;
pub mod password_reset;
pub mod plugin;
pub mod repository;
pub mod review;
pub mod role;
pub mod search;
pub mod seo;
pub mod session;
pub mod space;
pub mod ssg;
pub mod tags;
pub mod team;
pub mod user;
pub mod webhook;

use axum::Router;
use crate::config::GuestConfig;

/// Create a test router with all routes for integration testing
pub async fn create_router() -> Router {
    use tachyon_database::init_with_migrations;
    use crate::routes::activity::{ActivityState, create_activity_router};
    use crate::routes::billing::{BillingState, create_billing_router};
    use crate::routes::catalog::{CatalogState, create_catalog_router};
    use crate::routes::conflict::{ConflictState, create_conflict_router};
    use crate::routes::document::{DocumentState, create_document_router};
    use crate::routes::node::{NodeState, create_node_router};
    use crate::routes::notification::{NotificationState, create_notification_router};
    use crate::routes::onboarding::{OnboardingState, create_onboarding_router};
    use crate::routes::organization::{OrganizationState, create_organization_router};
    use crate::routes::repository::{RepositoryState, create_repository_router};
    use crate::routes::review::{ReviewState, create_review_router};
    use crate::routes::role::{RoleState, create_role_router};
    use crate::routes::search::{SearchState, create_search_router};
    use crate::routes::session::{SessionState, create_session_router};
    use crate::routes::seo::{SeoState, create_seo_router};
    use crate::routes::space::{SpaceState, create_space_router};
    use crate::routes::ssg::{SsgState, create_ssg_router};
    use crate::routes::tags::{TagsState, create_tags_router};
    use crate::routes::team::{TeamState, create_team_router};
    use crate::routes::user::{UserState, create_user_router};
    use crate::routes::webhook::{WebhookState, create_webhook_router};
    use crate::routes::plugin::{PluginState, create_plugin_router_with_state};
    use crate::websocket::ConnectionManager;

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

    let document_state = DocumentState::with_guest_config(pool.clone(), guest_config.clone(), reqwest::Client::new());
    let user_state = UserState::with_guest_config(
        pool.clone(),
        "test_secret_key".to_string(),
        3600,
        "tachyon".to_string(),
        "tachyon".to_string(),
        guest_config,
    );
    let session_state = SessionState::new(pool.clone(), 3600);
    let repository_state = RepositoryState::new();
    let node_state = NodeState::new(pool.clone());
    let catalog_state = CatalogState::new(pool.clone());
    let review_state = ReviewState::new(pool.clone(), reqwest::Client::new());
    let activity_state = ActivityState::new(pool.clone());
    let notification_state = NotificationState::new(pool.clone());
    let tags_state = TagsState { pool: pool.clone() };
    let webhook_state = WebhookState { pool: pool.clone() };
    let plugin_runtime = tachyon_plugin_runtime::PluginRuntime::new(
        std::env::temp_dir().join("tachyon-test-plugins"),
    );
    let plugin_state = PluginState { pool: pool.clone(), runtime: plugin_runtime };
    let space_state = SpaceState { pool: pool.clone() };
    let team_state = TeamState::new(pool.clone());
    let role_state = RoleState::new(pool.clone());
    let search_state = SearchState::new(pool.clone());
    let billing_state = BillingState::new(pool.clone(), None); // No TrueLayer in tests
    let organization_state = OrganizationState { pool: pool.clone() };
    let ssg_state = SsgState::new(pool.clone());
    let onboarding_state = OnboardingState { pool: pool.clone() };
    let conflict_state = ConflictState { pool: pool.clone() };
    let seo_state = SeoState {
        pool: pool.clone(),
        site_config: crate::config::SiteConfig::default(),
    };
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
        .merge(seo_router);

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
    search_documents, update_document, DocumentState, DocumentQuery, UpdateDocumentRequest,
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
    create_search_router, create_saved_search, delete_saved_search, get_saved_search,
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
    AddMemberRequest as TeamAddMemberRequest, CreateTeamRequest, TeamMemberResponse,
    TeamQuery, TeamResponse, TeamState, UpdateMemberRequest, UpdateTeamRequest,
};

// User exports
pub use user::{
    auth_status, authenticate, create_user, create_user_router, delete_user, get_me, get_user,
    list_users, logout, update_me, update_user, AuthenticateRequest, AuthenticateResponse,
    CreateUserRequest, UpdateProfileRequest, UpdateUserRequest, UserListResponse, UserQuery,
    UserResponse, UserState,
};

// SEO exports
pub use seo::{SeoState, create_seo_router};

// Review exports
pub use review::{
    create_review, create_review_router, list_reviews, update_review, create_comment, list_comments,
    get_review_status, ReviewState, ReviewResponse, ReviewStatusResponse, CommentResponse,
};

// Activity exports
pub use activity::{
    create_activity_router, list_activity, create_activity,
    ActivityState, ListActivityQuery, ActivityListResponse,
};

// Notification exports
pub use notification::{
    create_notification_router, list_notifications, unread_count,
    mark_notification_read, mark_all_read,
    NotificationState, ListNotificationsQuery, NotificationListResponse,
    UnreadCountResponse, MarkReadResponse, MarkAllReadResponse,
};

// Conflict exports
pub use conflict::{
    create_conflict_router, get_conflict_info, resolve_conflict, ConflictState,
    ConflictInfo, MergeResultInfo, ResolveConflictRequest,
};

// Tags exports
pub use tags::{
    create_tags_router, list_tags, TagsState, TagsResponse, TagInfo,
};

// Webhook exports
pub use webhook::{
    create_webhook_router, create_webhook, list_webhooks, delete_webhook,
    WebhookState, WebhookResponse, CreateWebhookBody,
};

// Plugin exports
pub use plugin::{
    create_plugin_router_with_state, list_plugins, get_plugin, create_plugin, update_plugin, delete_plugin,
    PluginState, PluginResponse, CreatePluginBody, UpdatePluginBody,
};

// Space exports
pub use space::{
    create_space_router, SpaceState, SpaceResponse, SpaceMemberResponse,
    CreateSpaceBody, UpdateSpaceBody, SpaceQuery, AddMemberBody, UpdateMemberBody,
};

// Billing exports
pub use billing::{
    create_billing_router, BillingState, Plan, PlanDetails, PlanInfo,
    PlansResponse, SubscriptionResponse, InvoicesResponse, UsageResponse,
    UsageMetrics, BillingErrorResponse,
    list_plans, create_subscription, get_subscription, list_invoices,
    get_usage, cancel_subscription,
};

// Organization exports
pub use organization::{
    create_organization_router, OrganizationState, OrganizationResponse,
    OrganizationMemberResponse, CreateOrganizationBody, UpdateOrganizationBody,
    OrganizationQuery, AddMemberBody as OrgAddMemberBody, UpdateMemberBody as OrgUpdateMemberBody,
    list_organizations, get_organization, create_organization, update_organization,
    delete_organization, list_members, add_member, update_member, remove_member,
};

// Password reset exports
pub use password_reset::{
    create_password_reset_router, PasswordResetState,
    PasswordResetRequest, PasswordResetConfirm,
    EmailVerifyRequest, EmailVerifyConfirm,
    MessageResponse as PasswordResetMessageResponse,
    ErrorResponse as PasswordResetErrorResponse,
};

// Files exports
pub use files::{
    create_files_router, FilesState,
};

// SSG exports
pub use ssg::{
    create_ssg_router, SsgState, SsgBuildRequest, SsgBuildResponse,
    SsgBuildResultWrapper, SsgNavLink, SsgErrorResponse,
    build_site, download_site,
};

// Onboarding exports
pub use onboarding::{
    create_onboarding_router, OnboardingState, OnboardingStatusResponse,
    CompleteStepRequest, CompleteStepResponse, SampleContentResponse,
    SuggestionsResponse, TemplateSuggestion,
};
