// Document type definitions
// Represents knowledge documents in Tachyon system

use crate::id::DocumentId;
use crate::id::UserId;
use crate::types::error::TachyonError;
use crate::util::{slugify, validate_tag_name};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Document Status
// ============================================================================

/// Document status lifecycle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DocumentStatus {
    /// Document is being edited
    #[serde(rename = "draft")]
    Draft,
    /// Document is published and visible
    #[serde(rename = "published")]
    Published,
    /// Document is archived (read-only, not visible)
    #[serde(rename = "archived")]
    Archived,
    /// Document is deleted (soft delete)
    #[serde(rename = "deleted")]
    Deleted,
}

impl DocumentStatus {
    /// Check if document is in a terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Archived | Self::Deleted)
    }

    /// Check if document is editable
    pub fn is_editable(&self) -> bool {
        matches!(self, Self::Draft)
    }

    /// Get valid status transitions from current state
    pub fn valid_transitions(&self) -> Vec<DocumentStatus> {
        match self {
            Self::Draft => vec![Self::Published, Self::Archived, Self::Deleted],
            Self::Published => vec![Self::Archived, Self::Draft],
            Self::Archived => vec![Self::Draft],
            Self::Deleted => vec![],
        }
    }

    /// Check if transition to target status is valid
    pub fn can_transition_to(&self, target: DocumentStatus) -> bool {
        self.valid_transitions().contains(&target)
    }
}

// ============================================================================
// Document Visibility
// ============================================================================

/// Document visibility settings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DocumentVisibility {
    /// Document is publicly visible
    #[serde(rename = "public")]
    Public,
    /// Document is visible to authenticated users
    #[serde(rename = "private")]
    Private,
    /// Document is visible only to specific users
    #[serde(rename = "restricted")]
    Restricted,
}

// ============================================================================
// Document Content
// ============================================================================

/// Document content types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DocumentContent {
    /// Markdown content
    #[serde(rename = "markdown")]
    Markdown {
        /// Raw markdown text
        content: String,
    },
    /// Plain text content
    #[serde(rename = "text")]
    Text {
        /// Raw text content
        content: String,
    },
    /// Binary content (files)
    #[serde(rename = "binary")]
    Binary {
        /// Binary data
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<Vec<u8>>,
        /// MIME type
        #[serde(skip_serializing_if = "Option::is_none")]
        mime_type: Option<String>,
        /// Original filename
        #[serde(skip_serializing_if = "Option::is_none")]
        filename: Option<String>,
        /// File hash for deduplication
        #[serde(skip_serializing_if = "Option::is_none")]
        content_hash: Option<String>,
    },
}

impl DocumentContent {
    /// Get text content if available
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Markdown { content } => Some(content),
            Self::Text { content } => Some(content),
            Self::Binary { .. } => None,
        }
    }

    /// Get content length
    pub fn len(&self) -> usize {
        match self {
            Self::Markdown { content } => content.len(),
            Self::Text { content } => content.len(),
            Self::Binary { content, .. } => content.as_ref().map_or(0, |v| v.len()),
        }
    }

    /// Check if content is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Create new markdown content
    pub fn markdown(content: String) -> Self {
        Self::Markdown { content }
    }

    /// Create new text content
    pub fn text(content: String) -> Self {
        Self::Text { content }
    }

    /// Create new binary content
    pub fn binary(content: Vec<u8>, mime_type: String, filename: String) -> Self {
        Self::Binary {
            content: Some(content),
            mime_type: Some(mime_type),
            filename: Some(filename),
            content_hash: None,
        }
    }
}

// ============================================================================
// Document Frontmatter
// ============================================================================

/// Frontmatter metadata from markdown documents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentFrontmatter {
    /// Document title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Document description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Tags
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Custom metadata fields
    #[serde(flatten)]
    pub custom: HashMap<String, serde_json::Value>,
}

impl Default for DocumentFrontmatter {
    fn default() -> Self {
        Self {
            title: None,
            description: None,
            tags: Vec::new(),
            custom: HashMap::new(),
        }
    }
}

// ============================================================================
// Document Metadata
// ============================================================================

/// Document metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    /// Document title
    pub title: String,
    /// Document slug (URL-friendly identifier)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    /// Author user ID
    pub author_id: UserId,
    /// Document description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Tags
    pub tags: Vec<String>,
    /// Frontmatter if present
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frontmatter: Option<DocumentFrontmatter>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
    /// Published timestamp (if published)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_at: Option<DateTime<Utc>>,
}

