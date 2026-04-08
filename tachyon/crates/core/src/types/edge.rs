// Edge type definitions
// Represents relationships (edges) between nodes in the knowledge graph

use crate::id::EdgeId;
use crate::id::NodeId;
use crate::id::UserId;
use crate::types::error::TachyonError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ============================================================================
// Edge Type
// ============================================================================

/// Edge type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EdgeType {
    /// One node references another
    #[serde(rename = "references")]
    References,
    /// One node depends on another
    #[serde(rename = "depends_on")]
    DependsOn,
    /// Nodes are similar
    #[serde(rename = "similar_to")]
    SimilarTo,
    /// One node is part of another
    #[serde(rename = "part_of")]
    PartOf,
    /// Generic related connection
    #[serde(rename = "related_to")]
    RelatedTo,
    /// Tag association
    #[serde(rename = "tagged_with")]
    TaggedWith,
}

impl EdgeType {
    /// Check if edge type is bidirectional
    pub fn is_bidirectional(&self) -> bool {
        matches!(self, Self::RelatedTo | Self::SimilarTo)
    }

    /// Check if edge type is directed
    pub fn is_directed(&self) -> bool {
        matches!(self, Self::References | Self::DependsOn | Self::PartOf)
    }
}

// ============================================================================
// Edge Weight
// ============================================================================

/// Edge weight for scoring and ranking relationships
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EdgeWeight {
    /// Weight value (higher = stronger relationship)
    pub weight: f64,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f64,
}

impl EdgeWeight {
    /// Create a new edge weight
    ///
    /// # Arguments
    /// * `weight` - Weight value
    /// * `confidence` - Confidence score
    pub fn new(weight: f64, confidence: f64) -> Self {
        Self { weight, confidence }
    }

    /// Validate edge weight
    ///
    /// # Returns
    /// Result indicating valid weight or error
    pub fn validate(&self) -> Result<(), TachyonError> {
        if self.weight < 0.0 || self.weight > 1.0 {
            return Err(TachyonError::field_validation(
                "weight",
                "Weight must be between 0.0 and 1.0",
            ));
        }

        if self.confidence < 0.0 || self.confidence > 1.0 {
            return Err(TachyonError::field_validation(
                "confidence",
                "Confidence must be between 0.0 and 1.0",
            ));
        }

        Ok(())
    }

    /// Get combined score
    pub fn combined_score(&self) -> f64 {
        self.weight * self.confidence
    }
}

impl Default for EdgeWeight {
    fn default() -> Self {
        Self::new(0.5, 0.5)
    }
}

// ============================================================================
// Edge Metadata
// ============================================================================

/// Edge metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeMetadata {
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
    /// When the edge was deactivated (None if currently active)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deactivated_at: Option<DateTime<Utc>>,
    /// Creator user ID
    pub created_by: UserId,
    /// Edge label (optional display text)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Edge description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl EdgeMetadata {
    /// Create new edge metadata
    ///
    /// # Arguments
    /// * `created_by` - Creator's user ID
    pub fn new(created_by: UserId) -> Self {
        let now = Utc::now();
        Self {
            created_at: now,
            updated_at: now,
            deactivated_at: None,
            created_by,
            label: None,
            description: None,
        }
    }

    /// Update timestamp
    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    /// Set label
    pub fn with_label(mut self, label: String) -> Self {
        self.label = Some(label);
        self
    }

    /// Set description
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
}

// ============================================================================
// Edge
// ============================================================================

/// Knowledge graph edge (relationship between nodes)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    /// Unique edge identifier
    pub id: EdgeId,
    /// Source node ID (from)
    pub source_id: NodeId,
    /// Target node ID (to)
    pub target_id: NodeId,
    /// Edge type
    pub edge_type: EdgeType,
    /// Edge metadata
    pub metadata: EdgeMetadata,
    /// Edge weight for scoring
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<EdgeWeight>,
    /// Whether edge is active
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,
}

impl Edge {
    /// Create a new edge
    ///
    /// # Arguments
    /// * `id` - Edge ID
    /// * `source_id` - Source node ID
    /// * `target_id` - Target node ID
    /// * `edge_type` - Edge type
    /// * `created_by` - Creator's user ID
    pub fn new(
        id: EdgeId,
        source_id: NodeId,
        target_id: NodeId,
        edge_type: EdgeType,
        created_by: UserId,
    ) -> Self {
        let metadata = EdgeMetadata::new(created_by);
        Self {
            id,
            source_id,
            target_id,
            edge_type,
            metadata,
            weight: None,
            is_active: Some(true),
        }
    }

