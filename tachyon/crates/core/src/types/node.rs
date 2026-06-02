// Node type definitions
// Represents nodes in Tachyon knowledge graph

use crate::id::DocumentId;
use crate::id::NodeId;
use crate::id::UserId;
use crate::types::error::TachyonError;
use crate::util::slugify;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

// ============================================================================
// Node Type
// ============================================================================

/// Node type classification in the knowledge graph
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeType {
    /// Document node
    #[serde(rename = "document")]
    Document,
    /// Concept node (idea, topic, etc.)
    #[serde(rename = "concept")]
    Concept,
    /// Reference node (external link)
    #[serde(rename = "reference")]
    Reference,
    /// Media node (image, video, etc.)
    #[serde(rename = "media")]
    Media,
}

impl NodeType {
    /// Check if node type can have content
    pub fn has_content(&self) -> bool {
        matches!(self, Self::Document | Self::Concept)
    }

    /// Check if node type can reference external resources
    pub fn can_reference(&self) -> bool {
        matches!(self, Self::Reference)
    }

    /// Check if node type can contain binary data
    pub fn can_have_media(&self) -> bool {
        matches!(self, Self::Media)
    }
}

// ============================================================================
// Node Visibility
// ============================================================================

/// Node visibility settings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeVisibility {
    /// Node is publicly visible
    #[serde(rename = "public")]
    Public,
    /// Node is visible to authenticated users
    #[serde(rename = "private")]
    Private,
    /// Node is visible only to specific users
    #[serde(rename = "restricted")]
    Restricted,
}

// ============================================================================
// Node Metadata
// ============================================================================

/// Node metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetadata {
    /// Node title
    pub title: String,
    /// Node slug (URL-friendly identifier)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    /// Node description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Creator user ID
    pub created_by: UserId,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
    /// Parent node ID (if nested)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<NodeId>,
    /// Associated document ID (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_id: Option<DocumentId>,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// Custom metadata fields
    #[serde(skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub custom_metadata: std::collections::BTreeMap<String, serde_json::Value>,
}

impl NodeMetadata {
    /// Create new node metadata
    ///
    /// # Arguments
    /// * `title` - Node title
    /// * `created_by` - Creator's user ID
    pub fn new(title: String, created_by: UserId) -> Self {
        let now = Utc::now();
        let slug = slugify(&title);
        Self {
            title,
            slug: Some(slug),
            description: None,
            created_by,
            created_at: now,
            updated_at: now,
            parent_id: None,
            document_id: None,
            tags: Vec::new(),
            custom_metadata: std::collections::BTreeMap::new(),
        }
    }

    /// Update timestamp
    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    /// Add a tag to the node
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    /// Remove a tag from the node
    pub fn remove_tag(&mut self, tag: &str) {
        self.tags.retain(|t| t != tag);
    }

    /// Set parent node ID
    pub fn with_parent_id(mut self, parent_id: NodeId) -> Self {
        self.parent_id = Some(parent_id);
        self
    }

    /// Set document ID
    pub fn with_document_id(mut self, document_id: DocumentId) -> Self {
        self.document_id = Some(document_id);
        self
    }

    /// Set description
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
}

// ============================================================================
// Node Relationship
// ============================================================================

/// Relationship between two nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeRelationship {
    /// Target node ID
    pub target_node_id: NodeId,
    /// Relationship type
    pub relationship_type: RelationshipType,
    /// Relationship metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<String>,
}

/// Type of relationship between nodes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RelationshipType {
    /// References another node
    #[serde(rename = "references")]
    References,
    /// Depends on another node
    #[serde(rename = "depends_on")]
    DependsOn,
    /// Similar to another node
    #[serde(rename = "similar_to")]
    SimilarTo,
    /// Part of another node
    #[serde(rename = "part_of")]
    PartOf,
    /// Related to another node
    #[serde(rename = "related_to")]
    RelatedTo,
}

// ============================================================================
// Node
// ============================================================================

/// Knowledge graph node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    /// Unique node identifier
    pub id: NodeId,
    /// Node type
    pub node_type: NodeType,
    /// Node metadata
    pub metadata: NodeMetadata,
    /// Node visibility
    pub visibility: NodeVisibility,
    /// Node content (text or HTML)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// Node relationships (edges)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub relationships: Vec<NodeRelationship>,
    /// Node weight (for ranking/scoring)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<f64>,
}

