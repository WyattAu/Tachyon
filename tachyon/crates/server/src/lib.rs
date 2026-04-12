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
//!     // Initialize tracing
//!     tracing_subscriber::fmt::init();
//!
//!     // Load configuration
//!     let config = ServerConfig::from_env();
//!
//!     // Run server
//!     run_server(config).await
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
pub mod graph_extractor;
pub mod middleware;
pub mod routes;
pub mod sync;
pub mod validation;
pub mod webhook_delivery;
pub mod websocket;

pub use api_docs::*;
pub use audit::*;
#[allow(ambiguous_glob_reexports)]
pub use config::*;
pub use middleware::*;

use axum::extract::State;
use tower_http::cors::CorsLayer;

/// Initialize application state from a [`ServerConfig`].
///
/// Creates the database pool, seeds the admin user (if needed),
/// and returns all state types needed by the router.
///
/// This is the single entry point for building a Tachyon server,
/// used by both `tachyon-server` (binary) and `tachyon serve` (CLI).
pub async fn init_app_state(
    config: &ServerConfig,
) -> anyhow::Result<(
    crate::routes::document::DocumentState,
    crate::routes::user::UserState,
    crate::routes::session::SessionState,
    crate::routes::repository::RepositoryState,
    crate::routes::node::NodeState,
    crate::routes::catalog::CatalogState,
    crate::routes::team::TeamState,
    crate::routes::role::RoleState,
    crate::routes::search::SearchState,
    crate::routes::seo::SeoState,
    crate::routes::review::ReviewState,
    crate::routes::activity::ActivityState,
    crate::routes::notification::NotificationState,
    crate::routes::tags::TagsState,
    crate::routes::webhook::WebhookState,
    crate::routes::conflict::ConflictState,
    crate::websocket::ConnectionManager,
    tachyon_database::DatabasePool,
)> {
    use crate::routes::review::ReviewState;
    use crate::routes::activity::ActivityState;
    use crate::routes::tags::TagsState;
    use crate::routes::webhook::WebhookState;
    use crate::routes::conflict::ConflictState;
    use crate::routes::user::UserState;
    use crate::routes::session::SessionState;
    use crate::routes::repository::RepositoryState;
    use crate::routes::node::NodeState;
    use crate::routes::catalog::CatalogState;
    use crate::routes::team::TeamState;
    use crate::routes::role::RoleState;
    use crate::routes::search::SearchState;
    use crate::routes::document::DocumentState;
    use crate::routes::notification::NotificationState;
    use crate::websocket::ConnectionManager;
    use tachyon_database::init_with_migrations;

    let database_url = if config.database_url.is_empty() {
        config.database_path.as_deref()
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
        let admin_username = std::env::var("TACHYON_ADMIN_USERNAME").unwrap_or_else(|_| "admin".into());
        let admin_password = std::env::var("TACHYON_ADMIN_PASSWORD").unwrap_or_else(|_| {
            uuid::Uuid::new_v4().to_string().replace('-', "")[..16].to_string()
        });
        let admin_email = std::env::var("TACHYON_ADMIN_EMAIL")
            .unwrap_or_else(|_| "admin@tachyon.local".into());

        match user_repo.seed_admin(&admin_username, "Administrator", &admin_email, &admin_password).await {
            Ok(Some(user)) => {
                tracing::info!(
                    "Initial admin user seeded: {} ({})",
                    user.username, user.id
                );
            }
            Ok(None) => {
                tracing::info!("Admin seed skipped: users already exist");
            }
            Err(e) => {
                tracing::warn!("Failed to seed admin user: {}. Users must be created manually.", e);
            }
        }
    }

    let document_state = DocumentState::with_guest_config(pool.clone(), config.guest.clone());
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
    let review_state = ReviewState::new(pool.clone());
    let activity_state = ActivityState::new(pool.clone());
    let notification_state = NotificationState::new(pool.clone());
    let tags_state = TagsState { pool: pool.clone() };
    let webhook_state = WebhookState { pool: pool.clone() };
    let conflict_state = ConflictState { pool: pool.clone() };
    let connection_manager = ConnectionManager::new();

    Ok((
        document_state, user_state, session_state, repository_state, node_state,
        catalog_state, team_state, role_state, search_state, seo_state,
        review_state, activity_state, notification_state, tags_state,
        webhook_state, conflict_state, connection_manager, pool,
    ))
}

