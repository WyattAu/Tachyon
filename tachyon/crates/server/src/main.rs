// Tachyon Server Entry Point
// HTTP server with Axum framework

use anyhow::{Context, Result};
use std::backtrace::Backtrace;
use std::net::SocketAddr;
use std::sync::OnceLock;
use std::time::Instant;
use tachyon_server::config::ServerConfig;
use tachyon_search::{IndexConfig, IndexManager};
use tracing::{error, info, warn};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt, Layer};

/// Global panic information storage for debugging
static PANIC_INFO: OnceLock<String> = OnceLock::new();

/// Server start time for uptime tracking
static START_TIME: OnceLock<Instant> = OnceLock::new();

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

/// Initialize tracing subscriber with configurable format.
///
/// Supports two formats:
/// - "text" (default): Human-readable plain text output
/// - "json": Structured JSON for production log aggregation (ELK, Datadog, etc.)
fn init_tracing(config: &ServerConfig) {
    let env_filter = if let Some(ref level) = config.log.level {
        // Use the explicit log level from config
        let filter_str = format!("tachyon_server={},tower_http=debug,axum=debug", level);
        EnvFilter::new(filter_str)
    } else {
        EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("tachyon_server=info,tower_http=debug,axum=debug"))
    };

    tracing_subscriber::registry()
        .with(env_filter)
        .with(match config.log.format.as_str() {
            "json" => fmt::layer().json().boxed(),
            _ => fmt::layer().boxed(),
        })
        .init();
}

/// Initialize Tantivy search index (best-effort).
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

/// Run the HTTP server using the shared library.
async fn run_server(config: ServerConfig) -> Result<()> {
    let addr: SocketAddr = config
        .bind_address()
        .parse()
        .context("Invalid server address")?;

    // Initialize all application state via the shared library.
    let state = tachyon_server::init_app_state(&config).await?;

    // Initialize optional Tantivy search index and inject into relevant states.
    let tantivy_index = init_tantivy_index().await;

    let document_state = match &tantivy_index {
        Some(mgr) => state.0.with_index_manager(mgr.clone()),
        None => state.0,
    };
    let search_state = match &tantivy_index {
        Some(mgr) => state.8.with_index_manager(mgr.clone()),
        None => state.8,
    };

    // Build the full Axum router with all routes and middleware.
    let app = tachyon_server::build_app(
        document_state, state.1, state.2, state.3, state.4, state.5,
        state.6, state.7, search_state, state.9, state.10, state.11,
        state.12, state.13, state.14, state.15, state.16, state.17,
        state.18,
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

    let _ = tokio::time::timeout(std::time::Duration::from_secs(30), server_task)
        .await
        .context("Server shutdown timeout")??;

    info!("Server shutdown complete");

    Ok(())
}

/// Main entry point
#[tokio::main]
async fn main() -> Result<()> {
    // Setup panic handler FIRST before any other initialization
    setup_panic_handler();
    START_TIME.get_or_init(Instant::now);

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
