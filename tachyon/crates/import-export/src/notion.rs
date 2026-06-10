//! Notion export importer.
//!
//! Imports a Notion workspace export (ZIP archive), handling:
//! - Markdown page content extracted from the export
//! - Notion database properties mapped to Tachyon tags
//! - Nested page hierarchy preserved as source_path tree
//! - CSV metadata files for database pages
//!
//! Notion exports typically contain:
//! - `export/` directory with markdown files
//! - `.csv` files for database content
//! - Nested directories matching the page tree

use crate::{
    ImportExportError, ImportSummary, ImportedDocument, error::ImportExportResult,
    frontmatter::Frontmatter,
};
use std::collections::{BTreeMap, HashSet};
use std::io::{Cursor, Read};
use std::path::Path;
use zip::ZipArchive;

/// Import Notion workspace exports.
pub struct NotionImporter;

/// Directories to skip in Notion exports.
const SKIP_DIRS: &[&str] = &[".git", "node_modules"];

impl NotionImporter {
    /// Import all pages from a Notion export ZIP archive.
    pub fn import_from_bytes(
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

        // Notion exports have a flat structure with markdown files and optional CSV metadata.
        // Each .md file is a page; CSV files contain database properties.
        let mut csv_data: BTreeMap<String, String> = BTreeMap::new();

        // First pass: collect CSV metadata
        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .map_err(|e| ImportExportError::zip(e.to_string()))?;

            let name = file.name().to_string();

            if name.ends_with(".csv") {
                let mut content = String::new();
                if file.read_to_string(&mut content).is_ok() {
                    // Store CSV keyed by filename (without extension) for property lookup
                    let key = Path::new(&name)
                        .file_stem()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    csv_data.insert(key, content);
                }
            }
        }

        // Second pass: import markdown pages
        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .map_err(|e| ImportExportError::zip(e.to_string()))?;

            let name = file.name().to_string();

            if file.is_dir() {
                summary.skipped += 1;
                continue;
            }

            if should_skip_path(&name) {
                summary.skipped += 1;
                continue;
            }

            if !name.ends_with(".md") {
                summary.skipped += 1;
                continue;
            }

            let mut content = String::new();
            if let Err(e) = file.read_to_string(&mut content) {
                tracing::warn!("Failed to read {}: {}", name, e);
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

            match parse_notion_page(&content, &name, &csv_data) {
                Ok(doc) => {
                    for tag in &doc.tags {
                        all_tags.insert(tag.clone());
                    }
                    summary.imported += 1;
                    summary.document_titles.push(doc.title.clone());
                    documents.push(doc);
                }
                Err(e) => {
                    tracing::warn!("Failed to parse {}: {}", name, e);
                    summary.failed += 1;
                    summary
                        .warnings
                        .push(format!("Failed to parse {}: {}", name, e));
                }
            }
        }

        summary.all_tags = all_tags.into_iter().collect();
        summary.all_tags.sort();
        Ok((documents, summary))
    }
}

