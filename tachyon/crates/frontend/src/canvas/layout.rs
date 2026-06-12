use crate::canvas::{CanvasEdge, CanvasNode, Position};

/// Layout algorithm type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LayoutAlgorithm {
    ForceDirected,
    Hierarchical,
    Radial,
}

impl Default for LayoutAlgorithm {
    fn default() -> Self {
        Self::ForceDirected
    }
}

/// Apply the specified layout algorithm to the given nodes and edges.
/// Returns new positions for each node (matched by index).
pub fn auto_layout(nodes: &mut [CanvasNode], edges: &[CanvasEdge], algorithm: LayoutAlgorithm) {
    match algorithm {
        LayoutAlgorithm::ForceDirected => force_directed_layout(nodes, edges),
        LayoutAlgorithm::Hierarchical => hierarchical_layout(nodes, edges),
        LayoutAlgorithm::Radial => radial_layout(nodes, edges),
    }
}

/// Force-directed layout using simple simulation
fn force_directed_layout(nodes: &mut [CanvasNode], edges: &[CanvasEdge]) {
    if nodes.is_empty() {
        return;
    }

    let n = nodes.len();
    let repulsion = 8000.0;
    let attraction = 0.005;
    let center_gravity = 0.01;
    let damping = 0.9;
    let iterations = 200;

    let mut vx = vec![0.0f64; n];
    let mut vy = vec![0.0f64; n];

    // Build adjacency for quick lookup
    let id_to_idx: std::collections::HashMap<&str, usize> = nodes
        .iter()
        .enumerate()
        .map(|(i, node)| (node.id.as_str(), i))
        .collect();

    let edge_pairs: Vec<(usize, usize)> = edges
        .iter()
        .filter_map(|e| {
            let s = id_to_idx.get(e.source_id.as_str())?;
            let t = id_to_idx.get(e.target_id.as_str())?;
            Some((*s, *t))
        })
        .collect();

    let center_x = 400.0;
    let center_y = 300.0;

    for _ in 0..iterations {
        let mut fx = vec![0.0f64; n];
        let mut fy = vec![0.0f64; n];

        // Repulsion between all pairs
        for i in 0..n {
            for j in (i + 1)..n {
                let dx = nodes[i].position.x - nodes[j].position.x;
                let dy = nodes[i].position.y - nodes[j].position.y;
                let dist_sq = dx * dx + dy * dy;
                let dist = dist_sq.sqrt().max(1.0);
                let force = (repulsion / dist_sq).min(200.0);
                let ux = dx / dist;
                let uy = dy / dist;
                fx[i] += force * ux;
                fy[i] += force * uy;
                fx[j] -= force * ux;
                fy[j] -= force * uy;
            }
        }

        // Attraction along edges
        for &(s, t) in &edge_pairs {
            let dx = nodes[t].position.x - nodes[s].position.x;
            let dy = nodes[t].position.y - nodes[s].position.y;
            let dist = (dx * dx + dy * dy).sqrt().max(1.0);
            let force = attraction * dist;
            let ux = dx / dist;
            let uy = dy / dist;
            fx[s] += force * ux;
            fy[s] += force * uy;
            fx[t] -= force * ux;
            fy[t] -= force * uy;
        }

        // Center gravity
        for i in 0..n {
            fx[i] += (center_x - nodes[i].position.x) * center_gravity;
            fy[i] += (center_y - nodes[i].position.y) * center_gravity;
        }

        // Apply forces
        for i in 0..n {
            vx[i] = (vx[i] + fx[i]) * damping;
            vy[i] = (vy[i] + fy[i]) * damping;
            nodes[i].position.x += vx[i];
            nodes[i].position.y += vy[i];
        }
    }
}

