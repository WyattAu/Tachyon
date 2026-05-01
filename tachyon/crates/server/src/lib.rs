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
//! - [User Guide](../../docs/user-guide/README.md)
//! - [API Documentation](../../docs/api/authentication.md)
//! - [Architecture](../../docs/developer/architecture.md)

pub mod api_docs;
pub mod audit;
pub mod config;
pub mod conflict;
pub mod crdt;
pub mod email;
pub mod error;
pub mod graph_extractor;
pub mod middleware;
pub mod routes;
pub mod storage;
pub mod sync;
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

use axum::extract::State;
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
    pub tags_state: crate::routes::tags::TagsState,
    pub webhook_state: crate::routes::webhook::WebhookState,
    pub plugin_state: crate::routes::plugin::PluginState,
    pub space_state: crate::routes::space::SpaceState,
    pub conflict_state: crate::routes::conflict::ConflictState,
    pub onboarding_state: crate::routes::onboarding::OnboardingState,
    pub connection_manager: crate::websocket::ConnectionManager,
    pub crdt_connection_manager: crate::websocket::CrdtConnectionManager,
    pub pool: tachyon_database::DatabasePool,
    pub http_client: reqwest::Client,
    pub email: crate::email::EmailService,
    pub metrics: Arc<crate::middleware::metrics::RequestMetrics>,
    pub api_cache: crate::middleware::api_cache::ApiCache,
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
    use crate::websocket::ConnectionManager;
    use crate::websocket::CrdtConnectionManager;
    use tachyon_database::init_with_migrations;

    let database_url = if config.database_url.is_empty() {
        config
            .database_path
            .as_deref()
            .unwrap_or("postgres://tachyon:tachyon@localhost:5432/tachyon")
    } else {
        &config.database_url
    };

    let pool = init_with_migrations(database_url)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to initialize database: {}", e))?;

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

    let document_state =
        DocumentState::with_guest_config(pool.clone(), config.guest.clone(), http_client.clone());
    let user_state = UserState::with_guest_config(
        pool.clone(),
        config.jwt.secret.clone(),
        config.jwt.expiration_secs,
        config.jwt.issuer.clone(),
        config.jwt.audience.clone(),
        config.guest.clone(),
    );
    let session_state = SessionState::new(pool.clone(), config.jwt.expiration_secs);
    let repository_state = RepositoryState::new();
    let node_state = NodeState::new(pool.clone());
    let catalog_state = CatalogState::new(pool.clone());
    let team_state = TeamState::new(pool.clone());
    let role_state = RoleState::new(pool.clone());
    let search_state = SearchState::new(pool.clone());
    let seo_state = crate::routes::seo::SeoState {
        pool: pool.clone(),
        site_config: config.site.clone(),
    };
    let review_state = ReviewState::new(pool.clone(), http_client.clone());
    let activity_state = ActivityState::new(pool.clone());
    let notification_state = NotificationState::new(pool.clone());
    let tags_state = TagsState { pool: pool.clone() };
    let webhook_state = WebhookState { pool: pool.clone() };
    let plugins_dir = std::env::current_dir()
        .unwrap_or_else(|_| std::env::temp_dir())
        .join("plugins");
    let plugin_runtime = tachyon_plugin_runtime::PluginRuntime::new(plugins_dir);
    let plugin_state = PluginState {
        pool: pool.clone(),
        runtime: plugin_runtime,
    };
    let space_state = SpaceState { pool: pool.clone() };
    let conflict_state = ConflictState { pool: pool.clone() };
    let onboarding_state = OnboardingState { pool: pool.clone() };
    let connection_manager = ConnectionManager::new();
    let crdt_connection_manager = CrdtConnectionManager::new();

    {
        let cleanup_cm = connection_manager.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
            loop {
                interval.tick().await;
                cleanup_cm.cleanup_stale_clients(300).await;
            }
        });
    }
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
        tags_state,
        webhook_state,
        plugin_state,
        space_state,
        conflict_state,
        onboarding_state,
        connection_manager,
        crdt_connection_manager,
        pool,
        http_client,
        email,
        metrics: Arc::new(crate::middleware::metrics::RequestMetrics::new()),
        api_cache: crate::middleware::api_cache::ApiCache::new(std::time::Duration::from_secs(60)),
    })
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
    let tags_state = state.tags_state;
    let webhook_state = state.webhook_state;
    let plugin_state = state.plugin_state;
    let space_state = state.space_state;
    let conflict_state = state.conflict_state;
    let onboarding_state = state.onboarding_state;
    let connection_manager = state.connection_manager;
    let crdt_connection_manager = state.crdt_connection_manager;
    let pool = state.pool;
    let http_client = state.http_client;
    let metrics = state.metrics;
    use crate::routes::activity::create_activity_router;
    use crate::routes::billing::{create_billing_router, BillingState};
    use crate::routes::catalog::create_catalog_router;
    use crate::routes::collaboration::{create_collaboration_router, CollaborationState};
    use crate::routes::conflict::create_conflict_router;
    use crate::routes::document::create_document_router;
    use crate::routes::ecosystem::{create_ecosystem_router, EcosystemState};
    use crate::routes::files::{create_files_router, FilesState};
    use crate::routes::mfa::create_mfa_router;
    use crate::routes::node::create_node_router;
    use crate::routes::notification::create_notification_router;
    use crate::routes::oauth2::{create_oauth2_router, OAuth2State};
    use crate::routes::onboarding::create_onboarding_router;
    use crate::routes::organization::{create_organization_router, OrganizationState};
    use crate::routes::password_reset::{create_password_reset_router, PasswordResetState};
    use crate::routes::plugin::create_plugin_router_with_state;
    use crate::routes::repository::create_repository_router;
    use crate::routes::review::create_review_router;
    use crate::routes::role::create_role_router;
    use crate::routes::search::create_search_router;
    use crate::routes::seo::create_seo_router;
    use crate::routes::session::create_session_router;
    use crate::routes::space::create_space_router;
    use crate::routes::ssg::{create_ssg_router, SsgState};
    use crate::routes::tags::create_tags_router;
    use crate::routes::team::create_team_router;
    use crate::routes::user::create_user_router;
    use crate::routes::webhook::create_webhook_router;
    use crate::websocket::handle_crdt_websocket_upgrade;
    use crate::websocket::handle_websocket_upgrade;
    use axum::{routing::get, Router};
    use tower::ServiceBuilder;
    use tower_http::{
        compression::CompressionLayer, limit::RequestBodyLimitLayer, trace::TraceLayer,
    };

    let document_router = create_document_router().with_state(document_state);
    let user_router = create_user_router().with_state(user_state.clone());
    let session_router = create_session_router().with_state(session_state);
    let repository_router = create_repository_router().with_state(repository_state);
    let node_router = create_node_router().with_state(node_state);
    let catalog_router = create_catalog_router().with_state(catalog_state);
    let team_router = create_team_router().with_state(team_state);
    let role_router = create_role_router().with_state(role_state);
    let search_router = create_search_router().with_state(search_state);
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
    let collaboration_state = CollaborationState::new(pool.clone(), connection_manager.clone());
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
    let billing_state = BillingState::new(pool.clone(), truelayer_client);
    let billing_router = create_billing_router().with_state(billing_state);
    let organization_state = OrganizationState { pool: pool.clone() };
    let organization_router = create_organization_router().with_state(organization_state);
    let ssg_state = SsgState::new(pool.clone());
    let ssg_router = create_ssg_router().with_state(ssg_state);

    // OAuth2 router (only enabled when providers are configured)
    let oauth2_state = OAuth2State {
        jwt_secret: config.jwt.secret.clone(),
        jwt_expiration_secs: config.jwt.expiration_secs,
        jwt_issuer: config.jwt.issuer.clone(),
        jwt_audience: config.jwt.audience.clone(),
        config: config.oauth2.clone(),
        pool: pool.clone(),
        client: http_client.clone(),
        csrf_states: std::sync::Arc::new(dashmap::DashMap::new()),
    };
    let oauth2_router = create_oauth2_router().with_state(oauth2_state);
    let password_reset_state = PasswordResetState {
        pool: pool.clone(),
        client: http_client.clone(),
    };
    let password_reset_router = create_password_reset_router().with_state(password_reset_state);

    let files_root =
        std::env::var("TACHYON_FILES_ROOT").unwrap_or_else(|_| "./content".to_string());
    let files_state = FilesState {
        root_path: std::path::PathBuf::from(&files_root),
        uploads_dir: std::path::PathBuf::from(&files_root).join("uploads"),
    };
    let files_router = create_files_router().with_state(files_state);
    let mfa_router = create_mfa_router().with_state(user_state.clone());

    let api_v1 = Router::new()
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
        .layer(RequestBodyLimitLayer::new(1024 * 1024))
        .merge(ssg_router.layer(RequestBodyLimitLayer::new(1024 * 1024)))
        .merge(oauth2_router.layer(RequestBodyLimitLayer::new(1024 * 1024)));

    let ws_router = Router::new()
        .route("/ws", get(handle_websocket_upgrade))
        .with_state(connection_manager)
        .layer(RequestBodyLimitLayer::new(1024 * 1024 * 1024));

    // CRDT WebSocket needs its own router since it uses different state
    // y-websocket appends room name to URL: ws://host/ws/crdt/{documentId}
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
        .with_state(HealthState {
            pool: pool.clone(),
            start_time,
            redis_enabled: config.rate_limit.enabled && config.rate_limit.redis_url.is_some(),
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
    let static_service = tower_http::services::ServeDir::new(&static_dir)
        .fallback(tower_http::services::ServeFile::new(format!(
            "{}/index.html",
            static_dir
        )));

    let mut router = Router::new()
        .merge(health_router)
        .merge(metrics_router)
        .merge(seo_router)
        .merge(ws_router)
        .merge(crdt_ws_router)
        .nest("/api/v1", api_v1)
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
                .layer(CompressionLayer::new())
                .layer(axum::middleware::from_fn(request_size_limit))
                .layer(RequestBodyLimitLayer::new(10 * 1024 * 1024))
                .layer(build_cors_layer(config))
                .map_response(move |response| {
                    add_security_headers_from_config(response, &security_config)
                }),
        );

    router = router.merge(swagger_ui);

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
    // Use the singleton installed by install_metrics() — calling
    // install_recorder() here would panic on the 2nd request.
    install_metrics().render()
}

/// Install the Prometheus metrics exporter.
///
/// Call this once at server startup. It installs a global metrics recorder
/// that can be queried via the `/metrics/prometheus` endpoint.
pub fn install_metrics() -> &'static metrics_exporter_prometheus::PrometheusHandle {
    // Use a lazy static to ensure we only install once
    use std::sync::OnceLock;
    static HANDLE: OnceLock<metrics_exporter_prometheus::PrometheusHandle> = OnceLock::new();

    HANDLE.get_or_init(|| {
        let handle = metrics_exporter_prometheus::PrometheusBuilder::new()
            .install_recorder()
            .expect("failed to install Prometheus metrics recorder");
        tracing::info!("Prometheus metrics exporter installed");
        handle
    })
}

