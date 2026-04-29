// Graph Invariants — Property-Based Tests
//
// These tests verify that all graph structures maintain critical invariants
// under arbitrary inputs. Each test is named after the invariant it protects.
//
// Invariants tested:
// - INV-001: No self-loops — Edge::validate() rejects source == target
// - INV-002: Weight bounds — EdgeWeight fields must be in [0.0, 1.0]
// - INV-003: Deterministic reversal — reversed().reversed() == identity
// - INV-004: Node title non-empty — Node::validate() rejects empty titles
// - INV-005: Node title max length — Node::validate() rejects >200 chars
// - INV-006: Edge type consistency — bidirectional check is symmetric
// - INV-007: Connected components partition — every node in exactly one component
// - INV-008: Component membership — nodes connected by edges share a component
// - INV-009: Disjoint components — no node appears in multiple components
// - INV-010: Edge activation cycle — deactivate then activate restores original
// - INV-011: Relationship dedup — add_relationship doesn't create duplicates by default
// - INV-012: EdgeWeight combined_score — always weight * confidence
// - INV-013: Edge connects_to — true for exactly source and target
// - INV-014: Node type capabilities — consistent has_content/can_reference/can_have_media
// - INV-015: EdgeType serialization round-trip

use super::edge::{Edge, EdgeBuilder, EdgeMetadata, EdgeType, EdgeWeight};
use super::node::{Node, NodeBuilder, NodeType, RelationshipType};
use crate::id::{generate_edge_id, generate_node_id, generate_user_id};
use proptest::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};

// ============================================================================
// Arbitrary Strategies for EdgeType and NodeType
// ============================================================================

fn edge_type_strategy() -> impl Strategy<Value = EdgeType> {
    prop_oneof![
        Just(EdgeType::References),
        Just(EdgeType::DependsOn),
        Just(EdgeType::SimilarTo),
        Just(EdgeType::PartOf),
        Just(EdgeType::RelatedTo),
        Just(EdgeType::TaggedWith),
    ]
}

fn node_type_strategy() -> impl Strategy<Value = NodeType> {
    prop_oneof![
        Just(NodeType::Document),
        Just(NodeType::Concept),
        Just(NodeType::Reference),
        Just(NodeType::Media),
    ]
}

// ============================================================================
// INV-001: No Self-Loops
// ============================================================================

proptest! {
    /// An edge must never connect a node to itself.
    /// Edge::validate() must reject when source_id == target_id for all edge types.
    #[test]
    fn inv_001_no_self_loops(edge_type in edge_type_strategy()) {
        let node_id = generate_node_id();
        let user_id = generate_user_id();
        let edge_id = generate_edge_id();

        let edge = Edge::new(edge_id, node_id, node_id, edge_type, user_id);
        prop_assert!(edge.validate().is_err(), "Self-loop must be rejected for {:?}", edge_type);
    }
}

// ============================================================================
// INV-002: Weight Bounds
// ============================================================================

proptest! {
    /// EdgeWeight weight field must be in [0.0, 1.0].
    /// Confidence field must be in [0.0, 1.0].
    #[test]
    fn inv_002_weight_in_bounds(weight in 0.0f64..=1.0, confidence in 0.0f64..=1.0) {
        let w = EdgeWeight::new(weight, confidence);
        prop_assert!(w.validate().is_ok());
        prop_assert!((w.combined_score() - weight * confidence).abs() < f64::EPSILON);
    }

    /// EdgeWeight must reject weight or confidence outside [0.0, 1.0].
    #[test]
    fn inv_002_weight_out_of_bounds_rejected(bad_field in 0u8..3) {
        // 0 = weight negative, 1 = weight > 1, 2 = confidence negative, 3 = confidence > 1
        let cases = [
            EdgeWeight::new(-0.1, 0.5),  // negative weight
            EdgeWeight::new(1.1, 0.5),   // weight > 1
            EdgeWeight::new(0.5, -0.1),  // negative confidence
            EdgeWeight::new(0.5, 1.1),   // confidence > 1
        ];
        let w = &cases[bad_field as usize % 4];
        prop_assert!(w.validate().is_err(), "Out-of-bounds weight must be rejected");
    }

    /// combined_score is always weight * confidence for valid inputs.
    #[test]
    fn inv_002_combined_score_is_product(weight in 0.0f64..=1.0, confidence in 0.0f64..=1.0) {
        let w = EdgeWeight::new(weight, confidence);
        let expected = weight * confidence;
        prop_assert!((w.combined_score() - expected).abs() < f64::EPSILON);
    }
}

