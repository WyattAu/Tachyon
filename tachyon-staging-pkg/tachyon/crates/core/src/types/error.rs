// Error handling type definitions
// Provides TachyonError and ErrorCategory for consistent error handling across all modules

use crate::util::PathError;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Type alias for error results in Tachyon core
pub type ErrorResult<T> = std::result::Result<T, TachyonError>;

/// Main error type for Tachyon core
/// Provides structured error handling with categories, codes, and context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TachyonError {
    /// Error category for classification
    pub category: ErrorCategory,
    /// Machine-readable error code
    pub code: String,
    /// Human-readable error message
    pub message: String,
    /// Additional context about the error
    pub context: Option<String>,
    /// Source error if wrapping another error
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

impl TachyonError {
    /// Create a new TachyonError
    ///
    /// # Arguments
    /// * `category` - Error category
    /// * `code` - Error code
    /// * `message` - Error message
    pub fn new(
        category: ErrorCategory,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            category,
            code: code.into(),
            message: message.into(),
            context: None,
            source: None,
        }
    }

    /// Add context to the error
    ///
    /// # Arguments
    /// * `context` - Additional context
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    /// Add a source error
    ///
    /// # Arguments
    /// * `source` - Source error string
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// Create a storage error
    pub fn storage(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(ErrorCategory::Storage, code, message)
    }

    /// Create an authentication error
    pub fn authentication(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(ErrorCategory::Authentication, code, message)
    }

    /// Create an authorization error
    pub fn authorization(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(ErrorCategory::Authorization, code, message)
    }

    /// Create a rendering error
    pub fn rendering(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(ErrorCategory::Rendering, code, message)
    }

    /// Create a Git error
    pub fn git(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(ErrorCategory::Git, code, message)
    }

    /// Create a validation error
    pub fn validation(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(ErrorCategory::Validation, code, message)
    }

    /// Create a not found error
    pub fn not_found(resource: impl Into<String>) -> Self {
        Self::new(
            ErrorCategory::Storage,
            "NOT_FOUND",
            format!("{} not found", resource.into()),
        )
    }

    /// Create a permission denied error
    pub fn permission_denied(action: impl Into<String>) -> Self {
        Self::new(
            ErrorCategory::Authorization,
            "PERMISSION_DENIED",
            format!("Permission denied: {}", action.into()),
        )
    }

    /// Create an internal error
    pub fn internal(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(ErrorCategory::Internal, code, message)
    }
}

impl fmt::Display for TachyonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}:{}] {}", self.category, self.code, self.message)?;
        if let Some(ref ctx) = self.context {
            write!(f, " (context: {})", ctx)?;
        }
        if let Some(ref src) = self.source {
            write!(f, " (caused by: {})", src)?;
        }
        Ok(())
    }
}

impl std::error::Error for TachyonError {}

/// Error categories for classification and handling
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ErrorCategory {
    /// Storage-related errors (file system, database)
    Storage,
    /// Authentication errors (login, credentials)
    Authentication,
    /// Authorization errors (permissions, RBAC)
    Authorization,
    /// Rendering errors (markdown, templates)
    Rendering,
    /// Git operation errors (clone, push, pull)
    Git,
    /// Validation errors (input validation)
    Validation,
    /// Network errors (API calls, connectivity)
    Network,
    /// Configuration errors (invalid settings)
    Configuration,
    /// Internal errors (unexpected conditions)
    Internal,
}

impl fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Storage => write!(f, "STORAGE"),
            Self::Authentication => write!(f, "AUTH"),
            Self::Authorization => write!(f, "AUTHZ"),
            Self::Rendering => write!(f, "RENDER"),
            Self::Git => write!(f, "GIT"),
            Self::Validation => write!(f, "VALIDATION"),
            Self::Network => write!(f, "NETWORK"),
            Self::Configuration => write!(f, "CONFIG"),
            Self::Internal => write!(f, "INTERNAL"),
        }
    }
}

