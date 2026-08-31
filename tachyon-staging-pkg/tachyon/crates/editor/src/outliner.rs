use std::collections::HashMap;

/// A unique identifier for an outliner node.
pub type NodeId = u64;

/// A single node in the outliner tree.
#[derive(Debug, Clone)]
pub struct OutlinerNode {
    pub id: NodeId,
    /// Content text for this node (one line in the document).
    pub content: String,
    /// Depth level (0 = root, 1 = first indent, etc.).
    pub depth: usize,
    /// Whether this node's children are collapsed.
    pub collapsed: bool,
    /// Block reference ID for transclusion. `None` means no block ref.
    pub block_ref: Option<String>,
}

impl OutlinerNode {
    pub fn new(id: NodeId, content: &str) -> Self {
        Self {
            id,
            content: content.to_string(),
            depth: 0,
            collapsed: false,
            block_ref: None,
        }
    }
}

/// State for the outliner mode — manages a flat list of nodes with tree semantics.
///
/// Tree structure is encoded via `depth` levels:
/// - A node is a child of the nearest preceding node with depth = current - 1.
/// - Collapsed nodes hide their descendants.
#[derive(Debug, Clone)]
pub struct OutlinerState {
    nodes: Vec<OutlinerNode>,
    /// Maps NodeId -> index in `nodes` for O(1) lookup.
    index: HashMap<NodeId, usize>,
    /// Monotonic ID generator.
    next_id: NodeId,
}

