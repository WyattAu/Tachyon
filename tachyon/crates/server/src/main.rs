// Tachyon Server Entry Point
// HTTP server with Axum framework

use anyhow::{Context, Result};
use axum::{
    Router,
    routing::{delete, get, post, put},
    http::{HeaderValue, Method},
};
use std::backtrace::Backtrace;
use std::net::SocketAddr;
use std::sync::OnceLock;
use tachyon_database::init_with_migrations;
use tachyon_server::api_docs::create_swagger_router;
use tachyon_server::config::ServerConfig;
use tachyon_server::middleware::{
    RateLimitConfig, RateLimitState, security_headers_middleware,
};
use tachyon_server::routes::catalog::{CatalogState, create_catalog_router};
use tachyon_server::routes::document::{DocumentState, create_document_router};
use tachyon_server::routes::node::{NodeState, create_node_router};
use tachyon_server::routes::repository::{RepositoryState, create_repository_router};
use tachyon_server::routes::role::{RoleState, create_role_router};
use tachyon_server::routes::search::{SearchState, create_search_router};
use tachyon_server::routes::session::{SessionState, create_session_router};
use tachyon_server::routes::team::{TeamState, create_team_router};
use tachyon_server::routes::user::{UserState, create_user_router};
use tachyon_server::websocket::{ConnectionManager, handle_websocket_upgrade};
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer, 
    cors::{CorsLayer, Any, AllowOrigin},
    limit::RequestBodyLimitLayer, 
    trace::TraceLayer,
};
use tracing::{error, info};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

/// Global panic information storage for debugging
static PANIC_INFO: OnceLock<String> = OnceLock::new();

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

    let document_state = DocumentState::with_guest_config(pool.clone(), config.guest.clone());
    let user_state = UserState::with_guest_config(
        pool.clone(),
        config.jwt.secret.clone(),
        config.jwt.expiration_secs,
        config.jwt.issuer.clone(),
        config.jwt.audience.clone(),
        config.guest.clone(),
    );
    let session_state = SessionState::new(config.jwt.expiration_secs);
    let repository_state = RepositoryState::new();
    let node_state = NodeState::new();
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
    let search_router = create_search_router().with_state(SearchState::new(pool.clone()));

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

    let swagger_router = create_swagger_router();

    let rate_limit_state = RateLimitState::new(RateLimitConfig {
        enabled: config.rate_limit.enabled,
        redis_url: config.rate_limit.redis_url.clone(),
        default_requests_per_minute: config.rate_limit.default_requests_per_minute,
        cleanup_interval_secs: config.rate_limit.cleanup_interval_secs,
        endpoint_limits: config.rate_limit.endpoint_limits.iter()
            .map(|(k, v)| (k.clone(), tachyon_server::middleware::rate_limit::RateLimit::new(v.max_requests, v.window_secs)))
            .collect(),
    });

    let mut router = Router::new()
        .route("/", get(root_handler))
        .route("/health", get(health_handler))
        .route("/metrics", get(metrics_handler))
        .merge(ws_router)
        .nest("/api/v1", api_v1)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CompressionLayer::new())
                .layer(RequestBodyLimitLayer::new(10 * 1024 * 1024))
                .layer(cors)
                .map_response(tachyon_server::middleware::add_security_headers),
        );
    
    // Note: Swagger UI temporarily disabled due to axum 0.8 compatibility
    // router = router.merge(swagger_router.into());
    
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

/// Health check handler - returns detailed health status
async fn health_handler() -> axum::Json<serde_json::Value> {
    let has_panic = get_last_panic_info().is_some();
    
    axum::Json(serde_json::json!({
        "status": if has_panic { "degraded" } else { "healthy" },
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": env!("CARGO_PKG_VERSION"),
        "panic_detected": has_panic,
    }))
}

/// Metrics handler - returns basic metrics and panic info if available
async fn metrics_handler() -> axum::Json<serde_json::Value> {
    let panic_info = get_last_panic_info();
    
    axum::Json(serde_json::json!({
        "metrics": {
            "requests_total": 0,  // TODO: Implement request counting
            "active_connections": 0,  // TODO: Implement connection tracking
        },
        "panic_info": panic_info,
    }))
}

/// Run the HTTP server
async fn run_server(config: ServerConfig) -> Result<()> {
    let addr: SocketAddr = config
        .bind_address()
        .parse()
        .context("Invalid server address")?;

    let (document_state, user_state, session_state, repository_state, node_state, catalog_state, connection_manager, pool) = init_state(&config).await?;
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

    tokio::time::timeout(std::time::Duration::from_secs(30), server_task)
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
