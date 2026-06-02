// Tachyon Server Entry Point
// HTTP server with Axum framework

use anyhow::{Context, Result};
use std::backtrace::Backtrace;
use std::net::SocketAddr;
use std::sync::OnceLock;
use tachyon_search::{IndexConfig, IndexManager};
use tachyon_server::config::ServerConfig;
use tracing::{error, info, warn};
use tracing_subscriber::{EnvFilter, Layer, fmt, layer::SubscriberExt, util::SubscriberInitExt};

/// Global panic information storage for debugging
static PANIC_INFO: OnceLock<String> = OnceLock::new();

/// Setup custom panic handler for better error reporting
fn setup_panic_handler() {
    std::panic::set_hook(Box::new(|panic_info| {
        let location = panic_info
            .location()
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

/// Initialize tracing subscriber with configurable format.
///
/// Supports two formats:
/// - "text" (default): Human-readable plain text output
/// - "json": Structured JSON for production log aggregation (ELK, Datadog, etc.)
fn init_tracing(config: &ServerConfig) {
    let env_filter = if let Some(ref level) = config.log.level {
        let filter_str = format!("tachyon_server={},tower_http=debug,axum=debug", level);
        EnvFilter::new(filter_str)
    } else if let Ok(filter) = std::env::var("TACHYON_LOG_FILTER") {
        match EnvFilter::try_new(&filter) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Invalid TACHYON_LOG_FILTER '{}': {}", filter, e);
                EnvFilter::new("tachyon_server=info,tower_http=debug,axum=debug")
            }
        }
    } else {
        EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("tachyon_server=info,tower_http=debug,axum=debug"))
    };

    tracing_subscriber::registry()
        .with(env_filter)
        .with(match config.log.format.as_str() {
            "json" => fmt::layer().json().with_target(false).boxed(),
            _ => fmt::layer().boxed(),
        })
        .init();
}

/// Initialize Tantivy search index (best-effort).
async fn init_tantivy_index() -> Option<std::sync::Arc<tokio::sync::Mutex<IndexManager>>> {
    let index_path = std::path::PathBuf::from(".tachyon/search_index");
    if let Err(e) = tokio::fs::create_dir_all(&index_path).await {
        warn!("Failed to create search index directory: {}", e);
        return None;
    }

    let index_config = IndexConfig::new("tachyon").with_index_path(".tachyon/search_index");

    match IndexManager::with_config(index_path.clone(), index_config).await {
        Ok(mgr) => {
            info!("Tantivy search index initialized at .tachyon/search_index");
            Some(std::sync::Arc::new(tokio::sync::Mutex::new(mgr)))
        }
        Err(e) if e.to_string().contains("Index already exists") => {
            info!("Tantivy search index already exists, opening: .tachyon/search_index");
            match IndexManager::open(index_path).await {
                Ok(mgr) => Some(std::sync::Arc::new(tokio::sync::Mutex::new(mgr))),
                Err(e2) => {
                    warn!("Failed to open existing Tantivy search index: {}", e2);
                    None
                }
            }
        }
        Err(e) => {
            warn!("Failed to initialize Tantivy search index: {}", e);
            None
        }
    }
}

/// Run the HTTP server using the shared library.
async fn run_server(config: ServerConfig) -> Result<()> {
    let addr: SocketAddr = config
        .bind_address()
        .parse()
        .context("Invalid server address")?;

    // Initialize all application state via the shared library.
    let mut state = tachyon_server::init_app_state(&config).await?;

    // Initialize optional Tantivy search index and inject into relevant states.
    let tantivy_index = init_tantivy_index().await;

    state.document_state = match &tantivy_index {
        Some(mgr) => state.document_state.with_index_manager(mgr.clone()),
        None => state.document_state,
    };
    state.search_state = match &tantivy_index {
        Some(mgr) => state.search_state.with_index_manager(mgr.clone()),
        None => state.search_state,
    };

    // Build the full Axum router with all routes and middleware.
    let app = tachyon_server::build_app(state, &config);

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
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("Failed to serve HTTP connections")?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        if let Err(e) = tokio::signal::ctrl_c().await {
            tracing::warn!("Failed to install Ctrl+C handler: {}", e);
            std::future::pending::<()>().await;
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut sig) => {
                sig.recv().await;
            }
            Err(e) => {
                tracing::warn!("Failed to install SIGTERM handler: {}", e);
                std::future::pending::<()>().await;
            }
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => info!("Received Ctrl+C, shutting down gracefully..."),
        _ = terminate => info!("Received SIGTERM, shutting down gracefully..."),
    }

    info!("Waiting 30 seconds for connections to drain...");
    tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
    info!("Shutdown complete");
}

/// Main entry point
#[tokio::main]
async fn main() -> Result<()> {
    // Setup panic handler FIRST before any other initialization
    setup_panic_handler();

    // Load configuration from environment variables
    let config = ServerConfig::from_env();

    // Initialize tracing with config-driven format
    init_tracing(&config);

    // Install Prometheus metrics exporter (must be before server starts)
    tachyon_server::install_metrics();

    info!("Starting Tachyon server");
    info!(
        log_format = %config.log.format,
        "Logging initialized"
    );
    info!(
        "Database URL: {}",
        if config.database_url.is_empty() {
            config.database_path.as_deref().unwrap_or("not configured")
        } else {
            // Don't log the full URL with password
            config
                .database_url
                .split('@')
                .next_back()
                .unwrap_or("configured")
        }
    );

    run_server(config).await
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