/// Hierarchical (tree-like) layout
fn hierarchical_layout(nodes: &mut [CanvasNode], edges: &[CanvasEdge]) {
    if nodes.is_empty() {
        return;
    }

    let id_to_idx: std::collections::HashMap<&str, usize> = nodes
        .iter()
        .enumerate()
        .map(|(i, node)| (node.id.as_str(), i))
        .collect();

    // Find root nodes (no incoming edges)
    let has_incoming: std::collections::HashSet<&str> =
        edges.iter().map(|e| e.target_id.as_str()).collect();

    let roots: Vec<usize> = nodes
        .iter()
        .enumerate()
        .filter(|(_, n)| !has_incoming.contains(n.id.as_str()))
        .map(|(i, _)| i)
        .collect();

    // BFS to assign levels
    let mut levels: Vec<i32> = vec![-1; nodes.len()];
    let mut queue = std::collections::VecDeque::new();

    if roots.is_empty() {
        // No roots found; treat first node as root
        levels[0] = 0;
        queue.push_back(0);
    } else {
        for &r in &roots {
            levels[r] = 0;
            queue.push_back(r);
        }
    }

    while let Some(current) = queue.pop_front() {
        let current_level = levels[current];
        for edge in edges {
            if let Some(&t) = id_to_idx.get(edge.target_id.as_str()) {
                if edge.source_id == nodes[current].id && levels[t] == -1 {
                    levels[t] = current_level + 1;
                    queue.push_back(t);
                }
            }
        }
    }

    // Unvisited nodes go to level 0
    for level in levels.iter_mut() {
        if *level == -1 {
            *level = 0;
        }
    }

    // Group by level
    let max_level = levels.iter().copied().max().unwrap_or(0);
    let mut level_groups: Vec<Vec<usize>> = vec![Vec::new(); (max_level + 1) as usize];
    for (i, &level) in levels.iter().enumerate() {
        level_groups[level as usize].push(i);
    }

    // Position nodes
    let h_spacing = 200.0;
    let v_spacing = 120.0;

    for (level, group) in level_groups.iter().enumerate() {
        let total_width = group.len() as f64 * h_spacing;
        let start_x = 200.0 - total_width / 2.0 + h_spacing / 2.0;

        for (col, &node_idx) in group.iter().enumerate() {
            nodes[node_idx].position = Position::new(
                start_x + col as f64 * h_spacing,
                100.0 + level as f64 * v_spacing,
            );
        }
    }
}

/// Radial layout: place nodes in concentric rings around a center
fn radial_layout(nodes: &mut [CanvasNode], edges: &[CanvasEdge]) {
    if nodes.is_empty() {
        return;
    }

    let center_x = 400.0;
    let center_y = 300.0;

    // Build degree count for each node
    let mut degree: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for node in nodes.iter() {
        degree.entry(node.id.clone()).or_insert(0);
    }
    for edge in edges {
        *degree.entry(edge.source_id.clone()).or_insert(0) += 1;
        *degree.entry(edge.target_id.clone()).or_insert(0) += 1;
    }

    // Sort nodes by degree (highest first) to place important nodes inner
    let mut sorted_indices: Vec<usize> = (0..nodes.len()).collect();
    sorted_indices.sort_by(|&a, &b| {
        let da = degree.get(&nodes[a].id).copied().unwrap_or(0);
        let db = degree.get(&nodes[b].id).copied().unwrap_or(0);
        db.cmp(&da)
    });

    let rings = ((nodes.len() as f64).sqrt().ceil() as usize).max(1);
    let per_ring = nodes.len() / rings + 1;

    for (rank, &idx) in sorted_indices.iter().enumerate() {
        let ring = rank / per_ring;
        let pos_in_ring = rank % per_ring;
        let ring_size = (ring + 1) * per_ring;
        let actual_in_ring = ring_size.min(nodes.len() - ring * per_ring).max(1);

        let radius = 80.0 + ring as f64 * 100.0;
        let angle = (pos_in_ring as f64 / actual_in_ring as f64) * std::f64::consts::PI * 2.0;

        nodes[idx].position = Position::new(
            center_x + radius * angle.cos(),
            center_y + radius * angle.sin(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::edge::CanvasEdge;

    fn make_node(id: &str, x: f64, y: f64) -> CanvasNode {
        CanvasNode::new_text(id, "test", x, y)
    }

    #[test]
    fn test_empty_layout() {
        let mut nodes: Vec<CanvasNode> = vec![];
        let edges: Vec<CanvasEdge> = vec![];
        force_directed_layout(&mut nodes, &edges);
        assert!(nodes.is_empty());
    }

    #[test]
    fn test_single_node_layout() {
        let mut nodes = vec![make_node("n1", 0.0, 0.0)];
        let edges: Vec<CanvasEdge> = vec![];
        force_directed_layout(&mut nodes, &edges);
        assert_eq!(nodes.len(), 1);
    }

    #[test]
    fn test_hierarchical_layout() {
        let mut nodes = vec![
            make_node("n1", 0.0, 0.0),
            make_node("n2", 0.0, 0.0),
            make_node("n3", 0.0, 0.0),
        ];
        let edges = vec![CanvasEdge::new_arrow("e1", "n1", "n2")];
        hierarchical_layout(&mut nodes, &edges);
        // n1 and n2 should be at different y-levels
        assert_ne!(nodes[0].position.y, nodes[1].position.y);
    }

    #[test]
    fn test_radial_layout() {
        let mut nodes = vec![
            make_node("n1", 0.0, 0.0),
            make_node("n2", 0.0, 0.0),
            make_node("n3", 0.0, 0.0),
        ];
        let edges: Vec<CanvasEdge> = vec![];
        radial_layout(&mut nodes, &edges);
        // Nodes should have different positions
        let positions: Vec<_> = nodes.iter().map(|n| (n.position.x, n.position.y)).collect();
        assert!(positions.iter().any(|p| *p != positions[0]));
    }
}
