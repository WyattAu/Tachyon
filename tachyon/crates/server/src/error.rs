use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use std::collections::BTreeMap;

/// Unified error type for all server route handlers.
/// Implements From for sub-crate error types so the ? operator works seamlessly.
#[derive(Debug)]
pub enum ServerError {
    Database(String),
    Auth(String),
    Rbac(String),
    Search(String),
    NotFound(String),
    Validation(String),
    Conflict(String),
    RateLimit(String),
    Internal(String),
}

impl ServerError {
    /// Attach optional extra details to this error.
    /// Stored as a map for structured field-level errors.
    pub fn with_details(self, details: BTreeMap<String, String>) -> ServerErrorWithDetails {
        ServerErrorWithDetails {
            inner: self,
            details: Some(details),
        }
    }

    /// Attach an optional string detail (for backward compat with modules that use `details: Option<String>`).
    pub fn with_detail_string(self, detail: String) -> ServerErrorWithDetails {
        let mut map = BTreeMap::new();
        map.insert("detail".to_string(), detail);
        ServerErrorWithDetails {
            inner: self,
            details: Some(map),
        }
    }
}

/// Wrapper that adds optional structured details to a ServerError.
/// Used by modules that need field-level error information.
pub struct ServerErrorWithDetails {
    inner: ServerError,
    details: Option<BTreeMap<String, String>>,
}

impl std::fmt::Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Database(e) => write!(f, "Database error: {}", e),
            Self::Auth(e) => write!(f, "Authentication error: {}", e),
            Self::Rbac(e) => write!(f, "Authorization error: {}", e),
            Self::Search(e) => write!(f, "Search error: {}", e),
            Self::NotFound(e) => write!(f, "Not found: {}", e),
            Self::Validation(e) => write!(f, "Validation error: {}", e),
            Self::Conflict(e) => write!(f, "Conflict: {}", e),
            Self::RateLimit(e) => write!(f, "Rate limited: {}", e),
            Self::Internal(e) => write!(f, "Internal error: {}", e),
        }
    }
}

impl std::error::Error for ServerError {}

impl ServerError {
    /// Create a "not found" error for a resource.
    pub fn not_found(resource: &str, id: &str) -> Self {
        Self::NotFound(format!("{} '{}' not found", resource, id))
    }

    /// Create a validation error.
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::Validation(message.into())
    }

    /// Create a conflict error.
    pub fn conflict(message: impl Into<String>) -> Self {
        Self::Conflict(message.into())
    }

    /// Create a rate limit error.
    pub fn rate_limited(retry_after: u64) -> Self {
        Self::RateLimit(format!(
            "Too many requests. Retry after {} seconds.",
            retry_after
        ))
    }

    /// Create a database error.
    pub fn database(message: impl Into<String>) -> Self {
        Self::Database(message.into())
    }

    /// Create an internal server error.
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal(message.into())
    }

    /// Create an authentication (401) error.
    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::Auth(message.into())
    }

    /// Create a forbidden (403) error.
    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::Rbac(message.into())
    }
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let (status, code, message) = self.status_code_message();
        let body = Json(json!({ "code": code, "message": message }));
        (status, body).into_response()
    }
}

impl IntoResponse for ServerErrorWithDetails {
    fn into_response(self) -> Response {
        let (status, code, message) = self.inner.status_code_message();
        let mut body = json!({ "code": code, "message": message });
        if let Some(details) = self.details {
            body["details"] = json!(details);
        }
        (status, Json(body)).into_response()
    }
}

impl ServerError {
    fn status_code_message(&self) -> (StatusCode, &'static str, String) {
        match self {
            Self::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg.clone()),
            Self::Validation(msg) => (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", msg.clone()),
            Self::Auth(msg) => (StatusCode::UNAUTHORIZED, "AUTH_ERROR", msg.clone()),
            Self::Rbac(msg) => (StatusCode::FORBIDDEN, "FORBIDDEN", msg.clone()),
            Self::Conflict(msg) => (StatusCode::CONFLICT, "CONFLICT", msg.clone()),
            Self::RateLimit(msg) => (StatusCode::TOO_MANY_REQUESTS, "RATE_LIMITED", msg.clone()),
            Self::Database(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "DATABASE_ERROR",
                msg.clone(),
            ),
            Self::Search(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "SEARCH_ERROR",
                msg.clone(),
            ),
            Self::Internal(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                msg.clone(),
            ),
        }
    }
}

impl From<tachyon_database::DatabaseError> for ServerError {
    fn from(e: tachyon_database::DatabaseError) -> Self {
        Self::Database(e.to_string())
    }
}

impl From<std::io::Error> for ServerError {
    fn from(e: std::io::Error) -> Self {
        Self::Internal(e.to_string())
    }
}

impl From<serde_json::Error> for ServerError {
    fn from(e: serde_json::Error) -> Self {
        Self::Internal(e.to_string())
    }
}

impl From<crate::middleware::auth::AuthError> for ServerError {
    fn from(e: crate::middleware::auth::AuthError) -> Self {
        match &e {
            crate::middleware::auth::AuthError::InsufficientPermissions => {
                Self::Rbac(e.to_string())
            }
            _ => Self::Auth(e.to_string()),
        }
    }
}

impl From<tachyon_search::SearchError> for ServerError {
    fn from(e: tachyon_search::SearchError) -> Self {
        Self::Search(e.to_string())
    }
}

impl From<tachyon_renderer::RendererError> for ServerError {
    fn from(e: tachyon_renderer::RendererError) -> Self {
        Self::Internal(e.to_string())
    }
}

impl From<tachyon_rbac::RbacError> for ServerError {
    fn from(e: tachyon_rbac::RbacError) -> Self {
        Self::Rbac(e.to_string())
    }
}

impl From<tachyon_ssg::SsgError> for ServerError {
    fn from(e: tachyon_ssg::SsgError) -> Self {
        Self::Internal(e.to_string())
    }
}

impl From<tachyon_import_export::ImportExportError> for ServerError {
    fn from(e: tachyon_import_export::ImportExportError) -> Self {
        Self::Internal(e.to_string())
    }
}

impl From<tachyon_plugin_runtime::PluginRuntimeError> for ServerError {
    fn from(e: tachyon_plugin_runtime::PluginRuntimeError) -> Self {
        Self::Internal(e.to_string())
    }
}

impl From<sqlx::Error> for ServerError {
    fn from(e: sqlx::Error) -> Self {
        Self::Database(e.to_string())
    }
}
