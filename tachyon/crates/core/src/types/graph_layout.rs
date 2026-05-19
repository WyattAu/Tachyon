//! Force-directed graph layout algorithm for visualization.

use super::edge::Edge;
use super::node::Node;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphPosition {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphLayout {
    pub positions: Vec<LayoutNode>,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutNode {
    pub id: String,
    pub label: String,
    pub position: GraphPosition,
    pub group: Option<String>,
}

pub struct ForceDirectedLayout {
    pub width: f64,
    pub height: f64,
    pub iterations: usize,
    pub repulsion: f64,
    pub attraction: f64,
    pub damping: f64,
}

impl Default for ForceDirectedLayout {
    fn default() -> Self {
        Self {
            width: 800.0,
            height: 600.0,
            iterations: 50,
            repulsion: 5000.0,
            attraction: 0.01,
            damping: 0.9,
        }
    }
}

impl ForceDirectedLayout {
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            width,
            height,
            ..Default::default()
        }
    }

    pub fn compute(&self, nodes: &[Node], edges: &[Edge]) -> GraphLayout {
        let n = nodes.len();
        if n == 0 {
            return GraphLayout {
                positions: vec![],
                width: self.width,
                height: self.height,
            };
        }

        let mut positions: Vec<(f64, f64)> = nodes
            .iter()
            .enumerate()
            .map(|(i, _)| {
                let angle = (i as f64 / n as f64) * std::f64::consts::TAU;
                let r = self.width.min(self.height) * 0.3;
                (
                    self.width / 2.0 + r * angle.cos(),
                    self.height / 2.0 + r * angle.sin(),
                )
            })
            .collect();

        let mut adj: Vec<Vec<usize>> = vec![vec![]; n];
        for edge in edges {
            let src = nodes
                .iter()
                .position(|nd| nd.id == edge.source_id)
                .unwrap_or(0);
            let tgt = nodes
                .iter()
                .position(|nd| nd.id == edge.target_id)
                .unwrap_or(0);
            if src < n && tgt < n {
                adj[src].push(tgt);
                adj[tgt].push(src);
            }
        }

        let area = self.width * self.height;
        let k = (area / n as f64).sqrt();

        for _ in 0..self.iterations {
            let mut displacements: Vec<(f64, f64)> = vec![(0.0, 0.0); n];

            for i in 0..n {
                for j in (i + 1)..n {
                    let dx = positions[i].0 - positions[j].0;
                    let dy = positions[i].1 - positions[j].1;
                    let dist = (dx * dx + dy * dy).sqrt().max(1.0);
                    let force = (k * k) / dist;
                    let fx = (dx / dist) * force;
                    let fy = (dy / dist) * force;
                    displacements[i].0 += fx;
                    displacements[i].1 += fy;
                    displacements[j].0 -= fx;
                    displacements[j].1 -= fy;
                }
            }

            for i in 0..n {
                for &j in &adj[i] {
                    let dx = positions[j].0 - positions[i].0;
                    let dy = positions[j].1 - positions[i].1;
                    let dist = (dx * dx + dy * dy).sqrt().max(1.0);
                    let force = (dist * dist) / k * self.attraction;
                    let fx = (dx / dist) * force;
                    let fy = (dy / dist) * force;
                    displacements[i].0 += fx;
                    displacements[i].1 += fy;
                }
            }

            for i in 0..n {
                let dx = displacements[i].0 * self.damping;
                let dy = displacements[i].1 * self.damping;
                let dist = (dx * dx + dy * dy).sqrt().max(1.0);
                let max_dist = self.width.min(self.height) * 0.1;
                let scale = dist.min(max_dist) / dist;
                positions[i].0 += dx * scale;
                positions[i].1 += dy * scale;
                positions[i].0 = positions[i].0.max(50.0).min(self.width - 50.0);
                positions[i].1 = positions[i].1.max(50.0).min(self.height - 50.0);
            }
        }

        GraphLayout {
            positions: nodes
                .iter()
                .enumerate()
                .map(|(i, node)| LayoutNode {
                    id: node.id.as_str(),
                    label: node.metadata.title.clone(),
                    position: GraphPosition {
                        x: positions[i].0,
                        y: positions[i].1,
                    },
                    group: node.metadata.tags.first().cloned(),
                })
                .collect(),
            width: self.width,
            height: self.height,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::id::{generate_edge_id, generate_node_id, generate_user_id};
    use crate::types::edge::EdgeType;
    use crate::types::node::{NodeType, NodeVisibility};

    fn make_node(id: crate::id::NodeId, label: &str) -> Node {
        let mut node = Node::new(id, NodeType::Concept, label.to_string(), generate_user_id());
        node.visibility = NodeVisibility::Public;
        node
    }

    fn make_edge(src: crate::id::NodeId, tgt: crate::id::NodeId) -> Edge {
        Edge::new(
            generate_edge_id(),
            src,
            tgt,
            EdgeType::References,
            generate_user_id(),
        )
    }

    #[test]
    fn test_empty_graph() {
        let layout = ForceDirectedLayout::default().compute(&[], &[]);
        assert!(layout.positions.is_empty());
    }

    #[test]
    fn test_single_node() {
        let nodes = vec![make_node(generate_node_id(), "A")];
        let layout = ForceDirectedLayout::default().compute(&nodes, &[]);
        assert_eq!(layout.positions.len(), 1);
        assert_eq!(layout.positions[0].label, "A");
    }

    #[test]
    fn test_connected_nodes() {
        let a = generate_node_id();
        let b = generate_node_id();
        let c = generate_node_id();
        let nodes = vec![make_node(a, "A"), make_node(b, "B"), make_node(c, "C")];
        let edges = vec![make_edge(a, b), make_edge(b, c)];
        let layout = ForceDirectedLayout::default().compute(&nodes, &edges);
        assert_eq!(layout.positions.len(), 3);
        let dist_ab = ((layout.positions[0].position.x - layout.positions[1].position.x).powi(2)
            + (layout.positions[0].position.y - layout.positions[1].position.y).powi(2))
        .sqrt();
        let dist_ac = ((layout.positions[0].position.x - layout.positions[2].position.x).powi(2)
            + (layout.positions[0].position.y - layout.positions[2].position.y).powi(2))
        .sqrt();
        assert!(
            dist_ac > dist_ab,
            "Connected nodes (dist={}) should be closer than non-connected (dist={})",
            dist_ab,
            dist_ac
        );
    }

    #[test]
    fn test_positions_within_bounds() {
        let nodes: Vec<Node> = (0..10)
            .map(|i| make_node(generate_node_id(), &format!("N{}", i)))
            .collect();
        let layout = ForceDirectedLayout::new(1000.0, 800.0).compute(&nodes, &[]);
        for ln in &layout.positions {
            assert!(
                ln.position.x >= 50.0 && ln.position.x <= 950.0,
                "x out of bounds: {}",
                ln.position.x
            );
            assert!(
                ln.position.y >= 50.0 && ln.position.y <= 750.0,
                "y out of bounds: {}",
                ln.position.y
            );
        }
    }

    #[test]
    fn test_group_from_tags() {
        let mut node = make_node(generate_node_id(), "Tagged");
        node.metadata.add_tag("group-a".to_string());
        let layout = ForceDirectedLayout::default().compute(&[node], &[]);
        assert_eq!(layout.positions[0].group.as_deref(), Some("group-a"));
    }

    #[test]
    fn test_custom_dimensions() {
        let nodes = vec![make_node(generate_node_id(), "X")];
        let layout = ForceDirectedLayout::new(1200.0, 900.0).compute(&nodes, &[]);
        assert_eq!(layout.width, 1200.0);
        assert_eq!(layout.height, 900.0);
    }
}
