//! Tachyon Server Library
//!
//! A high-performance HTTP/2 server for knowledge management built with Axum.
//!
//! # Features
//!
//! - **REST API**: Comprehensive REST API with OpenAPI documentation
//! - **WebSocket Support**: Real-time collaboration via WebSocket
//! - **Authentication**: JWT and API key based authentication
//! - **Rate Limiting**: Configurable rate limiting with Redis support
//! - **CORS**: Configurable CORS support
//! - **Security Headers**: Automatic security headers middleware
//! - **Audit Logging**: Comprehensive audit trail for security
//!
//! # Architecture
//!
//! The server is organized into several modules:
//!
//! - [`config`]: Server configuration and settings
//! - [`routes`]: HTTP route handlers
//! - [`middleware`]: HTTP middleware (auth, CORS, rate limiting, security)
//! - [`websocket`]: WebSocket handlers for real-time features
//! - [`api_docs`]: OpenAPI/Swagger documentation
//! - [`validation`]: Request validation utilities
//! - [`audit`]: Audit logging for security compliance
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use tachyon_server::config::ServerConfig;
//!
//! // Load configuration from environment
//! let config = ServerConfig::from_env();
//!
//! // Validate configuration
//! config.validate().expect("Invalid configuration");
//! ```
//!
//! # Configuration
//!
//! Server configuration can be provided via:
//!
//! 1. Environment variables (recommended for production)
//! 2. Configuration file (`tachyon.toml`)
//! 3. Programmatically via `ServerConfig` struct
//!
//! ## Environment Variables
//!
//! | Variable | Description | Default |
//! |----------|-------------|---------|
//! | `TACHYON_HOST` | Server bind address | `0.0.0.0` |
//! | `TACHYON_PORT` | Server port | `8080` |
//! | `DATABASE_URL` | PostgreSQL connection string | Required |
//! | `TACHYON_JWT_SECRET` | JWT signing secret | Required |
//!
//! # Example
//!
//! ```rust,no_run
//! use tachyon_server::config::ServerConfig;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Load configuration from environment
//!     let config = ServerConfig::from_env();
//!
//!     // Validate configuration (fails fast on misconfiguration)
//!     if let Err(errors) = config.validate() {
//!         anyhow::bail!("Configuration errors: {:?}", errors);
//!     }
//!
//!     // Initialize application state (database pool, search index, etc.)
//!     let state = tachyon_server::init_app_state(&config).await?;
//!
//!     // Build the Axum router with all routes and middleware
//!     let app = tachyon_server::build_app(state, &config);
//!
//!     // Serve with axum's listener
//!     let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
//!     axum::serve(listener, app).await?;
//!     Ok(())
//! }
//! ```
//!
//! # Modules
//!
//! ## Configuration (`config`)
//!
//! Server configuration types and loading logic.
//!
//! ```rust
//! use tachyon_server::config::ServerConfig;
//!
//! let config = ServerConfig::default();
//! assert_eq!(config.host, "0.0.0.0");
//! assert_eq!(config.port, 8080);
//! ```
//!
//! ## Middleware (`middleware`)
//!
//! HTTP middleware for authentication, CORS, rate limiting, and security.
//!
//! ## Routes (`routes`)
//!
//! HTTP route handlers for documents, users, projects, and search.
//!
//! ## WebSocket (`websocket`)
//!
//! WebSocket handlers for real-time document collaboration.
//!
//! ## Audit (`audit`)
//!
//! Audit logging for tracking user actions and security events.
//!
//! # See Also
//!
//! - [User Guide](../../docs/user/README.md)
//! - [API Documentation](../../docs/api/authentication.md)
//! - [Architecture](../../docs/developer/architecture.md)

pub mod ai;
pub mod api_docs;
pub mod broadcast_bus;
pub mod audit;
pub mod config;
pub mod csrf_store;
pub mod conflict;
pub mod crdt;
pub mod dlp;
pub mod email;
pub mod error;
pub mod graph_extractor;
pub mod graphql;
pub mod integrations;
pub mod middleware;
pub mod notification_dispatch;
pub mod pagination;
pub mod proofs;
pub mod push;
pub mod routes;
pub mod sms;
pub mod sso;
pub mod storage;
pub mod sync;
pub mod tantivy_search;
pub mod totp;
pub mod truelayer;
pub mod validation;
pub mod webhook_delivery;
pub mod websocket;

#[doc(hidden)]
pub use api_docs::*;
#[doc(hidden)]
pub use audit::*;
#[doc(hidden)]
#[allow(ambiguous_glob_reexports)]
pub use config::*;
#[doc(hidden)]
pub use middleware::*;

