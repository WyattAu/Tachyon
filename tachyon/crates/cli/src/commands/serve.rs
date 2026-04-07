// Serve command for Tachyon CLI

use crate::commands::Command;
use crate::config::TachyonConfig;
use crate::config::{DEFAULT_HOST, DEFAULT_HTTP_PORT, DEFAULT_WS_PORT};
use crate::error::{CliError, CliResult};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::mpsc;
use tachyon_core::{FileChangeEvent, FileWatcher, FileWatcherConfig};
use tachyon_database::DatabasePool;
use tachyon_server::sync::{FileSyncService, SyncConfig, SyncResult};
use tokio::signal;
use tokio::sync::broadcast;

/// Options for serve command
#[derive(Debug, Clone)]
pub struct ServeOptions {
    /// Host address to bind to
    pub host: String,

    /// HTTP port
    pub http_port: u16,

    /// WebSocket port
    pub ws_port: u16,

    /// Enable TLS
    pub tls_enabled: bool,

    /// TLS certificate path
    pub tls_cert: Option<String>,

    /// TLS key path
    pub tls_key: Option<String>,

    /// Repository path
    pub repo_path: PathBuf,

    /// Maximum request body size
    pub max_body_size: Option<usize>,

    /// Request timeout in seconds
    pub timeout: Option<u64>,

    /// Enable file watching and auto-sync
    pub watch: bool,

    /// Path to watch for changes (defaults to --repo-path or current directory)
    pub watch_path: Option<PathBuf>,
}

impl Default for ServeOptions {
    fn default() -> Self {
        Self {
            host: DEFAULT_HOST.to_string(),
            http_port: DEFAULT_HTTP_PORT,
            ws_port: DEFAULT_WS_PORT,
            tls_enabled: false,
            tls_cert: None,
            tls_key: None,
            repo_path: PathBuf::from(".tachyon"),
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
    pub fn from_args(
        host: Option<String>,
        http_port: Option<u16>,
        ws_port: Option<u16>,
        tls_enabled: bool,
        tls_cert: Option<String>,
        tls_key: Option<String>,
        repo_path: Option<PathBuf>,
        max_body_size: Option<usize>,
        timeout: Option<u64>,
        watch: bool,
        watch_path: Option<PathBuf>,
    ) -> Self {
        Self::new(ServeOptions {
            host: host.unwrap_or_else(|| DEFAULT_HOST.to_string()),
            http_port: http_port.unwrap_or(DEFAULT_HTTP_PORT),
            ws_port: ws_port.unwrap_or(DEFAULT_WS_PORT),
            tls_enabled,
            tls_cert,
            tls_key,
            repo_path: repo_path.unwrap_or_else(|| PathBuf::from(".tachyon")),
            max_body_size,
            timeout,
            watch,
            watch_path,
        })
    }

    /// Load configuration from file
    fn load_config(&self) -> CliResult<TachyonConfig> {
        let config_path = self.options.repo_path.join("tachyon.toml");

        if config_path.exists() {
            TachyonConfig::load_from_file(&config_path)
        } else {
            Ok(TachyonConfig::default())
        }
    }

    /// Validate repository path
    fn validate_repo_path(&self) -> CliResult<()> {
        if !self.options.repo_path.exists() {
            return Err(CliError::init_failed(format!(
                "Repository path does not exist: {}. Run 'tachyon init' first.",
                self.options.repo_path.display()
            )));
        }

        let db_path = self.options.repo_path.join("db");
        if !db_path.exists() {
            return Err(CliError::init_failed(format!(
                "Database directory does not exist: {}. Run 'tachyon init' first.",
                db_path.display()
            )));
        }

        Ok(())
    }

    /// Parse socket address
    fn parse_address(&self, host: &str, port: u16) -> CliResult<SocketAddr> {
        let addr_str = format!("{}:{}", host, port);
        addr_str
            .parse()
            .map_err(|e| CliError::invalid_argument(format!("Invalid address {}: {}", addr_str, e)))
    }

    /// Start HTTP server
    async fn start_http_server(&self) -> CliResult<()> {
        let addr = self.parse_address(&self.options.host, self.options.http_port)?;

        println!(
            "Starting HTTP/2 server on http://{}:{}...",
            self.options.host, self.options.http_port
        );

        // Create axum router with routes
        let app = axum::Router::new()
            .route(
                "/health",
                axum::routing::get(|| async { axum::Json(serde_json::json!({"status": "ok"})) }),
            )
            .route(
                "/api",
                axum::routing::get(|| async {
                    axum::Json(serde_json::json!({"message": "Tachyon API"}))
                }),
            );

        // Bind to address
        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(|e| CliError::server(format!("Failed to bind to {}: {}", addr, e)))?;

        println!("HTTP server listening on {}", addr);

        // Start server with graceful shutdown
        let mut shutdown_rx = self.shutdown.subscribe();

        axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                shutdown_rx.recv().await.ok();
                println!("Shutting down HTTP server...");
            })
            .await
            .map_err(|e| CliError::server(format!("HTTP server error: {}", e)))?;

        Ok(())
    }

