//! Generic Markdown Vault Importer.
//!
//! A format-agnostic importer that recursively scans a directory (or ZIP archive)
//! of markdown files, extracts YAML frontmatter and inline `#tags`, and
//! produces `ImportedDocument` instances ready for storage.
//!
//! Unlike the Obsidian or Docusaurus importers, this makes no assumptions
//! about wiki-link syntax, callout blocks, or directory conventions. It
//! handles any collection of plain markdown files with optional frontmatter.

use crate::error::{ImportExportError, ImportExportResult};
use crate::frontmatter::Frontmatter;
use crate::{ImportSummary, ImportedDocument, parse_date};
use std::collections::{BTreeMap, HashSet};
use std::path::Path;
use tracing::{debug, info, warn};
use walkdir::WalkDir;

// ============================================================================
// Shared helpers (elevated from obsidian.rs / docusaurus.rs)
// ============================================================================

/// Extensions to skip during vault import.
const SKIP_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "svg", "webp", "ico", "mp3", "mp4", "wav", "pdf", "zip",
];

/// Extract the first `# ` heading from markdown content.
pub fn extract_first_heading(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(stripped) = trimmed.strip_prefix("# ") {
            return Some(stripped.trim().to_string());
        }
        if !trimmed.is_empty() && !trimmed.starts_with('#') {
            break;
        }
    }
    None
}

/// Extract inline `#tags` from markdown content, skipping code blocks and headings.
pub fn extract_inline_tags(content: &str) -> Vec<String> {
    let mut tags = Vec::new();
    let mut in_code_block = false;

    for line in content.lines() {
        if line.trim().starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }
        if in_code_block {
            continue;
        }

        let trimmed = line.trim();
        if trimmed.starts_with('#') && (trimmed.chars().nth(1) == Some(' ')) {
            continue;
        }

        for word in trimmed.split_whitespace() {
            if let Some(tag) = word.strip_prefix('#') {
                let chars: Vec<char> = tag.chars().collect();
                if chars.first().is_some_and(|c| c.is_ascii_alphabetic())
                    && chars
                        .iter()
                        .all(|c| c.is_alphanumeric() || *c == '-' || *c == '/')
                {
                    tags.push(tag.to_string());
                }
            }
        }
    }

    tags
}

/// Derive a human-readable title from a file path (e.g. "my-note.md" -> "My Note").
pub fn title_from_path(path: &str) -> String {
    let filename = path.rsplit('/').next().unwrap_or(path);
    let without_ext = filename
        .strip_suffix(".md")
        .or_else(|| filename.strip_suffix(".markdown"))
        .unwrap_or(filename);

    without_ext
        .replace(['-', '_'], " ")
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

/// Strip UTF-8 BOM and decode bytes to string.
pub fn strip_bom_and_decode(bytes: &[u8]) -> Result<String, String> {
    let without_bom = if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        &bytes[3..]
    } else {
        bytes
    };

    String::from_utf8(without_bom.to_vec()).map_err(|e| format!("UTF-8 decode error: {}", e))
}

/// Check whether a file path should be skipped during import.
pub fn should_skip_path(path: &str) -> bool {
    if let Some(ext) = Path::new(path).extension()
        && SKIP_EXTENSIONS.contains(&ext.to_string_lossy().as_ref())
    {
        return true;
    }

    if path.contains("/.") || path.starts_with('.') {
        return true;
    }

    false
}

// ============================================================================
// MarkdownVaultImporter
// ============================================================================

/// Generic markdown vault importer.
///
/// Supports importing from a local directory or a ZIP archive. Extracts
/// frontmatter, inline tags, and derives titles from filenames or headings.
#[derive(Default)]
pub struct MarkdownVaultImporter {
    /// Whether to derive tags from directory path components.
    pub path_as_tags: bool,
}

impl MarkdownVaultImporter {
    /// Create a new vault importer with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a vault importer that derives tags from directory path components.
    pub fn with_path_tags() -> Self {
        Self { path_as_tags: true }
    }

