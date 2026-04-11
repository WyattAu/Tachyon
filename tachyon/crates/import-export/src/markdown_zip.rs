//! Markdown ZIP import and export.
//!
//! Import: Reads a ZIP archive containing `.md` files, parses frontmatter,
//! and produces a list of `ImportedDocument` values.
//!
//! Export: Takes document data and produces a ZIP archive of `.md` files
//! with YAML frontmatter.

use crate::{
    error::ImportExportResult, frontmatter::Frontmatter, slugify, ExportSummary, ImportExportError,
    ImportSummary, ImportedDocument,
};
use std::collections::HashSet;
use std::io::{Cursor, Read, Write};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

/// Import markdown documents from a ZIP archive.
pub struct MarkdownZipImporter;

impl MarkdownZipImporter {
    /// Import all `.md` files from a ZIP archive (provided as bytes).
    pub fn import_from_bytes(zip_bytes: &[u8]) -> ImportExportResult<ImportSummary> {
        let cursor = Cursor::new(zip_bytes);
        let mut archive =
            ZipArchive::new(cursor).map_err(|e| ImportExportError::zip(e.to_string()))?;

        let mut summary = ImportSummary {
            total_files: archive.len(),
            imported: 0,
            skipped: 0,
            failed: 0,
            document_titles: Vec::new(),
            all_tags: Vec::new(),
            warnings: Vec::new(),
        };

        let mut all_tags: HashSet<String> = HashSet::new();

        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .map_err(|e| ImportExportError::zip(e.to_string()))?;

            let name = file.name().to_string();
            let is_dir = file.is_dir();

            if is_dir {
                summary.skipped += 1;
                continue;
            }

            // Only process .md and .markdown files
            let is_markdown = name.ends_with(".md") || name.ends_with(".markdown");
            if !is_markdown {
                summary.skipped += 1;
                continue;
            }

            // Skip hidden files (e.g., .obsidian/)
            if name.contains("/.") || name.starts_with('.') {
                summary.skipped += 1;
                continue;
            }

            let mut content = String::new();
            if let Err(e) = file.read_to_string(&mut content) {
                tracing::warn!("Failed to read file {}: {}", name, e);
                summary.failed += 1;
                summary
                    .warnings
                    .push(format!("Failed to read {}: {}", name, e));
                continue;
            }

            if content.trim().is_empty() {
                summary.skipped += 1;
                continue;
            }

            // Parse frontmatter
            let (frontmatter, body) = Frontmatter::parse(&content);

            // Derive title from frontmatter or filename
            let title = frontmatter
                .title
                .clone()
                .unwrap_or_else(|| title_from_path(&name));

            // Collect tags
            for tag in &frontmatter.tags {
                all_tags.insert(tag.clone());
            }

            summary.imported += 1;
            summary.document_titles.push(title.clone());

            // Note: The actual ImportedDocument is created here but we don't
            // return it in the summary. The full import returns a Vec.
            // For the summary-only API, we just count.
            let _doc = ImportedDocument {
                title,
                slug: None, // Will be set by caller or via frontmatter.slug
                content: body.to_string(),
                frontmatter,
                tags: vec![], // Merged by caller
                source_path: name,
                created_at: None,
                updated_at: None,
                extra: std::collections::HashMap::new(),
            };
        }

        summary.all_tags = all_tags.into_iter().collect();
        summary.all_tags.sort();
        Ok(summary)
    }

    /// Import all `.md` files from a ZIP archive, returning the parsed documents.
    pub fn import_documents_from_bytes(
        zip_bytes: &[u8],
    ) -> ImportExportResult<(Vec<ImportedDocument>, ImportSummary)> {
        let cursor = Cursor::new(zip_bytes);
        let mut archive =
            ZipArchive::new(cursor).map_err(|e| ImportExportError::zip(e.to_string()))?;

        let mut summary = ImportSummary {
            total_files: archive.len(),
            imported: 0,
            skipped: 0,
            failed: 0,
            document_titles: Vec::new(),
            all_tags: Vec::new(),
            warnings: Vec::new(),
        };

        let mut documents = Vec::new();
        let mut all_tags: HashSet<String> = HashSet::new();

        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .map_err(|e| ImportExportError::zip(e.to_string()))?;

            let name = file.name().to_string();
            let is_dir = file.is_dir();

            if is_dir || !is_markdown_file(&name) || is_hidden(&name) {
                summary.skipped += 1;
                continue;
            }

            let mut content = String::new();
            if let Err(e) = file.read_to_string(&mut content) {
                tracing::warn!("Failed to read file {}: {}", name, e);
                summary.failed += 1;
                summary
                    .warnings
                    .push(format!("Failed to read {}: {}", name, e));
                continue;
            }

            if content.trim().is_empty() {
                summary.skipped += 1;
                continue;
            }

            let (frontmatter, body) = Frontmatter::parse(&content);

            let title = frontmatter
                .title
                .clone()
                .unwrap_or_else(|| title_from_path(&name));

            let tags = frontmatter.tags.clone();
            for tag in &tags {
                all_tags.insert(tag.clone());
            }

            summary.imported += 1;
            summary.document_titles.push(title.clone());

            let created_at = crate::parse_date(frontmatter.created.as_deref().unwrap_or(""));
            let updated_at = crate::parse_date(frontmatter.modified.as_deref().unwrap_or(""));

            let doc = ImportedDocument {
                title,
                slug: None,
                content: body.to_string(),
                frontmatter,
                tags,
                source_path: name,
                created_at,
                updated_at,
                extra: std::collections::HashMap::new(),
            };
            documents.push(doc);
        }

        summary.all_tags = all_tags.into_iter().collect();
        summary.all_tags.sort();
        Ok((documents, summary))
    }
}

