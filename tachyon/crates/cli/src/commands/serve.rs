// Serve command for Tachyon CLI
//
// Starts the full Tachyon server (HTTP + WebSocket) using the shared
// server library. Delegates to `tachyon_server::build_server()` for
// all routing, middleware, and state initialization.

use crate::commands::Command;
use crate::error::{CliError, CliResult};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::mpsc;
use tachyon_core::{FileChangeEvent, FileWatcher, FileWatcherConfig};
use tachyon_server::config::ServerConfig;
use tachyon_server::sync::{FileSyncService, SyncConfig, SyncResult};
use tokio::signal;
use tokio::sync::broadcast;

/// Options for serve command
#[derive(Debug, Clone)]
pub struct ServeOptions {
    /// Host address to bind to
    pub host: String,

    /// HTTP port (WebSocket shares this port)
    pub port: u16,

    /// Enable TLS
    pub tls_enabled: bool,

    /// TLS certificate path
    pub tls_cert: Option<String>,

    /// TLS key path
    pub tls_key: Option<String>,

    /// Database URL (PostgreSQL connection string)
    pub database_url: Option<String>,

    /// Maximum request body size
    pub max_body_size: Option<usize>,

    /// Request timeout in seconds
    pub timeout: Option<u64>,

    /// Enable file watching and auto-sync
    pub watch: bool,

    /// Path to watch for changes (defaults to current directory)
    pub watch_path: Option<PathBuf>,
}

impl Default for ServeOptions {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            tls_enabled: false,
            tls_cert: None,
            tls_key: None,
            database_url: None,
            max_body_size: None,
            timeout: None,
            watch: false,
            watch_path: None,
        }
    }
}

/// Server shutdown signal
#[derive(Clone)]
pub struct ShutdownSignal {
    tx: broadcast::Sender<()>,
}

impl Default for ShutdownSignal {
    fn default() -> Self {
        Self::new()
    }
}

impl ShutdownSignal {
    /// Create a new shutdown signal
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(1);
        Self { tx }
    }

    /// Send shutdown signal
    pub fn shutdown(&self) {
        let _ = self.tx.send(());
    }

    /// Subscribe to shutdown signal
    pub fn subscribe(&self) -> broadcast::Receiver<()> {
        self.tx.subscribe()
    }
}

/// Serve command handler
#[derive(Clone)]
pub struct ServeCommand {
    options: ServeOptions,
    shutdown: ShutdownSignal,
}

impl ServeCommand {
    /// Create a new serve command
    pub fn new(options: ServeOptions) -> Self {
        Self {
            options,
            shutdown: ShutdownSignal::new(),
        }
    }

    /// Create from clap arguments
    #[allow(clippy::too_many_arguments)]
    pub fn from_args(
        host: Option<String>,
        port: Option<u16>,
        tls_enabled: bool,
        tls_cert: Option<String>,
        tls_key: Option<String>,
        database_url: Option<String>,
        max_body_size: Option<usize>,
        timeout: Option<u64>,
        watch: bool,
        watch_path: Option<PathBuf>,
    ) -> Self {
        Self::new(ServeOptions {
            host: host.unwrap_or_else(|| "127.0.0.1".to_string()),
            port: port.unwrap_or(8080),
            tls_enabled,
            tls_cert,
            tls_key,
            database_url,
            max_body_size,
            timeout,
            watch,
            watch_path,
        })
    }

    /// Build a `tachyon_server::config::ServerConfig` from CLI options + env vars.
    ///
    /// CLI options take precedence over environment variables for host/port.
    /// Environment variables (DATABASE_URL, TACHYON_JWT_SECRET, etc.) are
    /// still respected for values not specified via CLI.
    fn build_server_config(&self) -> ServerConfig {
        let mut config = ServerConfig::from_env();

        // CLI options override env vars
        config.host = self.options.host.clone();
        config.port = self.options.port;
        config.enable_tls = self.options.tls_enabled;
        config.tls_cert_path = self.options.tls_cert.clone();
        config.tls_key_path = self.options.tls_key.clone();

        if let Some(ref db_url) = self.options.database_url {
            config.database_url = db_url.clone();
        }

        config
    }

    /// Wait for shutdown signal (Ctrl+C or SIGTERM)
    async fn wait_for_shutdown() {
        let ctrl_c = async {
            signal::ctrl_c()
                .await
                .expect("failed to install CTRL+C handler");
        };

        #[cfg(unix)]
        let terminate = async {
            signal::unix::signal(signal::unix::SignalKind::terminate())
                .expect("failed to install SIGTERM handler")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => {
                tracing::info!("Received CTRL+C, shutting down...");
            }
            _ = terminate => {
                tracing::info!("Received SIGTERM, shutting down...");
            }
        }
    }