impl OutlinerState {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            index: HashMap::new(),
            next_id: 1,
        }
    }

    /// Build an OutlinerState from a text document.
    /// Lines starting with `- ` or `* ` are treated as list items.
    /// Indentation (leading spaces/tabs) determines depth.
    /// Lines without a bullet prefix are treated as continuation text
    /// at the same depth as the preceding bullet.
    pub fn from_text(text: &str) -> Self {
        let mut state = Self::new();
        let mut _prev_depth: usize = 0;

        for line in text.lines() {
            let trimmed = line.trim_end();
            if trimmed.is_empty() {
                continue;
            }

            // Count leading indent (2 spaces = 1 depth level)
            let indent = trimmed
                .chars()
                .take_while(|&c| c == ' ' || c == '\t')
                .count();
            let depth = indent / 2;

            // Check for bullet prefix
            let stripped = trimmed.trim_start_matches([' ', '\t']);
            let content = if stripped.starts_with("- ") || stripped.starts_with("* ") {
                &stripped[2..]
            } else {
                // Continuation line — use same depth as prev
                _prev_depth = depth;
                stripped
            };

            let id = state.next_id;
            state.next_id += 1;

            let mut node = OutlinerNode::new(id, content);
            node.depth = depth;
            _prev_depth = depth;

            state.index.insert(id, state.nodes.len());
            state.nodes.push(node);
        }

        state
    }

    /// Convert the outliner tree back to a text document.
    pub fn to_text(&self) -> String {
        let visible = self.visible_nodes();
        let mut lines = Vec::new();
        for node in visible {
            let indent = "  ".repeat(node.depth);
            lines.push(format!("{}- {}", indent, node.content));
        }
        lines.join("\n")
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn nodes(&self) -> &[OutlinerNode] {
        &self.nodes
    }

    pub fn node_by_id(&self, id: NodeId) -> Option<&OutlinerNode> {
        self.index.get(&id).and_then(|&i| self.nodes.get(i))
    }

    pub fn node_by_id_mut(&mut self, id: NodeId) -> Option<&mut OutlinerNode> {
        self.index
            .get(&id)
            .copied()
            .and_then(|i| self.nodes.get_mut(i))
    }

    /// Get the index of a node by its ID.
    pub fn index_of(&self, id: NodeId) -> Option<usize> {
        self.index.get(&id).copied()
    }

    /// Add a new node at the end.
    pub fn push_node(&mut self, content: &str) -> NodeId {
        let id = self.next_id;
        self.next_id += 1;
        let mut node = OutlinerNode::new(id, content);
        // Root nodes always start at depth 0
        node.depth = 0;
        self.index.insert(id, self.nodes.len());
        self.nodes.push(node);
        id
    }

    /// Add a new node as a child of `parent_id`, after `parent_id`'s last descendant.
    pub fn push_child(&mut self, parent_id: NodeId, content: &str) -> Option<NodeId> {
        let parent_idx = self.index_of(parent_id)?;
        let parent_depth = self.nodes[parent_idx].depth;
        let child_depth = parent_depth + 1;

        // Find the insertion point: after the parent's last descendant
        let mut insert_at = parent_idx + 1;
        while insert_at < self.nodes.len() && self.nodes[insert_at].depth > parent_depth {
            insert_at += 1;
        }

        let id = self.next_id;
        self.next_id += 1;
        let mut node = OutlinerNode::new(id, content);
        node.depth = child_depth;

        self.nodes.insert(insert_at, node);
        self.rebuild_index();
        Some(id)
    }

    /// Remove a node and all its descendants.
    pub fn remove_node(&mut self, id: NodeId) -> Option<OutlinerNode> {
        let idx = self.index_of(id)?;
        let depth = self.nodes[idx].depth;
        let mut end = idx + 1;
        while end < self.nodes.len() && self.nodes[end].depth > depth {
            end += 1;
        }
        let removed: Vec<OutlinerNode> = self.nodes.drain(idx..end).collect();
        self.rebuild_index();
        removed.into_iter().next()
    }

    /// Get the visible (non-collapsed) nodes.
    pub fn visible_nodes(&self) -> Vec<&OutlinerNode> {
        let mut result = Vec::new();
        let mut skip_depth: Option<usize> = None;
        for node in &self.nodes {
            if let Some(sd) = skip_depth {
                if node.depth > sd {
                    continue;
                } else {
                    skip_depth = None;
                }
            }
            result.push(node);
            if node.collapsed {
                skip_depth = Some(node.depth);
            }
        }
        result
    }

    /// Get the visible node IDs.
    pub fn visible_ids(&self) -> Vec<NodeId> {
        self.visible_nodes().iter().map(|n| n.id).collect()
    }

    // ─── Indent / Outdent ──────────────────────────────────────────────

    /// Indent a node (increase depth by 1).
    /// The node becomes a child of the preceding node at depth - 1.
    /// Returns false if the node is already at depth 0 or has no valid parent.
    pub fn indent(&mut self, id: NodeId) -> bool {
        let idx = match self.index_of(id) {
            Some(i) => i,
            None => return false,
        };
        if idx == 0 {
            return false;
        }
        let current_depth = self.nodes[idx].depth;
        // Can only indent if the preceding node is at exactly depth - 1
        if current_depth == 0 {
            let prev_depth = self.nodes[idx - 1].depth;
            if prev_depth == 0 {
                return false;
            }
        }
        if current_depth > 0 {
            let prev_depth = self.nodes[idx - 1].depth;
            if prev_depth < current_depth {
                // Already a sibling, not a child of the preceding node at depth+1
                // We need the preceding node to be at exactly current_depth - 1
                if prev_depth != current_depth - 1 {
                    return false;
                }
            }
        }

        self.nodes[idx].depth += 1;

        // Also indent all children
        let mut child = idx + 1;
        while child < self.nodes.len() && self.nodes[child].depth > current_depth {
            self.nodes[child].depth += 1;
            child += 1;
        }
        true
    }

    /// Outdent a node (decrease depth by 1).
    /// The node becomes a sibling of its current parent.
    /// Returns false if the node is already at depth 0.
    pub fn outdent(&mut self, id: NodeId) -> bool {
        let idx = match self.index_of(id) {
            Some(i) => i,
            None => return false,
        };
        let current_depth = self.nodes[idx].depth;
        if current_depth == 0 {
            return false;
        }

        self.nodes[idx].depth -= 1;

        // Also outdent all descendants (nodes deeper than current depth that follow)
        let mut child = idx + 1;
        while child < self.nodes.len() && self.nodes[child].depth > current_depth {
            self.nodes[child].depth -= 1;
            child += 1;
        }
        true
    }

    // ─── Move Up / Down ────────────────────────────────────────────────

    /// Move a node and its children up (swap with previous sibling).
    pub fn move_up(&mut self, id: NodeId) -> bool {
        let idx = match self.index_of(id) {
            Some(i) => i,
            None => return false,
        };
        if idx == 0 {
            return false;
        }
        let depth = self.nodes[idx].depth;

        // Find the start of this node's subtree (including children)
        let mut subtree_end = idx + 1;
        while subtree_end < self.nodes.len() && self.nodes[subtree_end].depth > depth {
            subtree_end += 1;
        }
        let subtree_len = subtree_end - idx;

        // Find the previous sibling: walk backward to find a node at the same depth
        let mut prev_idx = idx;
        while prev_idx > 0 {
            prev_idx -= 1;
            if self.nodes[prev_idx].depth == depth {
                break;
            }
        }
        if self.nodes[prev_idx].depth != depth {
            return false;
        }

        // Find the start of the previous sibling's subtree
        let mut prev_subtree_start = prev_idx;
        while prev_subtree_start > 0 && self.nodes[prev_subtree_start - 1].depth > depth {
            prev_subtree_start -= 1;
        }
        let prev_subtree_len = prev_idx - prev_subtree_start + 1;

        // Extract both subtrees
        let this_subtree: Vec<OutlinerNode> = self.nodes.drain(idx..subtree_end).collect();
        let prev_subtree: Vec<OutlinerNode> = self
            .nodes
            .drain(prev_subtree_start..prev_subtree_start + prev_subtree_len)
            .collect();

        // Insert this_subtree before prev_subtree
        let insert_at = prev_subtree_start;
        for (i, node) in this_subtree.into_iter().enumerate() {
            self.nodes.insert(insert_at + i, node);
        }
        for (i, node) in prev_subtree.into_iter().enumerate() {
            self.nodes.insert(insert_at + subtree_len + i, node);
        }

        self.rebuild_index();
        true
    }

    /// Move a node and its children down (swap with next sibling).
    pub fn move_down(&mut self, id: NodeId) -> bool {
        let idx = match self.index_of(id) {
            Some(i) => i,
            None => return false,
        };
        let depth = self.nodes[idx].depth;

        // Find the end of this node's subtree
        let mut subtree_end = idx + 1;
        while subtree_end < self.nodes.len() && self.nodes[subtree_end].depth > depth {
            subtree_end += 1;
        }

        if subtree_end >= self.nodes.len() {
            return false;
        }

        // The next sibling starts at subtree_end
        let next_start = subtree_end;
        let next_depth = self.nodes[next_start].depth;
        if next_depth != depth {
            return false;
        }

        // Find the end of the next sibling's subtree
        let mut next_end = next_start + 1;
        while next_end < self.nodes.len() && self.nodes[next_end].depth > next_depth {
            next_end += 1;
        }

        // Extract both subtrees
        let this_subtree: Vec<OutlinerNode> = self.nodes.drain(idx..subtree_end).collect();
        let this_len = this_subtree.len();
        let next_subtree: Vec<OutlinerNode> = self
            .nodes
            .drain(next_start - this_len..next_end - this_len)
            .collect();

        // Insert next_subtree first, then this_subtree
        let insert_at = idx;
        for (i, node) in next_subtree.into_iter().enumerate() {
            self.nodes.insert(insert_at + i, node);
        }
        for (i, node) in this_subtree.into_iter().enumerate() {
            self.nodes
                .insert(insert_at + (next_end - next_start) + i, node);
        }

        self.rebuild_index();
        true
    }

    // ─── Collapse / Expand ─────────────────────────────────────────────

    /// Toggle the collapsed state of a node.
    /// Returns the new collapsed state, or None if node not found.
    pub fn toggle_collapse(&mut self, id: NodeId) -> Option<bool> {
        let idx = self.index_of(id)?;
        self.nodes[idx].collapsed = !self.nodes[idx].collapsed;
        Some(self.nodes[idx].collapsed)
    }

    /// Collapse a node (hide its children).
    pub fn collapse(&mut self, id: NodeId) -> bool {
        if let Some(idx) = self.index_of(id) {
            self.nodes[idx].collapsed = true;
            true
        } else {
            false
        }
    }

    /// Expand a node (show its children).
    pub fn expand(&mut self, id: NodeId) -> bool {
        if let Some(idx) = self.index_of(id) {
            self.nodes[idx].collapsed = false;
            true
        } else {
            false
        }
    }

    /// Collapse all nodes that have children.
    pub fn collapse_all(&mut self) {
        // Collect node IDs first to avoid borrow checker issues
        let node_ids: Vec<NodeId> = self.nodes.iter().map(|n| n.id).collect();
        let has_children: Vec<bool> = node_ids.iter().map(|&id| self.has_children(id)).collect();

        for (node, &has_child) in self.nodes.iter_mut().zip(has_children.iter()) {
            if has_child {
                node.collapsed = true;
            }
        }
    }

    /// Expand all nodes.
    pub fn expand_all(&mut self) {
        for node in &mut self.nodes {
            node.collapsed = false;
        }
    }

    /// Check if a node has children.
    pub fn has_children(&self, id: NodeId) -> bool {
        if let Some(idx) = self.index_of(id) {
            let depth = self.nodes[idx].depth;
            idx + 1 < self.nodes.len() && self.nodes[idx + 1].depth > depth
        } else {
            false
        }
    }

    /// Get the children of a node.
    pub fn children(&self, id: NodeId) -> Vec<&OutlinerNode> {
        if let Some(idx) = self.index_of(id) {
            let depth = self.nodes[idx].depth;
            let mut children = Vec::new();
            let mut i = idx + 1;
            while i < self.nodes.len() && self.nodes[i].depth > depth {
                if self.nodes[i].depth == depth + 1 {
                    children.push(&self.nodes[i]);
                }
                i += 1;
            }
            children
        } else {
            Vec::new()
        }
    }

    /// Get the parent of a node.
    pub fn parent(&self, id: NodeId) -> Option<&OutlinerNode> {
        let idx = self.index_of(id)?;
        let depth = self.nodes[idx].depth;
        if depth == 0 {
            return None;
        }
        let mut i = idx;
        while i > 0 {
            i -= 1;
            if self.nodes[i].depth == depth - 1 {
                return Some(&self.nodes[i]);
            }
        }
        None
    }

    // ─── Block References (Transclusion) ───────────────────────────────

    /// Set the block reference ID for a node.
    pub fn set_block_ref(&mut self, id: NodeId, block_ref: &str) -> bool {
        if let Some(node) = self.node_by_id_mut(id) {
            node.block_ref = Some(block_ref.to_string());
            true
        } else {
            false
        }
    }

    /// Get the block reference for a node.
    pub fn get_block_ref(&self, id: NodeId) -> Option<&str> {
        self.node_by_id(id).and_then(|n| n.block_ref.as_deref())
    }

    // ─── Helpers ───────────────────────────────────────────────────────

    fn rebuild_index(&mut self) {
        self.index.clear();
        for (i, node) in self.nodes.iter().enumerate() {
            self.index.insert(node.id, i);
        }
    }

    /// Validate tree integrity: all depths are consistent, no gaps > 1.
    pub fn validate(&self) -> Result<(), String> {
        if self.nodes.is_empty() {
            return Ok(());
        }
        if self.nodes[0].depth != 0 {
            return Err(format!(
                "First node has depth {} (expected 0)",
                self.nodes[0].depth
            ));
        }
        for i in 1..self.nodes.len() {
            let prev = self.nodes[i - 1].depth;
            let cur = self.nodes[i].depth;
            if cur > prev + 1 {
                return Err(format!(
                    "Depth jump from {} to {} at node {} (\"{}\")",
                    prev, cur, self.nodes[i].id, self.nodes[i].content
                ));
            }
            if cur > prev && self.nodes[i - 1].collapsed {
                return Err(format!(
                    "Child at depth {} follows collapsed node {} at depth {}",
                    cur,
                    self.nodes[i - 1].id,
                    prev
                ));
            }
        }
        Ok(())
    }
}

