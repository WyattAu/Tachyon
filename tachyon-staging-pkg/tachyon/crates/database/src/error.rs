// Database Error Types
// Comprehensive error handling for database operations

use thiserror::Error;

/// Type alias for Result with DatabaseError
pub type DatabaseResult<T> = Result<T, DatabaseError>;

/// Database error types for Tachyon database layer
#[derive(Error, Debug)]
pub enum DatabaseError {
    /// Connection error with the database
    #[error("Database connection error: {0}")]
    ConnectionError(#[from] sqlx::Error),

    /// Migration error during schema updates
    #[error("Migration error: {0}")]
    MigrationError(String),

    /// Query execution error
    #[error("Query error: {0}")]
    QueryError(String),

    /// Record not found error
    #[error("Record not found: {entity_type} with ID {id}")]
    NotFound { entity_type: String, id: String },

    /// Duplicate record error
    #[error("Duplicate {entity_type}: {message}")]
    Duplicate {
        entity_type: String,
        message: String,
    },

    /// Constraint violation error
    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Validation error for input data
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Transaction error
    #[error("Transaction error: {0}")]
    TransactionError(String),

    /// Lock acquisition error
    #[error("Lock acquisition error: {0}")]
    LockError(String),

    /// Session expired error
    #[error("Session expired: {0}")]
    SessionExpired(String),

    /// Session not found error
    #[error("Session not found: {0}")]
    SessionNotFound(String),

    /// RBAC policy error
    #[error("RBAC policy error: {0}")]
    RbacPolicyError(String),

    /// Internal error
    #[error("Internal database error: {0}")]
    InternalError(String),
}

impl DatabaseError {
    /// Create a not found error
    ///
    /// # Arguments
    /// * `entity_type` - Type of entity (e.g., "document", "session")
    /// * `id` - Entity ID
    ///
    /// # Returns
    /// New DatabaseError::NotFound instance
    pub fn not_found(entity_type: impl Into<String>, id: impl Into<String>) -> Self {
        Self::NotFound {
            entity_type: entity_type.into(),
            id: id.into(),
        }
    }

    /// Create a duplicate error
    ///
    /// # Arguments
    /// * `entity_type` - Type of entity
    /// * `message` - Error message
    ///
    /// # Returns
    /// New DatabaseError::Duplicate instance
    pub fn duplicate(entity_type: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Duplicate {
            entity_type: entity_type.into(),
            message: message.into(),
        }
    }

    /// Create a constraint violation error
    ///
    /// # Arguments
    /// * `message` - Error message
    ///
    /// # Returns
    /// New DatabaseError::ConstraintViolation instance
    pub fn constraint_violation(message: impl Into<String>) -> Self {
        Self::ConstraintViolation(message.into())
    }

    /// Create a validation error
    ///
    /// # Arguments
    /// * `message` - Error message
    ///
    /// # Returns
    /// New DatabaseError::ValidationError instance
    pub fn validation_error(message: impl Into<String>) -> Self {
        Self::ValidationError(message.into())
    }

    /// Create a query error
    ///
    /// # Arguments
    /// * `message` - Error message
    ///
    /// # Returns
    /// New DatabaseError::QueryError instance
    pub fn query_error(message: impl Into<String>) -> Self {
        Self::QueryError(message.into())
    }

    /// Create a transaction error
    ///
    /// # Arguments
    /// * `message` - Error message
    ///
    /// # Returns
    /// New DatabaseError::TransactionError instance
    pub fn transaction_error(message: impl Into<String>) -> Self {
        Self::TransactionError(message.into())
    }

    /// Create a session expired error
    ///
    /// # Arguments
    /// * `session_id` - Session ID
    ///
    /// # Returns
    /// New DatabaseError::SessionExpired instance
    pub fn session_expired(session_id: impl Into<String>) -> Self {
        Self::SessionExpired(session_id.into())
    }

    /// Create a session not found error
    ///
    /// # Arguments
    /// * `session_id` - Session ID
    ///
    /// # Returns
    /// New DatabaseError::SessionNotFound instance
    pub fn session_not_found(session_id: impl Into<String>) -> Self {
        Self::SessionNotFound(session_id.into())
    }
}

/// Convert from serde_json error to DatabaseError
impl From<serde_json::Error> for DatabaseError {
    fn from(err: serde_json::Error) -> Self {
        Self::SerializationError(err.to_string())
    }
}

/// Convert from sqlx::migrate::MigrateError to DatabaseError
impl From<sqlx::migrate::MigrateError> for DatabaseError {
    fn from(err: sqlx::migrate::MigrateError) -> Self {
        Self::MigrationError(err.to_string())
    }
}