    /// Run file watcher for auto-sync (optional, gated behind --watch).
    async fn run_file_watcher(
        watch_path: PathBuf,
        database_url: String,
        shutdown: ShutdownSignal,
    ) {
        if !watch_path.exists() {
            eprintln!("Watch path does not exist: {}", watch_path.display());
            return;
        }

        println!("Watching for changes in: {}", watch_path.display());

        let config = FileWatcherConfig {
            watch_path: watch_path.clone(),
            watch_extensions: vec![".md".to_string(), ".markdown".to_string()],
            debounce_ms: 500,
            recursive: true,
        };

        let mut watcher = match FileWatcher::new(config) {
            Ok(w) => w,
            Err(e) => {
                eprintln!("Failed to create file watcher: {}", e);
                return;
            }
        };

        let (tx, rx) = mpsc::channel::<FileChangeEvent>();

        if let Err(e) = watcher.start(tx) {
            eprintln!("Failed to start file watcher: {}", e);
            return;
        }

        let (async_tx, mut async_rx) = tokio::sync::mpsc::channel::<FileChangeEvent>(32);

        std::thread::spawn(move || {
            while let Ok(event) = rx.recv() {
                if async_tx.blocking_send(event).is_err() {
                    break;
                }
            }
        });

        let pool = match tachyon_database::DatabasePool::new(&database_url).await {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Failed to connect to database (watch sync disabled): {}", e);
                let mut shutdown_rx = shutdown.subscribe();
                let _ = shutdown_rx.recv().await;
                watcher.stop();
                return;
            }
        };

        let sync_config = SyncConfig::default();
        let sync_service = FileSyncService::new(pool, sync_config);

        let initial_events = watcher.scan_initial();
        if !initial_events.is_empty() {
            println!("Initial scan: found {} markdown files", initial_events.len());
            for event in &initial_events {
                Self::sync_and_report(&sync_service, event).await;
            }
        }

        let mut shutdown_rx = shutdown.subscribe();

        loop {
            tokio::select! {
                _ = shutdown_rx.recv() => {
                    println!("Stopping file watcher...");
                    watcher.stop();
                    break;
                }
                event = async_rx.recv() => {
                    match event {
                        Some(event) => {
                            Self::sync_and_report(&sync_service, &event).await;
                        }
                        None => {
                            println!("File watcher channel closed.");
                            break;
                        }
                    }
                }
            }
        }
    }

    async fn sync_and_report(sync_service: &FileSyncService, event: &FileChangeEvent) {
        let kind_str = match event.kind {
            tachyon_core::FileChangeKind::Created => "created",
            tachyon_core::FileChangeKind::Modified => "modified",
            tachyon_core::FileChangeKind::Deleted => "deleted",
        };
        println!("[watch] {} {}", kind_str, event.path.display());

        match sync_service.sync_file(event).await {
            SyncResult::Created { id, slug } => {
                println!("[watch]   -> created document: {} (slug: {})", id, slug);
            }
            SyncResult::Updated { id, slug, hash_changed, conflict } => {
                let mut msg = format!("[watch]   -> updated document: {} (slug: {})", id, slug);
                if !hash_changed {
                    msg.push_str(" (no content change)");
                }
                if conflict {
                    msg.push_str(" (conflict detected)");
                }
                println!("{}", msg);
            }
            SyncResult::Deleted { id, slug } => {
                println!("[watch]   -> deleted document: {} (slug: {})", id, slug);
            }
            SyncResult::Skipped { path, reason } => {
                println!("[watch]   -> skipped {}: {}", path.display(), reason);
            }
            SyncResult::Error { path, message } => {
                eprintln!("[watch]   -> error syncing {}: {}", path.display(), message);
            }
        }
    }
}

impl Command for ServeCommand {
    fn execute(&self) -> CliResult<()> {
        let server_config = self.build_server_config();

        println!();
        println!("Tachyon Server (via CLI)");
        println!("========================");
        println!("Host: {}", server_config.host);
        println!("Port: {}", server_config.port);
        println!("Database: {}", if server_config.database_url.is_empty() {
            server_config.database_path.as_deref().unwrap_or("not configured")
        } else {
            // Don't log the full URL with password
            server_config.database_url.split('@').next_back().unwrap_or("configured")
        });
        if self.options.watch {
            let watch_path = self.options.watch_path.clone()
                .unwrap_or_else(|| PathBuf::from("."));
            println!("Watch: {}", watch_path.display());
        }
        println!();
        println!("Press CTRL+C to stop");
        println!();

        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| CliError::server(format!("Failed to create runtime: {}", e)))?;