impl DocumentMetadata {
    /// Create new document metadata
    ///
    /// # Arguments
    /// * `title` - Document title
    /// * `author_id` - Author's user ID
    pub fn new(title: String, author_id: UserId) -> Self {
        let now = Utc::now();
        let slug = slugify(&title);
        Self {
            title,
            slug: Some(slug),
            author_id,
            description: None,
            tags: Vec::new(),
            frontmatter: None,
            created_at: now,
            updated_at: now,
            published_at: None,
        }
    }

    /// Update timestamp
    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    /// Add a tag to the document
    pub fn add_tag(&mut self, tag: String) -> Result<(), TachyonError> {
        validate_tag_name(&tag)?;
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
        Ok(())
    }

    /// Remove a tag from the document
    pub fn remove_tag(&mut self, tag: &str) {
        self.tags.retain(|t| t != tag);
    }

    /// Set description
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// Set frontmatter
    pub fn with_frontmatter(mut self, frontmatter: DocumentFrontmatter) -> Self {
        self.frontmatter = Some(frontmatter);
        self
    }
}

// ============================================================================
// Document Statistics
// ============================================================================

/// Document usage statistics
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct DocumentStats {
    /// Word count
    pub word_count: usize,
    /// Character count
    pub character_count: usize,
    /// Read count
    pub read_count: usize,
    /// Edit count
    pub edit_count: usize,
}

impl DocumentStats {
    /// Calculate stats from content
    pub fn from_content(content: &str) -> Self {
        let words = content.split_whitespace().count();
        let chars = content.chars().count();
        Self {
            word_count: words,
            character_count: chars,
            read_count: 0,
            edit_count: 1,
        }
    }

    /// Increment read count
    pub fn increment_read(&mut self) {
        self.read_count += 1;
    }

    /// Increment edit count
    pub fn increment_edit(&mut self) {
        self.edit_count += 1;
    }
}

// ============================================================================
// Document
// ============================================================================

/// Main document entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    /// Unique document identifier
    pub id: DocumentId,
    /// Document metadata
    pub metadata: DocumentMetadata,
    /// Document content
    pub content: DocumentContent,
    /// Document visibility
    pub visibility: DocumentVisibility,
    /// Document status
    pub status: DocumentStatus,
    /// Document statistics
    pub stats: DocumentStats,
    /// Repository ID (if in a repository)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository_id: Option<crate::id::RepositoryId>,
}

impl Document {
    /// Create a new document
    ///
    /// # Arguments
    /// * `id` - Document ID
    /// * `title` - Document title
    /// * `author_id` - Author's user ID
    /// * `content` - Document content
    pub fn new(id: DocumentId, title: String, author_id: UserId, content: DocumentContent) -> Self {
        let metadata = DocumentMetadata::new(title, author_id);
        let stats = match &content {
            DocumentContent::Markdown { content } | DocumentContent::Text { content } => {
                DocumentStats::from_content(content)
            }
            DocumentContent::Binary { .. } => DocumentStats::default(),
        };

        Self {
            id,
            metadata,
            content,
            visibility: DocumentVisibility::Private,
            status: DocumentStatus::Draft,
            stats,
            repository_id: None,
        }
    }

    /// Set document visibility
    pub fn with_visibility(mut self, visibility: DocumentVisibility) -> Self {
        self.visibility = visibility;
        self
    }

    /// Set repository ID
    pub fn with_repository_id(mut self, repo_id: crate::id::RepositoryId) -> Self {
        self.repository_id = Some(repo_id);
        self
    }

    /// Update document content
    pub fn update_content(&mut self, content: DocumentContent) {
        self.content = content;
        self.metadata.touch();
        self.stats.increment_edit();
    }

    /// Publish document
    pub fn publish(&mut self) -> Result<(), TachyonError> {
        if !self.status.can_transition_to(DocumentStatus::Published) {
            return Err(TachyonError::field_validation(
                "status",
                format!("Cannot transition from {:?} to Published", self.status),
            ));
        }
        self.status = DocumentStatus::Published;
        self.metadata.published_at = Some(Utc::now());
        self.metadata.touch();
        Ok(())
    }

