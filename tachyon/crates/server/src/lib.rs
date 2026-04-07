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
pub mod middleware;
pub mod routes;
pub mod validation;
pub mod websocket;

pub use api_docs::*;
pub use audit::*;
#[allow(ambiguous_glob_reexports)]
pub use config::*;
pub use middleware::*;
pub use validation::*;