impl Node {
    /// Create a new node
    ///
    /// # Arguments
    /// * `id` - Node ID
    /// * `node_type` - Node type
    /// * `title` - Node title
    /// * `created_by` - Creator's user ID
    pub fn new(id: NodeId, node_type: NodeType, title: String, created_by: UserId) -> Self {
        let metadata = NodeMetadata::new(title, created_by);
        Self {
            id,
            node_type,
            metadata,
            visibility: NodeVisibility::Private,
            content: None,
            relationships: Vec::new(),
            weight: None,
        }
    }

    /// Set node content
    pub fn with_content(mut self, content: String) -> Self {
        self.content = Some(content);
        self
    }

    /// Set node visibility
    pub fn with_visibility(mut self, visibility: NodeVisibility) -> Self {
        self.visibility = visibility;
        self
    }

    /// Set node weight
    pub fn with_weight(mut self, weight: f64) -> Self {
        self.weight = Some(weight);
        self
    }

    /// Add a relationship
    pub fn add_relationship(
        &mut self,
        target_node_id: NodeId,
        relationship_type: RelationshipType,
    ) {
        let relationship = NodeRelationship {
            target_node_id,
            relationship_type,
            metadata: None,
        };
        self.relationships.push(relationship);
        self.metadata.touch();
    }

    /// Remove a relationship
    pub fn remove_relationship(&mut self, target_node_id: &NodeId) {
        self.relationships
            .retain(|r| &r.target_node_id != target_node_id);
        self.metadata.touch();
    }

    /// Get relationships of a specific type
    pub fn get_relationships_by_type(
        &self,
        relationship_type: RelationshipType,
    ) -> Vec<&NodeRelationship> {
        self.relationships
            .iter()
            .filter(|r| r.relationship_type == relationship_type)
            .collect()
    }

    /// Get all connected node IDs
    pub fn connected_node_ids(&self) -> HashSet<NodeId> {
        self.relationships
            .iter()
            .map(|r| r.target_node_id)
            .collect()
    }

    /// Update node content
    pub fn update_content(&mut self, content: String) {
        self.content = Some(content);
        self.metadata.touch();
    }

    /// Validate node
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

        if let Some(ref content) = self.content
            && content.is_empty() && self.node_type.has_content() {
                return Err(TachyonError::field_validation(
                    "content",
                    "Content cannot be empty for this node type",
                ));
            }

        Ok(())
    }

    /// Check if node can be edited
    pub fn can_edit(&self) -> bool {
        self.node_type.has_content()
    }
}

// ============================================================================
// NodeBuilder for fluent construction
// ============================================================================

/// Builder for creating Node instances
pub struct NodeBuilder {
    id: Option<NodeId>,
    node_type: NodeType,
    title: String,
    created_by: UserId,
    content: Option<String>,
    visibility: NodeVisibility,
    parent_id: Option<NodeId>,
    document_id: Option<DocumentId>,
    tags: Vec<String>,
    description: Option<String>,
    weight: Option<f64>,
}

impl NodeBuilder {
    /// Create a new NodeBuilder
    ///
    /// # Arguments
    /// * `node_type` - Node type
    /// * `title` - Node title
    /// * `created_by` - Creator's user ID
    pub fn new(node_type: NodeType, title: String, created_by: UserId) -> Self {
        Self {
            id: None,
            node_type,
            title,
            created_by,
            content: None,
            visibility: NodeVisibility::Private,
            parent_id: None,
            document_id: None,
            tags: Vec::new(),
            description: None,
            weight: None,
        }
    }

    /// Set node ID
    pub fn id(mut self, id: NodeId) -> Self {
        self.id = Some(id);
        self
    }

    /// Set node content
    pub fn content(mut self, content: String) -> Self {
        self.content = Some(content);
        self
    }

    /// Set node visibility
    pub fn visibility(mut self, visibility: NodeVisibility) -> Self {
        self.visibility = visibility;
        self
    }

    /// Set parent node ID
    pub fn parent_id(mut self, parent_id: NodeId) -> Self {
        self.parent_id = Some(parent_id);
        self
    }

