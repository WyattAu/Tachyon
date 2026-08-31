//! Rich text editor API — ProseMirror-compatible document format.

use serde::{Deserialize, Serialize};

/// ProseMirror-compatible document node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProseMirrorDoc {
    #[serde(rename = "type")]
    pub node_type: String,
    pub content: Vec<ProseMirrorNode>,
    pub attrs: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProseMirrorNode {
    #[serde(rename = "type")]
    pub node_type: String,
    pub content: Option<Vec<ProseMirrorNode>>,
    pub text: Option<String>,
    pub marks: Option<Vec<ProseMirrorMark>>,
    pub attrs: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProseMirrorMark {
    #[serde(rename = "type")]
    pub mark_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attrs: Option<serde_json::Value>,
}

/// Convert ProseMirror JSON to Markdown text.
pub fn prose_mirror_to_markdown(doc: &ProseMirrorDoc) -> String {
    doc.content.iter().map(node_to_markdown).collect::<Vec<_>>().join("\n")
}

fn node_to_markdown(node: &ProseMirrorNode) -> String {
    match node.node_type.as_str() {
        "heading" => {
            let level = node.attrs.as_ref()
                .and_then(|a| a.get("level"))
                .and_then(|l| l.as_u64())
                .unwrap_or(1) as usize;
            let text = extract_text(node);
            "#".repeat(level) + " " + &text
        }
        "paragraph" => extract_text(node),
        "code_block" => {
            let text = extract_text(node);
            let lang = node.attrs.as_ref()
                .and_then(|a| a.get("language"))
                .and_then(|l| l.as_str())
                .unwrap_or("");
            format!("```{}\n{}\n```", lang, text)
        }
        "blockquote" => {
            let text = node.content.as_ref()
                .map(|c| c.iter().map(node_to_markdown).collect::<Vec<_>>().join("\n"))
                .unwrap_or_default();
            text.lines().map(|l| format!("> {}", l)).collect::<Vec<_>>().join("\n")
        }
        "bullet_list" | "ordered_list" => {
            let marker = if node.node_type == "bullet_list" { "- " } else { "1. " };
            node.content.as_ref()
                .map(|c| c.iter().map(|n| format!("{}{}", marker, extract_text(n))).collect::<Vec<_>>().join("\n"))
                .unwrap_or_default()
        }
        "horizontal_rule" => "---".to_string(),
        _ => extract_text(node),
    }
}

fn extract_text(node: &ProseMirrorNode) -> String {
    if let Some(ref text) = node.text {
        return text.clone();
    }
    node.content.as_ref()
        .map(|c| c.iter().map(extract_text).collect::<Vec<_>>().join(""))
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prose_mirror_heading() {
        let doc = ProseMirrorDoc {
            node_type: "doc".to_string(),
            attrs: None,
            content: vec![ProseMirrorNode {
                node_type: "heading".to_string(),
                attrs: Some(serde_json::json!({"level": 2})),
                text: None,
                content: Some(vec![ProseMirrorNode {
                    node_type: "text".to_string(),
                    text: Some("Hello".to_string()),
                    marks: None,
                    content: None,
                    attrs: None,
                }]),
                marks: None,
            }],
        };
        let md = prose_mirror_to_markdown(&doc);
        assert_eq!(md, "## Hello");
    }

    #[test]
    fn test_prose_mirror_paragraph() {
        let doc = ProseMirrorDoc {
            node_type: "doc".to_string(),
            attrs: None,
            content: vec![ProseMirrorNode {
                node_type: "paragraph".to_string(),
                text: None,
                content: Some(vec![ProseMirrorNode {
                    node_type: "text".to_string(),
                    text: Some("World".to_string()),
                    marks: None,
                    content: None,
                    attrs: None,
                }]),
                marks: None,
                attrs: None,
            }],
        };
        let md = prose_mirror_to_markdown(&doc);
        assert_eq!(md, "World");
    }

    #[test]
    fn test_mark_serialization() {
        let mark = ProseMirrorMark {
            mark_type: "bold".to_string(),
            attrs: None,
        };
        let json = serde_json::to_string(&mark).unwrap();
        assert_eq!(json, r#"{"type":"bold"}"#);
    }

    #[test]
    fn test_code_block_with_language() {
        let doc = ProseMirrorDoc {
            node_type: "doc".to_string(),
            attrs: None,
            content: vec![ProseMirrorNode {
                node_type: "code_block".to_string(),
                attrs: Some(serde_json::json!({"language": "rust"})),
                text: Some("fn main() {}".to_string()),
                content: None,
                marks: None,
            }],
        };
        let md = prose_mirror_to_markdown(&doc);
        assert!(md.contains("```rust"));
    }
}
