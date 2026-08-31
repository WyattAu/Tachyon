use crate::canvas::{CanvasEntityId, Position};
use serde::{Deserialize, Serialize};

/// All supported canvas node types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum CanvasNodeData {
    #[serde(rename = "text")]
    Text(TextNodeData),
    #[serde(rename = "image")]
    Image(ImageNodeData),
    #[serde(rename = "link")]
    Link(LinkNodeData),
    #[serde(rename = "document")]
    Document(DocumentNodeData),
    #[serde(rename = "shape")]
    Shape(ShapeNodeData),
}

/// A node on the canvas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasNode {
    pub id: CanvasEntityId,
    pub position: Position,
    pub data: CanvasNodeData,
}

/// Text node content
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TextNodeData {
    pub content: String,
    #[serde(default = "default_font_size")]
    pub font_size: f64,
    #[serde(default = "default_color")]
    pub color: String,
}

fn default_font_size() -> f64 {
    16.0
}

fn default_color() -> String {
    "#000000".to_string()
}

/// Image node content
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImageNodeData {
    pub src: String,
    #[serde(default)]
    pub alt: String,
    pub width: Option<f64>,
    pub height: Option<f64>,
}

/// Link node content
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LinkNodeData {
    pub url: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub description: String,
}

/// Document reference node
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DocumentNodeData {
    pub document_id: String,
    #[serde(default)]
    pub title: String,
}

/// Shape node variants
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ShapeType {
    Rectangle,
    Circle,
    Diamond,
}

/// Shape node content
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ShapeNodeData {
    #[serde(rename = "type")]
    pub shape_type: ShapeType,
    #[serde(default = "default_fill")]
    pub fill: String,
    #[serde(default = "default_stroke")]
    pub stroke: String,
}

fn default_fill() -> String {
    "#ffffff".to_string()
}

fn default_stroke() -> String {
    "#000000".to_string()
}

impl CanvasNode {
    pub fn new_text(id: impl Into<String>, content: impl Into<String>, x: f64, y: f64) -> Self {
        Self {
            id: id.into(),
            position: Position::new(x, y),
            data: CanvasNodeData::Text(TextNodeData {
                content: content.into(),
                font_size: default_font_size(),
                color: default_color(),
            }),
        }
    }

    pub fn new_image(
        id: impl Into<String>,
        src: impl Into<String>,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    ) -> Self {
        Self {
            id: id.into(),
            position: Position::new(x, y),
            data: CanvasNodeData::Image(ImageNodeData {
                src: src.into(),
                alt: String::new(),
                width: Some(width),
                height: Some(height),
            }),
        }
    }

    pub fn new_link(id: impl Into<String>, url: impl Into<String>, x: f64, y: f64) -> Self {
        Self {
            id: id.into(),
            position: Position::new(x, y),
            data: CanvasNodeData::Link(LinkNodeData {
                url: url.into(),
                title: String::new(),
                description: String::new(),
            }),
        }
    }

    pub fn new_document(
        id: impl Into<String>,
        document_id: impl Into<String>,
        title: impl Into<String>,
        x: f64,
        y: f64,
    ) -> Self {
        Self {
            id: id.into(),
            position: Position::new(x, y),
            data: CanvasNodeData::Document(DocumentNodeData {
                document_id: document_id.into(),
                title: title.into(),
            }),
        }
    }

    pub fn new_shape(id: impl Into<String>, shape_type: ShapeType, x: f64, y: f64) -> Self {
        Self {
            id: id.into(),
            position: Position::new(x, y),
            data: CanvasNodeData::Shape(ShapeNodeData {
                shape_type,
                fill: default_fill(),
                stroke: default_stroke(),
            }),
        }
    }

    /// Bounding box width for hit testing
    pub fn width(&self) -> f64 {
        match &self.data {
            CanvasNodeData::Text(d) => (d.content.len() as f64 * d.font_size * 0.6).max(60.0),
            CanvasNodeData::Image(d) => d.width.unwrap_or(200.0),
            CanvasNodeData::Link(_) => 180.0,
            CanvasNodeData::Document(_) => 180.0,
            CanvasNodeData::Shape(d) => match d.shape_type {
                ShapeType::Rectangle => 160.0,
                ShapeType::Circle => 120.0,
                ShapeType::Diamond => 120.0,
            },
        }
    }

    /// Bounding box height for hit testing
    pub fn height(&self) -> f64 {
        match &self.data {
            CanvasNodeData::Text(d) => (d.font_size * 1.4).max(24.0),
            CanvasNodeData::Image(d) => d.height.unwrap_or(150.0),
            CanvasNodeData::Link(_) => 60.0,
            CanvasNodeData::Document(_) => 60.0,
            CanvasNodeData::Shape(d) => match d.shape_type {
                ShapeType::Rectangle => 80.0,
                ShapeType::Circle => 120.0,
                ShapeType::Diamond => 120.0,
            },
        }
    }

    /// Center position of the node
    pub fn center(&self) -> Position {
        Position::new(
            self.position.x + self.width() / 2.0,
            self.position.y + self.height() / 2.0,
        )
    }

    /// Hit test: does the point (px, py) fall within this node?
    pub fn contains_point(&self, px: f64, py: f64) -> bool {
        let w = self.width();
        let h = self.height();
        px >= self.position.x
            && px <= self.position.x + w
            && py >= self.position.y
            && py <= self.position.y + h
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_node_creation() {
        let node = CanvasNode::new_text("n1", "Hello", 10.0, 20.0);
        assert_eq!(node.id, "n1");
        assert_eq!(node.position.x, 10.0);
        match &node.data {
            CanvasNodeData::Text(d) => assert_eq!(d.content, "Hello"),
            _ => panic!("Expected text node"),
        }
    }

    #[test]
    fn test_node_hit_test() {
        let node = CanvasNode::new_text("n1", "Test", 0.0, 0.0);
        assert!(node.contains_point(10.0, 10.0));
        assert!(!node.contains_point(-10.0, 10.0));
    }

    #[test]
    fn test_node_center() {
        let node = CanvasNode::new_text("n1", "Hello World", 100.0, 50.0);
        let center = node.center();
        assert!(center.x > 100.0);
        assert!(center.y > 50.0);
    }
}