use axum::extract::{Extension, State};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

/// Application state shared across all routes and middleware.
///
/// Contains all initialized services, pools, and configuration
/// needed by the Tachyon server.
#[derive(Clone)]
pub struct AppState {
    pub start_time: std::time::Instant,
    pub document_state: crate::routes::document::DocumentState,
    pub user_state: crate::routes::user::UserState,
    pub session_state: crate::routes::session::SessionState,
    pub repository_state: crate::routes::repository::RepositoryState,
    pub node_state: crate::routes::node::NodeState,
    pub catalog_state: crate::routes::catalog::CatalogState,
    pub team_state: crate::routes::team::TeamState,
    pub role_state: crate::routes::role::RoleState,
    pub search_state: crate::routes::search::SearchState,
    pub seo_state: crate::routes::seo::SeoState,
    pub review_state: crate::routes::review::ReviewState,
    pub activity_state: crate::routes::activity::ActivityState,
    pub notification_state: crate::routes::notification::NotificationState,
    pub notification_dispatcher: crate::notification_dispatch::NotificationDispatcher,
    pub tags_state: crate::routes::tags::TagsState,
    pub webhook_state: crate::routes::webhook::WebhookState,
    pub plugin_state: crate::routes::plugin::PluginState,
    pub space_state: crate::routes::space::SpaceState,
    pub conflict_state: crate::routes::conflict::ConflictState,
    pub onboarding_state: crate::routes::onboarding::OnboardingState,
    pub comment_state: crate::routes::comments::CommentState,
    pub digest_state: crate::routes::digest::DigestState,
    pub crdt_connection_manager: crate::websocket::CrdtConnectionManager,
    pub broadcast_bus: Arc<crate::broadcast_bus::SharedBroadcastBus>,
    pub ai_manager: Arc<crate::ai::AiManager>,
    pub pool: tachyon_database::DatabasePool,
    pub http_client: reqwest::Client,
    pub email: crate::email::EmailService,
    pub metrics: Arc<crate::middleware::metrics::RequestMetrics>,
    pub api_cache: crate::middleware::api_cache::ApiCache,
    pub audit_logger: crate::audit::AuditLogger,
    pub oidc_state: Option<crate::sso::OidcState>,
    pub saml_state: Option<crate::sso::SamlState>,
    pub ldap_state: Option<crate::sso::LdapState>,
}

