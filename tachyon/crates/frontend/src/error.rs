// Frontend Error Types
// Error handling for the Leptos frontend

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Frontend error type
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum FrontendError {
    #[error("API error: {0}")]
    ApiError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl From<serde_json::Error> for FrontendError {
    fn from(e: serde_json::Error) -> Self {
        FrontendError::SerializationError(e.to_string())
    }
}

impl From<gloo_net::Error> for FrontendError {
    fn from(e: gloo_net::Error) -> Self {
        FrontendError::NetworkError(e.to_string())
    }
}

/// Result type for frontend operations
pub type FrontendResult<T> = Result<T, FrontendError>;