/// Export markdown documents to a ZIP archive.
pub struct MarkdownZipExporter;

impl MarkdownZipExporter {
    /// Export a single document as a markdown file with frontmatter.
    pub fn document_to_markdown(
        title: &str,
        content: &str,
        tags: &[String],
        description: Option<&str>,
        created_at: Option<chrono::DateTime<chrono::Utc>>,
        updated_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> String {
        let mut fm = Frontmatter::default();
        fm.title = Some(title.to_string());
        fm.description = description.map(|s| s.to_string());
        fm.tags = tags.to_vec();
        if let Some(dt) = created_at {
            fm.created = Some(dt.to_rfc3339());
        }
        if let Some(dt) = updated_at {
            fm.modified = Some(dt.to_rfc3339());
        }

        let frontmatter_block = fm.to_frontmatter_block();
        format!("{}{}", frontmatter_block, content)
    }

    /// Export multiple documents to a ZIP archive (returned as bytes).
    pub fn export_to_bytes(
        documents: &[(&str, &str, &str)], // (title, content, path_in_zip)
    ) -> ImportExportResult<Vec<u8>> {
        let buf = Cursor::new(Vec::new());
        let mut zip = ZipWriter::new(buf);
        let options = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .compression_level(Some(6));

        for (title, content, path) in documents {
            let md_content = Self::document_to_markdown(title, content, &[], None, None, None);
            zip.start_file(path, options.clone())
                .map_err(|e| ImportExportError::zip(e.to_string()))?;
            zip.write_all(md_content.as_bytes())
                .map_err(|e| ImportExportError::zip(e.to_string()))?;
        }

        let buf = zip
            .finish()
            .map_err(|e| ImportExportError::zip(e.to_string()))?;
        Ok(buf.into_inner())
    }

    /// Export multiple documents with metadata to a ZIP archive.
    pub fn export_with_metadata(
        documents: &[ExportDocument],
    ) -> ImportExportResult<(Vec<u8>, ExportSummary)> {
        let buf = Cursor::new(Vec::new());
        let mut zip = ZipWriter::new(buf);
        let options = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .compression_level(Some(6));

        let mut warnings = Vec::new();

        for doc in documents {
            let path = if doc.path.is_empty() {
                let slug = slugify(&doc.title);
                format!("{}.md", slug)
            } else {
                doc.path.clone()
            };

            let md_content = Self::document_to_markdown(
                &doc.title,
                &doc.content,
                &doc.tags,
                doc.description.as_deref(),
                doc.created_at,
                doc.updated_at,
            );

            if md_content.len() > 10_000_000 {
                warnings.push(format!(
                    "Large document: {} ({} bytes)",
                    doc.title,
                    md_content.len()
                ));
            }

            zip.start_file(&path, options.clone())
                .map_err(|e| ImportExportError::zip(e.to_string()))?;
            zip.write_all(md_content.as_bytes())
                .map_err(|e| ImportExportError::zip(e.to_string()))?;
        }

        let buf = zip
            .finish()
            .map_err(|e| ImportExportError::zip(e.to_string()))?;
        let bytes = buf.into_inner();

        let summary = ExportSummary {
            exported: documents.len(),
            format: "markdown-zip".to_string(),
            file_size_bytes: Some(bytes.len() as u64),
            warnings,
        };

        Ok((bytes, summary))
    }
}

/// A document ready for export.
pub struct ExportDocument {
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub description: Option<String>,
    pub path: String,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

// --- Helper functions ---

fn is_markdown_file(name: &str) -> bool {
    name.ends_with(".md") || name.ends_with(".markdown")
}

fn is_hidden(name: &str) -> bool {
    name.contains("/.") || name.starts_with('.')
}

fn title_from_path(path: &str) -> String {
    // Extract filename without extension
    let filename = path.rsplit('/').next().unwrap_or(path);
    let without_ext = filename
        .strip_suffix(".md")
        .or_else(|| filename.strip_suffix(".markdown"))
        .unwrap_or(filename);

    // Convert hyphens/underscores to spaces, title-case
    without_ext
        .replace('-', " ")
        .replace('_', " ")
        .split(' ')
        .filter(|s| !s.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_zip(files: &[(&str, &str)]) -> Vec<u8> {
        let buf = Cursor::new(Vec::new());
        let mut zip = ZipWriter::new(buf);
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);

        for (name, content) in files {
            zip.start_file(name, options.clone()).unwrap();
            zip.write_all(content.as_bytes()).unwrap();
        }

        let buf = zip.finish().unwrap();
        buf.into_inner()
    }

    #[test]
    fn test_import_simple_markdown_zip() {
        let zip_bytes = create_test_zip(&[
            ("doc1.md", "# Doc One\n\nContent here."),
            (
                "doc2.md",
                "---\ntitle: \"Second Doc\"\ntags: [rust, web]\n---\n\n# Second\n\nBody.",
            ),
            ("notes/readme.txt", "Not markdown"),
            ("empty.md", ""),
        ]);

        let (docs, summary) = MarkdownZipImporter::import_documents_from_bytes(&zip_bytes).unwrap();
        assert_eq!(summary.total_files, 4);
        assert_eq!(summary.imported, 2);
        assert_eq!(summary.skipped, 2); // readme.txt + empty.md
        assert_eq!(summary.failed, 0);
        assert_eq!(docs.len(), 2);

        // doc1.md — title derived from filename (no separator in "doc1")
        assert_eq!(docs[0].title, "Doc1");
        assert_eq!(docs[0].source_path, "doc1.md");
        assert!(docs[0].content.contains("Content here"));

        // doc2.md — title from frontmatter
        assert_eq!(docs[1].title, "Second Doc");
        assert_eq!(docs[1].tags, vec!["rust", "web"]);
        assert!(docs[1].content.contains("# Second"));
    }

    #[test]
    fn test_export_to_bytes() {
        let docs = vec![
            ("My Document", "# Hello\n\nWorld", "my-document.md"),
            ("Another Doc", "Plain text", "another-doc.md"),
        ];

        let bytes = MarkdownZipExporter::export_to_bytes(&docs).unwrap();
        assert!(!bytes.is_empty());

        // Verify we can read it back
        let cursor = Cursor::new(bytes);
        let mut archive = ZipArchive::new(cursor).unwrap();
        assert_eq!(archive.len(), 2);

        let mut file = archive.by_name("my-document.md").unwrap();
        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();
        assert!(content.contains("title: My Document"));
        assert!(content.contains("# Hello"));
    }

    #[test]
    fn test_export_with_metadata() {
        let docs = vec![ExportDocument {
            title: "Test Doc".to_string(),
            content: "# Test\n\nBody".to_string(),
            tags: vec!["test".to_string()],
            description: Some("A test".to_string()),
            path: "test.md".to_string(),
            created_at: None,
            updated_at: None,
        }];

        let (bytes, summary) = MarkdownZipExporter::export_with_metadata(&docs).unwrap();
        assert_eq!(summary.exported, 1);
        assert!(summary.file_size_bytes.unwrap() > 0);
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_title_from_path() {
        assert_eq!(title_from_path("my-document.md"), "My Document");
        assert_eq!(title_from_path("sub/dir/API Reference.md"), "API Reference");
        assert_eq!(title_from_path("hello_world.md"), "Hello World");
        assert_eq!(title_from_path("UPPER.md"), "UPPER");
    }

    #[test]
    fn test_roundtrip() {
        let original_docs = vec![("Source Doc", "# Source\n\nOriginal content.", "source.md")];

        let bytes = MarkdownZipExporter::export_to_bytes(&original_docs).unwrap();
        let (imported, _) = MarkdownZipImporter::import_documents_from_bytes(&bytes).unwrap();

        assert_eq!(imported.len(), 1);
        assert_eq!(imported[0].title, "Source Doc");
        assert!(imported[0].content.contains("Original content"));
    }
}