/// Initialize application state from a [`ServerConfig`].
///
/// Creates the database pool, seeds the admin user (if needed),
/// and returns all state types needed by the router.
///
/// This is the single entry point for building a Tachyon server,
/// used by both `tachyon-server` (binary) and `tachyon serve` (CLI).
pub async fn init_app_state(config: &ServerConfig) -> anyhow::Result<AppState> {
    if let Err(errors) = config.validate() {
        for e in &errors {
            tracing::error!(config_error = %e, "Configuration validation failed");
        }
        anyhow::bail!(
            "Configuration validation failed with {} error(s):\n  {}",
            errors.len(),
            errors.join("\n  ")
        );
    }

    use crate::routes::activity::ActivityState;
    use crate::routes::catalog::CatalogState;
    use crate::routes::conflict::ConflictState;
    use crate::routes::document::DocumentState;
    use crate::routes::node::NodeState;
    use crate::routes::notification::NotificationState;
    use crate::routes::onboarding::OnboardingState;
    use crate::routes::plugin::PluginState;
    use crate::routes::repository::RepositoryState;
    use crate::routes::review::ReviewState;
    use crate::routes::role::RoleState;
    use crate::routes::search::SearchState;
    use crate::routes::session::SessionState;
    use crate::routes::space::SpaceState;
    use crate::routes::tags::TagsState;
    use crate::routes::team::TeamState;
    use crate::routes::user::UserState;
    use crate::routes::webhook::WebhookState;
    use crate::websocket::CrdtConnectionManager;
    use tachyon_database::init;
    use tachyon_database::init_with_migrations;

    let database_url = if config.database_url.is_empty() {
        config
            .database_path
            .as_deref()
            .unwrap_or("postgres://tachyon:tachyon@localhost:5432/tachyon")
    } else {
        &config.database_url
    };

    let pool = if std::env::var("TACHYON_SKIP_MIGRATIONS").is_ok() {
        tracing::warn!("TACHYON_SKIP_MIGRATIONS is set -- skipping database migrations");
        init(database_url).await.map_err(|e| anyhow::anyhow!("Failed to initialize database: {}", e))?
    } else {
        init_with_migrations(database_url)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to initialize database: {}", e))?
    };

    // Seed initial admin user if no users exist
    {
        let user_repo = tachyon_database::UserRepository::new(pool.clone());
        let admin_username =
            std::env::var("TACHYON_ADMIN_USERNAME").unwrap_or_else(|_| "admin".into());
        let admin_password = std::env::var("TACHYON_ADMIN_PASSWORD").unwrap_or_else(|_| {
            uuid::Uuid::new_v4().to_string().replace('-', "")[..16].to_string()
        });
        let admin_email =
            std::env::var("TACHYON_ADMIN_EMAIL").unwrap_or_else(|_| "admin@tachyon.local".into());

        match user_repo
            .seed_admin(
                &admin_username,
                "Administrator",
                &admin_email,
                &admin_password,
            )
            .await
        {
            Ok(Some(user)) => {
                tracing::info!("Initial admin user seeded: {} ({})", user.username, user.id);
            }
            Ok(None) => {
                tracing::info!("Admin seed skipped: users already exist");
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to seed admin user: {}. Users must be created manually.",
                    e
                );
            }
        }
    }

    let http_client = reqwest::Client::new();

    let audit_logger = crate::audit::AuditLogger::new(10_000).with_database(pool.inner().clone());

    let ai_manager = Arc::new(crate::ai::AiManager::from_env());
    if ai_manager.is_available() {
        tracing::info!("AI provider configured: {}", ai_manager.provider_name());
    }

    let document_state =
        DocumentState::with_guest_config(pool.clone(), config.guest.clone(), http_client.clone())
            .with_audit_logger(audit_logger.clone())
            .with_ai_manager(ai_manager.clone());
    let user_state = UserState::with_guest_config(
        pool.clone(),
        config.jwt.secrets.clone(),
        config.jwt.expiration_secs,
        config.jwt.issuer.clone(),
        config.jwt.audience.clone(),
        config.guest.clone(),
    )
    .with_audit_logger(audit_logger.clone());
    let session_state = SessionState::new(pool.clone(), config.jwt.expiration_secs)
        .with_audit_logger(audit_logger.clone());
    let repository_state =
        RepositoryState::new(pool.clone()).with_audit_logger(audit_logger.clone());
    let node_state = NodeState::new(pool.clone()).with_audit_logger(audit_logger.clone());
    let catalog_state = CatalogState::new(pool.clone()).with_audit_logger(audit_logger.clone());
    let team_state = TeamState::new(pool.clone()).with_audit_logger(audit_logger.clone());
    let role_state = RoleState::new(pool.clone()).with_audit_logger(audit_logger.clone());
    let search_state = SearchState::new(pool.clone()).with_audit_logger(audit_logger.clone());
    let seo_state = crate::routes::seo::SeoState {
        pool: pool.clone(),
        site_config: config.site.clone(),
    };
    let activity_state = ActivityState::new(pool.clone());
    let notification_state = NotificationState::new(pool.clone());
    let broadcast_bus = Arc::new(broadcast_bus::SharedBroadcastBus::new(1024));
    let notification_dispatcher = crate::notification_dispatch::NotificationDispatcher::new(
        pool.clone(),
        broadcast_bus.clone(),
        notification_state.sse_tx.clone(),
    );
    let review_state = ReviewState::new(pool.clone(), http_client.clone())
        .with_audit_logger(audit_logger.clone())
        .with_notification_dispatcher(notification_dispatcher.clone());
    let team_state =
        team_state.with_notification_dispatcher(notification_dispatcher.clone());
    let tags_state = TagsState { pool: pool.clone() };
    let webhook_state = WebhookState {
        pool: pool.clone(),
        audit_logger: audit_logger.clone(),
    };
    let plugins_dir = std::env::current_dir()
        .unwrap_or_else(|_| std::env::temp_dir())
        .join("plugins");
    let plugin_runtime = tachyon_plugin_runtime::PluginRuntime::new(plugins_dir);
    let plugin_state = PluginState {
        pool: pool.clone(),
        runtime: plugin_runtime,
        audit_logger: audit_logger.clone(),
    };
    let space_state = SpaceState {
        pool: pool.clone(),
        audit_logger: audit_logger.clone(),
    };
    let conflict_state = ConflictState { pool: pool.clone() };
    let onboarding_state = OnboardingState { pool: pool.clone() };
    let comment_state = crate::routes::comments::CommentState::new(pool.clone());
    let digest_state = crate::routes::digest::DigestState { pool: pool.clone() };
    let crdt_connection_manager = CrdtConnectionManager::with_pool(pool.inner().clone(), broadcast_bus.clone());

    let oidc_state = if !config.sso_oidc.is_empty() {
        Some(crate::sso::OidcState {
            configs: config.sso_oidc.clone(),
            pool: pool.clone(),
            jwt_secret: config.jwt.signing_secret().to_string(),
            http_client: reqwest::Client::new(),
            csrf_store: crate::csrf_store::CsrfStoreType::new(config.rate_limit.redis_url.as_deref()),
        })
    } else {
        None
    };

    let saml_state = config.sso_saml.as_ref().map(|cfg| crate::sso::SamlState {
        config: cfg.clone(),
        pool: pool.clone(),
        jwt_secret: config.jwt.signing_secret().to_string(),
        csrf_store: crate::csrf_store::CsrfStoreType::new(config.rate_limit.redis_url.as_deref()),
    });

    let ldap_state = config.sso_ldap.as_ref().map(|cfg| crate::sso::LdapState {
        config: cfg.clone(),
        pool: pool.clone(),
        jwt_secret: config.jwt.signing_secret().to_string(),
    });

    {
        let cleanup_crdt = crdt_connection_manager.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
            loop {
                interval.tick().await;
                cleanup_crdt.cleanup_stale_clients(300).await;
            }
        });
    }
    {
        let crdt_mgr = crdt_connection_manager.crdt_manager().clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
            loop {
                interval.tick().await;
                crdt_mgr.flush_dirty().await;
            }
        });
    }

    let email = crate::email::EmailService::new(config);

    Ok(AppState {
        start_time: std::time::Instant::now(),
        document_state,
        user_state,
        session_state,
        repository_state,
        node_state,
        catalog_state,
        team_state,
        role_state,
        search_state,
        seo_state,
        review_state,
        activity_state,
        notification_state,
        notification_dispatcher,
        tags_state,
        webhook_state,
        plugin_state,
        space_state,
        conflict_state,
        onboarding_state,
        comment_state,
        digest_state,
        crdt_connection_manager,
        broadcast_bus,
        pool,
        ai_manager,
        http_client,
        email,
        metrics: Arc::new(crate::middleware::metrics::RequestMetrics::new()),
        api_cache: crate::middleware::api_cache::ApiCache::new(std::time::Duration::from_secs(60)),
        audit_logger,
        oidc_state,
        saml_state,
        ldap_state,
    })
}

