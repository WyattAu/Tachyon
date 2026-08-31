// Storage Backend Trait
//
// Defines the interface for pluggable document storage backends.
// This enables offline-first mode (v0.25) where the desktop app can
// use SQLite for local storage while syncing with a remote PostgreSQL server.
//
// Implementations:
// - `PostgresStore` (v0.23+): Delegates to `tachyon-database::DocumentRepository`
// - `SqliteStore` (v0.25): Local embedded storage via `rusqlite`
// - `MemoryStore` (testing): In-memory HashMap-based storage

use crate::id::DocumentId;
use crate::id::UserId;
use crate::types::document::{
    Document, DocumentContent, DocumentMetadata, DocumentStatus, DocumentVisibility,
};
use crate::types::error::TachyonError;
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;
use std::result::Result as StdResult;

/// Result type for storage operations.
pub type StorageResult<T> = StdResult<T, StorageError>;

/// Errors that can occur during storage operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageError {
    /// The requested document was not found.
    NotFound { id: String },
    /// A conflict was detected (e.g., concurrent edit).
    Conflict {
        id: String,
        expected_version: u64,
        actual_version: u64,
    },
    /// A constraint violation occurred (e.g., duplicate slug).
    ConstraintViolation { field: String, value: String },
    /// The storage backend is unavailable.
    Unavailable { reason: String },
    /// An internal error occurred.
    Internal { message: String },
    /// Validation failed.
    Validation { field: String, message: String },
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound { id } => write!(f, "Document not found: {}", id),
            Self::Conflict {
                id,
                expected_version,
                actual_version,
            } => {
                write!(
                    f,
                    "Conflict on document {}: expected version {}, got {}",
                    id, expected_version, actual_version
                )
            }
            Self::ConstraintViolation { field, value } => {
                write!(f, "Constraint violation on {}: {}", field, value)
            }
            Self::Unavailable { reason } => write!(f, "Storage unavailable: {}", reason),
            Self::Internal { message } => write!(f, "Internal storage error: {}", message),
            Self::Validation { field, message } => {
                write!(f, "Validation error on {}: {}", field, message)
            }
        }
    }
}

impl std::error::Error for StorageError {}

impl From<StorageError> for TachyonError {
    fn from(err: StorageError) -> Self {
        match &err {
            StorageError::NotFound { id: _ } => TachyonError::not_found("document"),
            StorageError::Conflict {
                id,
                expected_version,
                actual_version,
            } => TachyonError::storage(
                "conflict",
                format!(
                    "document {}: expected version {}, got {}",
                    id, expected_version, actual_version
                ),
            ),
            StorageError::ConstraintViolation { field, value } => {
                TachyonError::field_validation(field, value)
            }
            StorageError::Unavailable { reason } => TachyonError::storage("unavailable", reason),
            StorageError::Internal { message } => TachyonError::storage("internal", message),
            StorageError::Validation { field, message } => {
                TachyonError::field_validation(field, message)
            }
        }
    }
}

/// Parameters for listing documents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListParams {
    /// Page number (1-indexed).
    pub page: usize,
    /// Number of items per page.
    pub page_size: usize,
    /// Filter by author ID.
    pub author_id: Option<UserId>,
    /// Filter by status.
    pub status: Option<DocumentStatus>,
    /// Filter by visibility.
    pub visibility: Option<DocumentVisibility>,
    /// Filter by tags (any match).
    pub tags: Vec<String>,
    /// Search query (full-text search).
    pub query: Option<String>,
    /// Sort field.
    pub sort_by: SortField,
    /// Sort direction.
    pub sort_dir: SortDirection,
}

impl Default for ListParams {
    fn default() -> Self {
        Self {
            page: 1,
            page_size: 20,
            author_id: None,
            status: None,
            visibility: None,
            tags: Vec::new(),
            query: None,
            sort_by: SortField::default(),
            sort_dir: SortDirection::default(),
        }
    }
}

