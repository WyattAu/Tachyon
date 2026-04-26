use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

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
    Internal(String),
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
            Self::Internal(e) => write!(f, "Internal error: {}", e),
        }
    }
}

impl std::error::Error for ServerError {}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            Self::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg.as_str()),
            Self::Validation(msg) => (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", msg.as_str()),
            Self::Auth(msg) => (StatusCode::UNAUTHORIZED, "AUTH_ERROR", msg.as_str()),
            Self::Rbac(msg) => (StatusCode::FORBIDDEN, "FORBIDDEN", msg.as_str()),
            Self::Database(msg) => (StatusCode::INTERNAL_SERVER_ERROR, "DATABASE_ERROR", msg.as_str()),
            Self::Search(msg) => (StatusCode::INTERNAL_SERVER_ERROR, "SEARCH_ERROR", msg.as_str()),
            Self::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", msg.as_str()),
        };
        let body = Json(json!({ "code": code, "message": message }));
        (status, body).into_response()
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
            crate::middleware::auth::AuthError::InsufficientPermissions => Self::Rbac(e.to_string()),
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