async fn graphql_handler(
    State(pool): State<tachyon_database::DatabasePool>,
    Extension(auth_context): Extension<crate::middleware::AuthContext>,
    request: async_graphql_axum::GraphQLRequest,
) -> async_graphql_axum::GraphQLResponse {
    let gql_auth = graphql::GraphqlAuthContext::from(auth_context);
    let schema = graphql::build_schema_with_data(pool);
    let request = request.into_inner().data(gql_auth);
    schema.execute(request).await.into()
}

async fn graphql_playground() -> impl axum::response::IntoResponse {
    axum::response::Html(async_graphql::http::playground_source(
        async_graphql::http::GraphQLPlaygroundConfig::new("/graphql"),
    ))
}

/// Build the full Axum router with all routes, middleware, and state.
///
/// This is the single entry point for creating the Tachyon HTTP/WebSocket
/// application, used by both `tachyon-server` (binary) and `tachyon serve` (CLI).
pub fn build_app(state: AppState, config: &ServerConfig) -> axum::Router {
    let start_time = state.start_time;
    let document_state = state.document_state;
    let user_state = state.user_state;
    let session_state = state.session_state;
    let repository_state = state.repository_state;
    let node_state = state.node_state;
    let catalog_state = state.catalog_state;
    let team_state = state.team_state;
    let role_state = state.role_state;
    let search_state = state.search_state;
    let seo_state = state.seo_state;
    let review_state = state.review_state;
    let activity_state = state.activity_state;
    let notification_state = state.notification_state;
    let notification_dispatcher = state.notification_dispatcher;
    let _notification_dispatcher = notification_dispatcher;
    let tags_state = state.tags_state;
    let webhook_state = state.webhook_state;
    let plugin_state = state.plugin_state;
    let space_state = state.space_state;
    let conflict_state = state.conflict_state;
    let onboarding_state = state.onboarding_state;
    let crdt_connection_manager = state.crdt_connection_manager;
    let broadcast_bus = state.broadcast_bus;
    let pool = state.pool;
    let ai_manager = state.ai_manager;
    let http_client = state.http_client;
    let metrics = state.metrics;
    let audit_logger = state.audit_logger;
    let oidc_state = state.oidc_state;
    let saml_state = state.saml_state;
    let ldap_state = state.ldap_state;
    use crate::routes::activity::create_activity_router;
    use crate::routes::billing::{BillingState, create_billing_router};
    use crate::routes::catalog::create_catalog_router;
    use crate::routes::collaboration::{CollaborationState, create_collaboration_router};
    use crate::routes::conflict::create_conflict_router;
    use crate::routes::digest::create_digest_router;
    use crate::routes::document::create_document_router;
    use crate::routes::ecosystem::{EcosystemState, create_ecosystem_router};
    use crate::routes::files::{FilesState, create_files_router};
    use crate::routes::mfa::create_mfa_router;
    use crate::routes::node::create_node_router;
    use crate::routes::notification::create_notification_router;
    use crate::routes::oauth2::{OAuth2State, create_oauth2_router};
    use crate::routes::onboarding::create_onboarding_router;
    use crate::routes::organization::{OrganizationState, create_organization_router};
    use crate::routes::password_reset::{PasswordResetState, create_password_reset_router};
    use crate::routes::plugin::create_plugin_router_with_state;
    use crate::routes::repository::create_repository_router;
    use crate::routes::review::create_review_router;
    use crate::routes::role::create_role_router;
    use crate::routes::search::create_search_router;
    use crate::routes::seo::create_seo_router;
    use crate::routes::session::create_session_router;
    use crate::routes::sms_otp::create_sms_otp_router;
    use crate::routes::space::create_space_router;
    use crate::routes::ssg::{SsgState, create_ssg_router};
    use crate::routes::tags::create_tags_router;
    use crate::routes::team::create_team_router;
    use crate::routes::user::create_user_router;
    use crate::routes::webhook::create_webhook_router;
    use crate::websocket::handle_crdt_websocket_upgrade;
    use axum::{Router, routing::get};
    use tower::ServiceBuilder;
    use tower_http::{
        compression::CompressionLayer, limit::RequestBodyLimitLayer, trace::TraceLayer,
    };

    let document_router = create_document_router().with_state(document_state.clone());
    let user_router = create_user_router().with_state(user_state.clone());
    let session_router = create_session_router().with_state(session_state);
    let repository_router = create_repository_router().with_state(repository_state);
    let node_router = create_node_router().with_state(node_state);
    let catalog_router = create_catalog_router().with_state(catalog_state);
    let team_router = create_team_router().with_state(team_state);
    let role_router = create_role_router().with_state(role_state);
    let search_router = create_search_router().with_state(search_state.clone());
    let seo_router = create_seo_router().with_state(seo_state);
    let review_router = create_review_router().with_state(review_state);
    let activity_router = create_activity_router().with_state(activity_state);
    let notification_router = create_notification_router().with_state(notification_state);
    let tags_router = create_tags_router().with_state(tags_state);
    let webhook_router = create_webhook_router().with_state(webhook_state);
    let conflict_router = create_conflict_router().with_state(conflict_state);
    let plugin_router = create_plugin_router_with_state(plugin_state);
    let space_router = create_space_router().with_state(space_state);
    let onboarding_router = create_onboarding_router().with_state(onboarding_state);
    let collaboration_state = CollaborationState::new(pool.clone(), broadcast_bus.clone());
    let collaboration_router = create_collaboration_router().with_state(collaboration_state);
    let ecosystem_state = EcosystemState::new(pool.clone());
    let ecosystem_router = create_ecosystem_router().with_state(ecosystem_state);

    // Billing, Organization, SSG routers (pool-backed, created inline)
    let truelayer_client = if config.truelayer.enabled {
        let client = crate::truelayer::TrueLayerClient::new(&config.truelayer, http_client.clone());
        if client.is_enabled() {
            tracing::info!("TrueLayer payment processing enabled");
            Some(client)
        } else {
            tracing::warn!("TrueLayer enabled but missing configuration");
            None
        }
    } else {
        None
    };
    let billing_state =
        BillingState::new(pool.clone(), truelayer_client).with_audit_logger(audit_logger.clone());
    let billing_router = create_billing_router().with_state(billing_state);
    let organization_state = OrganizationState {
        pool: pool.clone(),
        audit_logger: audit_logger.clone(),
    };
    let organization_router = create_organization_router().with_state(organization_state);
    let ssg_state = SsgState::new(pool.clone());
    let ssg_router = create_ssg_router().with_state(ssg_state);

    // OAuth2 router (only enabled when providers are configured)
    let oauth2_state = OAuth2State {
        jwt_secrets: config.jwt.secrets.clone(),
        jwt_expiration_secs: config.jwt.expiration_secs,
        jwt_issuer: config.jwt.issuer.clone(),
        jwt_audience: config.jwt.audience.clone(),
        config: config.oauth2.clone(),
        pool: pool.clone(),
        client: http_client.clone(),
        csrf_store: crate::csrf_store::CsrfStoreType::new(config.rate_limit.redis_url.as_deref()),
    };
    let oauth2_router = create_oauth2_router().with_state(oauth2_state);
    let password_reset_state = PasswordResetState {
        pool: pool.clone(),
        client: http_client.clone(),
        audit_logger: audit_logger.clone(),
    };
    let password_reset_router = create_password_reset_router().with_state(password_reset_state);

    let files_root =
        std::env::var("TACHYON_FILES_ROOT").unwrap_or_else(|_| "./content".to_string());
    let files_state = FilesState {
        root_path: std::path::PathBuf::from(&files_root),
        uploads_dir: std::path::PathBuf::from(&files_root).join("uploads"),
        audit_logger: audit_logger.clone(),
    };
    let files_router = create_files_router().with_state(files_state);
    let mfa_router = create_mfa_router().with_state(user_state.clone());

    let sms_provider = crate::sms::build_sms_provider(&config.sms_otp, http_client.clone());
    let sms_otp_state = crate::routes::sms_otp::SmsOtpRouteState {
        pool: pool.clone(),
        client: http_client.clone(),
        audit_logger: audit_logger.clone(),
        jwt_secrets: config.jwt.secrets.clone(),
        token_expiration_secs: config.jwt.expiration_secs,
        jwt_issuer: config.jwt.issuer.clone(),
        jwt_audience: config.jwt.audience.clone(),
        ttl_secs: config.sms_otp.ttl_secs as i64,
        sms_provider: sms_provider.map(std::sync::Arc::from),
    };
    let sms_otp_router = if config.sms_otp.enabled {
        Some(create_sms_otp_router().with_state(sms_otp_state))
    } else {
        None
    };

    let ai_router = crate::routes::ai_routes::create_ai_router().with_state(ai_manager);

    let comment_router =
        crate::routes::comments::create_comment_router().with_state(state.comment_state);

    let digest_router = create_digest_router().with_state(state.digest_state);

    let mut api_v1 = Router::new()
        .merge(document_router)
        .merge(user_router)
        .merge(session_router)
        .merge(repository_router)
        .merge(node_router)
        .merge(catalog_router)
        .merge(team_router)
        .merge(role_router)
        .merge(search_router)
        .merge(review_router)
        .merge(activity_router)
        .merge(notification_router)
        .merge(tags_router)
        .merge(webhook_router)
        .merge(conflict_router)
        .merge(plugin_router)
        .merge(space_router)
        .merge(onboarding_router)
        .merge(collaboration_router)
        .merge(ecosystem_router)
        .merge(billing_router)
        .merge(organization_router)
        .merge(password_reset_router)
        .merge(files_router)
        .merge(mfa_router)
        .merge(ai_router)
        .layer(RequestBodyLimitLayer::new(1024 * 1024))
        .merge(ssg_router.layer(RequestBodyLimitLayer::new(1024 * 1024)))
        .merge(oauth2_router.layer(RequestBodyLimitLayer::new(1024 * 1024)))
        .merge(comment_router)
        .merge(digest_router);

    if let Some(sms_otp_router) = sms_otp_router {
        api_v1 = api_v1.merge(sms_otp_router);
    }

    if let Some(oidc_state) = oidc_state {
        let router = crate::sso::create_oidc_router()
            .with_state(oidc_state)
            .layer(RequestBodyLimitLayer::new(1024 * 1024));
        api_v1 = api_v1.merge(router);
    }
    if let Some(saml_state) = saml_state {
        let router = crate::sso::create_saml_router()
            .with_state(saml_state)
            .layer(RequestBodyLimitLayer::new(1024 * 1024));
        api_v1 = api_v1.merge(router);
    }
    if let Some(ldap_state) = ldap_state {
        let router = crate::sso::create_ldap_router()
            .with_state(ldap_state)
            .layer(RequestBodyLimitLayer::new(1024 * 1024));
        api_v1 = api_v1.merge(router);
    }

    let v2_state = crate::routes::v2::V2State {
        document_state: document_state.clone(),
        user_state: user_state.clone(),
        search_state: search_state.clone(),
    };
    let api_v2 = crate::routes::v2::create_v2_router(v2_state);

    let crdt_ws_router = Router::new()
        .route("/ws/crdt/{room}", get(handle_crdt_websocket_upgrade))
        .with_state(crdt_connection_manager)
        .layer(RequestBodyLimitLayer::new(1024 * 1024 * 1024));

    let swagger_ui = crate::api_docs::create_swagger_ui();

    let rate_limit_state =
        crate::middleware::RateLimitState::new(crate::middleware::RateLimitConfig {
            enabled: config.rate_limit.enabled,
            redis_url: config.rate_limit.redis_url.clone(),
            default_requests_per_minute: config.rate_limit.default_requests_per_minute,
            cleanup_interval_secs: config.rate_limit.cleanup_interval_secs,
            endpoint_limits: config
                .rate_limit
                .endpoint_limits
                .iter()
                .map(|(k, v)| {
                    (
                        k.clone(),
                        crate::middleware::rate_limit::RateLimit::new(
                            v.max_requests,
                            v.window_secs,
                        ),
                    )
                })
                .collect(),
        });

    let health_router = Router::new()
        .route("/health", get(crate::routes::health::health_check))
        .route("/ready", get(crate::routes::health::readiness_check))
        .route("/metrics", get(metrics_handler))
        .route("/metrics/prometheus", get(prometheus_metrics_handler))
        .route("/", get(crate::routes::landing::landing_page))
        .with_state(HealthState {
            pool: pool.clone(),
            start_time,
            redis_enabled: config.rate_limit.enabled && config.rate_limit.redis_url.is_some(),
            redis_url: config.rate_limit.redis_url.clone(),
            smtp_configured: config.smtp_url.is_some(),
        });

    let metrics_router = Router::new()
        .route(
            "/metrics/app",
            get(crate::routes::metrics::prometheus_metrics),
        )
        .with_state(crate::routes::metrics::MetricsState {
            metrics: metrics.clone(),
            start_time,
        });

    let security_config = Arc::new(config.security.clone());

    let static_dir = crate::config::static_dir();
    let static_service = tower_http::services::ServeDir::new(&static_dir);

    let mut router = Router::new()
        .merge(health_router)
        .merge(metrics_router)
        .merge(seo_router)
        .merge(crdt_ws_router)
        .nest("/api/v1", api_v1)
        .nest("/api/v2", api_v2)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(axum::middleware::from_fn_with_state(
                    crate::middleware::request_tracing::RequestTracingState {
                        metrics: metrics.clone(),
                    },
                    crate::middleware::request_logging_with_metrics,
                ))
                .layer(axum::middleware::from_fn(audit_middleware))
                .layer(axum::middleware::from_fn(request_id_middleware))
                .layer(axum::middleware::from_fn(cache_control_middleware))
                .layer(axum::middleware::from_fn(
                    crate::middleware::security_headers::csp_nonce_middleware,
                ))
                .layer(CompressionLayer::new())
                .layer(axum::middleware::from_fn(request_size_limit))
                .layer(RequestBodyLimitLayer::new(10 * 1024 * 1024))
                .layer(build_cors_layer(config))
                .map_response(move |response| {
                    add_security_headers_from_config(response, &security_config)
                }),
        );

    // GraphQL and Swagger routes are merged after the main middleware layer,
    // so we wrap them with audit + request-id middleware explicitly to ensure
    // all requests are audit-logged.
    let graphql_router = axum::Router::new()
        .route("/graphql", axum::routing::post(graphql_handler))
        .route("/graphql/playground", get(graphql_playground))
        .with_state(pool.clone())
        .layer(axum::middleware::from_fn(audit_middleware))
        .layer(axum::middleware::from_fn(request_id_middleware));

    let swagger_routes = crate::routes::swagger::routes()
        .layer(axum::middleware::from_fn(audit_middleware))
        .layer(axum::middleware::from_fn(request_id_middleware));

    router = router.merge(graphql_router);
    router = router.merge(swagger_ui);
    router = router.merge(swagger_routes);

    let auth_state = crate::middleware::AuthState::new(config.clone(), pool.clone());
    let auth_layer =
        axum::middleware::from_fn_with_state(auth_state, crate::middleware::auth_middleware);
    router = router.layer(auth_layer);

    if config.rate_limit.enabled {
        router = router.layer(axum::middleware::from_fn_with_state(
            rate_limit_state,
            crate::middleware::rate_limit_middleware,
        ));
    }

    router.fallback_service(static_service)
}