/// Error codes for specific error scenarios
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorCode {
    // Storage errors
    DocumentNotFound,
    UserNotFound,
    RepositoryNotFound,
    FileReadError,
    FileWriteError,
    DatabaseError,

    // Authentication errors
    InvalidCredentials,
    UserNotFoundAuth,
    PasswordMismatch,
    TokenExpired,
    TokenInvalid,

    // Authorization errors
    PermissionDenied,
    InsufficientRole,
    ResourceAccessDenied,

    // Rendering errors
    MarkdownParseError,
    TemplateError,
    InvalidSyntax,

    // Git errors
    GitNotInitialized,
    GitCloneFailed,
    GitPushFailed,
    GitPullFailed,
    GitMergeConflict,

    // Validation errors
    InvalidInput,
    ValidationError,
    ConstraintViolation,

    // Network errors
    ConnectionError,
    TimeoutError,

    // Configuration errors
    InvalidConfig,
    MissingConfig,

    // Internal errors
    UnexpectedError,
    NotImplemented,
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let code = match self {
            Self::DocumentNotFound => "DOCUMENT_NOT_FOUND",
            Self::UserNotFound => "USER_NOT_FOUND",
            Self::RepositoryNotFound => "REPOSITORY_NOT_FOUND",
            Self::FileReadError => "FILE_READ_ERROR",
            Self::FileWriteError => "FILE_WRITE_ERROR",
            Self::DatabaseError => "DATABASE_ERROR",
            Self::InvalidCredentials => "INVALID_CREDENTIALS",
            Self::UserNotFoundAuth => "USER_NOT_FOUND_AUTH",
            Self::PasswordMismatch => "PASSWORD_MISMATCH",
            Self::TokenExpired => "TOKEN_EXPIRED",
            Self::TokenInvalid => "TOKEN_INVALID",
            Self::PermissionDenied => "PERMISSION_DENIED",
            Self::InsufficientRole => "INSUFFICIENT_ROLE",
            Self::ResourceAccessDenied => "RESOURCE_ACCESS_DENIED",
            Self::MarkdownParseError => "MARKDOWN_PARSE_ERROR",
            Self::TemplateError => "TEMPLATE_ERROR",
            Self::InvalidSyntax => "INVALID_SYNTAX",
            Self::GitNotInitialized => "GIT_NOT_INITIALIZED",
            Self::GitCloneFailed => "GIT_CLONE_FAILED",
            Self::GitPushFailed => "GIT_PUSH_FAILED",
            Self::GitPullFailed => "GIT_PULL_FAILED",
            Self::GitMergeConflict => "GIT_MERGE_CONFLICT",
            Self::InvalidInput => "INVALID_INPUT",
            Self::ValidationError => "VALIDATION_ERROR",
            Self::ConstraintViolation => "CONSTRAINT_VIOLATION",
            Self::ConnectionError => "CONNECTION_ERROR",
            Self::TimeoutError => "TIMEOUT_ERROR",
            Self::InvalidConfig => "INVALID_CONFIG",
            Self::MissingConfig => "MISSING_CONFIG",
            Self::UnexpectedError => "UNEXPECTED_ERROR",
            Self::NotImplemented => "NOT_IMPLEMENTED",
        };
        write!(f, "{}", code)
    }
}

// ============================================================================
// From implementations for standard error types
// ============================================================================

impl From<std::io::Error> for TachyonError {
    fn from(err: std::io::Error) -> Self {
        Self::storage("IO_ERROR", err.to_string()).with_source(err.kind().to_string())
    }
}

impl From<serde_json::Error> for TachyonError {
    fn from(err: serde_json::Error) -> Self {
        Self::storage("SERIALIZATION_ERROR", err.to_string()).with_source(err.to_string())
    }
}

impl From<PathError> for TachyonError {
    fn from(err: PathError) -> Self {
        Self::storage("PATH_ERROR", err.to_string())
    }
}

impl From<chrono::ParseError> for TachyonError {
    fn from(err: chrono::ParseError) -> Self {
        Self::validation("DATE_PARSE_ERROR", err.to_string())
    }
}

impl From<String> for TachyonError {
    fn from(message: String) -> Self {
        Self::validation("ERROR", message)
    }
}

impl From<&str> for TachyonError {
    fn from(message: &str) -> Self {
        Self::validation("ERROR", message.to_string())
    }
}