/// Fields that documents can be sorted by.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum SortField {
    #[default]
    UpdatedAt,
    CreatedAt,
    Title,
}

/// Sort direction.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum SortDirection {
    #[default]
    Desc,
    Asc,
}

/// Result of a list operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResult {
    /// The returned documents.
    pub items: Vec<Document>,
    /// Total number of matching documents.
    pub total: usize,
    /// Current page number.
    pub page: usize,
    /// Page size used.
    pub page_size: usize,
}

/// Summary statistics for a document list.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DocumentListSummary {
    pub total_documents: usize,
    pub draft_count: usize,
    pub published_count: usize,
    pub archived_count: usize,
    pub total_word_count: usize,
    pub total_tags: usize,
}

/// Storage backend trait for document persistence.
///
/// This trait abstracts over the storage backend (PostgreSQL, SQLite, etc.)
/// and provides a consistent interface for document CRUD operations.
/// Implementations must be `Send + Sync` for use in async contexts.
///
/// # Design Decisions
///
/// - Uses `Pin<Box<dyn Future>>` instead of `async_trait` to avoid
///   adding a dependency on the `async-trait` crate.
/// - All methods take `&self` (not `&mut self`) because the backend
///   manages its own internal state (connection pool, etc.).
/// - Document IDs are strings to be backend-agnostic (UUIDs, slugs, etc.).
pub trait DocumentStore: Send + Sync {
    // --- Document CRUD ---

    /// Create a new document.
    fn create_document<'a>(
        &'a self,
        metadata: DocumentMetadata,
        content: DocumentContent,
    ) -> Pin<Box<dyn Future<Output = StorageResult<Document>> + Send + 'a>>;

    /// Get a document by ID.
    fn get_document<'a>(
        &'a self,
        id: &'a DocumentId,
    ) -> Pin<Box<dyn Future<Output = StorageResult<Document>> + Send + 'a>>;

    /// Update a document's content.
    fn update_document_content<'a>(
        &'a self,
        id: &'a DocumentId,
        content: DocumentContent,
        expected_version: Option<u64>,
    ) -> Pin<Box<dyn Future<Output = StorageResult<Document>> + Send + 'a>>;

    /// Update a document's metadata.
    fn update_document_metadata<'a>(
        &'a self,
        id: &'a DocumentId,
        metadata: DocumentMetadata,
        expected_version: Option<u64>,
    ) -> Pin<Box<dyn Future<Output = StorageResult<Document>> + Send + 'a>>;

    /// Delete a document (soft delete by setting status to Deleted).
    fn delete_document<'a>(
        &'a self,
        id: &'a DocumentId,
    ) -> Pin<Box<dyn Future<Output = StorageResult<()>> + Send + 'a>>;

    /// Permanently delete a document.
    fn permanently_delete_document<'a>(
        &'a self,
        id: &'a DocumentId,
    ) -> Pin<Box<dyn Future<Output = StorageResult<()>> + Send + 'a>>;

    // --- Listing & Search ---

    /// List documents with optional filters and pagination.
    fn list_documents<'a>(
        &'a self,
        params: ListParams,
    ) -> Pin<Box<dyn Future<Output = StorageResult<ListResult>> + Send + 'a>>;

    /// Full-text search across documents.
    fn search_documents<'a>(
        &'a self,
        query: &'a str,
        page: usize,
        page_size: usize,
    ) -> Pin<Box<dyn Future<Output = StorageResult<ListResult>> + Send + 'a>>;

    /// Get list summary statistics.
    fn get_list_summary<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = StorageResult<DocumentListSummary>> + Send + 'a>>;

    // --- Tags ---

    /// Get all unique tags across all documents.
    fn get_all_tags<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = StorageResult<Vec<String>>> + Send + 'a>>;

    /// Get documents by tag.
    fn get_documents_by_tag<'a>(
        &'a self,
        tag: &'a str,
        page: usize,
        page_size: usize,
    ) -> Pin<Box<dyn Future<Output = StorageResult<ListResult>> + Send + 'a>>;

    // --- Health ---

    /// Check if the storage backend is available and healthy.
    fn is_available<'a>(&'a self)
    -> Pin<Box<dyn Future<Output = StorageResult<bool>> + Send + 'a>>;
}