/// Build the full Axum router with all routes, middleware, and state.
///
/// This is the single entry point for creating the Tachyon HTTP/WebSocket
/// application, used by both `tachyon-server` (binary) and `tachyon serve` (CLI).
pub fn build_app(
    document_state: crate::routes::document::DocumentState,
    user_state: crate::routes::user::UserState,
    session_state: crate::routes::session::SessionState,
    repository_state: crate::routes::repository::RepositoryState,
    node_state: crate::routes::node::NodeState,
    catalog_state: crate::routes::catalog::CatalogState,
    team_state: crate::routes::team::TeamState,
    role_state: crate::routes::role::RoleState,
    search_state: crate::routes::search::SearchState,
    seo_state: crate::routes::seo::SeoState,
    review_state: crate::routes::review::ReviewState,
    activity_state: crate::routes::activity::ActivityState,
    notification_state: crate::routes::notification::NotificationState,
    tags_state: crate::routes::tags::TagsState,
    webhook_state: crate::routes::webhook::WebhookState,
    conflict_state: crate::routes::conflict::ConflictState,
    connection_manager: crate::websocket::ConnectionManager,
    pool: tachyon_database::DatabasePool,
    config: &ServerConfig,
) -> axum::Router {
    use crate::routes::document::create_document_router;
    use crate::routes::user::create_user_router;
    use crate::routes::session::create_session_router;
    use crate::routes::repository::create_repository_router;
    use crate::routes::node::create_node_router;
    use crate::routes::catalog::create_catalog_router;
    use crate::routes::team::create_team_router;
    use crate::routes::role::create_role_router;
    use crate::routes::search::create_search_router;
    use crate::routes::seo::create_seo_router;
    use crate::routes::review::create_review_router;
    use crate::routes::activity::create_activity_router;
    use crate::routes::notification::create_notification_router;
    use crate::routes::tags::create_tags_router;
    use crate::routes::webhook::create_webhook_router;
    use crate::routes::conflict::create_conflict_router;
    use crate::websocket::handle_websocket_upgrade;
    use axum::{Router, routing::get};
    use tower::ServiceBuilder;
    use tower_http::{
        compression::CompressionLayer,
        limit::RequestBodyLimitLayer,
        trace::TraceLayer,
    };

    let document_router = create_document_router().with_state(document_state);
    let user_router = create_user_router().with_state(user_state);
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
        .merge(conflict_router);

    let ws_router = Router::new()
        .route("/ws", get(handle_websocket_upgrade))
        .with_state(connection_manager);

    let swagger_ui = crate::api_docs::create_swagger_ui();

    let rate_limit_state = crate::middleware::RateLimitState::new(crate::middleware::RateLimitConfig {
        enabled: config.rate_limit.enabled,
        redis_url: config.rate_limit.redis_url.clone(),
        default_requests_per_minute: config.rate_limit.default_requests_per_minute,
        cleanup_interval_secs: config.rate_limit.cleanup_interval_secs,
        endpoint_limits: config.rate_limit.endpoint_limits.iter()
            .map(|(k, v)| (k.clone(), crate::middleware::rate_limit::RateLimit::new(v.max_requests, v.window_secs)))
            .collect(),
    });

    let health_router = Router::new()
        .route("/health", get(health_handler))
        .route("/metrics", get(metrics_handler))
        .with_state(HealthState { pool: pool.clone() });

    let mut router = Router::new()
        .route("/", get(root_handler))
        .merge(health_router)
        .merge(seo_router)
        .merge(ws_router)
        .nest("/api/v1", api_v1)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(axum::middleware::from_fn(cache_control_middleware))
                .layer(CompressionLayer::new())
                .layer(RequestBodyLimitLayer::new(10 * 1024 * 1024))
                .layer(build_cors_layer(config))
                .map_response(add_security_headers),
        );

    router = router.merge(swagger_ui);

    let auth_state = crate::middleware::AuthState::new(config.clone(), pool.clone());
    let auth_layer = axum::middleware::from_fn_with_state(
        auth_state,
        crate::middleware::auth_middleware,
    );
    router = router.layer(auth_layer);

    if config.rate_limit.enabled {
        router = router.layer(axum::middleware::from_fn_with_state(
            rate_limit_state,
            crate::middleware::rate_limit_middleware,
        ));
    }

    router
}

/// Convenience function: initialize state and build the full router in one call.
///
/// This is the recommended entry point for embedders (CLI, tests, etc.)
/// that don't need to inject custom state (e.g., Tantivy search).
/// If you need to customize individual state types, use [`init_app_state`]
/// and [`build_app`] separately.
pub async fn build_server(config: &ServerConfig) -> anyhow::Result<axum::Router> {
    let state = init_app_state(config).await?;
    Ok(build_app(
        state.0, state.1, state.2, state.3, state.4, state.5,
        state.6, state.7, state.8, state.9, state.10, state.11,
        state.12, state.13, state.14, state.15, state.16, state.17,
        config,
    ))
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
        let origins: Vec<HeaderValue> = config.cors.allowed_origins
            .iter()
            .filter_map(|origin| origin.parse().ok())
            .collect();
        AllowOrigin::list(origins)
    };

    let allow_methods: Vec<Method> = config.cors.allowed_methods
        .iter()
        .filter_map(|m| m.parse().ok())
        .collect();

    let allow_headers: Vec<axum::http::HeaderName> = config.cors.allowed_headers
        .iter()
        .filter_map(|h| h.parse().ok())
        .collect();

    let expose_headers: Vec<axum::http::HeaderName> = config.cors.exposed_headers
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
}

async fn health_handler(
    State(state): State<HealthState>,
) -> axum::Json<serde_json::Value> {
    let db_status = match state.pool.execute("SELECT 1").await {
        Ok(_) => "healthy",
        Err(_) => "unhealthy",
    };

    axum::Json(serde_json::json!({
        "status": if db_status == "healthy" { "healthy" } else { "unhealthy" },
        "version": env!("CARGO_PKG_VERSION"),
        "components": { "database": db_status },
    }))
}

async fn metrics_handler(
    State(state): State<HealthState>,
) -> axum::Json<serde_json::Value> {
    let db_stats: serde_json::Value = state.pool.statistics().await
        .unwrap_or_else(|_| serde_json::json!({}));

    axum::Json(serde_json::json!({
        "metrics": {
            "active_connections": db_stats.get("pool_size").and_then(|v: &serde_json::Value| v.as_u64()).unwrap_or(0),
            "idle_connections": db_stats.get("idle_connections").and_then(|v: &serde_json::Value| v.as_u64()).unwrap_or(0),
        },
    }))
}

async fn root_handler() -> &'static str {
    "Tachyon Knowledge Management System"
}
