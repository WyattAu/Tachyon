// Error handling module
// Provides centralized error types and conversion utilities

use thiserror::Error;
use crate::types::error::AppErrorCode;

/// Core error type for all Tachyon operations
#[derive(Debug, Error)]
pub enum Error {
    /// Document not found
    #[error("Document not found")]
    NotFound,
    
    /// Permission denied
    #[error("Permission denied")]
    PermissionDenied,
    
    /// Authentication required
    #[error("Authentication required")]
    Unauthorized,
    
    /// Validation error with details
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    /// Resource not available
    #[error("Resource unavailable")]
    ResourceUnavailable,
    
    /// Internal server error
    #[error("Internal error: {0}")]
    InternalError(String),
    
    /// Database error
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    /// Network error
    #[error("Network error: {0}")]
    NetworkError(String),
    
    /// File system error
    #[error("File system error: {0}")]
    FileSystemError(String),
    
    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    /// Rate limit exceeded
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    /// Invalid input with details
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    /// Application-specific error with code
    #[error("Application error: {code:?} - {message}")]
    AppError {
        code: AppErrorCode,
        message: String,
    },
}

/// Result type alias for convenience
pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    /// Get the HTTP status code for this error
    pub fn status_code(&self) -> u16 {
        match self {
            Error::NotFound => 404,
            Error::PermissionDenied => 403,
            Error::Unauthorized => 401,
            Error::ValidationError(_) | Error::InvalidInput(_) => 400,
            Error::ResourceUnavailable => 503,
            Error::InternalError(_) | Error::DatabaseError(_) | Error::FileSystemError(_) => 500,
            Error::NetworkError(_) => 503,
            Error::SerializationError(_) => 422,
            Error::RateLimitExceeded => 429,
            Error::AppError { code, .. } => match code {
                AppErrorCode::InvalidDocumentId => 400,
                AppErrorCode::InvalidContentType => 415,
                AppErrorCode::DocumentTitleTooLong => 400,
                AppErrorCode::DocumentTitleTooShort => 400,
                AppErrorCode::InvalidTag => 400,
                AppErrorCode::InvalidVisibility => 400,
                AppErrorCode::InvalidStatusTransition => 400,
                AppErrorCode::DocumentLocked => 409,
                AppErrorCode::DocumentAlreadyLinked => 409,
                AppErrorCode::InvalidRepository => 400,
                AppErrorCode::InvalidUser => 400,
                AppErrorCode::SearchError => 500,
                AppErrorCode::AuthError => 401,
                AppErrorCode::AuthzError => 403,
                AppErrorCode::ConflictError => 409,
                AppErrorCode::RateLimitError => 429,
            },
        }
    }
}
