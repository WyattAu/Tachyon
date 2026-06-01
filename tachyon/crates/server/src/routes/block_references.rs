//! Block references and transclusion — Logseq/Roam-style.

use serde::{Deserialize, Serialize};

/// A block reference linking one document block to another.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockReference {
    pub id: String,
    pub source_document_id: String,
    pub source_block_id: String,
    pub target_document_id: String,
    pub target_block_id: String,
    pub reference_type: String,
}

/// Expand block references in document content.
pub fn expand_block_references(content: &str, documents: &std::collections::HashMap<String, String>) -> String {
    let re = regex::Regex::new(r"\[\[!([a-f0-9-]+)\]\]").unwrap();
    re.replace_all(content, |caps: &regex::Captures| {
        let doc_id = &caps[1];
        documents.get(doc_id).cloned().unwrap_or_else(|| format!("[[!{}]]", doc_id))
    }).to_string()
}

/// Parse embedded block references from content.
pub fn parse_embed_references(content: &str) -> Vec<BlockReference> {
    let mut refs = Vec::new();
    let re = regex::Regex::new(r"\[\[!([a-f0-9-]+)(?:#([a-f0-9-]+))?\]\]").unwrap();
    for caps in re.captures_iter(content) {
        refs.push(BlockReference {
            id: uuid::Uuid::new_v4().to_string(),
            source_document_id: String::new(),
            source_block_id: String::new(),
            target_document_id: caps[1].to_string(),
            target_block_id: caps.get(2).map(|m| m.as_str().to_string()).unwrap_or_default(),
            reference_type: "embed".to_string(),
        });
    }
    refs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_reference_serialization() {
        let br = BlockReference {
            id: "1".to_string(),
            source_document_id: "doc1".to_string(),
            source_block_id: "block1".to_string(),
            target_document_id: "doc2".to_string(),
            target_block_id: "block2".to_string(),
            reference_type: "embed".to_string(),
        };
        let json = serde_json::to_string(&br).unwrap();
        assert!(json.contains("embed"));
        assert!(json.contains("doc2"));
    }

    #[test]
    fn test_expand_block_references() {
        let content = "See [[!abc-123]] for details.";
        let mut docs = std::collections::HashMap::new();
        docs.insert("abc-123".to_string(), "Embedded content here.".to_string());
        let expanded = expand_block_references(content, &docs);
        assert!(expanded.contains("Embedded content here."));
        assert!(!expanded.contains("[[!"));
    }

    #[test]
    fn test_expand_unknown_reference() {
        let content = "Unknown: [[!missing]]";
        let docs = std::collections::HashMap::new();
        let expanded = expand_block_references(content, &docs);
        assert!(expanded.contains("[[!missing]]"));
    }

    #[test]
    fn test_parse_embed_references() {
        let content = "Intro [[!doc1#block1]] and [[!doc2]]";
        let refs = parse_embed_references(content);
        assert_eq!(refs.len(), 2);
        assert_eq!(refs[0].target_block_id, "block1");
        assert_eq!(refs[1].target_block_id, "");
    }

    #[test]
    fn test_parse_no_references() {
        let content = "No references here";
        let refs = parse_embed_references(content);
        assert!(refs.is_empty());
    }
}