        let config = server_config;
        let port = config.port;
        let host = config.host.clone();
        let watch = self.options.watch;
        let watch_path = self.options.watch_path.clone();
        let database_url = config.database_url.clone();
        let shutdown = self.shutdown.clone();

        rt.block_on(async move {
            let addr: SocketAddr = format!("{}:{}", host, port)
                .parse()
                .map_err(|e| CliError::invalid_argument(format!("Invalid address: {}", e)))?;

            // Build the full server (all routes, middleware, state)
            let app = tachyon_server::build_server(&config)
                .await
                .map_err(|e| CliError::server(format!("Failed to initialize server: {}", e)))?;

            let listener = tokio::net::TcpListener::bind(addr)
                .await
                .map_err(|e| CliError::server(format!("Failed to bind to {}: {}", addr, e)))?;

            println!("Server listening on http://{}/", addr);
            println!("API:     http://{}/api/v1/", addr);
            println!("Docs:    http://{}/api/docs", addr);

            // Start the HTTP server
            let server_handle = tokio::spawn(async move {
                axum::serve(listener, app)
                    .with_graceful_shutdown(async {
                        // This future resolves when the CLI receives Ctrl+C/SIGTERM.
                        // We use a separate task for the signal.
                        std::future::pending::<()>().await;
                    })
                    .await
            });

            // Start file watcher if --watch is enabled
            let watch_handle = if watch {
                let effective_watch_path = watch_path.unwrap_or_else(|| PathBuf::from("."));
                let shutdown_for_watch = shutdown.clone();
                Some(tokio::spawn(async move {
                    Self::run_file_watcher(effective_watch_path, database_url, shutdown_for_watch).await;
                }))
            } else {
                None
            };

            // Wait for shutdown signal
            Self::wait_for_shutdown().await;

            // Trigger shutdown for file watcher
            shutdown.shutdown();

            // Abort server
            server_handle.abort();

            // Wait for tasks to finish
            let _ = server_handle.await;
            if let Some(h) = watch_handle {
                let _ = h.await;
            }

            println!("Server stopped.");

            Ok::<(), CliError>(())
        })?;

        Ok(())
    }

    fn name(&self) -> &str {
        "serve"
    }

    fn description(&self) -> &str {
        "Start the Tachyon server (HTTP + WebSocket)"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serve_options_default() {
        let options = ServeOptions::default();
        assert_eq!(options.host, "127.0.0.1");
        assert_eq!(options.port, 8080);
        assert!(!options.tls_enabled);
        assert!(!options.watch);
        assert!(options.watch_path.is_none());
        assert!(options.database_url.is_none());
    }

    #[test]
    fn test_serve_command_from_args() {
        let cmd = ServeCommand::from_args(
            Some("0.0.0.0".to_string()),
            Some(9000),
            false,
            None,
            None,
            Some("postgres://localhost/test".to_string()),
            Some(1024),
            Some(60),
            true,
            Some(PathBuf::from("/tmp/watch")),
        );

        assert_eq!(cmd.options.host, "0.0.0.0");
        assert_eq!(cmd.options.port, 9000);
        assert_eq!(cmd.options.database_url, Some("postgres://localhost/test".to_string()));
        assert_eq!(cmd.options.max_body_size, Some(1024));
        assert_eq!(cmd.options.timeout, Some(60));
        assert!(cmd.options.watch);
        assert_eq!(cmd.options.watch_path, Some(PathBuf::from("/tmp/watch")));
    }

    #[test]
    fn test_build_server_config_defaults() {
        let cmd = ServeCommand::new(ServeOptions::default());
        let config = cmd.build_server_config();

        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 8080);
        assert!(!config.enable_tls);
    }

    #[test]
    fn test_build_server_config_overrides() {
        let cmd = ServeCommand::new(ServeOptions {
            host: "0.0.0.0".to_string(),
            port: 3000,
            database_url: Some("postgres://user:pass@db:5432/mydb".to_string()),
            tls_enabled: true,
            tls_cert: Some("/cert.pem".to_string()),
            tls_key: Some("/key.pem".to_string()),
            ..Default::default()
        });
        let config = cmd.build_server_config();

        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 3000);
        assert_eq!(config.database_url, "postgres://user:pass@db:5432/mydb");
        assert!(config.enable_tls);
        assert_eq!(config.tls_cert_path, Some("/cert.pem".to_string()));
        assert_eq!(config.tls_key_path, Some("/key.pem".to_string()));
    }

    #[test]
    fn test_shutdown_signal() {
        let shutdown = ShutdownSignal::new();

        // Subscribe BEFORE sending the signal
        let mut rx = shutdown.subscribe();

        // Test sending shutdown signal
        shutdown.shutdown();

        // The channel should have received the signal
        let result = rx.try_recv();
        assert!(result.is_ok());
    }
}