    /// Set document ID
    pub fn document_id(mut self, document_id: DocumentId) -> Self {
        self.document_id = Some(document_id);
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

    /// Set weight
    pub fn weight(mut self, weight: f64) -> Self {
        self.weight = Some(weight);
        self
    }

    /// Build the Node
    ///
    /// # Returns
    /// Result containing Node or error
    pub fn build(self) -> Result<Node, TachyonError> {
        let id = self.id.unwrap_or_else(crate::id::generate_node_id);
        let mut node = Node::new(id, self.node_type, self.title, self.created_by);

        if let Some(content) = self.content {
            node = node.with_content(content);
        }

        node = node.with_visibility(self.visibility);

        if let Some(weight) = self.weight {
            node = node.with_weight(weight);
        }

        if let Some(parent_id) = self.parent_id {
            node.metadata = node.metadata.with_parent_id(parent_id);
        }

        if let Some(document_id) = self.document_id {
            node.metadata = node.metadata.with_document_id(document_id);
        }

        if !self.tags.is_empty() {
            for tag in self.tags {
                node.metadata.add_tag(tag);
            }
        }

        if let Some(description) = self.description {
            node.metadata = node.metadata.with_description(description);
        }

        node.validate()?;

        Ok(node)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_creation() {
        let node_id = crate::id::generate_node_id();
        let user_id = crate::id::generate_user_id();

        let node = Node::new(
            node_id,
            NodeType::Concept,
            "Test Concept".to_string(),
            user_id,
        );

        assert_eq!(node.id, node_id);
        assert_eq!(node.node_type, NodeType::Concept);
        assert_eq!(node.metadata.title, "Test Concept");
        assert!(node.can_edit());
    }

    #[test]
    fn test_node_relationships() {
        let node_id = crate::id::generate_node_id();
        let target_id = crate::id::generate_node_id();
        let user_id = crate::id::generate_user_id();

        let mut node = Node::new(node_id, NodeType::Document, "Test".to_string(), user_id);
        node.add_relationship(target_id, RelationshipType::References);

        assert_eq!(node.relationships.len(), 1);
        assert_eq!(node.relationships[0].target_node_id, target_id);
        assert!(node.connected_node_ids().contains(&target_id));
    }

    #[test]
    fn test_node_validation() {
        let node_id = crate::id::generate_node_id();
        let user_id = crate::id::generate_user_id();

        // Valid node
        let node = Node::new(
            node_id,
            NodeType::Concept,
            "Valid Title".to_string(),
            user_id,
        )
        .with_content("Content".to_string());
        assert!(node.validate().is_ok());

        // Empty title
        let invalid_node = Node::new(
            crate::id::generate_node_id(),
            NodeType::Concept,
            "".to_string(),
            user_id,
        );
        assert!(invalid_node.validate().is_err());
    }

    #[test]
    fn test_node_builder() {
        let user_id = crate::id::generate_user_id();
        let parent_id = crate::id::generate_node_id();
        let doc_id = crate::id::generate_document_id();

        let node = NodeBuilder::new(NodeType::Document, "Test Node".to_string(), user_id)
            .content("Node content".to_string())
            .visibility(NodeVisibility::Public)
            .parent_id(parent_id)
            .document_id(doc_id)
            .tag("test".to_string())
            .description("Test description".to_string())
            .weight(0.5)
            .build()
            .expect("Should build node");

        assert_eq!(node.metadata.title, "Test Node");
        assert_eq!(node.visibility, NodeVisibility::Public);
        assert_eq!(node.metadata.parent_id, Some(parent_id));
        assert_eq!(node.metadata.document_id, Some(doc_id));
        assert_eq!(node.metadata.tags.len(), 1);
        assert_eq!(node.weight, Some(0.5));
    }

    #[test]
    fn test_node_type_properties() {
        assert!(NodeType::Document.has_content());
        assert!(NodeType::Concept.has_content());
        assert!(!NodeType::Reference.has_content());
        assert!(NodeType::Reference.can_reference());
        assert!(!NodeType::Document.can_reference());
        assert!(NodeType::Media.can_have_media());
        assert!(!NodeType::Concept.can_have_media());
    }

    #[test]
    fn test_relationships_by_type() {
        let node_id = crate::id::generate_node_id();
        let target1 = crate::id::generate_node_id();
        let target2 = crate::id::generate_node_id();
        let user_id = crate::id::generate_user_id();

        let mut node = Node::new(node_id, NodeType::Document, "Test".to_string(), user_id);
        node.add_relationship(target1, RelationshipType::References);
        node.add_relationship(target2, RelationshipType::DependsOn);

        let refs = node.get_relationships_by_type(RelationshipType::References);
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].relationship_type, RelationshipType::References);

        let deps = node.get_relationships_by_type(RelationshipType::DependsOn);
        assert_eq!(deps.len(), 1);
    }
}
