//! JSON export for documents.
//!
//! Serializes documents to a JSON array, with optional metadata and pretty-printing.

use crate::{error::ImportExportResult, ImportExportError};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::Write;
use std::path::Path;

/// A document ready for JSON export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportableDocument {
    pub id: String,
    pub title: String,
    pub content: String,
    pub slug: String,
    pub tags: Vec<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<BTreeMap<String, serde_json::Value>>,
}

/// JSON exporter for documents.
pub struct JsonExporter {
    include_metadata: bool,
    pretty_print: bool,
}

impl Default for JsonExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl JsonExporter {
    pub fn new() -> Self {
        Self {
            include_metadata: false,
            pretty_print: false,
        }
    }

    pub fn with_metadata(mut self) -> Self {
        self.include_metadata = true;
        self
    }

    pub fn with_pretty_print(mut self) -> Self {
        self.pretty_print = true;
        self
    }

    pub fn export(&self, documents: Vec<ExportableDocument>) -> ImportExportResult<Vec<u8>> {
        let docs: Vec<_> = if self.include_metadata {
            documents
        } else {
            documents
                .into_iter()
                .map(|mut doc| {
                    doc.metadata = None;
                    doc
                })
                .collect()
        };

        let bytes = if self.pretty_print {
            serde_json::to_vec_pretty(&docs)
        } else {
            serde_json::to_vec(&docs)
        }
        .map_err(|e| ImportExportError::export(format!("JSON serialization failed: {}", e)))?;

        Ok(bytes)
    }

    pub fn export_to_file(
        &self,
        documents: Vec<ExportableDocument>,
        path: &Path,
    ) -> ImportExportResult<()> {
        let bytes = self.export(documents)?;
        let mut file =
            std::fs::File::create(path).map_err(|e| ImportExportError::io(path, e.to_string()))?;
        file.write_all(&bytes)
            .map_err(|e| ImportExportError::io(path, e.to_string()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_doc(id: &str, title: &str, content: &str) -> ExportableDocument {
        ExportableDocument {
            id: id.to_string(),
            title: title.to_string(),
            content: content.to_string(),
            slug: crate::slugify(title),
            tags: vec![],
            created_at: None,
            updated_at: None,
            metadata: None,
        }
    }

    fn test_doc_with_metadata(
        id: &str,
        title: &str,
        content: &str,
    ) -> ExportableDocument {
        let mut meta = BTreeMap::new();
        meta.insert("author".to_string(), serde_json::json!("test-user"));
        ExportableDocument {
            id: id.to_string(),
            title: title.to_string(),
            content: content.to_string(),
            slug: crate::slugify(title),
            tags: vec!["tag1".to_string()],
            created_at: Some("2024-01-15T10:30:00Z".to_string()),
            updated_at: Some("2024-01-16T12:00:00Z".to_string()),
            metadata: Some(meta),
        }
    }

    #[test]
    fn test_export_single_document() {
        let exporter = JsonExporter::new();
        let docs = vec![test_doc("1", "Hello World", "# Hello\n\nContent")];
        let bytes = exporter.export(docs).unwrap();
        let json_str = String::from_utf8(bytes).unwrap();
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&json_str).unwrap();

        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0]["id"], "1");
        assert_eq!(parsed[0]["title"], "Hello World");
        assert_eq!(parsed[0]["slug"], "hello-world");
        assert!(parsed[0].get("metadata").is_none());
    }

    #[test]
    fn test_export_multiple_documents() {
        let exporter = JsonExporter::new();
        let docs = vec![
            test_doc("1", "First Doc", "Content 1"),
            test_doc("2", "Second Doc", "Content 2"),
            test_doc("3", "Third Doc", "Content 3"),
        ];
        let bytes = exporter.export(docs).unwrap();
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&String::from_utf8(bytes).unwrap()).unwrap();

        assert_eq!(parsed.len(), 3);
        assert_eq!(parsed[0]["id"], "1");
        assert_eq!(parsed[1]["id"], "2");
        assert_eq!(parsed[2]["id"], "3");
    }

    #[test]
    fn test_export_with_metadata() {
        let exporter = JsonExporter::new().with_metadata();
        let docs = vec![test_doc_with_metadata("1", "With Meta", "Content")];
        let bytes = exporter.export(docs).unwrap();
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&String::from_utf8(bytes).unwrap()).unwrap();

        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0]["metadata"]["author"], "test-user");
        assert_eq!(parsed[0]["tags"][0], "tag1");
        assert_eq!(parsed[0]["created_at"], "2024-01-15T10:30:00Z");
    }

    #[test]
    fn test_export_without_metadata_strips_it() {
        let exporter = JsonExporter::new();
        let docs = vec![test_doc_with_metadata("1", "Strip Meta", "Content")];
        let bytes = exporter.export(docs).unwrap();
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&String::from_utf8(bytes).unwrap()).unwrap();

        assert!(parsed[0].get("metadata").is_none());
    }

    #[test]
    fn test_export_with_pretty_print() {
        let exporter = JsonExporter::new().with_pretty_print();
        let docs = vec![test_doc("1", "Pretty", "Content")];
        let bytes = exporter.export(docs).unwrap();
        let json_str = String::from_utf8(bytes).unwrap();

        assert!(json_str.contains('\n'));
        assert!(json_str.contains("  "));
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&json_str).unwrap();
        assert_eq!(parsed.len(), 1);
    }

    #[test]
    fn test_export_compact_no_pretty() {
        let exporter = JsonExporter::new();
        let docs = vec![test_doc("1", "Compact", "Content")];
        let bytes = exporter.export(docs).unwrap();
        let json_str = String::from_utf8(bytes).unwrap();

        assert!(!json_str.contains('\n'));
    }

    #[test]
    fn test_export_to_file() {
        let dir = std::env::temp_dir().join("tachyon_json_export_test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("export.json");

        let exporter = JsonExporter::new().with_pretty_print();
        let docs = vec![test_doc("1", "File Test", "Content")];
        exporter.export_to_file(docs, &path).unwrap();

        let contents = std::fs::read_to_string(&path).unwrap();
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&contents).unwrap();
        assert_eq!(parsed[0]["id"], "1");

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_roundtrip_export_import() {
        let exporter = JsonExporter::new().with_metadata();
        let original = vec![
            test_doc_with_metadata("abc-123", "Roundtrip Doc", "# Title\n\nBody text"),
            test_doc_with_metadata("def-456", "Another Doc", "Plain content"),
        ];

        let bytes = exporter.export(original.clone()).unwrap();
        let json_str = String::from_utf8(bytes).unwrap();

        let imported: Vec<ExportableDocument> = serde_json::from_str(&json_str).unwrap();
        assert_eq!(imported.len(), 2);
        assert_eq!(imported[0].id, original[0].id);
        assert_eq!(imported[0].title, original[0].title);
        assert_eq!(imported[0].content, original[0].content);
        assert_eq!(imported[0].tags, original[0].tags);
        assert_eq!(imported[1].id, original[1].id);
        assert_eq!(imported[1].metadata.as_ref().unwrap()["author"], "test-user");
    }
}