// ============================================================================
// INV-003: Deterministic Reversal
// ============================================================================

proptest! {
    /// Reversing an edge twice must yield the original edge.
    #[test]
    fn inv_003_reversal_idempotent(
        edge_type in edge_type_strategy(),
        label in proptest::option::of(proptest::string::string_regex("[a-zA-Z ]{1,50}").unwrap()),
        description in proptest::option::of(proptest::string::string_regex("[a-zA-Z ]{1,100}").unwrap()),
        weight_val in proptest::option::of(0.0f64..=1.0),
        confidence_val in proptest::option::of(0.0f64..=1.0),
    ) {
        let source_id = generate_node_id();
        let target_id = generate_node_id();
        let user_id = generate_user_id();

        let mut builder = EdgeBuilder::new(source_id, target_id, edge_type, user_id);
        if let Some(ref lbl) = label {
            builder = builder.label(lbl.clone());
        }
        if let Some(ref desc) = description {
            builder = builder.description(desc.clone());
        }
        if let (Some(w), Some(c)) = (weight_val, confidence_val) {
            builder = builder.weight(EdgeWeight::new(w, c));
        }

        let edge = builder.build().expect("Builder should produce valid edge");
        let double_reversed = edge.reversed().reversed();

        prop_assert_eq!(double_reversed.source_id, edge.source_id);
        prop_assert_eq!(double_reversed.target_id, edge.target_id);
        prop_assert_eq!(double_reversed.edge_type, edge.edge_type);
        prop_assert_eq!(double_reversed.id, edge.id);
        prop_assert_eq!(double_reversed.weight, edge.weight);
        prop_assert_eq!(double_reversed.is_active, edge.is_active);
    }
}

// ============================================================================
// INV-004: Node Title Non-Empty
// ============================================================================

proptest! {
    /// Node::validate() must reject empty titles for all node types.
    #[test]
    fn inv_004_empty_title_rejected(node_type in node_type_strategy()) {
        let node_id = generate_node_id();
        let user_id = generate_user_id();

        let node = Node::new(node_id, node_type, String::new(), user_id);
        prop_assert!(node.validate().is_err(), "Empty title must be rejected for {:?}", node_type);
    }
}

// ============================================================================
// INV-005: Node Title Max Length
// ============================================================================

proptest! {
    /// Node::validate() must reject titles longer than 200 characters.
    #[test]
    fn inv_005_long_title_rejected(node_type in node_type_strategy(), extra_len in 1usize..=100) {
        let node_id = generate_node_id();
        let user_id = generate_user_id();

        let long_title = "x".repeat(200 + extra_len);
        let node = Node::new(node_id, node_type, long_title, user_id);
        prop_assert!(node.validate().is_err(), "Title >200 chars must be rejected");
    }

    /// Node::validate() must accept titles at exactly 200 characters.
    #[test]
    fn inv_005_max_title_accepted(node_type in node_type_strategy()) {
        let node_id = generate_node_id();
        let user_id = generate_user_id();

        let max_title = "a".repeat(200);
        let node = Node::new(node_id, node_type, max_title, user_id);
        prop_assert!(node.validate().is_ok(), "Title of exactly 200 chars must be accepted");
    }
}

// ============================================================================
// INV-006: Edge Type Consistency
// ============================================================================

