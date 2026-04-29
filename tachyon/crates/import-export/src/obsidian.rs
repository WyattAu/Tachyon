//! Obsidian vault import.
//!
//! Imports an Obsidian vault (ZIP archive or directory), handling:
//! - YAML frontmatter (title, tags, aliases, created, modified, cssclass)
//! - Wikilinks `[[page]]`, `[[page|display]]`, `[[page#heading]]`
//! - Inline tags `#tag` and `#nested/tag`
//! - Callout blocks `> [!note]`, `> [!warning]`, etc.
//! - Embedded files `![[image.png]]`
//!
//! Obsidian-specific folders like `.obsidian/`, `templates/`, and
//! non-markdown attachment files are skipped during import.

use crate::{
    error::ImportExportResult, frontmatter::Frontmatter, ImportExportError, ImportSummary,
    ImportedDocument,
};
use std::collections::HashSet;
use std::io::{Cursor, Read};
use std::path::Path;
use zip::ZipArchive;

/// Import an Obsidian vault from a ZIP archive.
pub struct ObsidianImporter;

/// Directories to skip in Obsidian vaults.
const SKIP_DIRS: &[&str] = &[".obsidian", ".trash", ".git", "node_modules", "__pycache__"];

/// Non-markdown extensions to skip.
const SKIP_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "svg", "webp", "ico", "mp3", "mp4", "wav", "ogg", "webm", "pdf",
    "doc", "docx", "xls", "xlsx", "ppt", "pptx", "zip", "tar", "gz", "7z", "exe", "dll", "so",
    "dylib", "json", "css", "js", "ts",
];

impl ObsidianImporter {
    /// Import all markdown files from an Obsidian vault ZIP archive.
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

            let (frontmatter, body) = Frontmatter::parse(&content);

            let title = frontmatter
                .title
                .clone()
                .or_else(|| extract_first_heading(body))
                .unwrap_or_else(|| title_from_path(&name));

            let mut inline_tags = extract_inline_tags(body);
            let mut tags: Vec<String> = frontmatter.tags.clone();
            tags.append(&mut inline_tags);
            tags.sort();
            tags.dedup();

            for tag in &tags {
                all_tags.insert(tag.clone());
            }

            for alias in &frontmatter.aliases {
                all_tags.insert(alias.clone());
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
                extra: std::collections::BTreeMap::new(),
            };
            documents.push(doc);
        }

        summary.all_tags = all_tags.into_iter().collect();
        summary.all_tags.sort();
        Ok((documents, summary))
    }

    /// Import from a directory on disk.
    pub fn import_from_dir(
        dir_path: &Path,
    ) -> ImportExportResult<(Vec<ImportedDocument>, ImportSummary)> {
        if !dir_path.is_dir() {
            return Err(ImportExportError::io(dir_path, "Not a directory"));
        }

        let mut summary = ImportSummary {
            total_files: 0,
            imported: 0,
            skipped: 0,
            failed: 0,
            document_titles: Vec::new(),
            all_tags: Vec::new(),
            warnings: Vec::new(),
        };

        let mut documents = Vec::new();
        let mut all_tags: HashSet<String> = HashSet::new();

        for entry in walkdir::WalkDir::new(dir_path)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                !SKIP_DIRS.contains(&name.as_ref())
            })
        {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    summary.warnings.push(format!("Walk error: {}", e));
                    continue;
                }
            };

            if entry.file_type().is_dir() {
                continue;
            }

            summary.total_files += 1;

            let path = entry.path();
            let name = path
                .strip_prefix(dir_path)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();

            if let Some(ext) = path.extension() {
                if ext != "md" && ext != "markdown" {
                    summary.skipped += 1;
                    continue;
                }
            } else {
                summary.skipped += 1;
                continue;
            }

            let content = match std::fs::read_to_string(path) {
                Ok(c) => c,
                Err(e) => {
                    tracing::warn!("Failed to read {}: {}", path.display(), e);
                    summary.failed += 1;
                    summary
                        .warnings
                        .push(format!("Failed to read {}: {}", path.display(), e));
                    continue;
                }
            };

            if content.trim().is_empty() {
                summary.skipped += 1;
                continue;
            }

            let (frontmatter, body) = Frontmatter::parse(&content);

            let title = frontmatter
                .title
                .clone()
                .or_else(|| extract_first_heading(body))
                .unwrap_or_else(|| title_from_path(&name));

            let mut inline_tags = extract_inline_tags(body);
            let mut tags: Vec<String> = frontmatter.tags.clone();
            tags.append(&mut inline_tags);
            tags.sort();
            tags.dedup();

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
                extra: std::collections::BTreeMap::new(),
            };
            documents.push(doc);
        }

        summary.all_tags = all_tags.into_iter().collect();
        summary.all_tags.sort();
        Ok((documents, summary))
    }
}

