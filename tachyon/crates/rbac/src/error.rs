// RBAC Error Types
// Comprehensive error handling for RBAC operations

use thiserror::Error;

/// Result type alias for RBAC operations
pub type RbacResult<T> = Result<T, RbacError>;

/// RBAC error types
#[derive(Error, Debug)]
pub enum RbacError {
    /// Permission denied error
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Invalid subject error
    #[error("Invalid subject: {0}")]
    InvalidSubject(String),

    /// Invalid resource error
    #[error("Invalid resource: {0}")]
    InvalidResource(String),

    /// Invalid policy error
    #[error("Invalid policy: {0}")]
    InvalidPolicy(String),

    /// Policy evaluation error
    #[error("Policy evaluation failed: {0}")]
    PolicyEvaluationFailed(String),

    /// Session error
    #[error("Session error: {0}")]
    SessionError(String),

    /// Cache error
    #[error("Cache error: {0}")]
    CacheError(String),

    /// Database error
    #[error("Database error: {0}")]
    DatabaseError(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Internal error
    #[error("Internal error: {0}")]
    InternalError(String),

    /// Not found error
    #[error("Not found: {0}")]
    NotFound(String),
}

impl RbacError {
    /// Create a permission denied error
    pub fn permission_denied(message: impl Into<String>) -> Self {
        Self::PermissionDenied(message.into())
    }

    /// Create an invalid subject error
    pub fn invalid_subject(message: impl Into<String>) -> Self {
        Self::InvalidSubject(message.into())
    }

    /// Create an invalid resource error
    pub fn invalid_resource(message: impl Into<String>) -> Self {
        Self::InvalidResource(message.into())
    }

    /// Create an invalid policy error
    pub fn invalid_policy(message: impl Into<String>) -> Self {
        Self::InvalidPolicy(message.into())
    }

    /// Create a policy evaluation error
    pub fn policy_evaluation_failed(message: impl Into<String>) -> Self {
        Self::PolicyEvaluationFailed(message.into())
    }

    /// Create a session error
    pub fn session_error(message: impl Into<String>) -> Self {
        Self::SessionError(message.into())
    }

    /// Create a cache error
    pub fn cache_error(message: impl Into<String>) -> Self {
        Self::CacheError(message.into())
    }

    /// Create a database error
    pub fn database_error(message: impl Into<String>) -> Self {
        Self::DatabaseError(message.into())
    }

    /// Create a configuration error
    pub fn configuration_error(message: impl Into<String>) -> Self {
        Self::ConfigurationError(message.into())
    }

    /// Create a serialization error
    pub fn serialization_error(message: impl Into<String>) -> Self {
        Self::SerializationError(message.into())
    }

    /// Create an internal error
    pub fn internal(message: impl Into<String>) -> Self {
        Self::InternalError(message.into())
    }

    /// Create a not found error
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::NotFound(message.into())
    }

    /// Check if error is recoverable
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::CacheError(_)
                | Self::SessionError(_)
                | Self::ConfigurationError(_)
                | Self::InternalError(_)
        )
    }

    /// Check if error is permission related
    pub fn is_permission_error(&self) -> bool {
        matches!(self, Self::PermissionDenied(_))
    }

    /// Check if error is validation related
    pub fn is_validation_error(&self) -> bool {
        matches!(
            self,
            Self::InvalidSubject(_) | Self::InvalidResource(_) | Self::InvalidPolicy(_)
        )
    }

    /// Get error code for API responses
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::PermissionDenied(_) => "PERMISSION_DENIED",
            Self::InvalidSubject(_) => "INVALID_SUBJECT",
            Self::InvalidResource(_) => "INVALID_RESOURCE",
            Self::InvalidPolicy(_) => "INVALID_POLICY",
            Self::PolicyEvaluationFailed(_) => "POLICY_EVALUATION_FAILED",
            Self::SessionError(_) => "SESSION_ERROR",
            Self::CacheError(_) => "CACHE_ERROR",
            Self::DatabaseError(_) => "DATABASE_ERROR",
            Self::ConfigurationError(_) => "CONFIGURATION_ERROR",
            Self::SerializationError(_) => "SERIALIZATION_ERROR",
            Self::InternalError(_) => "INTERNAL_ERROR",
            Self::NotFound(_) => "NOT_FOUND",
        }
    }

    /// Get HTTP status code for error
    pub fn http_status_code(&self) -> u16 {
        match self {
            Self::PermissionDenied(_) => 403,
            Self::InvalidSubject(_) | Self::InvalidResource(_) | Self::InvalidPolicy(_) => 400,
            Self::PolicyEvaluationFailed(_) | Self::SessionError(_) | Self::CacheError(_) => 422,
            Self::NotFound(_) => 404,
            Self::DatabaseError(_) | Self::ConfigurationError(_) | Self::SerializationError(_) => {
                500
            }
            Self::InternalError(_) => 500,
        }
    }
}

/// Convert from sqlx::Error to RbacError
impl From<sqlx::Error> for RbacError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => Self::not_found(err.to_string()),
            _ => Self::database_error(err.to_string()),
        }
    }
}

/// Convert from serde_json::Error to RbacError
impl From<serde_json::Error> for RbacError {
    fn from(err: serde_json::Error) -> Self {
        Self::serialization_error(err.to_string())
    }
}

/// Convert from std::io::Error to RbacError
impl From<std::io::Error> for RbacError {
    fn from(err: std::io::Error) -> Self {
        Self::internal(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        let error = RbacError::permission_denied("Access denied");
        assert_eq!(error.error_code(), "PERMISSION_DENIED");
        assert_eq!(error.http_status_code(), 403);
    }

    #[test]
    fn test_recoverable_errors() {
        assert!(RbacError::cache_error("Cache miss").is_recoverable());
        assert!(!RbacError::permission_denied("No access").is_recoverable());
    }

    #[test]
    fn test_permission_errors() {
        assert!(RbacError::permission_denied("No access").is_permission_error());
        assert!(!RbacError::invalid_subject("Bad subject").is_permission_error());
    }

    #[test]
    fn test_validation_errors() {
        assert!(RbacError::invalid_subject("Bad subject").is_validation_error());
        assert!(RbacError::invalid_resource("Bad resource").is_validation_error());
        assert!(!RbacError::permission_denied("No access").is_validation_error());
    }

    #[test]
    fn test_error_display() {
        let error = RbacError::permission_denied("Access denied");
        assert_eq!(error.to_string(), "Permission denied: Access denied");
    }
}