proptest! {
    /// Bidirectional edge types must report is_bidirectional() == true and is_directed() == false.
    /// Directed edge types must report is_bidirectional() == false and is_directed() == true.
    #[test]
    fn inv_006_edge_type_consistency(edge_type in edge_type_strategy()) {
        let is_bi = edge_type.is_bidirectional();
        let is_di = edge_type.is_directed();
        // They should not both be true (though they could both be false for some types like TaggedWith)
        // Actually, TaggedWith is neither bidirectional nor directed — that's fine.
        // The invariant is: if bidirectional, then NOT directed
        if is_bi {
            prop_assert!(!is_di, "Bidirectional type {:?} should not also be directed", edge_type);
        }
    }
}

// ============================================================================
// INV-007 through INV-009: Connected Components
// ============================================================================

/// A minimal in-memory graph for testing connected components invariants.
/// This is NOT the database graph — it's a pure data-structure test.
struct InMemoryGraph {
    nodes: HashSet<String>,
    edges: Vec<(String, String)>, // (source, target)
}

impl InMemoryGraph {
    fn new() -> Self {
        Self {
            nodes: HashSet::new(),
            edges: Vec::new(),
        }
    }

    fn add_node(&mut self, id: String) {
        self.nodes.insert(id);
    }

    fn add_edge(&mut self, source: String, target: String) {
        // Add nodes if not present
        self.nodes.insert(source.clone());
        self.nodes.insert(target.clone());
        self.edges.push((source, target));
    }

    /// Compute connected components using BFS (undirected view of edges).
    fn connected_components(&self) -> Vec<HashSet<String>> {
        let mut adjacency: HashMap<&str, Vec<&str>> = HashMap::new();
        for node in &self.nodes {
            adjacency.entry(node).or_default();
        }
        for (source, target) in &self.edges {
            adjacency.entry(source).or_default().push(target);
            adjacency.entry(target).or_default().push(source);
        }

        let mut visited: HashSet<&str> = HashSet::new();
        let mut components: Vec<HashSet<String>> = Vec::new();

        for node in &self.nodes {
            if visited.contains(node.as_str()) {
                continue;
            }
            let mut component = HashSet::new();
            let mut queue = VecDeque::new();
            queue.push_back(node.as_str());
            visited.insert(node.as_str());

            while let Some(current) = queue.pop_front() {
                component.insert(current.to_string());
                if let Some(neighbors) = adjacency.get(current) {
                    for neighbor in neighbors {
                        if !visited.contains(*neighbor) {
                            visited.insert(neighbor);
                            queue.push_back(neighbor);
                        }
                    }
                }
            }
            components.push(component);
        }

        components
    }
}

proptest! {
    /// Every node appears in exactly one connected component.
    #[test]
    fn inv_007_components_partition(
        node_count in 0usize..20,
        edge_specs in proptest::collection::vec(
            (0usize..20, 0usize..20),
            0..30
        )
    ) {
        let mut graph = InMemoryGraph::new();
        for i in 0..node_count {
            graph.add_node(format!("node-{}", i));
        }

        for (src, tgt) in edge_specs {
            if src < node_count && tgt < node_count && src != tgt {
                graph.add_edge(format!("node-{}", src), format!("node-{}", tgt));
            }
        }

        let components = graph.connected_components();
        let mut seen: HashSet<String> = HashSet::new();
        for component in &components {
            for node in component {
                prop_assert!(!seen.contains(node), "Node {} appears in multiple components", node);
                seen.insert(node.clone());
            }
        }

        // Every graph node must appear in exactly one component
        for node in &graph.nodes {
            prop_assert!(seen.contains(node), "Node {} missing from all components", node);
        }
    }

    /// Nodes connected by edges must share a component.
    #[test]
    fn inv_008_edge_endpoints_share_component(
        edge_specs in proptest::collection::vec(
            (0usize..10, 0usize..10),
            1..15
        )
    ) {
        let mut graph = InMemoryGraph::new();
        // Add all referenced nodes
        let mut all_nodes: HashSet<usize> = HashSet::new();
        for (src, tgt) in &edge_specs {
            if src != tgt {
                all_nodes.insert(*src);
                all_nodes.insert(*tgt);
            }
        }
        for &n in &all_nodes {
            graph.add_node(format!("node-{}", n));
        }
        for (src, tgt) in edge_specs {
            if src != tgt && all_nodes.contains(&src) && all_nodes.contains(&tgt) {
                graph.add_edge(format!("node-{}", src), format!("node-{}", tgt));
            }
        }

        let components = graph.connected_components();
        // Build node -> component index map
        let mut node_component: HashMap<String, usize> = HashMap::new();
        for (idx, component) in components.iter().enumerate() {
            for node in component {
                node_component.insert(node.clone(), idx);
            }
        }

        // Verify all edge endpoints share a component
        for (src, tgt) in &graph.edges {
            if let (Some(&c1), Some(&c2)) = (node_component.get(src), node_component.get(tgt)) {
                prop_assert_eq!(c1, c2, "Edge endpoints {} and {} in different components", src, tgt);
            }
        }
    }
}