    /// Import all markdown files from a directory tree.
    ///
    /// Recursively walks the directory, skips non-markdown and hidden files,
    /// and parses each file into an `ImportedDocument`.
    pub fn import_from_dir(
        &self,
        dir: &Path,
    ) -> ImportExportResult<(Vec<ImportedDocument>, ImportSummary)> {
        if !dir.exists() {
            return Err(ImportExportError::io(
                dir,
                format!("Directory not found: {}", dir.display()),
            ));
        }

        if !dir.is_dir() {
            return Err(ImportExportError::Import(format!(
                "Path is not a directory: {}",
                dir.display()
            )));
        }

        info!("Importing markdown vault from: {}", dir.display());

        let mut documents = Vec::new();
        let mut total_files = 0usize;
        let mut skipped = 0usize;
        let mut failed = 0usize;
        let mut all_tags = HashSet::new();
        let warnings = Vec::new();

        for entry in WalkDir::new(dir).follow_links(true).into_iter() {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    warn!("Skipping unreadable entry: {}", e);
                    skipped += 1;
                    continue;
                }
            };

            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let relative = match path.strip_prefix(dir) {
                Ok(r) => r.to_string_lossy().to_string(),
                Err(_) => path.display().to_string(),
            };

            let relative_str = relative.as_str();

            if should_skip_path(relative_str) {
                skipped += 1;
                continue;
            }

            // Only process .md and .markdown files
            let is_markdown = path
                .extension()
                .is_some_and(|ext| ext == "md" || ext == "markdown");

            if !is_markdown {
                skipped += 1;
                continue;
            }

            total_files += 1;

            let bytes = match std::fs::read(path) {
                Ok(b) => b,
                Err(e) => {
                    warn!("Failed to read {}: {}", relative, e);
                    failed += 1;
                    continue;
                }
            };