    /// Set edge weight
    pub fn with_weight(mut self, weight: EdgeWeight) -> Self {
        self.weight = Some(weight);
        self
    }

    /// Set whether edge is active
    pub fn with_active(mut self, active: bool) -> Self {
        self.is_active = Some(active);
        self
    }

    /// Get reversed edge (swap source and target)
    pub fn reversed(&self) -> Self {
        Self {
            id: self.id.clone(),
            source_id: self.target_id.clone(),
            target_id: self.source_id.clone(),
            edge_type: self.edge_type,
            metadata: EdgeMetadata {
                created_at: self.metadata.created_at,
                updated_at: self.metadata.updated_at,
                deactivated_at: self.metadata.deactivated_at,
                created_by: self.metadata.created_by,
                label: self.metadata.label.clone(),
                description: self.metadata.description.clone(),
            },
            weight: self.weight,
            is_active: self.is_active,
        }
    }

    /// Update edge weight
    pub fn update_weight(&mut self, weight: EdgeWeight) {
        weight.validate().expect("Invalid weight");
        self.weight = Some(weight);
        self.metadata.touch();
    }

    /// Deactivate edge
    pub fn deactivate(&mut self) {
        self.is_active = Some(false);
        self.metadata.deactivated_at = Some(Utc::now());
        self.metadata.touch();
    }

    /// Activate edge
    pub fn activate(&mut self) {
        self.is_active = Some(true);
        self.metadata.deactivated_at = None;
        self.metadata.touch();
    }

    /// Check if edge is active
    pub fn is_active(&self) -> bool {
        self.is_active.unwrap_or(true)
    }

    /// Validate edge
    pub fn validate(&self) -> Result<(), TachyonError> {
        if self.source_id == self.target_id {
            return Err(TachyonError::field_validation(
                "nodes",
                "Source and target nodes cannot be the same",
            ));
        }

        if let Some(ref weight) = self.weight {
            weight.validate()?;
        }

        Ok(())
    }

    /// Check if edge connects to a specific node
    pub fn connects_to(&self, node_id: &NodeId) -> bool {
        self.source_id == *node_id || self.target_id == *node_id
    }
}

// ============================================================================
// EdgeBuilder for fluent construction
// ============================================================================

/// Builder for creating Edge instances
pub struct EdgeBuilder {
    id: Option<EdgeId>,
    source_id: NodeId,
    target_id: NodeId,
    edge_type: EdgeType,
    created_by: UserId,
    label: Option<String>,
    description: Option<String>,
    weight: Option<EdgeWeight>,
    is_active: bool,
}

impl EdgeBuilder {
    /// Create a new EdgeBuilder
    ///
    /// # Arguments
    /// * `source_id` - Source node ID
    /// * `target_id` - Target node ID
    /// * `edge_type` - Edge type
    /// * `created_by` - Creator's user ID
    pub fn new(
        source_id: NodeId,
        target_id: NodeId,
        edge_type: EdgeType,
        created_by: UserId,
    ) -> Self {
        Self {
            id: None,
            source_id,
            target_id,
            edge_type,
            created_by,
            label: None,
            description: None,
            weight: None,
            is_active: true,
        }
    }

    /// Set edge ID
    pub fn id(mut self, id: EdgeId) -> Self {
        self.id = Some(id);
        self
    }

    /// Set label
    pub fn label(mut self, label: String) -> Self {
        self.label = Some(label);
        self
    }

    /// Set description
    pub fn description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// Set weight
    pub fn weight(mut self, weight: EdgeWeight) -> Self {
        self.weight = Some(weight);
        self
    }

    /// Set whether edge is active
    pub fn active(mut self, active: bool) -> Self {
        self.is_active = active;
        self
    }