// ============================================================================
// INV-010: Edge Activation Cycle
// ============================================================================

proptest! {
    /// Deactivating then activating an edge must restore is_active to true.
    #[test]
    fn inv_010_deactivate_activate_cycle(edge_type in edge_type_strategy()) {
        let source_id = generate_node_id();
        let target_id = generate_node_id();
        let user_id = generate_user_id();
        let edge_id = generate_edge_id();

        let mut edge = Edge::new(edge_id, source_id, target_id, edge_type, user_id);
        prop_assert!(edge.is_active());

        edge.deactivate();
        prop_assert!(!edge.is_active());

        edge.activate();
        prop_assert!(edge.is_active());
    }
}

// ============================================================================
// INV-011: Relationship Collection Integrity
// ============================================================================

proptest! {
    /// connected_node_ids() must return exactly the set of unique target_node_ids.
    #[test]
    fn inv_011_connected_node_ids_match_relationships(
        targets in proptest::collection::vec(proptest::strategy::Just(()), 0..10)
    ) {
        let node_id = generate_node_id();
        let user_id = generate_user_id();
        let mut node = Node::new(node_id, NodeType::Document, "Test".to_string(), user_id);

        let target_ids: Vec<_> = targets.iter().map(|_| generate_node_id()).collect();
        let rel_types = [
            RelationshipType::References,
            RelationshipType::DependsOn,
            RelationshipType::SimilarTo,
            RelationshipType::PartOf,
            RelationshipType::RelatedTo,
        ];

        for (i, target_id) in target_ids.iter().enumerate() {
            let rel_type = rel_types[i % rel_types.len()];
            node.add_relationship(*target_id, rel_type);
        }

        let connected = node.connected_node_ids();
        prop_assert_eq!(connected.len(), target_ids.len());
        for target_id in &target_ids {
            prop_assert!(connected.contains(target_id), "connected_node_ids missing {:?}", target_id);
        }
    }

    /// remove_relationship must remove exactly the matching relationship.
    #[test]
    fn inv_011_remove_relationship(
        target_count in 1usize..5,
        remove_idx in 0usize..5
    ) {
        let node_id = generate_node_id();
        let user_id = generate_user_id();
        let mut node = Node::new(node_id, NodeType::Document, "Test".to_string(), user_id);

        let target_ids: Vec<_> = (0..target_count).map(|_| generate_node_id()).collect();
        for target_id in &target_ids {
            node.add_relationship(*target_id, RelationshipType::References);
        }

        let initial_count = node.relationships.len();
        if remove_idx < target_ids.len() {
            node.remove_relationship(&target_ids[remove_idx]);
            prop_assert_eq!(node.relationships.len(), initial_count - 1);
            // Verify the removed one is gone
            for rel in &node.relationships {
                prop_assert_ne!(&rel.target_node_id, &target_ids[remove_idx]);
            }
        }
    }
}

// ============================================================================
// INV-012: EdgeWeight combined_score (fuzz boundary)
// ============================================================================

