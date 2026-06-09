//! Tachyon Import/Export Library
//!
//! Provides import and export functionality for the Tachyon knowledge
//! management system, supporting:
//!
//! - Markdown vault import (generic recursive folder scan with frontmatter/tags)
//! - Markdown ZIP import/export (with YAML frontmatter)
//! - Obsidian vault import (frontmatter, wikilinks, tags, callouts)
//! - Docusaurus import (MDX, frontmatter, sidebar data)
//! - HTML export (rendered pages suitable for Confluence or static hosting)
//! - JSON export (structured document data)

pub mod csv_import;
pub mod docusaurus;
pub mod error;
pub mod frontmatter;
pub mod gdpr_export;
pub mod html_export;
pub mod json_export;
pub mod markdown_zip;

#[cfg(feature = "docx")]
pub mod docx_export;
#[cfg(feature = "docx")]
pub mod docx_import;
pub mod obsidian;
#[cfg(feature = "pdf-export")]
pub mod pdf_export;
pub mod vault_importer;

// Re-export commonly used types
pub use docusaurus::DocusaurusImporter;
#[cfg(feature = "docx")]
pub use docx_export::{DocxExportOptions, DocxExporter, PageSize};
#[cfg(feature = "docx")]
pub use docx_import::{DocxImportOptions, DocxImporter};
pub use error::{ImportExportError, ImportExportResult};
pub use frontmatter::Frontmatter;
pub use gdpr_export::{
    GdprExportBuilder, GdprUserExport, UserActivity, UserComment, UserDocument, UserProfile,
};
pub use html_export::{HtmlExportConfig, HtmlExportDocument, HtmlExporter};
pub use json_export::{ExportableDocument, JsonExporter};
pub use markdown_zip::{ExportDocument, MarkdownZipExporter, MarkdownZipImporter};
pub use obsidian::ObsidianImporter;
#[cfg(feature = "pdf-export")]
pub use pdf_export::{PdfExportConfig, PdfExportDocument, PdfExporter};
pub use vault_importer::MarkdownVaultImporter;

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A document parsed from an import source, ready to be stored.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportedDocument {
    /// Document title (from frontmatter or filename)
    pub title: String,
    /// URL-friendly slug (from frontmatter or derived from title)
    pub slug: Option<String>,
    /// Raw markdown content (body, without frontmatter)
    pub content: String,
    /// Parsed YAML frontmatter
    pub frontmatter: Frontmatter,
    /// Tags (merged from frontmatter and inline #tags)
    pub tags: Vec<String>,
    /// Original file path in the source archive/directory
    pub source_path: String,
    /// Creation date (from frontmatter or file metadata)
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Last modification date (from frontmatter or file metadata)
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Additional metadata from frontmatter not mapped to specific fields
    pub extra: BTreeMap<String, serde_json::Value>,
}

/// Summary of an import operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportSummary {
    /// Total files found in the source
    pub total_files: usize,
    /// Files successfully parsed as documents
    pub imported: usize,
    /// Files skipped (non-markdown, empty, etc.)
    pub skipped: usize,
    /// Files that failed to parse
    pub failed: usize,
    /// Titles of imported documents
    pub document_titles: Vec<String>,
    /// Tags found across all documents
    pub all_tags: Vec<String>,
    /// Warnings encountered during import
    pub warnings: Vec<String>,
}

/// Summary of an export operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportSummary {
    /// Total documents exported
    pub exported: usize,
    /// Export format used
    pub format: String,
    /// Output file size in bytes (if applicable)
    pub file_size_bytes: Option<u64>,
    /// Warnings encountered during export
    pub warnings: Vec<String>,
}

/// Convert an ImportedDocument into a Tachyon DocumentContent.
impl ImportedDocument {
    /// Get the content as markdown text.
    pub fn markdown_content(&self) -> &str {
        &self.content
    }

    /// Generate a slug from the title if none is set.
    pub fn effective_slug(&self) -> String {
        self.slug.clone().unwrap_or_else(|| slugify(&self.title))
    }
}

/// Simple slugification: lowercase, replace non-alphanumeric with hyphens.
pub(crate) fn slugify(text: &str) -> String {
    tachyon_core::util::slugify(text)
}

/// Parse a date string from frontmatter into chrono::DateTime.
/// Supports ISO 8601 and common date formats.
pub fn parse_date(s: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    // Try ISO 8601 first
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
        return Some(dt.with_timezone(&chrono::Utc));
    }
    // Try date-only (YYYY-MM-DD)
    if let Ok(d) = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        return d.and_hms_opt(0, 0, 0).map(|dt| dt.and_utc());
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("Hello   World"), "hello-world");
        assert_eq!(slugify("API Reference"), "api-reference");
        assert_eq!(slugify("C++ Guide"), "c-guide");
        assert_eq!(slugify(""), "");
    }

    #[test]
    fn test_parse_date_iso8601() {
        let dt = parse_date("2024-01-15T10:30:00Z").unwrap();
        assert_eq!(dt.year(), 2024);
        assert_eq!(dt.month(), 1);
        assert_eq!(dt.day(), 15);
    }

    #[test]
    fn test_parse_date_only() {
        let dt = parse_date("2024-01-15").unwrap();
        assert_eq!(dt.year(), 2024);
        assert_eq!(dt.month(), 1);
        assert_eq!(dt.day(), 15);
    }

    #[test]
    fn test_parse_date_invalid() {
        assert!(parse_date("not-a-date").is_none());
    }

    #[test]
    fn test_imported_document_effective_slug() {
        let doc = ImportedDocument {
            title: "My Document".to_string(),
            slug: None,
            content: "Hello".to_string(),
            frontmatter: Frontmatter::default(),
            tags: vec![],
            source_path: "my-document.md".to_string(),
            created_at: None,
            updated_at: None,
            extra: BTreeMap::new(),
        };
        assert_eq!(doc.effective_slug(), "my-document");

        let doc_with_slug = ImportedDocument {
            slug: Some("custom-slug".to_string()),
            ..doc.clone()
        };
        assert_eq!(doc_with_slug.effective_slug(), "custom-slug");
    }
}
