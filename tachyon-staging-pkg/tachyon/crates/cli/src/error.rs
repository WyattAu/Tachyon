// CLI error types and result handling

use std::path::PathBuf;
use thiserror::Error;

/// Result type alias for CLI operations
pub type CliResult<T> = Result<T, CliError>;

/// CLI error types
#[derive(Error, Debug)]
pub enum CliError {
    /// Configuration file errors
    #[error("Configuration error: {0}")]
    Config(String),

    /// File system operation errors
    #[error("File system error: {0}")]
    FileSystem(String),

    /// Git operation errors
    #[error("Git error: {0}")]
    Git(String),

    /// Database operation errors
    #[error("Database error: {0}")]
    Database(String),

    /// Server operation errors
    #[error("Server error: {0}")]
    Server(String),

    /// Desktop application errors
    #[error("Desktop error: {0}")]
    Desktop(String),

    /// Build operation errors
    #[error("Build error: {0}")]
    Build(String),

    /// Invalid command arguments
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    /// Command execution errors
    #[error("Command failed: {0}")]
    Command(String),

    /// IO errors
    #[error("IO error: {path:?}: {message}")]
    Io { path: PathBuf, message: String },

    /// Permission errors
    #[error("Permission denied: {0}")]
    Permission(String),

    /// Repository initialization errors
    #[error("Repository initialization failed: {0}")]
    InitFailed(String),

    /// Shutdown signal errors
    #[error("Shutdown error: {0}")]
    Shutdown(String),

    /// Generic errors
    #[error("{0}")]
    Generic(String),
}

impl CliError {
    /// Creates a configuration error
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }

    /// Creates a file system error
    pub fn filesystem(msg: impl Into<String>) -> Self {
        Self::FileSystem(msg.into())
    }

    /// Creates a git error
    pub fn git(msg: impl Into<String>) -> Self {
        Self::Git(msg.into())
    }

    /// Creates a database error
    pub fn database(msg: impl Into<String>) -> Self {
        Self::Database(msg.into())
    }

    /// Creates a server error
    pub fn server(msg: impl Into<String>) -> Self {
        Self::Server(msg.into())
    }

    /// Creates a desktop error
    pub fn desktop(msg: impl Into<String>) -> Self {
        Self::Desktop(msg.into())
    }

    /// Creates a build error
    pub fn build(msg: impl Into<String>) -> Self {
        Self::Build(msg.into())
    }

    /// Creates an invalid argument error
    pub fn invalid_argument(msg: impl Into<String>) -> Self {
        Self::InvalidArgument(msg.into())
    }

    /// Creates a command error
    pub fn command(msg: impl Into<String>) -> Self {
        Self::Command(msg.into())
    }

    /// Creates an IO error
    pub fn io(path: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        Self::Io {
            path: path.into(),
            message: message.into(),
        }
    }

    /// Creates a permission error
    pub fn permission(msg: impl Into<String>) -> Self {
        Self::Permission(msg.into())
    }

    /// Creates an initialization failed error
    pub fn init_failed(msg: impl Into<String>) -> Self {
        Self::InitFailed(msg.into())
    }

    /// Creates a shutdown error
    pub fn shutdown(msg: impl Into<String>) -> Self {
        Self::Shutdown(msg.into())
    }

    /// Creates a generic error
    pub fn generic(msg: impl Into<String>) -> Self {
        Self::Generic(msg.into())
    }

    /// Returns the exit code for this error
    pub fn exit_code(&self) -> i32 {
        match self {
            CliError::Config(_) => 1,
            CliError::FileSystem(_) => 2,
            CliError::Git(_) => 3,
            CliError::Database(_) => 4,
            CliError::Server(_) => 5,
            CliError::Desktop(_) => 6,
            CliError::Build(_) => 7,
            CliError::InvalidArgument(_) => 64, // EX_USAGE
            CliError::Command(_) => 1,
            CliError::Io { .. } => 2,
            CliError::Permission(_) => 126, // EX_NOPERM
            CliError::InitFailed(_) => 1,
            CliError::Shutdown(_) => 130, // SIGINT
            CliError::Generic(_) => 1,
        }
    }
}

// Implement conversions from standard error types
impl From<std::io::Error> for CliError {
    fn from(err: std::io::Error) -> Self {
        Self::Io {
            path: PathBuf::new(),
            message: err.to_string(),
        }
    }
}

impl From<serde_json::Error> for CliError {
    fn from(err: serde_json::Error) -> Self {
        Self::Config(err.to_string())
    }
}

impl From<git2::Error> for CliError {
    fn from(err: git2::Error) -> Self {
        Self::Git(err.to_string())
    }
}

impl From<tachyon_core::types::TachyonError> for CliError {
    fn from(err: tachyon_core::types::TachyonError) -> Self {
        Self::Generic(err.to_string())
    }
}

impl From<tachyon_database::DatabaseError> for CliError {
    fn from(err: tachyon_database::DatabaseError) -> Self {
        Self::Database(err.to_string())
    }
}