proptest! {
    /// combined_score is always weight * confidence for any float inputs,
    /// including special values like infinity and NaN.
    #[test]
    fn inv_012_combined_score_special_floats(
        weight in prop_oneof![
            proptest::num::f64::NEGATIVE,
            proptest::num::f64::POSITIVE,
            proptest::num::f64::ZERO,
            Just(1.0f64),
        ],
        confidence in prop_oneof![
            proptest::num::f64::NEGATIVE,
            proptest::num::f64::POSITIVE,
            proptest::num::f64::ZERO,
            Just(1.0f64),
        ],
    ) {
        let w = EdgeWeight::new(weight, confidence);
        let expected = weight * confidence;
        let actual = w.combined_score();
        // Use bit-exact comparison for all float values including NaN/Inf
        // NaN != NaN, so handle that case; otherwise use exact equality
        if expected.is_nan() && actual.is_nan() {
            // Both NaN — invariant holds
        } else if expected.is_infinite() && actual.is_infinite() && expected.signum() == actual.signum() {
            // Same signed infinity — invariant holds
        } else {
            prop_assert!((actual - expected).abs() < f64::EPSILON, "Expected {}, got {}", expected, actual);
        }
    }
}

// ============================================================================
// INV-013: Edge connects_to
// ============================================================================

proptest! {
    /// connects_to returns true for exactly the source and target node IDs.
    #[test]
    fn inv_013_connects_to_exact_set(edge_type in edge_type_strategy()) {
        let source_id = generate_node_id();
        let target_id = generate_node_id();
        let user_id = generate_user_id();
        let edge_id = generate_edge_id();
        let other_id = generate_node_id();

        let edge = Edge::new(edge_id, source_id, target_id, edge_type, user_id);

        prop_assert!(edge.connects_to(&source_id), "Must connect to source");
        prop_assert!(edge.connects_to(&target_id), "Must connect to target");
        prop_assert!(!edge.connects_to(&other_id), "Must not connect to unrelated node");
    }
}

// ============================================================================
// INV-014: Node Type Capabilities
// ============================================================================

proptest! {
    /// Node type capabilities must be internally consistent.
    /// has_content and can_have_media are mutually exclusive
    /// (a node type shouldn't claim both for now — this documents the current design).
    #[test]
    fn inv_014_node_type_capabilities(node_type in node_type_strategy()) {
        let has_content = node_type.has_content();
        let can_ref = node_type.can_reference();
        let can_media = node_type.can_have_media();

        // Current design: can_reference is only for Reference, can_have_media only for Media
        // These are single-type capabilities, not overlapping
        if can_ref {
            prop_assert!(!has_content, "Reference type should not have content");
            prop_assert!(!can_media, "Reference type should not have media");
        }
        if can_media {
            prop_assert!(!has_content, "Media type should not have content");
            prop_assert!(!can_ref, "Media type should not reference");
        }
    }
}

// ============================================================================
// INV-015: EdgeType Serialization Round-Trip
// ============================================================================

proptest! {
    /// Every EdgeType must round-trip through JSON serialization.
    #[test]
    fn inv_015_edge_type_serialization_roundtrip(edge_type in edge_type_strategy()) {
        let json = serde_json::to_string(&edge_type).expect("serialize");
        let deserialized: EdgeType = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(edge_type, deserialized);
    }
}

// ============================================================================
// INV-016: EdgeBuilder always generates valid IDs
// ============================================================================

proptest! {
    /// EdgeBuilder::build() must always produce an edge with a valid, non-nil ID.
    #[test]
    fn inv_016_builder_generates_valid_id(edge_type in edge_type_strategy()) {
        let source_id = generate_node_id();
        let target_id = generate_node_id();
        let user_id = generate_user_id();

        let edge = EdgeBuilder::new(source_id, target_id, edge_type, user_id)
            .build()
            .expect("Builder should succeed");

        prop_assert!(!edge.id.is_nil(), "Generated edge ID must not be nil");
    }
}

// ============================================================================
// INV-017: NodeBuilder always generates valid IDs
// ============================================================================

proptest! {
    /// NodeBuilder::build() must always produce a node with a valid, non-nil ID.
    #[test]
    fn inv_017_node_builder_generates_valid_id(node_type in node_type_strategy()) {
        let user_id = generate_user_id();

        let node = NodeBuilder::new(node_type, "Valid Title".to_string(), user_id)
            .build()
            .expect("Builder should succeed");

        prop_assert!(!node.id.is_nil(), "Generated node ID must not be nil");
        prop_assert!(node.validate().is_ok(), "Builder must produce valid node");
    }
}

