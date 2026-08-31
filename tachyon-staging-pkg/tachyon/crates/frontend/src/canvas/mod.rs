pub mod edge;
pub mod layout;
pub mod node;
pub mod renderer;

use serde::{Deserialize, Serialize};

pub use edge::*;
pub use layout::*;
pub use node::*;
pub use renderer::*;

/// Unique identifier for canvas entities
pub type CanvasEntityId = String;

/// 2D position on the canvas
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

impl Default for Position {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

impl Position {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

/// Canvas view state for infinite scrolling
#[derive(Debug, Clone)]
pub struct ViewState {
    pub offset_x: f64,
    pub offset_y: f64,
    pub zoom: f64,
}

impl Default for ViewState {
    fn default() -> Self {
        Self {
            offset_x: 0.0,
            offset_y: 0.0,
            zoom: 1.0,
        }
    }
}

/// The main canvas state
#[derive(Debug, Clone, Default)]
pub struct CanvasState {
    pub nodes: Vec<CanvasNode>,
    pub edges: Vec<CanvasEdge>,
    pub selected_node_id: Option<CanvasEntityId>,
    pub selected_edge_id: Option<CanvasEntityId>,
    pub view: ViewState,
    pub is_panning: bool,
    pub connecting_from: Option<CanvasEntityId>,
}

impl CanvasState {
    pub fn find_node(&self, id: &str) -> Option<&CanvasNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    pub fn find_node_mut(&mut self, id: &str) -> Option<&mut CanvasNode> {
        self.nodes.iter_mut().find(|n| n.id == id)
    }

    pub fn find_edge(&self, id: &str) -> Option<&CanvasEdge> {
        self.edges.iter().find(|e| e.id == id)
    }

    pub fn remove_node(&mut self, id: &str) {
        self.nodes.retain(|n| n.id != id);
        self.edges
            .retain(|e| e.source_id != id && e.target_id != id);
    }

    pub fn remove_edge(&mut self, id: &str) {
        self.edges.retain(|e| e.id != id);
    }

    pub fn add_node(&mut self, node: CanvasNode) {
        self.nodes.push(node);
    }

    pub fn add_edge(&mut self, edge: CanvasEdge) {
        self.edges.push(edge);
    }
}
