// Tachyon Server Entry Point
// HTTP server with Axum framework

use anyhow::{Context, Result};
use axum::{
    Router,
    extract::State,
    routing::get,
    http::{HeaderValue, Method},
};
use std::backtrace::Backtrace;
use std::net::SocketAddr;
use std::sync::OnceLock;
use std::time::Instant;
use tachyon_database::init_with_migrations;
use tachyon_server::api_docs::create_swagger_ui;
use tachyon_server::config::ServerConfig;
use tachyon_server::middleware::{
    RateLimitConfig, RateLimitState, AuthState, cache_control_middleware,
};
use tachyon_server::routes::catalog::{CatalogState, create_catalog_router};
use tachyon_server::routes::document::{DocumentState, create_document_router};
use tachyon_server::routes::node::{NodeState, create_node_router};
use tachyon_server::routes::repository::{RepositoryState, create_repository_router};
use tachyon_server::routes::role::{RoleState, create_role_router};
use tachyon_server::routes::search::{SearchState, create_search_router};
use tachyon_server::routes::session::{SessionState, create_session_router};
use tachyon_server::routes::seo::{SeoState, create_seo_router};
use tachyon_server::routes::team::{TeamState, create_team_router};
use tachyon_server::routes::user::{UserState, create_user_router};
use tachyon_server::websocket::{ConnectionManager, handle_websocket_upgrade};
use tachyon_search::{IndexConfig, IndexManager};
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer, 
    cors::{CorsLayer, AllowOrigin},
    limit::RequestBodyLimitLayer, 
    trace::TraceLayer,
};
use tracing::{error, info, warn};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

/// Global panic information storage for debugging
static PANIC_INFO: OnceLock<String> = OnceLock::new();

/// Server start time for uptime tracking
static START_TIME: OnceLock<Instant> = OnceLock::new();

/// Application state for health and metrics endpoints
#[derive(Clone)]
struct AppState {
    pool: tachyon_database::DatabasePool,
}

/// Setup custom panic handler for better error reporting
fn setup_panic_handler() {
    std::panic::set_hook(Box::new(|panic_info| {
        let location = panic_info.location()
            .map(|loc| format!("{}:{}:{}", loc.file(), loc.line(), loc.column()))
            .unwrap_or_else(|| "unknown location".to_string());

        let message = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic message".to_string()
        };

        // Capture backtrace
        let backtrace = Backtrace::capture();
        let backtrace_str = match backtrace.status() {
            std::backtrace::BacktraceStatus::Captured => backtrace.to_string(),
            _ => "Backtrace not available (compile with RUST_BACKTRACE=1)".to_string(),
        };

        let full_info = format!(
            "PANIC: {}\nLocation: {}\nBacktrace:\n{}",
            message, location, backtrace_str
        );

        // Store for later retrieval
        let _ = PANIC_INFO.set(full_info.clone());

        // Log to tracing
        error!(
            target: "tachyon::panic",
            location = %location,
            message = %message,
            backtrace = %backtrace_str,
            "Server panic occurred"
        );

        // Also print to stderr for immediate visibility
        eprintln!("\n{}", "=".repeat(70));
        eprintln!("TACHYON SERVER PANIC");
        eprintln!("{}", "=".repeat(70));
        eprintln!("Message: {}", message);
        eprintln!("Location: {}", location);
        eprintln!("\nBacktrace:\n{}", backtrace_str);
        eprintln!("{}\n", "=".repeat(70));
    }));
}

/// Get last panic info (for health/metrics endpoints)
pub fn get_last_panic_info() -> Option<&'static str> {
    PANIC_INFO.get().map(|s| s.as_str())
}