impl Default for OutlinerState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_tree() -> OutlinerState {
        let mut state = OutlinerState::new();
        let a = state.push_node("A");
        let _b = state.push_child(a, "B").unwrap();
        let c = state.push_child(a, "C").unwrap();
        let _d = state.push_child(c, "D").unwrap();
        state.push_node("E");
        state
    }

    #[test]
    fn new_state_is_empty() {
        let state = OutlinerState::new();
        assert!(state.is_empty());
        assert_eq!(state.len(), 0);
    }

    #[test]
    fn push_node_and_child() {
        let state = make_tree();
        assert_eq!(state.len(), 5);
        let a = state.node_by_id(1).unwrap();
        assert_eq!(a.depth, 0);
        assert_eq!(a.content, "A");
        let b = state.node_by_id(2).unwrap();
        assert_eq!(b.depth, 1);
        assert_eq!(b.content, "B");
    }

    // ─── Indent / Outdent ──────────────────────────────────────────────

    #[test]
    fn indent_increases_depth() {
        let _state = make_tree();
        // B is at depth 1, parent A is at depth 0 — indent B
        // B should go to depth 2 (child of A, after itself)
        // Actually B is already a child of A. Indenting B would make it depth 2,
        // but the preceding node at depth 1 is B itself, so it needs a node at depth 1 before it.
        // Let's test indent on E (depth 0, preceding node A has depth 0)
        // Actually E is at depth 0, and C's last descendant D is at depth 2.
        // The node before E is D (depth 2). Indenting E would make it depth 3 — not useful.
        // Let's build a simpler tree for indent testing.
        let mut state2 = OutlinerState::new();
        let a = state2.push_node("A");
        let _b = state2.push_child(a, "B").unwrap();
        state2.push_node("C");

        // Indent C (depth 0) — preceding node B is at depth 1, so C can become depth 1
        let c_id = 3;
        assert!(state2.indent(c_id));
        assert_eq!(state2.node_by_id(c_id).unwrap().depth, 1);
    }

    #[test]
    fn indent_root_fails() {
        let mut state = OutlinerState::new();
        let a = state.push_node("A");
        assert!(!state.indent(a));
    }

    #[test]
    fn outdent_decreases_depth() {
        let mut state = make_tree();
        // B is at depth 1, outdent should make it depth 0
        let b_id = 2;
        assert!(state.outdent(b_id));
        assert_eq!(state.node_by_id(b_id).unwrap().depth, 0);
        // C is NOT a child of B (C is child of A at same depth), so C stays at depth 1
        assert_eq!(state.node_by_id(3).unwrap().depth, 1);
        // D is a child of C (not B), so D stays at depth 2
        assert_eq!(state.node_by_id(4).unwrap().depth, 2);
    }

    #[test]
    fn outdent_root_fails() {
        let mut state = OutlinerState::new();
        let a = state.push_node("A");
        assert!(!state.outdent(a));
    }

    // ─── Move Up / Down ────────────────────────────────────────────────

    #[test]
    fn move_up_swaps_with_prev_sibling() {
        let mut state = make_tree();
        // Nodes: A(0), B(1), C(1), D(2), E(0)
        // Move C up — should swap with B (C and its child D move before B)
        let c_id = 3;
        assert!(state.move_up(c_id));
        let visible = state.visible_nodes();
        let contents: Vec<&str> = visible.iter().map(|n| n.content.as_str()).collect();
        assert_eq!(contents, vec!["A", "C", "D", "B", "E"]);
    }

    #[test]
    fn move_up_first_node_fails() {
        let mut state = make_tree();
        assert!(!state.move_up(1)); // A is first
    }

    #[test]
    fn move_down_swaps_with_next_sibling() {
        let mut state = make_tree();
        // Nodes: A(0), B(1), C(1), D(2), E(0)
        // Move B down — should swap with C (B moves after C's subtree)
        let b_id = 2;
        assert!(state.move_down(b_id));
        let visible = state.visible_nodes();
        let contents: Vec<&str> = visible.iter().map(|n| n.content.as_str()).collect();
        assert_eq!(contents, vec!["A", "C", "D", "B", "E"]);
    }

    #[test]
    fn move_down_last_sibling_fails() {
        let mut state = make_tree();
        // C is the last child of A
        assert!(!state.move_down(3));
    }

    // ─── Collapse / Expand ─────────────────────────────────────────────

    #[test]
    fn toggle_collapse() {
        let mut state = make_tree();
        let a_id = 1;
        assert!(!state.node_by_id(a_id).unwrap().collapsed);
        let collapsed = state.toggle_collapse(a_id).unwrap();
        assert!(collapsed);
        assert!(state.node_by_id(a_id).unwrap().collapsed);
    }

    #[test]
    fn collapse_hides_children() {
        let mut state = make_tree();
        state.collapse(1); // Collapse A
        let visible = state.visible_ids();
        assert_eq!(visible, vec![1, 5]); // A and E
    }

    #[test]
    fn expand_shows_children() {
        let mut state = make_tree();
        state.collapse(1);
        state.expand(1);
        let visible = state.visible_ids();
        assert_eq!(visible, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn collapse_all() {
        let mut state = make_tree();
        state.collapse_all();
        let visible = state.visible_ids();
        assert_eq!(visible, vec![1, 5]); // Only root nodes
    }

    #[test]
    fn expand_all() {
        let mut state = make_tree();
        state.collapse_all();
        state.expand_all();
        let visible = state.visible_ids();
        assert_eq!(visible, vec![1, 2, 3, 4, 5]);
    }

    // ─── Tree Structure ────────────────────────────────────────────────

    #[test]
    fn has_children() {
        let state = make_tree();
        assert!(state.has_children(1)); // A has B, C
        assert!(!state.has_children(2)); // B has no children
        assert!(state.has_children(3)); // C has D
    }

    #[test]
    fn children_of_node() {
        let state = make_tree();
        let children = state.children(1);
        assert_eq!(children.len(), 2);
        assert_eq!(children[0].content, "B");
        assert_eq!(children[1].content, "C");
    }

    #[test]
    fn parent_of_node() {
        let state = make_tree();
        let parent = state.parent(4); // D's parent is C
        assert!(parent.is_some());
        assert_eq!(parent.unwrap().content, "C");
    }

    #[test]
    fn parent_of_root_is_none() {
        let state = make_tree();
        assert!(state.parent(1).is_none());
    }

    // ─── Block References ──────────────────────────────────────────────

    #[test]
    fn set_get_block_ref() {
        let mut state = make_tree();
        assert!(state.set_block_ref(2, "ref-abc"));
        assert_eq!(state.get_block_ref(2), Some("ref-abc"));
    }

    // ─── Text Roundtrip ────────────────────────────────────────────────

    #[test]
    fn text_roundtrip() {
        let text = "- A\n  - B\n  - C\n    - D\n- E";
        let state = OutlinerState::from_text(text);
        assert_eq!(state.len(), 5);
        let output = state.to_text();
        // E stays at depth 0 as authored — the outliner preserves original indentation
        assert_eq!(output, "- A\n  - B\n  - C\n    - D\n- E");
    }

    #[test]
    fn from_text_empty() {
        let state = OutlinerState::from_text("");
        assert!(state.is_empty());
    }

    // ─── Validation ────────────────────────────────────────────────────

    #[test]
    fn validate_ok() {
        let state = make_tree();
        assert!(state.validate().is_ok());
    }

    #[test]
    fn validate_depth_jump() {
        let mut state = OutlinerState::new();
        state.push_node("A");
        let mut node = OutlinerNode::new(2, "B");
        node.depth = 3; // Invalid jump from 0 to 3
        state.index.insert(2, 1);
        state.nodes.push(node);
        assert!(state.validate().is_err());
    }

    // ─── Remove ────────────────────────────────────────────────────────

    #[test]
    fn remove_node_and_children() {
        let mut state = make_tree();
        // Remove C (has child D)
        state.remove_node(3);
        assert_eq!(state.len(), 3); // A, B, E
        assert!(state.node_by_id(3).is_none());
        assert!(state.node_by_id(4).is_none()); // D removed too
    }
}