/// Configuration for embedded storage (used in desktop mode).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddedStorageConfig {
    /// Storage backend type.
    pub backend: StorageBackendType,
    /// Path to the database file (for SQLite).
    pub database_path: Option<String>,
    /// Connection string (for PostgreSQL).
    pub database_url: Option<String>,
    /// Maximum number of connections.
    pub max_connections: u32,
}

/// Available storage backend types.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum StorageBackendType {
    /// PostgreSQL (default for server mode).
    #[default]
    Postgres,
    /// SQLite (for embedded/offline mode).
    Sqlite,
    /// In-memory (for testing).
    Memory,
}

impl Default for EmbeddedStorageConfig {
    fn default() -> Self {
        Self {
            backend: StorageBackendType::Postgres,
            database_path: None,
            database_url: None,
            max_connections: 5,
        }
    }
}

impl EmbeddedStorageConfig {
    /// Create a SQLite embedded config with the given database path.
    pub fn sqlite(path: impl Into<String>) -> Self {
        Self {
            backend: StorageBackendType::Sqlite,
            database_path: Some(path.into()),
            database_url: None,
            max_connections: 1,
        }
    }

    /// Create a PostgreSQL config with the given connection string.
    pub fn postgres(url: impl Into<String>) -> Self {
        Self {
            backend: StorageBackendType::Postgres,
            database_path: None,
            database_url: Some(url.into()),
            max_connections: 5,
        }
    }

    /// Create an in-memory config for testing.
    pub fn memory() -> Self {
        Self {
            backend: StorageBackendType::Memory,
            database_path: None,
            database_url: None,
            max_connections: 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_error_display() {
        let err = StorageError::NotFound {
            id: "doc-123".to_string(),
        };
        assert_eq!(format!("{}", err), "Document not found: doc-123");

        let err = StorageError::Conflict {
            id: "doc-123".to_string(),
            expected_version: 2,
            actual_version: 3,
        };
        assert!(format!("{}", err).contains("expected version 2"));

        let err = StorageError::Validation {
            field: "title".to_string(),
            message: "Title cannot be empty".to_string(),
        };
        assert!(format!("{}", err).contains("title"));
    }

    #[test]
    fn test_storage_error_into_tachyon_error() {
        let err: StorageError = StorageError::NotFound {
            id: "doc-123".to_string(),
        };
        let tachyon_err: TachyonError = err.into();
        assert!(format!("{}", tachyon_err).contains("not found"));

        let err: StorageError = StorageError::Unavailable {
            reason: "connection refused".to_string(),
        };
        let tachyon_err: TachyonError = err.into();
        assert!(format!("{}", tachyon_err).contains("unavailable"));
    }

    #[test]
    fn test_embedded_storage_config() {
        let sqlite = EmbeddedStorageConfig::sqlite("/tmp/tachyon.db");
        assert_eq!(sqlite.backend, StorageBackendType::Sqlite);
        assert_eq!(sqlite.database_path, Some("/tmp/tachyon.db".to_string()));

        let postgres = EmbeddedStorageConfig::postgres("postgres://localhost/tachyon");
        assert_eq!(postgres.backend, StorageBackendType::Postgres);
        assert_eq!(
            postgres.database_url,
            Some("postgres://localhost/tachyon".to_string())
        );

        let memory = EmbeddedStorageConfig::memory();
        assert_eq!(memory.backend, StorageBackendType::Memory);
    }

    #[test]
    fn test_list_params() {
        let params = ListParams::default();
        assert_eq!(params.page, 1);
        assert_eq!(params.page_size, 20);
        assert!(params.author_id.is_none());

        let params = ListParams {
            page: 2,
            page_size: 50,
            ..Default::default()
        };
        assert_eq!(params.page, 2);
        assert_eq!(params.page_size, 50);
    }
}