/// Parse a single Notion markdown page into an ImportedDocument.
fn parse_notion_page(
    content: &str,
    source_path: &str,
    _csv_data: &BTreeMap<String, String>,
) -> ImportExportResult<ImportedDocument> {
    // Notion pages may have frontmatter-like metadata at the top,
    // but typically they're just raw markdown with some Notion-specific syntax.
    let (frontmatter, body) = Frontmatter::parse(content);

    // Notion embeds page properties as inline metadata lines like:
    //   Created: 2024-01-15
    //   Tags: rust, web
    // We extract these from the body if they exist before frontmatter.
    let (properties, clean_body) = extract_notion_properties(body);

    let title = frontmatter
        .title
        .clone()
        .or_else(|| crate::vault_importer::extract_first_heading(clean_body))
        .or_else(|| extract_title_from_path(source_path))
        .unwrap_or_else(|| title_from_path(source_path));

    // Derive tags from frontmatter + Notion properties + path components + inline tags
    let mut tags: Vec<String> = frontmatter.tags.clone();

    // Extract inline tags from body
    let mut inline_tags = crate::vault_importer::extract_inline_tags(clean_body);
    tags.append(&mut inline_tags);

    // Extract tags from Notion database properties
    if let Some(notion_tags) = properties.get("Tags") {
        tags.extend(
            notion_tags
                .split(',')
                .map(|s| s.trim().to_lowercase())
                .filter(|s| !s.is_empty()),
        );
    }
    if let Some(category) = properties.get("Category")
        && !category.is_empty()
    {
        tags.push(category.trim().to_lowercase());
    }

    // Add path-derived tags
    let path_tags = crate::obsidian::infer_tags_from_path(source_path);
    tags.extend(path_tags);

    tags.sort();
    tags.dedup();

    // Extract dates from properties or frontmatter
    let created_at = properties
        .get("Created")
        .and_then(|s| crate::parse_date(s))
        .or_else(|| frontmatter.created.as_deref().and_then(crate::parse_date));
    let updated_at = properties
        .get("Last edited")
        .and_then(|s| crate::parse_date(s))
        .or_else(|| frontmatter.modified.as_deref().and_then(crate::parse_date));

    let slug = frontmatter
        .extra
        .get("slug")
        .and_then(|v| v.as_str().map(|s| s.to_string()));

    let mut extra: BTreeMap<String, serde_json::Value> = BTreeMap::new();
    // Store all Notion properties as extra metadata
    for (key, value) in &properties {
        extra.insert(key.clone(), serde_json::Value::String(value.clone()));
    }

    Ok(ImportedDocument {
        title,
        slug,
        content: clean_body.to_string(),
        frontmatter,
        tags,
        source_path: source_path.to_string(),
        created_at,
        updated_at,
        extra,
    })
}

/// Extract Notion page properties from the first few lines of content.
///
/// Notion exports often have lines like:
/// ```text
/// Created: 2024-01-15
/// Tags: rust, web
/// Status: Published
/// ```
///
/// Returns (properties_map, content_without_properties).
fn extract_notion_properties(content: &str) -> (BTreeMap<String, String>, &str) {
    let mut props = BTreeMap::new();
    let mut lines_consumed = 0;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            // Empty line signals end of properties block
            break;
        }
        if let Some((key, value)) = trimmed.split_once(':') {
            let key = key.trim();
            let value = value.trim();
            // Only treat as property if key looks like a property name (no spaces, reasonable length)
            if !key.is_empty()
                && key.len() < 50
                && !key.starts_with('#')
                && !key.starts_with('[')
                && !key.starts_with('!')
            {
                props.insert(key.to_string(), value.to_string());
                lines_consumed += 1;
                continue;
            }
        }
        // If we hit a line that doesn't look like a property, stop
        break;
    }

    if lines_consumed > 0 {
        let remaining = content
            .lines()
            .skip(lines_consumed)
            .collect::<Vec<_>>()
            .join("\n");
        (props, Box::leak(remaining.into_boxed_str()))
    } else {
        (props, content)
    }
}

fn should_skip_path(path: &str) -> bool {
    for dir in SKIP_DIRS {
        if path.contains(&format!("/{}/", dir)) || path.starts_with(&format!("{}/", dir)) {
            return true;
        }
    }

    if path.contains("/.") || path.starts_with('.') {
        return true;
    }

    false
}

/// Extract title from a Notion-style path like `My Page 1a2b3c4d5e6f.md`
fn extract_title_from_path(path: &str) -> Option<String> {
    let filename = path.rsplit('/').next()?;
    let without_ext = filename.strip_suffix(".md")?;

    // Notion often appends a UUID-like suffix: "Page Title 1a2b3c4d5e6f"
    // Try to strip it
    let cleaned = strip_notion_id_suffix(without_ext);
    if cleaned.is_empty() {
        return None;
    }

    Some(cleaned.replace(['-', '_'], " "))
}