    /// Start WebSocket server
    async fn start_ws_server(&self) -> CliResult<()> {
        let addr = self.parse_address(&self.options.host, self.options.ws_port)?;

        println!(
            "Starting WebSocket server on ws://{}:{}...",
            self.options.host, self.options.ws_port
        );

        // Create axum router with WebSocket route
        let app = axum::Router::new().route(
            "/ws",
            axum::routing::get(|ws: axum::extract::WebSocketUpgrade| async {
                ws.on_upgrade(|mut socket| async move {
                    while let Some(Ok(msg)) = socket.recv().await {
                        if let axum::extract::ws::Message::Text(text) = msg {
                            if let Err(e) = socket
                                .send(axum::extract::ws::Message::Text(format!("Echo: {}", text).into()))
                                .await
                            {
                                eprintln!("WebSocket error: {}", e);
                                break;
                            }
                        }
                    }
                })
            }),
        );

        // Bind to address
        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(|e| CliError::server(format!("Failed to bind to {}: {}", addr, e)))?;

        println!("WebSocket server listening on {}", addr);

        // Start server with graceful shutdown
        let mut shutdown_rx = self.shutdown.subscribe();

        axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                shutdown_rx.recv().await.ok();
                println!("Shutting down WebSocket server...");
            })
            .await
            .map_err(|e| CliError::server(format!("WebSocket server error: {}", e)))?;

        Ok(())
    }

    /// Wait for shutdown signal
    async fn wait_for_shutdown(&self) {
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
            Some(())
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => {
                println!("Received CTRL+C, shutting down...");
            }
            _ = terminate => {
                println!("Received SIGTERM, shutting down...");
            }
        }
    }

    async fn run_file_watcher(
        watch_path: PathBuf,
        db_path: PathBuf,
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

        let pool = match DatabasePool::new(&format!("sqlite:{}", db_path.join("tachyon.db").display())).await {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Failed to open database (watch sync disabled): {}", e);
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
        // Validate repository path
        self.validate_repo_path()?;

        // Load configuration
        let _config = self.load_config()?;

        println!("");
        println!("Tachyon Server");
        println!("==============");
        println!("Repository: {}", self.options.repo_path.display());
        println!(
            "HTTP: http://{}:{}",
            self.options.host, self.options.http_port
        );
        println!(
            "WebSocket: ws://{}:{}",
            self.options.host, self.options.ws_port
        );
        println!("");
        println!("Press CTRL+C to stop");
        println!("");

        // Create runtime for async execution
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| CliError::server(format!("Failed to create runtime: {}", e)))?;

        // Clone necessary data for async block
        let http_port = self.options.http_port;
        let ws_port = self.options.ws_port;
        let host = self.options.host.clone();
        let repo_path = self.options.repo_path.clone();
        let shutdown = self.shutdown.clone();
        let watch = self.options.watch;
        let watch_path = self.options.watch_path.clone();

        rt.block_on(async move {
            let cmd = ServeCommand {
                options: ServeOptions {
                    host: host.clone(),
                    http_port,
                    ws_port,
                    tls_enabled: false,
                    tls_cert: None,
                    tls_key: None,
                    repo_path: repo_path.clone(),
                    max_body_size: None,
                    timeout: None,
                    watch,
                    watch_path: watch_path.clone(),
                },
                shutdown: shutdown.clone(),
            };

            // Start HTTP server
            let cmd_http = cmd.clone();
            let http_handle = tokio::spawn(async move {
                if let Err(e) = cmd_http.start_http_server().await {
                    eprintln!("HTTP server error: {}", e);
                }
            });

            // Start WebSocket server
            let cmd_ws = cmd.clone();
            let ws_handle = tokio::spawn(async move {
                if let Err(e) = cmd_ws.start_ws_server().await {
                    eprintln!("WebSocket server error: {}", e);
                }
            });

            // Start file watcher if --watch is enabled
            let watch_handle = if watch {
                let effective_watch_path = watch_path.unwrap_or_else(|| repo_path.clone());
                let db_path = repo_path.join("db");
                let shutdown_for_watch = shutdown.clone();

                Some(tokio::spawn(async move {
                    Self::run_file_watcher(effective_watch_path, db_path, shutdown_for_watch).await;
                }))
            } else {
                None
            };

            // Wait for shutdown signal
            cmd.wait_for_shutdown().await;

            // Trigger shutdown
            cmd.shutdown.shutdown();

            // Wait for servers to shut down
            let mut handles: Vec<tokio::task::JoinHandle<()>> = vec![http_handle, ws_handle];
            if let Some(h) = watch_handle {
                handles.push(h);
            }
            for h in handles {
                let _ = h.await;
            }

            Ok::<(), CliError>(())
        })?;

        println!("Server stopped.");

        Ok(())
    }

    fn name(&self) -> &str {
        "serve"
    }

    fn description(&self) -> &str {
        "Start HTTP/2 and WebSocket servers"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serve_options_default() {
        let options = ServeOptions::default();
        assert_eq!(options.host, DEFAULT_HOST);
        assert_eq!(options.http_port, DEFAULT_HTTP_PORT);
        assert_eq!(options.ws_port, DEFAULT_WS_PORT);
        assert!(!options.tls_enabled);
        assert!(!options.watch);
        assert!(options.watch_path.is_none());
    }

    #[test]
    fn test_serve_command_from_args() {
        let cmd = ServeCommand::from_args(
            Some("0.0.0.0".to_string()),
            Some(9000),
            Some(9001),
            false,
            None,
            None,
            Some(PathBuf::from("/tmp/test")),
            Some(1024),
            Some(60),
            true,
            Some(PathBuf::from("/tmp/watch")),
        );

        assert_eq!(cmd.options.host, "0.0.0.0");
        assert_eq!(cmd.options.http_port, 9000);
        assert_eq!(cmd.options.ws_port, 9001);
        assert_eq!(cmd.options.repo_path, PathBuf::from("/tmp/test"));
        assert_eq!(cmd.options.max_body_size, Some(1024));
        assert_eq!(cmd.options.timeout, Some(60));
        assert!(cmd.options.watch);
        assert_eq!(cmd.options.watch_path, Some(PathBuf::from("/tmp/watch")));
    }

    #[test]
    fn test_parse_address_valid() {
        let options = ServeOptions::default();
        let cmd = ServeCommand::new(options);

        let addr = cmd.parse_address("127.0.0.1", 8080).unwrap();
        assert_eq!(addr.port(), 8080);
        assert_eq!(addr.ip().to_string(), "127.0.0.1");
    }

    #[test]
    fn test_parse_address_invalid() {
        let options = ServeOptions::default();
        let cmd = ServeCommand::new(options);

        let result = cmd.parse_address("invalid", 8080);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_repo_path_not_exists() {
        let options = ServeOptions {
            repo_path: PathBuf::from("/nonexistent/path"),
            ..Default::default()
        };
        let cmd = ServeCommand::new(options);

        let result = cmd.validate_repo_path();
        assert!(result.is_err());
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