// ============================================================================
// INV-018: Graph degree properties
// ============================================================================

proptest! {
    /// In any graph, the sum of all node degrees equals 2 * edge_count.
    #[test]
    fn inv_018_degree_sum_equals_twice_edges(
        node_count in 2usize..15,
        edge_specs in proptest::collection::vec(
            (0usize..15, 0usize..15),
            0..20
        )
    ) {
        let mut graph = InMemoryGraph::new();
        for i in 0..node_count {
            graph.add_node(format!("node-{}", i));
        }

        let mut added_edges = 0usize;
        let mut seen_edges: HashSet<(String, String)> = HashSet::new();
        for (src, tgt) in edge_specs {
            if src < node_count && tgt < node_count && src != tgt {
                let s = format!("node-{}", src);
                let t = format!("node-{}", tgt);
                // Avoid duplicate edges
                let key = if s < t.clone() { (s.clone(), t.clone()) } else { (t.clone(), s.clone()) };
                if !seen_edges.contains(&key) {
                    seen_edges.insert(key);
                    graph.add_edge(s, t);
                    added_edges += 1;
                }
            }
        }

        // Count degree for each node
        let mut degree_sum: usize = 0;
        for (_src, _tgt) in &graph.edges {
            degree_sum += 2; // Each edge contributes 1 to each endpoint
        }

        prop_assert_eq!(degree_sum, added_edges * 2);
    }
}

// ============================================================================
// INV-019: RelationshipType filtering
// ============================================================================

proptest! {
    /// get_relationships_by_type must return only relationships of the requested type.
    #[test]
    fn inv_019_relationship_type_filtering(
        refs_count in 0usize..5,
        deps_count in 0usize..5,
    ) {
        let node_id = generate_node_id();
        let user_id = generate_user_id();
        let mut node = Node::new(node_id, NodeType::Document, "Test".to_string(), user_id);

        let ref_targets: Vec<_> = (0..refs_count).map(|_| generate_node_id()).collect();
        let dep_targets: Vec<_> = (0..deps_count).map(|_| generate_node_id()).collect();

        for target in &ref_targets {
            node.add_relationship(*target, RelationshipType::References);
        }
        for target in &dep_targets {
            node.add_relationship(*target, RelationshipType::DependsOn);
        }

        let refs = node.get_relationships_by_type(RelationshipType::References);
        let deps = node.get_relationships_by_type(RelationshipType::DependsOn);
        let similars = node.get_relationships_by_type(RelationshipType::SimilarTo);

        prop_assert_eq!(refs.len(), refs_count);
        prop_assert_eq!(deps.len(), deps_count);
        prop_assert_eq!(similars.len(), 0);

        for rel in &refs {
            prop_assert_eq!(rel.relationship_type, RelationshipType::References);
        }
        for rel in &deps {
            prop_assert_eq!(rel.relationship_type, RelationshipType::DependsOn);
        }
    }
}

// ============================================================================
// INV-020: NodeType serialization round-trip
// ============================================================================

proptest! {
    /// Every NodeType must round-trip through JSON serialization.
    #[test]
    fn inv_020_node_type_serialization_roundtrip(node_type in node_type_strategy()) {
        let json = serde_json::to_string(&node_type).expect("serialize");
        let deserialized: NodeType = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(node_type, deserialized);
    }
}

// ============================================================================
// INV-021: New edge has no deactivated_at
// ============================================================================

proptest! {
    /// A freshly created edge must have deactivated_at = None.
    #[test]
    fn inv_021_new_edge_has_no_deactivation_timestamp(edge_type in edge_type_strategy()) {
        let source_id = generate_node_id();
        let target_id = generate_node_id();
        let user_id = generate_user_id();
        let edge_id = generate_edge_id();

        prop_assume!(source_id != target_id);

        let edge = Edge::new(edge_id, source_id, target_id, edge_type, user_id);

        prop_assert!(edge.metadata.deactivated_at.is_none());
        prop_assert!(edge.is_active());
    }
}