/// Strip a Notion-style hex ID suffix from a page title.
///
/// e.g., "My Page 1a2b3c4d5e6f" -> "My Page"
fn strip_notion_id_suffix(title: &str) -> String {
    let trimmed = title.trim();
    // Check if the last word is a hex string (8-32 chars)
    if let Some(last_space_idx) = trimmed.rfind(' ') {
        let potential_id = &trimmed[last_space_idx + 1..];
        if potential_id.len() >= 8
            && potential_id.len() <= 32
            && potential_id.chars().all(|c| c.is_ascii_hexdigit())
        {
            return trimmed[..last_space_idx].trim().to_string();
        }
    }
    trimmed.to_string()
}

fn title_from_path(path: &str) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn create_notion_zip(files: &[(&str, &str)]) -> Vec<u8> {
        let buf = Cursor::new(Vec::new());
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
    fn test_import_notion_export() {
        let zip_bytes = create_notion_zip(&[
            (
                "export/My Page 1a2b3c4d.md",
                "# My Page\n\nThis is a Notion page with some content.\n\n#notion #import",
            ),
            (
                "export/Sub Page 5e6f7a8b.md",
                "---\ntitle: Custom Title\ntags:\n  - custom\n  - tag\n---\n\nContent here.",
            ),
            (
                "export/Database Page 12345678.md",
                "Created: 2024-01-15\nTags: rust, web\nCategory: Programming\n\n# Database Entry\n\nPage content.",
            ),
            ("export/image.png", "PNG_DATA"),
            (".git/config", "stuff"),
        ]);

        let (docs, summary) = NotionImporter::import_from_bytes(&zip_bytes).unwrap();

        assert_eq!(summary.imported, 3);
        assert_eq!(docs.len(), 3);

        // First doc: title extracted from path, hex suffix stripped
        assert_eq!(docs[0].title, "My Page");
        assert!(docs[0].tags.contains(&"notion".to_string()));
        assert!(docs[0].tags.contains(&"import".to_string()));

        // Second doc: custom title from frontmatter
        assert_eq!(docs[1].title, "Custom Title");
        assert!(docs[1].tags.contains(&"custom".to_string()));
        assert!(docs[1].tags.contains(&"tag".to_string()));

        // Third doc: Notion properties parsed
        assert_eq!(docs[2].title, "Database Entry");
        assert!(docs[2].tags.contains(&"rust".to_string()));
        assert!(docs[2].tags.contains(&"web".to_string()));
        assert!(docs[2].tags.contains(&"programming".to_string()));
        assert!(docs[2].extra.contains_key("Category"));
    }

    #[test]
    fn test_strip_notion_id_suffix() {
        assert_eq!(strip_notion_id_suffix("My Page 1a2b3c4d5e6f"), "My Page");
        assert_eq!(
            strip_notion_id_suffix("Long Title Here abcdef1234567890abcdef"),
            "Long Title Here"
        );
        // No hex suffix
        assert_eq!(strip_notion_id_suffix("Just a Title"), "Just a Title");
        // Short hex string (not Notion ID)
        assert_eq!(strip_notion_id_suffix("Title abcd"), "Title abcd");
    }

    #[test]
    fn test_extract_notion_properties() {
        let content = "Created: 2024-01-15\nTags: rust, web\nCategory: Programming\n\n# Actual Content\n\nBody text.";
        let (props, body) = extract_notion_properties(content);
        assert_eq!(props.get("Created").unwrap(), "2024-01-15");
        assert_eq!(props.get("Tags").unwrap(), "rust, web");
        assert_eq!(props.get("Category").unwrap(), "Programming");
        assert!(body.contains("# Actual Content"));
    }

    #[test]
    fn test_extract_title_from_path() {
        assert_eq!(
            extract_title_from_path("export/My Page 1a2b3c4d.md"),
            Some("My Page".to_string())
        );
        assert_eq!(
            extract_title_from_path("Simple Title.md"),
            Some("Simple Title".to_string())
        );
        assert_eq!(extract_title_from_path("image.png"), None);
    }

    #[test]
    fn test_should_skip_path() {
        assert!(should_skip_path(".git/config"));
        assert!(should_skip_path("export/.hidden/file.md"));
        assert!(!should_skip_path("export/My Page.md"));
    }
}
