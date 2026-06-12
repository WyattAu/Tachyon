#![allow(dead_code)]

use serde::{Deserialize, Serialize};

/// A rectangle region on a PDF page
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Annotation color
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AnnotationColor {
    Yellow,
    Blue,
    Green,
    Pink,
}

impl AnnotationColor {
    pub fn to_hex(&self) -> &str {
        match self {
            AnnotationColor::Yellow => "#FFFF00",
            AnnotationColor::Blue => "#4A90D9",
            AnnotationColor::Green => "#50C878",
            AnnotationColor::Pink => "#FFB6C1",
        }
    }

    pub fn to_rgba(&self, opacity: f64) -> String {
        match self {
            AnnotationColor::Yellow => format!("rgba(255, 255, 0, {})", opacity),
            AnnotationColor::Blue => format!("rgba(74, 144, 217, {})", opacity),
            AnnotationColor::Green => format!("rgba(80, 200, 120, {})", opacity),
            AnnotationColor::Pink => format!("rgba(255, 182, 193, {})", opacity),
        }
    }
}

/// Annotation type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AnnotationType {
    Highlight,
    Underline,
    Strikethrough,
    StickyNote,
}

/// A text annotation (highlight, underline, strikethrough)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TextAnnotation {
    pub id: String,
    pub page: u32,
    pub rects: Vec<Rect>,
    pub color: AnnotationColor,
    pub note: Option<String>,
    pub kind: AnnotationType,
    pub created_at: String,
}

/// A sticky note annotation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StickyNoteAnnotation {
    pub id: String,
    pub page: u32,
    pub x: f64,
    pub y: f64,
    pub content: String,
    pub created_at: String,
}

/// Union of all annotation types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Annotation {
    Text(TextAnnotation),
    StickyNote(StickyNoteAnnotation),
}

impl Annotation {
    pub fn id(&self) -> &str {
        match self {
            Annotation::Text(a) => &a.id,
            Annotation::StickyNote(a) => &a.id,
        }
    }

    pub fn page(&self) -> u32 {
        match self {
            Annotation::Text(a) => a.page,
            Annotation::StickyNote(a) => a.page,
        }
    }
}

/// PDF document metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PdfDocument {
    pub url: String,
    pub title: Option<String>,
    pub page_count: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect_creation() {
        let rect = Rect {
            x: 10.0,
            y: 20.0,
            width: 100.0,
            height: 50.0,
        };
        assert_eq!(rect.x, 10.0);
        assert_eq!(rect.width, 100.0);
    }

    #[test]
    fn test_annotation_color_hex() {
        assert_eq!(AnnotationColor::Yellow.to_hex(), "#FFFF00");
        assert_eq!(AnnotationColor::Blue.to_hex(), "#4A90D9");
    }

    #[test]
    fn test_annotation_color_rgba() {
        let rgba = AnnotationColor::Yellow.to_rgba(0.5);
        assert!(rgba.contains("0.5"));
    }

    #[test]
    fn test_text_annotation_creation() {
        let annotation = TextAnnotation {
            id: "test-1".to_string(),
            page: 1,
            rects: vec![Rect {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 20.0,
            }],
            color: AnnotationColor::Yellow,
            note: Some("Test note".to_string()),
            kind: AnnotationType::Highlight,
            created_at: "2024-01-01T00:00:00Z".to_string(),
        };
        assert_eq!(annotation.id, "test-1");
        assert_eq!(annotation.page, 1);
        assert_eq!(annotation.rects.len(), 1);
    }

    #[test]
    fn test_sticky_note_annotation() {
        let note = StickyNoteAnnotation {
            id: "note-1".to_string(),
            page: 1,
            x: 50.0,
            y: 100.0,
            content: "Important point".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
        };
        assert_eq!(note.content, "Important point");
    }

    #[test]
    fn test_annotation_union() {
        let text = Annotation::Text(TextAnnotation {
            id: "a1".to_string(),
            page: 1,
            rects: vec![],
            color: AnnotationColor::Blue,
            note: None,
            kind: AnnotationType::Underline,
            created_at: "2024-01-01T00:00:00Z".to_string(),
        });
        assert_eq!(text.id(), "a1");
        assert_eq!(text.page(), 1);
    }
}