            match self.parse_markdown_file(&bytes, &relative) {
                Ok(doc) => {
                    for tag in &doc.tags {
                        all_tags.insert(tag.clone());
                    }
                    documents.push(doc);
                }
                Err(e) => {
                    warn!("Failed to parse {}: {}", relative, e);
                    failed += 1;
                }
            }
        }

        let imported = documents.len();
        let document_titles: Vec<String> = documents.iter().map(|d| d.title.clone()).collect();

        let summary = ImportSummary {
            total_files,
            imported,
            skipped,
            failed,
            document_titles,
            all_tags: all_tags.into_iter().collect(),
            warnings,
        };

        info!(
            "Vault import complete: {} imported, {} skipped, {} failed (of {} total)",
            imported, skipped, failed, total_files
        );

        Ok((documents, summary))
    }

    /// Import markdown files from a ZIP archive.
    pub fn import_from_bytes(
        &self,
        zip_bytes: &[u8],
    ) -> ImportExportResult<(Vec<ImportedDocument>, ImportSummary)> {
        let cursor = std::io::Cursor::new(zip_bytes);
        let mut archive = match zip::ZipArchive::new(cursor) {
            Ok(a) => a,
            Err(e) => {
                return Err(ImportExportError::Zip(format!(
                    "Failed to open ZIP archive: {}",
                    e
                )));
            }
        };

        info!(
            "Importing markdown vault from ZIP ({} entries)",
            archive.len()
        );

        let mut documents = Vec::new();
        let mut total_files = 0usize;
        let mut skipped = 0usize;
        let mut failed = 0usize;
        let mut all_tags = HashSet::new();
        let warnings = Vec::new();

        for i in 0..archive.len() {
            let mut entry = match archive.by_index(i) {
                Ok(e) => e,
                Err(_) => continue,
            };

            let name = entry.name().to_string();

            // Skip directories, hidden files, and non-markdown
            if name.ends_with('/') || should_skip_path(&name) {
                skipped += 1;
                continue;
            }

            let is_markdown = Path::new(&name)
                .extension()
                .is_some_and(|ext| ext == "md" || ext == "markdown");

            if !is_markdown {
                skipped += 1;
                continue;
            }

            total_files += 1;

            let mut bytes = Vec::new();
            if let Err(e) = std::io::Read::read_to_end(&mut entry, &mut bytes) {
                warn!("Failed to read ZIP entry {}: {}", name, e);
                failed += 1;
                continue;
            }

            match self.parse_markdown_file(&bytes, &name) {
                Ok(doc) => {
                    for tag in &doc.tags {
                        all_tags.insert(tag.clone());
                    }
                    documents.push(doc);
                }
                Err(e) => {
                    warn!("Failed to parse {}: {}", name, e);
                    failed += 1;
                }
            }
        }

        let imported = documents.len();
        let document_titles: Vec<String> = documents.iter().map(|d| d.title.clone()).collect();

        let summary = ImportSummary {
            total_files,
            imported,
            skipped,
            failed,
            document_titles,
            all_tags: all_tags.into_iter().collect(),
            warnings,
        };

        Ok((documents, summary))
    }

    /// Parse a single markdown file's bytes into an `ImportedDocument`.
    fn parse_markdown_file(
        &self,
        bytes: &[u8],
        path: &str,
    ) -> ImportExportResult<ImportedDocument> {
        let text = strip_bom_and_decode(bytes).map_err(ImportExportError::import)?;

        if text.trim().is_empty() {
            return Err(ImportExportError::Import(format!("Empty file: {}", path)));
        }

        let (mut frontmatter, body) = Frontmatter::parse(&text);

        // Extract all frontmatter fields before consuming the struct
        let fm_title = frontmatter.title.clone();
        let fm_tags = frontmatter.tags.clone();
        let fm_created = frontmatter.created.clone();
        let fm_modified = frontmatter.modified.clone();
        let fm_aliases = frontmatter.aliases.clone();
        let fm_extra = std::mem::take(&mut frontmatter.extra);

        // Title: frontmatter > first heading > filename
        let title = fm_title
            .or_else(|| extract_first_heading(body))
            .unwrap_or_else(|| title_from_path(path));

        // Slug: frontmatter slug > derived from title
        let slug = fm_extra
            .get("slug")
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .filter(|s| !s.is_empty());

        // Tags: merge frontmatter tags + inline #tags + optional path-derived tags
        let mut tags: HashSet<String> = fm_tags.iter().cloned().collect();
        tags.extend(extract_inline_tags(body));

        if self.path_as_tags {
            let parent = Path::new(path)
                .parent()
                .and_then(|p| p.to_str())
                .unwrap_or("");
            for component in parent.split('/') {
                if !component.is_empty() && component != "." {
                    tags.insert(component.to_string());
                }
            }
        }

        // Dates from frontmatter
        let created_at = fm_created.as_deref().and_then(parse_date);
        let updated_at = fm_modified.as_deref().and_then(parse_date);

        // Extra metadata from frontmatter
        let mut extra: BTreeMap<String, serde_json::Value> = fm_extra
            .into_iter()
            .filter_map(|(k, v)| {
                // Skip fields already mapped to dedicated struct fields
                if matches!(
                    k.as_str(),
                    "slug"
                        | "title"
                        | "tags"
                        | "description"
                        | "created"
                        | "modified"
                        | "aliases"
                        | "cssclass"
                        | "category"
                        | "template"
                        | "author"
                        | "status"
                        | "visibility"
                ) {
                    None
                } else {
                    serde_json::to_value(v).ok().map(|val| (k, val))
                }
            })
            .collect();

        // Store aliases if present
        if !fm_aliases.is_empty() {
            extra.insert("aliases".to_string(), serde_json::json!(fm_aliases));
        }

        debug!(
            "Parsed document: '{}' (tags: {}, source: {})",
            title,
            tags.len(),
            path
        );

        Ok(ImportedDocument {
            title,
            slug,
            content: body.to_string(),
            frontmatter,
            tags: tags.into_iter().collect(),
            source_path: path.to_string(),
            created_at,
            updated_at,
            extra,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn create_test_vault(files: &[(&str, &str)]) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        for (path, content) in files {
            let full_path = dir.path().join(path);
            if let Some(parent) = full_path.parent() {
                std::fs::create_dir_all(parent).unwrap();
            }
            std::fs::write(full_path, content).unwrap();
        }
        dir
    }

    fn create_test_zip(files: &[(&str, &str)]) -> Vec<u8> {
        let buf = std::io::Cursor::new(Vec::new());
        let mut zip = zip::ZipWriter::new(buf);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);

        for (name, content) in files {
            zip.start_file(name, options).unwrap();
            zip.write_all(content.as_bytes()).unwrap();
        }

        zip.finish().unwrap().into_inner()
    }

    #[test]
    fn test_extract_first_heading() {
        assert_eq!(
            extract_first_heading("# Hello\nWorld"),
            Some("Hello".to_string())
        );
        assert_eq!(extract_first_heading("No heading here"), None);
        assert_eq!(
            extract_first_heading("\n\n## Sub heading\n"),
            None // First heading must be h1
        );
    }

    #[test]
    fn test_extract_inline_tags() {
        let content = "# Title\n\nSome text with #rust and #web-dev tags.\n```rust\n# not a tag\n```\nMore #testing content.";
        let tags = extract_inline_tags(content);
        assert!(tags.contains(&"rust".to_string()));
        assert!(tags.contains(&"web-dev".to_string()));
        assert!(tags.contains(&"testing".to_string()));
        assert_eq!(tags.len(), 3);
    }

    #[test]
    fn test_title_from_path() {
        assert_eq!(title_from_path("my-note.md"), "My Note");
        assert_eq!(title_from_path("path/to/api_reference.md"), "Api Reference");
        assert_eq!(title_from_path("README.markdown"), "README");
    }

    #[test]
    fn test_should_skip_path() {
        assert!(should_skip_path("image.png"));
        assert!(should_skip_path(".hidden/file.md"));
        assert!(should_skip_path("dir/.git/config"));
        assert!(!should_skip_path("notes/doc.md"));
        assert!(!should_skip_path("README.md"));
    }

    #[test]
    fn test_strip_bom() {
        let mut with_bom = vec![0xEF, 0xBB, 0xBF];
        with_bom.extend_from_slice(b"hello");
        assert_eq!(strip_bom_and_decode(&with_bom).unwrap(), "hello");
        assert_eq!(strip_bom_and_decode(b"hello").unwrap(), "hello");
    }

    #[test]
    fn test_vault_import_from_dir() {
        let dir = create_test_vault(&[
            (
                "doc1.md",
                "# Document One\nContent here with #tag1 and #more.",
            ),
            (
                "sub/doc2.md",
                "---\ntitle: Second Doc\ntags:\n  - alpha\n  - beta\n---\nBody.",
            ),
            ("image.png", "not markdown"),
            ("empty.md", ""),
        ]);

        let importer = MarkdownVaultImporter::new();
        let (docs, summary) = importer.import_from_dir(dir.path()).unwrap();

        assert_eq!(summary.imported, 2);
        assert_eq!(summary.skipped, 1); // image.png
        assert_eq!(summary.failed, 1); // empty.md
        assert_eq!(docs.len(), 2);

        let titles: Vec<&str> = docs.iter().map(|d| d.title.as_str()).collect();
        assert!(titles.contains(&"Document One"));
        assert!(titles.contains(&"Second Doc"));

        let doc_one = docs.iter().find(|d| d.title == "Document One").unwrap();
        assert!(doc_one.tags.contains(&"tag1".to_string()));

        let doc_two = docs.iter().find(|d| d.title == "Second Doc").unwrap();
        assert!(doc_two.tags.contains(&"alpha".to_string()));
    }

    #[test]
    fn test_vault_import_from_zip() {
        let zip_bytes = create_test_zip(&[
            ("notes/readme.md", "# Readme\nWelcome to the vault."),
            (
                "notes/daily.md",
                "---\ntitle: Daily Note\ntags: [journal, log]\n---\nEntry.",
            ),
        ]);

        let importer = MarkdownVaultImporter::new();
        let (docs, summary) = importer.import_from_bytes(&zip_bytes).unwrap();

        assert_eq!(summary.imported, 2);
        assert_eq!(docs[1].title, "Daily Note");
        assert!(docs[1].tags.contains(&"journal".to_string()));
    }

    #[test]
    fn test_vault_import_with_path_tags() {
        let dir = create_test_vault(&[("engineering/rust/guide.md", "# Rust Guide\nContent.")]);

        let importer = MarkdownVaultImporter::with_path_tags();
        let (docs, _summary) = importer.import_from_dir(dir.path()).unwrap();

        assert_eq!(docs.len(), 1);
        assert!(docs[0].tags.contains(&"engineering".to_string()));
        assert!(docs[0].tags.contains(&"rust".to_string()));
    }

    #[test]
    fn test_vault_import_nonexistent_dir() {
        let importer = MarkdownVaultImporter::new();
        let result = importer.import_from_dir(Path::new("/nonexistent/path"));
        assert!(result.is_err());
    }
}
