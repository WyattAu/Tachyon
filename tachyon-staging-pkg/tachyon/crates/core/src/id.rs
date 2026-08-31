// ID generation module
// Provides time-ordered unique ID generation for all entities

use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::str::FromStr;
use uuid::{Uuid, Version};

/// Type alias for Document ID
pub type DocumentId = Id;

/// Type alias for User ID
pub type UserId = Id;

/// Type alias for Session ID
pub type SessionId = Id;

/// Type alias for Repository ID
pub type RepositoryId = Id;

/// Type alias for Node ID
pub type NodeId = Id;

/// Type alias for Edge ID
pub type EdgeId = Id;

/// Type alias for Tag ID
pub type TagId = Id;

/// Generic ID wrapper for type-safe entity identification
/// Uses UUID v7 for time-ordered, sortable IDs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Id(Uuid);

impl Id {
    /// Generate a new ID using UUID v7 (time-ordered)
    /// UUID v7 provides:
    /// - Time-ordered sorting capability
    /// - Monotonic increasing values within the same millisecond
    /// - Good database indexing performance
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    /// Create an ID from a raw UUID
    ///
    /// # Arguments
    /// * `uuid` - The UUID value to wrap
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Get the underlying UUID value
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }

    /// Get the ID as a string (hyphenated format)
    pub fn as_str(&self) -> String {
        self.0.as_hyphenated().to_string()
    }

    /// Parse an ID from a string
    ///
    /// # Arguments
    /// * `s` - String representation of a UUID
    ///
    /// # Returns
    /// Result containing the parsed ID or an error
    pub fn parse_str(s: &str) -> Result<Self, IdParseError> {
        Uuid::parse_str(s)
            .map(Self)
            .map_err(|e| IdParseError::InvalidFormat(e.to_string()))
    }

    /// Validate that this ID uses UUID v7 format
    pub fn validate_v7(&self) -> bool {
        self.0.get_version() == Some(Version::SortRand)
    }

    /// Check if the ID is nil (all zeros)
    pub fn is_nil(&self) -> bool {
        self.0.is_nil()
    }
}

impl Default for Id {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for Id {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.0)
    }
}

impl FromStr for Id {
    type Err = IdParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse_str(s)
    }
}

impl From<Uuid> for Id {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<Id> for Uuid {
    fn from(id: Id) -> Self {
        id.0
    }
}

/// Error type for ID parsing failures
#[derive(Debug, Clone, PartialEq)]
pub enum IdParseError {
    /// Invalid UUID format
    InvalidFormat(String),
    /// UUID version mismatch (expected v7)
    VersionMismatch { got: Option<Version> },
}

impl std::error::Error for IdParseError {}

impl Display for IdParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::InvalidFormat(msg) => write!(f, "Invalid ID format: {}", msg),
            Self::VersionMismatch { got } => {
                write!(f, "ID version mismatch: expected v7, got {:?}", got)
            }
        }
    }
}

/// Generate a new Document ID
pub fn generate_document_id() -> DocumentId {
    Id::new()
}

/// Generate a new User ID
pub fn generate_user_id() -> UserId {
    Id::new()
}

/// Generate a new Session ID
pub fn generate_session_id() -> SessionId {
    Id::new()
}

/// Generate a new Repository ID
pub fn generate_repository_id() -> RepositoryId {
    Id::new()
}

/// Generate a new Node ID
pub fn generate_node_id() -> NodeId {
    Id::new()
}

/// Generate a new Edge ID
pub fn generate_edge_id() -> EdgeId {
    Id::new()
}

/// Generate a new Tag ID
pub fn generate_tag_id() -> TagId {
    Id::new()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_generation() {
        let id = Id::new();
        assert!(!id.is_nil());
        assert!(id.validate_v7());
    }

    #[test]
    fn test_id_parsing() {
        let id = Id::new();
        let id_str = id.as_str();
        let parsed = Id::parse_str(&id_str).expect("Should parse");
        assert_eq!(id, parsed);
    }

    #[test]
    fn test_id_from_uuid() {
        let uuid = Uuid::now_v7();
        let id = Id::from_uuid(uuid);
        assert_eq!(id.as_uuid(), uuid);
    }

    #[test]
    fn test_id_display() {
        let id = Id::new();
        let display = format!("{}", id);
        let parsed = Id::parse_str(&display).expect("Should parse");
        assert_eq!(id, parsed);
    }

    #[test]
    fn test_id_from_str() {
        let id = Id::new();
        let id_str = id.as_str();
        let parsed: Result<Id, _> = id_str.parse();
        assert!(parsed.is_ok());
        assert_eq!(id, parsed.unwrap());
    }

    #[test]
    fn test_nil_id() {
        let nil_id = Id::from_uuid(Uuid::nil());
        assert!(nil_id.is_nil());
    }

    #[test]
    fn test_type_aliases() {
        let doc_id: DocumentId = generate_document_id();
        let user_id: UserId = generate_user_id();
        let session_id: SessionId = generate_session_id();
        let repo_id: RepositoryId = generate_repository_id();
        let node_id: NodeId = generate_node_id();
        let edge_id: EdgeId = generate_edge_id();
        let tag_id: TagId = generate_tag_id();

        assert_ne!(doc_id, user_id);
        assert_ne!(session_id, repo_id);
        assert_ne!(node_id, edge_id);
        assert_ne!(tag_id, doc_id);
    }
}