/// Convenience function: initialize state and build the full router in one call.
///
/// This is the recommended entry point for embedders (CLI, tests, etc.)
/// that don't need to inject custom state (e.g., Tantivy search).
/// If you need to customize individual state types, use [`init_app_state`]
/// and [`build_app`] separately.
pub async fn build_server(config: &ServerConfig) -> anyhow::Result<axum::Router> {
    let state = init_app_state(config).await?;
    Ok(build_app(state, config))
}

/// Build CORS layer from server config.
pub fn build_cors_layer(config: &ServerConfig) -> CorsLayer {
    use axum::http::{HeaderValue, Method};
    use tower_http::cors::AllowOrigin;

    if !config.cors.enabled {
        return CorsLayer::new();
    }

    if config.cors.allowed_origins.contains(&"*".to_string()) {
        tracing::warn!(
            "CORS is configured with wildcard origin '*' — this allows any website to make cross-origin requests. \
             Set TACHYON_CORS_ALLOWED_ORIGINS to a comma-separated list of trusted origins in production."
        );
    }

    let allow_origin = if config.cors.allowed_origins.contains(&"*".to_string()) {
        AllowOrigin::any()
    } else {
        let origins: Vec<HeaderValue> = config
            .cors
            .allowed_origins
            .iter()
            .filter_map(|origin| origin.parse().ok())
            .collect();
        AllowOrigin::list(origins)
    };

    let allow_methods: Vec<Method> = config
        .cors
        .allowed_methods
        .iter()
        .filter_map(|m| m.parse().ok())
        .collect();

    let allow_headers: Vec<axum::http::HeaderName> = config
        .cors
        .allowed_headers
        .iter()
        .filter_map(|h| h.parse().ok())
        .collect();

    let expose_headers: Vec<axum::http::HeaderName> = config
        .cors
        .exposed_headers
        .iter()
        .filter_map(|h| h.parse().ok())
        .collect();

    let mut cors = CorsLayer::new()
        .allow_origin(allow_origin)
        .allow_methods(allow_methods)
        .allow_headers(allow_headers);

    if !expose_headers.is_empty() {
        cors = cors.expose_headers(expose_headers);
    }

    if config.cors.allow_credentials {
        cors = cors.allow_credentials(true);
    }

    if let Some(max_age) = config.cors.max_age_secs {
        cors = cors.max_age(std::time::Duration::from_secs(max_age));
    }

    cors
}

