//! Error types for the SSG engine.

use thiserror::Error;

/// SSG engine error types.
#[derive(Debug, Error)]
pub enum SsgError {
    /// I/O error (file read/write)
    #[error("I/O error: {0}")]
    Io(String),

    /// ZIP archive error
    #[error("ZIP error: {0}")]
    Zip(String),

    /// Rendering error
    #[error("Render error: {0}")]
    Render(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),
}

/// SSG result type alias.
pub type SsgResult<T> = Result<T, SsgError>;