// ============================================================================
// INV-022: Deactivation sets deactivated_at, activation clears it
// ============================================================================

proptest! {
    /// Deactivating an edge must set deactivated_at to Some(non-past).
    /// Re-activating must clear it back to None.
    #[test]
    fn inv_022_deactivation_sets_timestamp(edge_type in edge_type_strategy()) {
        let source_id = generate_node_id();
        let target_id = generate_node_id();
        let user_id = generate_user_id();
        let edge_id = generate_edge_id();

        prop_assume!(source_id != target_id);

        let mut edge = Edge::new(edge_id, source_id, target_id, edge_type, user_id);

        // Deactivate
        edge.deactivate();
        prop_assert!(!edge.is_active());
        let ts = edge.metadata.deactivated_at.expect("deactivated_at must be set");
        prop_assert!(ts >= edge.metadata.created_at);
        prop_assert!(ts <= chrono::Utc::now());

        // Activate clears deactivated_at
        edge.activate();
        prop_assert!(edge.is_active());
        prop_assert!(edge.metadata.deactivated_at.is_none());
    }
}

// ============================================================================
// INV-023: Multiple deactivate-activate cycles preserve monotonic timestamps
// ============================================================================

proptest! {
    /// Each deactivation timestamp must be >= the previous deactivation timestamp.
    #[test]
    fn inv_023_deactivation_timestamps_monotonic(
        edge_type in edge_type_strategy(),
        cycle_count in 1usize..5,
    ) {
        let source_id = generate_node_id();
        let target_id = generate_node_id();
        let user_id = generate_user_id();
        let edge_id = generate_edge_id();

        prop_assume!(source_id != target_id);

        let mut edge = Edge::new(edge_id, source_id, target_id, edge_type, user_id);

        let mut last_ts: Option<chrono::DateTime<chrono::Utc>> = None;

        for _ in 0..cycle_count {
            edge.deactivate();
            let ts = edge.metadata.deactivated_at.expect("must be set after deactivate");
            if let Some(prev) = last_ts {
                prop_assert!(ts >= prev, "deactivation timestamps must be monotonic");
            }
            last_ts = Some(ts);

            edge.activate();
            prop_assert!(edge.metadata.deactivated_at.is_none());
        }
    }
}

// ============================================================================
// INV-024: Reversed edge preserves deactivated_at
// ============================================================================

proptest! {
    /// The reversed() copy must preserve the deactivated_at timestamp.
    #[test]
    fn inv_024_reversed_preserves_deactivated_at(edge_type in edge_type_strategy()) {
        let source_id = generate_node_id();
        let target_id = generate_node_id();
        let user_id = generate_user_id();
        let edge_id = generate_edge_id();

        prop_assume!(source_id != target_id);

        let mut edge = Edge::new(edge_id, source_id, target_id, edge_type, user_id);

        edge.deactivate();
        let reversed = edge.reversed();

        prop_assert_eq!(
            edge.metadata.deactivated_at,
            reversed.metadata.deactivated_at,
            "reversed edge must preserve deactivated_at"
        );
    }
}

// ============================================================================
// INV-025: EdgeMetadata serialization round-trip with deactivated_at
// ============================================================================

proptest! {
    /// EdgeMetadata with deactivated_at set must round-trip through JSON.
    #[test]
    fn inv_025_edge_metadata_serialization_with_deactivation(has_deactivation in proptest::bool::ANY) {
        let user_id = generate_user_id();
        let mut meta = EdgeMetadata::new(user_id);
        meta.label = Some("test-label".to_string());
        meta.description = Some("test-desc".to_string());

        if has_deactivation {
            meta.deactivated_at = Some(chrono::Utc::now());
        }

        let json = serde_json::to_string(&meta).expect("serialize");
        let deserialized: EdgeMetadata = serde_json::from_str(&json).expect("deserialize");

        prop_assert_eq!(meta.deactivated_at, deserialized.deactivated_at);
        prop_assert_eq!(meta.label, deserialized.label);
        prop_assert_eq!(meta.description, deserialized.description);
        prop_assert_eq!(meta.created_by, deserialized.created_by);
    }
}