    /// Archive document
    pub fn archive(&mut self) -> Result<(), TachyonError> {
        if !self.status.can_transition_to(DocumentStatus::Archived) {
            return Err(TachyonError::field_validation(
                "status",
                format!("Cannot transition from {:?} to Archived", self.status),
            ));
        }
        self.status = DocumentStatus::Archived;
        self.metadata.touch();
        Ok(())
    }

    /// Delete document
    pub fn delete(&mut self) -> Result<(), TachyonError> {
        if !self.status.can_transition_to(DocumentStatus::Deleted) {
            return Err(TachyonError::field_validation(
                "status",
                format!("Cannot transition from {:?} to Deleted", self.status),
            ));
        }
        self.status = DocumentStatus::Deleted;
        self.metadata.touch();
        Ok(())
    }

    /// Restore document (from archived/deleted)
    pub fn restore(&mut self) -> Result<(), TachyonError> {
        if !self.status.can_transition_to(DocumentStatus::Draft) {
            return Err(TachyonError::field_validation(
                "status",
                format!("Cannot transition from {:?} to Draft", self.status),
            ));
        }
        self.status = DocumentStatus::Draft;
        self.metadata.touch();
        Ok(())
    }

    /// Check if document can be edited
    pub fn can_edit(&self) -> bool {
        self.status.is_editable()
    }

    /// Validate document
    pub fn validate(&self) -> Result<(), TachyonError> {
        if self.metadata.title.is_empty() {
            return Err(TachyonError::field_validation(
                "title",
                "Title cannot be empty",
            ));
        }

        if self.metadata.title.len() > 200 {
            return Err(TachyonError::field_validation(
                "title",
                "Title too long (max 200 characters)",
            ));
        }

        if self.content.is_empty() {
            // Empty content is allowed for new documents; they start blank
        }

        for tag in &self.metadata.tags {
            validate_tag_name(tag)?;
        }

        Ok(())
    }

    /// Increment read count
    pub fn record_read(&mut self) {
        self.stats.increment_read();
    }
}

// ============================================================================
// DocumentBuilder for fluent construction
// ============================================================================

/// Builder for creating Document instances
pub struct DocumentBuilder {
    id: Option<DocumentId>,
    title: String,
    author_id: UserId,
    content: Option<DocumentContent>,
    visibility: DocumentVisibility,
    tags: Vec<String>,
    description: Option<String>,
    repository_id: Option<crate::id::RepositoryId>,
}

impl DocumentBuilder {
    /// Create a new DocumentBuilder
    ///
    /// # Arguments
    /// * `title` - Document title
    /// * `author_id` - Author's user ID
    pub fn new(title: String, author_id: UserId) -> Self {
        Self {
            id: None,
            title,
            author_id,
            content: None,
            visibility: DocumentVisibility::Private,
            tags: Vec::new(),
            description: None,
            repository_id: None,
        }
    }

    /// Set document ID
    pub fn id(mut self, id: DocumentId) -> Self {
        self.id = Some(id);
        self
    }

    /// Set document content
    pub fn content(mut self, content: DocumentContent) -> Self {
        self.content = Some(content);
        self
    }

    /// Set document visibility
    pub fn visibility(mut self, visibility: DocumentVisibility) -> Self {
        self.visibility = visibility;
        self
    }

    /// Add a tag
    pub fn tag(mut self, tag: String) -> Self {
        self.tags.push(tag);
        self
    }

    /// Set tags
    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Set description
    pub fn description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// Set repository ID
    pub fn repository_id(mut self, repo_id: crate::id::RepositoryId) -> Self {
        self.repository_id = Some(repo_id);
        self
    }

