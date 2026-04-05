// Search error types
// Provides comprehensive error handling for Tachyon search operations

use serde::Serialize;
use std::fmt;

/// Type alias for search results
pub type SearchResult<T> = std::result::Result<T, SearchError>;

/// Main error type for Tachyon search operations
/// Provides structured error handling with categories and detailed messages
#[derive(Debug, Clone, Serialize)]
pub struct SearchError {
    /// Error category for classification
    pub category: ErrorCategory,
    /// Machine-readable error code
    pub code: String,
    /// Human-readable error message
    pub message: String,
    /// Additional context about the error
    pub context: Option<String>,
    /// Source error if wrapping another error
    pub source: Option<String>,
}

impl SearchError {
    /// Create a new SearchError
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

    /// Create an index error
    pub fn index(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(ErrorCategory::Index, code, message)
    }

    /// Create a query error
    pub fn query(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(ErrorCategory::Query, code, message)
    }

    /// Create a ranking error
    pub fn ranking(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(ErrorCategory::Ranking, code, message)
    }

    /// Create an API error
    pub fn api(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(ErrorCategory::Api, code, message)
    }

    /// Create a document not found error
    pub fn document_not_found(id: impl fmt::Display) -> Self {
        Self::index(
            "DOCUMENT_NOT_FOUND",
            format!("Document {} not found in index", id),
        )
    }

    /// Create an index not found error
    pub fn index_not_found(name: impl fmt::Display) -> Self {
        Self::index("INDEX_NOT_FOUND", format!("Index {} not found", name))
    }

    /// Create an invalid query error
    pub fn invalid_query(details: impl Into<String>) -> Self {
        Self::query(
            "INVALID_QUERY",
            format!("Invalid query: {}", details.into()),
        )
    }

    /// Create a parse error
    pub fn parse_error(details: impl Into<String>) -> Self {
        Self::query(
            "PARSE_ERROR",
            format!("Failed to parse query: {}", details.into()),
        )
    }

    /// Create a configuration error
    pub fn configuration_error(details: impl Into<String>) -> Self {
        Self::index(
            "CONFIG_ERROR",
            format!("Configuration error: {}", details.into()),
        )
    }

    /// Create an IO error
    pub fn io_error(details: impl Into<String>) -> Self {
        Self::new(
            ErrorCategory::Io,
            "IO_ERROR",
            format!("IO error: {}", details.into()),
        )
    }

    /// Create a serialization error
    pub fn serialization_error(details: impl Into<String>) -> Self {
        Self::new(
            ErrorCategory::Index,
            "SERIALIZATION_ERROR",
            format!("Serialization error: {}", details.into()),
        )
    }

    /// Create a field validation error
    pub fn field_validation(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::query(
            "FIELD_VALIDATION_ERROR",
            format!("Field '{}': {}", field.into(), message.into()),
        )
    }
}

impl fmt::Display for SearchError {
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

impl std::error::Error for SearchError {}

/// Error categories for classification and handling
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum ErrorCategory {
    /// Index-related errors (creation, deletion, updates)
    Index,
    /// Query-related errors (parsing, execution)
    Query,
    /// Ranking-related errors (scoring, sorting)
    Ranking,
    /// API-related errors (HTTP, WebSocket)
    Api,
    /// Configuration errors
    Configuration,
    /// IO errors
    Io,
}

impl fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Index => write!(f, "INDEX"),
            Self::Query => write!(f, "QUERY"),
            Self::Ranking => write!(f, "RANKING"),
            Self::Api => write!(f, "API"),
            Self::Configuration => write!(f, "CONFIG"),
            Self::Io => write!(f, "IO"),
        }
    }
}

// ============================================================================
// From implementations for standard error types
// ============================================================================

impl From<std::io::Error> for SearchError {
    fn from(err: std::io::Error) -> Self {
        Self::io_error(err.to_string()).with_source(err.kind().to_string())
    }
}

impl From<serde_json::Error> for SearchError {
    fn from(err: serde_json::Error) -> Self {
        Self::serialization_error(err.to_string()).with_source(err.to_string())
    }
}

impl From<tantivy::TantivyError> for SearchError {
    fn from(err: tantivy::TantivyError) -> Self {
        Self::index("TANTIVY_ERROR", err.to_string()).with_source(err.to_string())
    }
}

impl From<String> for SearchError {
    fn from(message: String) -> Self {
        Self::query("ERROR", message)
    }
}

impl From<&str> for SearchError {
    fn from(message: &str) -> Self {
        Self::query("ERROR", message.to_string())
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
        let error = SearchError::new(ErrorCategory::Index, "TEST_ERROR", "Test error message");
        assert_eq!(error.category, ErrorCategory::Index);
        assert_eq!(error.code, "TEST_ERROR");
        assert_eq!(error.message, "Test error message");
    }

    #[test]
    fn test_error_with_context() {
        let error =
            SearchError::query("VALIDATION_ERROR", "Invalid input").with_context("Field: query");
        assert_eq!(error.context, Some("Field: query".to_string()));
    }

    #[test]
    fn test_error_with_source() {
        let error =
            SearchError::index("IO_ERROR", "File read failed").with_source("Permission denied");
        assert_eq!(error.source, Some("Permission denied".to_string()));
    }

    #[test]
    fn test_error_display() {
        let error = SearchError::document_not_found("doc-123");
        let display = format!("{}", error);
        assert!(display.contains("doc-123"));
        assert!(display.contains("DOCUMENT_NOT_FOUND"));
    }

    #[test]
    fn test_error_category_display() {
        assert_eq!(format!("{}", ErrorCategory::Index), "INDEX");
        assert_eq!(format!("{}", ErrorCategory::Query), "QUERY");
        assert_eq!(format!("{}", ErrorCategory::Ranking), "RANKING");
    }

    #[test]
    fn test_helper_constructors() {
        let doc_error = SearchError::document_not_found("doc-123");
        assert_eq!(doc_error.category, ErrorCategory::Index);
        assert!(doc_error.message.contains("doc-123"));

        let query_error = SearchError::invalid_query("syntax error");
        assert_eq!(query_error.category, ErrorCategory::Query);

        let config_error = SearchError::configuration_error("invalid path");
        assert_eq!(config_error.category, ErrorCategory::Index);
    }

    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let search_err: SearchError = io_err.into();
        assert_eq!(search_err.category, ErrorCategory::Io);
        assert!(search_err.message.contains("File not found"));
    }

    #[test]
    fn test_error_serialization() {
        let error = SearchError::new(ErrorCategory::Query, "TEST_CODE", "Test message")
            .with_context("Test context");

        let json = serde_json::to_string(&error).expect("Should serialize");
        assert!(json.contains("TEST_CODE"));
        assert!(json.contains("Test message"));
    }
}
