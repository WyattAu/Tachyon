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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_error_display() {
        let err = FrontendError::ApiError("HTTP 404".to_string());
        assert_eq!(format!("{}", err), "API error: HTTP 404");
    }

    #[test]
    fn test_network_error_display() {
        let err = FrontendError::NetworkError("timeout".to_string());
        assert_eq!(format!("{}", err), "Network error: timeout");
    }

    #[test]
    fn test_unauthorized_display() {
        let err = FrontendError::Unauthorized;
        assert_eq!(format!("{}", err), "Unauthorized");
    }

    #[test]
    fn test_validation_error_display() {
        let err = FrontendError::ValidationError("field required".to_string());
        assert_eq!(format!("{}", err), "Validation error: field required");
    }

    #[test]
    fn test_not_found_display() {
        let err = FrontendError::NotFound("document 123".to_string());
        assert_eq!(format!("{}", err), "Not found: document 123");
    }

    #[test]
    fn test_serialization_error_display() {
        let err = FrontendError::SerializationError("bad json".to_string());
        assert_eq!(format!("{}", err), "Serialization error: bad json");
    }

    #[test]
    fn test_error_clone() {
        let err = FrontendError::ApiError("test".to_string());
        let cloned = err.clone();
        assert_eq!(format!("{}", err), format!("{}", cloned));
    }

    #[test]
    fn test_error_debug() {
        let err = FrontendError::Unknown("something".to_string());
        let debug = format!("{:?}", err);
        assert!(debug.contains("Unknown"));
        assert!(debug.contains("something"));
    }

    #[test]
    fn test_result_type() {
        let ok: FrontendResult<i32> = Ok(42);
        let err: FrontendResult<i32> = Err(FrontendError::Unauthorized);
        assert_eq!(ok.unwrap(), 42);
        assert!(err.is_err());
    }

    #[test]
    fn test_from_serde_json_error() {
        let serde_err: Result<i32, _> = serde_json::from_str("not a number");
        let frontend_err: FrontendError = serde_err.unwrap_err().into();
        match frontend_err {
            FrontendError::SerializationError(msg) => assert!(!msg.is_empty()),
            _ => panic!("Expected SerializationError"),
        }
    }

    #[test]
    fn test_error_serialization_roundtrip() {
        let err = FrontendError::ApiError("bad request".to_string());
        let json = serde_json::to_string(&err).unwrap();
        let parsed: FrontendError = serde_json::from_str(&json).unwrap();
        assert_eq!(format!("{}", parsed), "API error: bad request");
    }
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