    /// Build the Document
    ///
    /// # Returns
    /// Result containing Document or error
    pub fn build(self) -> Result<Document, TachyonError> {
        let id = self.id.unwrap_or_else(crate::id::generate_document_id);
        let content = self
            .content
            .unwrap_or_else(|| DocumentContent::text(String::new()));

        let mut doc = Document::new(id, self.title, self.author_id, content);

        doc.visibility = self.visibility;
        doc.metadata.tags = self.tags;
        doc.metadata.description = self.description;
        doc.repository_id = self.repository_id;

        doc.validate()?;

        Ok(doc)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_status_transitions() {
        assert!(DocumentStatus::Draft.can_transition_to(DocumentStatus::Published));
        assert!(DocumentStatus::Published.can_transition_to(DocumentStatus::Archived));
        assert!(!DocumentStatus::Deleted.can_transition_to(DocumentStatus::Published));
    }

    #[test]
    fn test_document_creation() {
        let doc_id = crate::id::generate_document_id();
        let user_id = crate::id::generate_user_id();
        let content = DocumentContent::markdown("# Test Document".to_string());

        let doc = Document::new(
            doc_id.clone(),
            "Test Document".to_string(),
            user_id,
            content,
        );

        assert_eq!(doc.id, doc_id);
        assert_eq!(doc.metadata.title, "Test Document");
        assert_eq!(doc.status, DocumentStatus::Draft);
        assert!(doc.can_edit());
    }

    #[test]
    fn test_document_publish() {
        let doc_id = crate::id::generate_document_id();
        let user_id = crate::id::generate_user_id();
        let content = DocumentContent::markdown("# Test".to_string());

        let mut doc = Document::new(doc_id, "Test".to_string(), user_id, content);
        assert!(doc.publish().is_ok());
        assert_eq!(doc.status, DocumentStatus::Published);
        assert!(!doc.can_edit());
    }

    #[test]
    fn test_document_archive() {
        let doc_id = crate::id::generate_document_id();
        let user_id = crate::id::generate_user_id();
        let content = DocumentContent::markdown("# Test".to_string());

        let mut doc = Document::new(doc_id, "Test".to_string(), user_id, content);
        doc.publish().unwrap();
        assert!(doc.archive().is_ok());
        assert_eq!(doc.status, DocumentStatus::Archived);
    }

    #[test]
    fn test_document_validation() {
        let doc_id = crate::id::generate_document_id();
        let user_id = crate::id::generate_user_id();

        // Valid document
        let doc = Document::new(
            doc_id.clone(),
            "Valid Title".to_string(),
            user_id.clone(),
            DocumentContent::markdown("Content".to_string()),
        );
        assert!(doc.validate().is_ok());

        // Empty title
        let invalid_doc = Document::new(
            crate::id::generate_document_id(),
            "".to_string(),
            user_id.clone(),
            DocumentContent::markdown("Content".to_string()),
        );
        assert!(invalid_doc.validate().is_err());

        // Empty content
        let invalid_doc = Document::new(
            crate::id::generate_document_id(),
            "Title".to_string(),
            user_id,
            DocumentContent::markdown("".to_string()),
        );
        assert!(invalid_doc.validate().is_err());
    }

    #[test]
    fn test_document_builder() {
        let user_id = crate::id::generate_user_id();
        let repo_id = crate::id::generate_repository_id();

        let doc = DocumentBuilder::new("Test Document".to_string(), user_id)
            .content(DocumentContent::markdown("Content".to_string()))
            .visibility(DocumentVisibility::Public)
            .tag("rust".to_string())
            .tag("tachyon".to_string())
            .repository_id(repo_id)
            .build()
            .expect("Should build document");

        assert_eq!(doc.metadata.title, "Test Document");
        assert_eq!(doc.visibility, DocumentVisibility::Public);
        assert_eq!(doc.metadata.tags.len(), 2);
        assert_eq!(doc.repository_id, Some(repo_id));
    }

    #[test]
    fn test_document_content() {
        let markdown = DocumentContent::markdown("# Test".to_string());
        assert_eq!(markdown.as_text(), Some("# Test"));
        assert_eq!(markdown.len(), 6);

        let text = DocumentContent::text("Plain text".to_string());
        assert_eq!(text.as_text(), Some("Plain text"));

        let binary = DocumentContent::binary(
            vec![1, 2, 3],
            "application/octet-stream".to_string(),
            "file.bin".to_string(),
        );
        assert!(binary.as_text().is_none());
        assert_eq!(binary.len(), 3);
    }

    #[test]
    fn test_document_stats() {
        let content = "Hello world";
        let stats = DocumentStats::from_content(content);
        assert_eq!(stats.word_count, 2);
        assert_eq!(stats.character_count, 11);
        assert_eq!(stats.read_count, 0);
        assert_eq!(stats.edit_count, 1);

        let mut stats = stats;
        stats.increment_read();
        assert_eq!(stats.read_count, 1);
    }
}