// ============================================================================
// Helper constructors for common error patterns
// ============================================================================

impl TachyonError {
    /// Create a document not found error
    pub fn document_not_found(id: impl fmt::Display) -> Self {
        Self::not_found(format!("Document {}", id))
    }

    /// Create a user not found error
    pub fn user_not_found(id: impl fmt::Display) -> Self {
        Self::not_found(format!("User {}", id))
    }

    /// Create a repository not found error
    pub fn repository_not_found(id: impl fmt::Display) -> Self {
        Self::not_found(format!("Repository {}", id))
    }

    /// Create an invalid credentials error
    pub fn invalid_credentials() -> Self {
        Self::authentication("INVALID_CREDENTIALS", "Invalid username or password")
    }

    /// Create a token expired error
    pub fn token_expired() -> Self {
        Self::authentication("TOKEN_EXPIRED", "Authentication token has expired")
    }

    /// Create a validation error with field and message
    pub fn field_validation(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::validation(
            "FIELD_VALIDATION",
            format!("{}: {}", field.into(), message.into()),
        )
    }

    /// Create a git operation error
    pub fn git_operation(operation: impl Into<String>, details: impl Into<String>) -> Self {
        Self::git(
            "GIT_OPERATION_FAILED",
            format!("{} failed: {}", operation.into(), details.into()),
        )
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = TachyonError::new(ErrorCategory::Storage, "TEST_ERROR", "Test error message");
        assert_eq!(error.category, ErrorCategory::Storage);
        assert_eq!(error.code, "TEST_ERROR");
        assert_eq!(error.message, "Test error message");
    }

    #[test]
    fn test_error_with_context() {
        let error = TachyonError::validation("VALIDATION_ERROR", "Invalid input")
            .with_context("Field: username");
        assert_eq!(error.context, Some("Field: username".to_string()));
    }

    #[test]
    fn test_error_with_source() {
        let error =
            TachyonError::storage("IO_ERROR", "File read failed").with_source("Permission denied");
        assert_eq!(error.source, Some("Permission denied".to_string()));
    }

    #[test]
    fn test_error_display() {
        let error = TachyonError::not_found("Document 123");
        let display = format!("{}", error);
        assert!(display.contains("Document 123"));
        assert!(display.contains("NOT_FOUND"));
    }

    #[test]
    fn test_error_category_display() {
        assert_eq!(format!("{}", ErrorCategory::Storage), "STORAGE");
        assert_eq!(format!("{}", ErrorCategory::Authentication), "AUTH");
        assert_eq!(format!("{}", ErrorCategory::Authorization), "AUTHZ");
    }

    #[test]
    fn test_error_code_display() {
        assert_eq!(
            format!("{}", ErrorCode::DocumentNotFound),
            "DOCUMENT_NOT_FOUND"
        );
        assert_eq!(
            format!("{}", ErrorCode::InvalidCredentials),
            "INVALID_CREDENTIALS"
        );
    }

    #[test]
    fn test_helper_constructors() {
        let doc_error = TachyonError::document_not_found("doc-123");
        assert_eq!(doc_error.category, ErrorCategory::Storage);
        assert!(doc_error.message.contains("Document doc-123"));

        let auth_error = TachyonError::invalid_credentials();
        assert_eq!(auth_error.category, ErrorCategory::Authentication);

        let validation_error = TachyonError::field_validation("email", "Invalid format");
        assert_eq!(validation_error.category, ErrorCategory::Validation);
    }

    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let tachyon_err: TachyonError = io_err.into();
        assert_eq!(tachyon_err.category, ErrorCategory::Storage);
        assert!(tachyon_err.message.contains("File not found"));
    }

    #[test]
    fn test_error_serialization() {
        let error = TachyonError::new(ErrorCategory::Validation, "TEST_CODE", "Test message")
            .with_context("Test context");

        let json = serde_json::to_string(&error).expect("Should serialize");
        let deserialized: TachyonError = serde_json::from_str(&json).expect("Should deserialize");

        assert_eq!(deserialized.category, error.category);
        assert_eq!(deserialized.code, error.code);
        assert_eq!(deserialized.context, error.context);
    }
}