// --- Internal state types for health/metrics -- only used by the binary ---

/// Application state for health and metrics endpoints (internal).
#[derive(Clone)]
pub(crate) struct HealthState {
    pub(crate) pool: tachyon_database::DatabasePool,
    pub(crate) start_time: std::time::Instant,
    pub(crate) redis_enabled: bool,
    pub(crate) redis_url: Option<String>,
    pub(crate) smtp_configured: bool,
}

async fn metrics_handler(State(state): State<HealthState>) -> axum::Json<serde_json::Value> {
    let db_stats: serde_json::Value = state
        .pool
        .statistics()
        .await
        .unwrap_or_else(|_| serde_json::json!({}));

    axum::Json(serde_json::json!({
        "metrics": {
            "active_connections": db_stats.get("pool_size").and_then(|v: &serde_json::Value| v.as_u64()).unwrap_or(0),
            "idle_connections": db_stats.get("idle_connections").and_then(|v: &serde_json::Value| v.as_u64()).unwrap_or(0),
        },
    }))
}

/// Prometheus-format metrics endpoint.
///
/// Returns metrics in the Prometheus exposition format for scraping by
/// Prometheus, Grafana, or any compatible monitoring system.
///
/// To add custom metrics, use the `metrics` crate in your handlers:
/// ```rust,ignore
/// metrics::counter!("requests_total", "method" => "GET", "path" => "/api/v1/documents").increment(1);
/// metrics::histogram!("request_duration_seconds").record(duration.as_secs_f64());
/// ```
async fn prometheus_metrics_handler() -> String {
    match install_metrics() {
        Some(handle) => handle.render(),
        None => {
            tracing::warn!("Prometheus metrics handle unavailable, returning empty response");
            String::new()
        }
    }
}

/// Install the Prometheus metrics exporter.
///
/// Call this once at server startup. It installs a global metrics recorder
/// that can be queried via the `/metrics/prometheus` endpoint.
///
/// Returns `None` if a global metrics recorder was already installed by
/// another component.
pub fn install_metrics() -> Option<&'static metrics_exporter_prometheus::PrometheusHandle> {
    use std::sync::OnceLock;
    static HANDLE: OnceLock<metrics_exporter_prometheus::PrometheusHandle> = OnceLock::new();

    if HANDLE.get().is_some() {
        return HANDLE.get();
    }

    match metrics_exporter_prometheus::PrometheusBuilder::new().install_recorder() {
        Ok(handle) => {
            tracing::info!("Prometheus metrics exporter installed");
            Some(HANDLE.get_or_init(|| handle))
        }
        Err(e) => {
            tracing::warn!(
                error = %e,
                "Failed to install Prometheus metrics recorder (global recorder already installed?). \
                 Metrics endpoint will return empty results."
            );
            None
        }
    }
}