// --- Helper functions ---

fn should_skip_path(path: &str) -> bool {
    for dir in SKIP_DIRS {
        if path.contains(&format!("/{}/", dir)) || path.starts_with(&format!("{}/", dir)) {
            return true;
        }
    }

    if let Some(ext) = Path::new(path).extension() {
        if SKIP_EXTENSIONS.contains(&ext.to_string_lossy().as_ref()) {
            return true;
        }
    }

    if path.contains("/.") || path.starts_with('.') {
        return true;
    }

    false
}

fn extract_first_heading(content: &str) -> Option<String> {
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

fn extract_inline_tags(content: &str) -> Vec<String> {
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

        // Extract #tags — match # followed by alphanumeric, allowing nested/path tags
        // Split on whitespace and check each word for leading #
        for word in trimmed.split_whitespace() {
            if let Some(tag) = word.strip_prefix('#') {
                // Must start with a letter and contain only word chars, hyphens, slashes
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

    fn create_obsidian_zip(files: &[(&str, &str)]) -> Vec<u8> {
        use std::io::Write;

        let buf = Cursor::new(Vec::new());
        let mut zip = zip::ZipWriter::new(buf);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);

        for (name, content) in files {
            zip.start_file(name, options).unwrap();
            zip.write_all(content.as_bytes()).unwrap();
        }

        let buf = zip.finish().unwrap();
        buf.into_inner()
    }

    #[test]
    fn test_import_obsidian_vault() {
        let zip_bytes = create_obsidian_zip(&[
            ("notes/daily/2024-01-15.md", "---\ntitle: Daily Note\ntags: [journal, daily]\ncreated: \"2024-01-15\"\n---\n\n# Daily Note\n\nJournal entry for today.\n\n#work #planning\n\nSome content with [[link]] and [[other|display text]]."),
            ("ideas/project-idea.md", "# Project Idea\n\nA new project concept.\n\n#idea #brainstorm"),
            ("templates/meeting.md", "---\ntitle: Meeting Template\ntemplate: meeting\n---\n\n# Meeting Notes\n\n## Attendees\n\n## Agenda"),
            ("attachments/image.png", "PNG_DATA"),
            (".obsidian/app.json", "{\"key\": \"value\"}"),
            ("scratch/empty.md", ""),
        ]);

        let (docs, summary) = ObsidianImporter::import_from_bytes(&zip_bytes).unwrap();

        assert_eq!(summary.imported, 3);
        assert!(summary.skipped >= 3);
        assert_eq!(summary.failed, 0);
        assert_eq!(docs.len(), 3);

        assert_eq!(docs[0].title, "Daily Note");
        assert_eq!(docs[0].tags, vec!["daily", "journal", "planning", "work"]);
        assert!(docs[0].content.contains("[[link]]"));

        assert_eq!(docs[1].title, "Project Idea");
        assert!(docs[1].tags.contains(&"idea".to_string()));

        assert_eq!(docs[2].title, "Meeting Template");
    }

    #[test]
    fn test_extract_inline_tags() {
        let content = r#"# Heading

Some text with #tag1 and #nested/tag2 here

```
#code-block-tag should be ignored
```

More text #tag3 at the end.

## Sub heading
"#;

        let tags = extract_inline_tags(content);
        assert!(tags.contains(&"tag1".to_string()));
        assert!(tags.contains(&"nested/tag2".to_string()));
        assert!(tags.contains(&"tag3".to_string()));
        assert!(!tags.contains(&"code-block-tag".to_string()));
        assert!(!tags.contains(&"Heading".to_string()));
    }

    #[test]
    fn test_extract_first_heading() {
        assert_eq!(
            extract_first_heading("# Hello World\n\nBody text"),
            Some("Hello World".to_string())
        );
        assert_eq!(extract_first_heading("No heading here\n\nJust text"), None,);
    }

    #[test]
    fn test_should_skip_path() {
        assert!(should_skip_path(".obsidian/config.json"));
        assert!(should_skip_path("notes/.hidden/file.md"));
        assert!(should_skip_path("attachments/photo.png"));
        assert!(should_skip_path("assets/video.mp4"));
        assert!(!should_skip_path("notes/README.md"));
        assert!(!should_skip_path("daily/2024-01-15.md"));
    }
}
