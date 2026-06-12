use crate::canvas::CanvasEntityId;
use serde::{Deserialize, Serialize};

/// Edge style variants
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum EdgeStyle {
    Solid,
    Dotted,
    Dashed,
}

impl Default for EdgeStyle {
    fn default() -> Self {
        Self::Solid
    }
}

/// All supported canvas edge types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum CanvasEdgeData {
    #[serde(rename = "arrow")]
    Arrow(ArrowEdgeData),
    #[serde(rename = "line")]
    Line(LineEdgeData),
    #[serde(rename = "dotted")]
    Dotted(DottedEdgeData),
}

/// An edge connecting two nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasEdge {
    pub id: CanvasEntityId,
    pub source_id: CanvasEntityId,
    pub target_id: CanvasEntityId,
    pub data: CanvasEdgeData,
}

/// Arrow edge (with arrowhead)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArrowEdgeData {
    #[serde(default = "default_edge_color")]
    pub color: String,
    #[serde(default)]
    pub style: EdgeStyle,
}

/// Simple line edge
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LineEdgeData {
    #[serde(default = "default_edge_color")]
    pub color: String,
    #[serde(default)]
    pub style: EdgeStyle,
}

/// Dotted edge
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DottedEdgeData {
    #[serde(default = "default_edge_color")]
    pub color: String,
}

fn default_edge_color() -> String {
    "#6B7280".to_string()
}

impl CanvasEdge {
    pub fn new_arrow(
        id: impl Into<String>,
        source_id: impl Into<String>,
        target_id: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            source_id: source_id.into(),
            target_id: target_id.into(),
            data: CanvasEdgeData::Arrow(ArrowEdgeData {
                color: default_edge_color(),
                style: EdgeStyle::default(),
            }),
        }
    }

    pub fn new_line(
        id: impl Into<String>,
        source_id: impl Into<String>,
        target_id: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            source_id: source_id.into(),
            target_id: target_id.into(),
            data: CanvasEdgeData::Line(LineEdgeData {
                color: default_edge_color(),
                style: EdgeStyle::default(),
            }),
        }
    }

    pub fn new_dotted(
        id: impl Into<String>,
        source_id: impl Into<String>,
        target_id: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            source_id: source_id.into(),
            target_id: target_id.into(),
            data: CanvasEdgeData::Dotted(DottedEdgeData {
                color: default_edge_color(),
            }),
        }
    }

    /// Edge color for rendering
    pub fn color(&self) -> &str {
        match &self.data {
            CanvasEdgeData::Arrow(d) => &d.color,
            CanvasEdgeData::Line(d) => &d.color,
            CanvasEdgeData::Dotted(d) => &d.color,
        }
    }

    /// Edge style for rendering
    pub fn style(&self) -> &EdgeStyle {
        match &self.data {
            CanvasEdgeData::Arrow(d) => &d.style,
            CanvasEdgeData::Line(d) => &d.style,
            CanvasEdgeData::Dotted(_) => &EdgeStyle::Dotted,
        }
    }

    /// Whether this edge has an arrowhead
    pub fn has_arrowhead(&self) -> bool {
        matches!(self.data, CanvasEdgeData::Arrow(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arrow_edge_creation() {
        let edge = CanvasEdge::new_arrow("e1", "n1", "n2");
        assert_eq!(edge.source_id, "n1");
        assert_eq!(edge.target_id, "n2");
        assert!(edge.has_arrowhead());
    }

    #[test]
    fn test_line_edge_creation() {
        let edge = CanvasEdge::new_line("e1", "n1", "n2");
        assert!(!edge.has_arrowhead());
        assert_eq!(edge.style(), &EdgeStyle::Solid);
    }

    #[test]
    fn test_dotted_edge_creation() {
        let edge = CanvasEdge::new_dotted("e1", "n1", "n2");
        assert_eq!(edge.style(), &EdgeStyle::Dotted);
    }

    #[test]
    fn test_edge_color_default() {
        let edge = CanvasEdge::new_arrow("e1", "n1", "n2");
        assert_eq!(edge.color(), "#6B7280");
    }
}