/// Initialize application state
async fn init_state(config: &ServerConfig) -> Result<(DocumentState, UserState, SessionState, RepositoryState, NodeState, CatalogState, ConnectionManager, tachyon_database::DatabasePool)> {
    // Use database_url for PostgreSQL, fallback to database_path for backwards compatibility
    let database_url = if config.database_url.is_empty() {
        config.database_path.as_deref().unwrap_or("postgres://tachyon:tachyon@localhost:5432/tachyon")
    } else {
        &config.database_url
    };
    
    // Initialize database with migrations
    let pool = init_with_migrations(database_url)
        .await
        .context("Failed to initialize database")?;

    // Seed initial admin user if no users exist.
    // Configurable via TACHYON_ADMIN_USERNAME / TACHYON_ADMIN_PASSWORD / TACHYON_ADMIN_EMAIL.
    {
        let user_repo = tachyon_database::UserRepository::new(pool.clone());
        let admin_username = std::env::var("TACHYON_ADMIN_USERNAME").unwrap_or_else(|_| "admin".into());
        let admin_password = std::env::var("TACHYON_ADMIN_PASSWORD").unwrap_or_else(|_| {
            // Generate a random password if not set, so the default isn't a security hole
            uuid::Uuid::new_v4().to_string().replace('-', "")[..16].to_string()
        });
        let admin_email = std::env::var("TACHYON_ADMIN_EMAIL").unwrap_or_else(|_| "admin@tachyon.local".into());

        match user_repo.seed_admin(&admin_username, "Administrator", &admin_email, &admin_password).await {
            Ok(Some(user)) => {
                info!(
                    "Initial admin user seeded: {} ({}) — save these credentials!",
                    user.username, user.id
                );
                info!("  Username: {}", admin_username);
                info!("  Password: {}", admin_password);
                info!("  (Set TACHYON_ADMIN_PASSWORD env var to use a custom password)");
            }
            Ok(None) => {
                info!("Admin seed skipped: users already exist");
            }
            Err(e) => {
                // Log but don't fail startup — admin seed is best-effort
                warn!("Failed to seed admin user: {}. Users must be created manually.", e);
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
    let connection_manager = ConnectionManager::new();

    Ok((document_state, user_state, session_state, repository_state, node_state, catalog_state, connection_manager, pool.clone()))
}

/// Build Axum router with all routes and middleware
fn build_router(
    document_state: DocumentState, 
    user_state: UserState, 
    session_state: SessionState, 
    repository_state: RepositoryState, 
    node_state: NodeState, 
    catalog_state: CatalogState, 
    connection_manager: ConnectionManager,
    pool: tachyon_database::DatabasePool,
    config: &ServerConfig,
    tantivy_index: Option<std::sync::Arc<tokio::sync::Mutex<IndexManager>>>,
) -> Router {
    let cors = build_cors_layer(config);

    let document_router = create_document_router().with_state(document_state);

    let user_router = create_user_router().with_state(user_state);
    
    let session_router = create_session_router().with_state(session_state);
    
    let repository_router = create_repository_router().with_state(repository_state);
    
    let node_router = create_node_router().with_state(node_state);
    
    let catalog_router = create_catalog_router().with_state(catalog_state);

    let team_router = create_team_router().with_state(TeamState::new(pool.clone()));
    let role_router = create_role_router().with_state(RoleState::new(pool.clone()));
    let search_router = create_search_router().with_state(
        match &tantivy_index {
            Some(mgr) => SearchState::new(pool.clone()).with_index_manager(mgr.clone()),
            None => SearchState::new(pool.clone()),
        }
    );

    let seo_router = create_seo_router().with_state(SeoState {
        pool: pool.clone(),
        site_config: config.site.clone(),
    });

    let api_v1 = Router::new()
        .merge(document_router)
        .merge(user_router)
        .merge(session_router)
        .merge(repository_router)
        .merge(node_router)
        .merge(catalog_router)
        .merge(team_router)
        .merge(role_router)
        .merge(search_router);

    let ws_router = Router::new()
        .route("/ws", get(handle_websocket_upgrade))
        .with_state(connection_manager);

    let swagger_ui = create_swagger_ui();

    let rate_limit_state = RateLimitState::new(RateLimitConfig {
        enabled: config.rate_limit.enabled,
        redis_url: config.rate_limit.redis_url.clone(),
        default_requests_per_minute: config.rate_limit.default_requests_per_minute,
        cleanup_interval_secs: config.rate_limit.cleanup_interval_secs,
        endpoint_limits: config.rate_limit.endpoint_limits.iter()
            .map(|(k, v)| (k.clone(), tachyon_server::middleware::rate_limit::RateLimit::new(v.max_requests, v.window_secs)))
            .collect(),
    });

    let health_router = Router::new()
        .route("/health", get(health_handler))
        .route("/metrics", get(metrics_handler))
        .with_state(AppState { pool: pool.clone() });

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
                .layer(cors)
                .map_response(tachyon_server::middleware::add_security_headers),
        );

    // Merge Swagger UI (utoipa-swagger-ui 9.x supports axum 0.8)
    router = router.merge(swagger_ui);

    // Wire authentication middleware — protects all /api/v1 routes except auth endpoints.
    // Bypasses: /health, /metrics, /api/v1/auth/*, SEO routes, /ws, /api/docs.
    let auth_state = AuthState::new(config.clone(), pool.clone());
    let auth_layer = axum::middleware::from_fn_with_state(
        auth_state,
        tachyon_server::middleware::auth_middleware,
    );
    // Apply auth to the full router; auth_middleware internally skips its bypass paths.
    router = router.layer(auth_layer);
    
    if config.rate_limit.enabled {
        router = router.layer(axum::middleware::from_fn_with_state(
            rate_limit_state,
            tachyon_server::middleware::rate_limit_middleware,
        ));
    }
    
    router
}

fn build_cors_layer(config: &ServerConfig) -> CorsLayer {
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

/// Root handler
async fn root_handler() -> &'static str {
    "Tachyon Knowledge Management System - Server"
}

async fn check_db_health(pool: &tachyon_database::DatabasePool) -> &'static str {
    match pool.execute("SELECT 1").await {
        Ok(_) => "healthy",
        Err(_) => "unhealthy",
    }
}

fn format_uptime() -> String {
    let elapsed = START_TIME.get().map(|t| t.elapsed()).unwrap_or_default();
    let secs = elapsed.as_secs();
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;
    if days > 0 {
        format!("{}d {}h {}m {}s", days, hours, minutes, seconds)
    } else if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}

async fn health_handler(State(state): State<AppState>) -> axum::Json<serde_json::Value> {
    let has_panic = get_last_panic_info().is_some();
    let db_status = check_db_health(&state.pool).await;

    let overall_status = match (has_panic, db_status) {
        (false, "healthy") => "healthy",
        (false, "degraded") | (_, "unhealthy") => "unhealthy",
        _ => "degraded",
    };

    let mut components = serde_json::Map::new();
    components.insert("database".to_string(), serde_json::json!(db_status));
    components.insert("server".to_string(), serde_json::json!(if has_panic { "degraded" } else { "healthy" }));

    axum::Json(serde_json::json!({
        "status": overall_status,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": env!("CARGO_PKG_VERSION"),
        "uptime": format_uptime(),
        "components": components,
        "panic_detected": has_panic,
    }))
}

async fn metrics_handler(State(state): State<AppState>) -> axum::Json<serde_json::Value> {
    let panic_info = get_last_panic_info();
    let db_stats = state.pool.statistics().await.unwrap_or(serde_json::json!({}));

    axum::Json(serde_json::json!({
        "metrics": {
            "requests_total": 0,
            "active_connections": db_stats.get("pool_size").and_then(|v| v.as_u64()).unwrap_or(0),
            "idle_connections": db_stats.get("idle_connections").and_then(|v| v.as_u64()).unwrap_or(0),
        },
        "uptime": format_uptime(),
        "panic_info": panic_info,
    }))
}

/// Run the HTTP server
async fn init_tantivy_index() -> Option<std::sync::Arc<tokio::sync::Mutex<IndexManager>>> {
    let index_path = std::path::PathBuf::from(".tachyon/search_index");
    if let Err(e) = std::fs::create_dir_all(&index_path) {
        warn!("Failed to create search index directory: {}", e);
        return None;
    }

    let index_config = IndexConfig::new("tachyon")
        .with_index_path(".tachyon/search_index");

    match IndexManager::with_config(index_path, index_config).await {
        Ok(mgr) => {
            info!("Tantivy search index initialized at .tachyon/search_index");
            Some(std::sync::Arc::new(tokio::sync::Mutex::new(mgr)))
        }
        Err(e) => {
            warn!("Failed to initialize Tantivy search index: {}", e);
            None
        }
    }
}

async fn run_server(config: ServerConfig) -> Result<()> {
    let addr: SocketAddr = config
        .bind_address()
        .parse()
        .context("Invalid server address")?;

    let (document_state, user_state, session_state, repository_state, node_state, catalog_state, connection_manager, pool) = init_state(&config).await?;

    let tantivy_index = init_tantivy_index().await;

    let document_state = match &tantivy_index {
        Some(mgr) => document_state.with_index_manager(mgr.clone()),
        None => document_state,
    };

    let app = build_router(
        document_state, 
        user_state, 
        session_state, 
        repository_state, 
        node_state, 
        catalog_state, 
        connection_manager,
        pool,
        &config,
        tantivy_index,
    );

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .context("Failed to bind to address")?;

    info!("Tachyon server listening on {}", addr);
    info!("API endpoints available at http://{}/api/v1/", addr);
    info!("WebSocket endpoint available at ws://{}/ws", addr);
    info!(
        "Authentication endpoints available at http://{}/api/v1/auth/",
        addr
    );

    axum::serve(listener, app)
        .await
        .context("Failed to serve HTTP connections")?;

    Ok(())
}

/// Setup graceful shutdown
async fn run_with_graceful_shutdown(config: ServerConfig) -> Result<()> {
    let server_task = tokio::spawn(run_server(config.clone()));

    // Wait for Ctrl+C (SIGINT) or SIGTERM (Docker stop signal)
    let ctrl_c = tokio::signal::ctrl_c();
    
    #[cfg(unix)]
    let sigterm = async {
        let mut signal = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler");
        signal.recv().await;
    };
    #[cfg(not(unix))]
    let sigterm = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C signal, initiating graceful shutdown...");
        }
        _ = sigterm => {
            info!("Received SIGTERM signal, initiating graceful shutdown...");
        }
    }

    server_task.abort();

    let _ = tokio::time::timeout(std::time::Duration::from_secs(30), server_task)
        .await
        .context("Server shutdown timeout")??;

    info!("Server shutdown complete");

    Ok(())
}

/// Initialize tracing subscriber
fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("tachyon_server=info,tower_http=debug,axum=debug"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer())
        .init();
}

/// Main entry point
#[tokio::main]
async fn main() -> Result<()> {
    // Setup panic handler FIRST before any other initialization
    setup_panic_handler();
    START_TIME.get_or_init(Instant::now);
    
    init_tracing();

    // Load configuration from environment variables
    let config = ServerConfig::from_env();

    info!("Starting Tachyon server");
    info!("Database URL: {}", if config.database_url.is_empty() { 
        config.database_path.as_deref().unwrap_or("not configured") 
    } else { 
        // Don't log the full URL with password
        config.database_url.split('@').last().unwrap_or("configured")
    });

    run_with_graceful_shutdown(config).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = ServerConfig::default();
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 8080);
    }
}