    /// Build the Edge
    ///
    /// # Returns
    /// Result containing Edge or error
    pub fn build(self) -> Result<Edge, TachyonError> {
        let id = self.id.unwrap_or_else(crate::id::generate_edge_id);
        let mut edge = Edge::new(
            id.clone(),
            self.source_id,
            self.target_id,
            self.edge_type,
            self.created_by,
        );

        if let Some(weight) = self.weight {
            edge = edge.with_weight(weight);
        }

        edge = edge.with_active(self.is_active);

        if let Some(label) = self.label {
            edge.metadata = edge.metadata.with_label(label);
        }

        if let Some(description) = self.description {
            edge.metadata = edge.metadata.with_description(description);
        }

        edge.validate()?;

        Ok(edge)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edge_type_properties() {
        assert!(EdgeType::RelatedTo.is_bidirectional());
        assert!(EdgeType::SimilarTo.is_bidirectional());
        assert!(!EdgeType::References.is_bidirectional());
        assert!(EdgeType::DependsOn.is_directed());
        assert!(!EdgeType::RelatedTo.is_directed());
    }

    #[test]
    fn test_edge_weight() {
        let weight = EdgeWeight::new(0.75, 0.9);
        assert_eq!(weight.weight, 0.75);
        assert_eq!(weight.confidence, 0.9);
        assert_eq!(weight.combined_score(), 0.675);
        assert!(weight.validate().is_ok());

        let invalid_weight = EdgeWeight::new(1.5, 0.5);
        assert!(invalid_weight.validate().is_err());
    }

    #[test]
    fn test_edge_creation() {
        let edge_id = crate::id::generate_edge_id();
        let source_id = crate::id::generate_node_id();
        let target_id = crate::id::generate_node_id();
        let user_id = crate::id::generate_user_id();

        let edge = Edge::new(
            edge_id.clone(),
            source_id.clone(),
            target_id.clone(),
            EdgeType::References,
            user_id,
        );

        assert_eq!(edge.id, edge_id);
        assert_eq!(edge.source_id, source_id);
        assert_eq!(edge.target_id, target_id);
        assert_eq!(edge.edge_type, EdgeType::References);
        assert!(edge.is_active());
    }

    #[test]
    fn test_edge_validation() {
        let node_id = crate::id::generate_node_id();
        let user_id = crate::id::generate_user_id();
        let edge_id = crate::id::generate_edge_id();

        // Valid edge
        let edge = Edge::new(
            crate::id::generate_edge_id(),
            crate::id::generate_node_id(),
            node_id.clone(),
            EdgeType::RelatedTo,
            user_id,
        );
        assert!(edge.validate().is_ok());

        // Self-referencing edge
        let invalid_edge = Edge::new(
            edge_id,
            node_id.clone(),
            node_id,
            EdgeType::References,
            user_id,
        );
        assert!(invalid_edge.validate().is_err());
    }

    #[test]
    fn test_edge_reversed() {
        let edge_id = crate::id::generate_edge_id();
        let source_id = crate::id::generate_node_id();
        let target_id = crate::id::generate_node_id();
        let user_id = crate::id::generate_user_id();

        let edge = Edge::new(
            edge_id.clone(),
            source_id.clone(),
            target_id.clone(),
            EdgeType::DependsOn,
            user_id,
        );
        let reversed = edge.reversed();

        assert_eq!(reversed.source_id, target_id);
        assert_eq!(reversed.target_id, source_id);
        assert_eq!(reversed.edge_type, EdgeType::DependsOn);
    }

    #[test]
    fn test_edge_deactivation() {
        let edge_id = crate::id::generate_edge_id();
        let source_id = crate::id::generate_node_id();
        let target_id = crate::id::generate_node_id();
        let user_id = crate::id::generate_user_id();

        let mut edge = Edge::new(edge_id, source_id, target_id, EdgeType::References, user_id);
        assert!(edge.is_active());

        edge.deactivate();
        assert!(!edge.is_active());

        edge.activate();
        assert!(edge.is_active());
    }

    #[test]
    fn test_edge_builder() {
        let source_id = crate::id::generate_node_id();
        let target_id = crate::id::generate_node_id();
        let user_id = crate::id::generate_user_id();
        let weight = EdgeWeight::new(0.8, 0.7);

        let edge = EdgeBuilder::new(source_id, target_id, EdgeType::SimilarTo, user_id)
            .label("Test Edge".to_string())
            .description("Test edge description".to_string())
            .weight(weight)
            .active(true)
            .build()
            .expect("Should build edge");

        assert_eq!(edge.metadata.label, Some("Test Edge".to_string()));
        assert_eq!(
            edge.metadata.description,
            Some("Test edge description".to_string())
        );
        assert_eq!(edge.weight, Some(weight));
    }

    #[test]
    fn test_edge_connects_to() {
        let node_id = crate::id::generate_node_id();
        let source_id = crate::id::generate_node_id();
        let target_id = crate::id::generate_node_id();
        let user_id = crate::id::generate_user_id();
        let edge_id = crate::id::generate_edge_id();

        let edge = Edge::new(edge_id, source_id, target_id, EdgeType::References, user_id);

        assert!(edge.connects_to(&source_id));
        assert!(edge.connects_to(&target_id));
        assert!(!edge.connects_to(&node_id));
    }
}
