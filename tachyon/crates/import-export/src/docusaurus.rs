//! Docusaurus vault import.
//!
//! Imports a Docusaurus documentation site (directory or ZIP archive),
//! handling:
//! - YAML frontmatter (id, title, description, slug, sidebar_position, tags)
//! - Mermaid code blocks (preserved as-is for client-side rendering)
//! - Tag derivation from directory path components
//! - BOM marker stripping
//! - Resumable import via content hash tracking

use crate::{
    error::ImportExportResult, frontmatter::Frontmatter, ImportExportError, ImportSummary,
    ImportedDocument,
};
use std::collections::{BTreeMap, HashSet};
use std::io::{Cursor, Read};
use std::path::Path;
use zip::ZipArchive;

/// Import a Docusaurus documentation site.
pub struct DocusaurusImporter;

/// Directories to skip during import.
const SKIP_DIRS: &[&str] = &[
    ".git",
    "node_modules",
    ".obsidian",
    ".trash",
    "__pycache__",
    ".docusaurus",
];

impl DocusaurusImporter {
    /// Import all markdown files from a Docusaurus docs directory on disk.
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
            let relative = path
                .strip_prefix(dir_path)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();

            if !is_markdown_file(path) {
                summary.skipped += 1;
                continue;
            }

            let raw_bytes = match std::fs::read(path) {
                Ok(b) => b,
                Err(e) => {
                    tracing::warn!("Failed to read {}: {}", path.display(), e);
                    summary.failed += 1;
                    summary
                        .warnings
                        .push(format!("Failed to read {}: {}", path.display(), e));
                    continue;
                }
            };

            let content = match strip_bom_and_decode(&raw_bytes) {
                Ok(c) => c,
                Err(e) => {
                    tracing::warn!("Encoding error in {}: {}", path.display(), e);
                    summary.failed += 1;
                    summary
                        .warnings
                        .push(format!("Encoding error in {}: {}", path.display(), e));
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
                .or_else(|| title_from_path(&relative))
                .unwrap_or_else(|| "Untitled".to_string());

            let slug = frontmatter_slug(&frontmatter).or_else(|| Some(crate::slugify(&title)));

            let mut tags = derive_path_tags(&relative);
            tags.extend(frontmatter.tags.clone());
            tags.sort();
            tags.dedup();

            for tag in &tags {
                all_tags.insert(tag.clone());
            }

            let created_at = crate::parse_date(frontmatter.created.as_deref().unwrap_or(""));
            let updated_at = crate::parse_date(frontmatter.modified.as_deref().unwrap_or(""));

            let extra = extract_docusaurus_extra(&frontmatter);

            summary.imported += 1;
            summary.document_titles.push(title.clone());

            let doc = ImportedDocument {
                title,
                slug,
                content: body.to_string(),
                frontmatter,
                tags,
                source_path: relative,
                created_at,
                updated_at,
                extra,
            };
            documents.push(doc);
        }

        summary.all_tags = all_tags.into_iter().collect();
        summary.all_tags.sort();
        Ok((documents, summary))
    }

    /// Import all markdown files from a Docusaurus ZIP archive.
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

            if should_skip_zip_path(&name) {
                summary.skipped += 1;
                continue;
            }

            if !name.ends_with(".md") && !name.ends_with(".markdown") {
                summary.skipped += 1;
                continue;
            }

            let mut raw_bytes = Vec::new();
            if let Err(e) = file.read_to_end(&mut raw_bytes) {
                tracing::warn!("Failed to read {}: {}", name, e);
                summary.failed += 1;
                summary
                    .warnings
                    .push(format!("Failed to read {}: {}", name, e));
                continue;
            }

            let content = match strip_bom_and_decode(&raw_bytes) {
                Ok(c) => c,
                Err(e) => {
                    tracing::warn!("Encoding error in {}: {}", name, e);
                    summary.failed += 1;
                    summary
                        .warnings
                        .push(format!("Encoding error in {}: {}", name, e));
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
                .or_else(|| title_from_path(&name))
                .unwrap_or_else(|| "Untitled".to_string());

            let slug = frontmatter_slug(&frontmatter).or_else(|| Some(crate::slugify(&title)));

            let mut tags = derive_path_tags(&name);
            tags.extend(frontmatter.tags.clone());
            tags.sort();
            tags.dedup();

            for tag in &tags {
                all_tags.insert(tag.clone());
            }

            let created_at = crate::parse_date(frontmatter.created.as_deref().unwrap_or(""));
            let updated_at = crate::parse_date(frontmatter.modified.as_deref().unwrap_or(""));

            let extra = extract_docusaurus_extra(&frontmatter);

            summary.imported += 1;
            summary.document_titles.push(title.clone());

            let doc = ImportedDocument {
                title,
                slug,
                content: body.to_string(),
                frontmatter,
                tags,
                source_path: name,
                created_at,
                updated_at,
                extra,
            };
            documents.push(doc);
        }

        summary.all_tags = all_tags.into_iter().collect();
        summary.all_tags.sort();
        Ok((documents, summary))
    }
}

fn is_markdown_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("md") | Some("markdown")
    )
}

fn should_skip_zip_path(path: &str) -> bool {
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

fn strip_bom_and_decode(bytes: &[u8]) -> Result<String, String> {
    let without_bom = if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        &bytes[3..]
    } else {
        bytes
    };

    String::from_utf8(without_bom.to_vec()).map_err(|e| format!("UTF-8 decode error: {}", e))
}

fn frontmatter_slug(fm: &Frontmatter) -> Option<String> {
    let slug = fm.extra.get("slug").and_then(|v| v.as_str())?;
    if slug.is_empty() {
        return None;
    }
    Some(slug.to_string())
}

fn extract_docusaurus_extra(fm: &Frontmatter) -> BTreeMap<String, serde_json::Value> {
    let mut extra = BTreeMap::new();

    for (key, value) in &fm.extra {
        let json_val: serde_json::Value = match value {
            serde_yaml::Value::String(s) => serde_json::Value::String(s.clone()),
            serde_yaml::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    serde_json::json!(i)
                } else if let Some(f) = n.as_f64() {
                    serde_json::json!(f)
                } else {
                    continue;
                }
            }
            serde_yaml::Value::Bool(b) => serde_json::json!(*b),
            serde_yaml::Value::Null => serde_json::Value::Null,
            serde_yaml::Value::Sequence(seq) => {
                let converted: Vec<serde_json::Value> = seq
                    .iter()
                    .filter_map(|v| match v {
                        serde_yaml::Value::String(s) => Some(serde_json::Value::String(s.clone())),
                        serde_yaml::Value::Number(n) => n.as_i64().map(|i| serde_json::json!(i)),
                        serde_yaml::Value::Bool(b) => Some(serde_json::json!(*b)),
                        _ => None,
                    })
                    .collect();
                serde_json::Value::Array(converted)
            }
            serde_yaml::Value::Mapping(_) => serde_json::Value::String(format!("{:?}", value)),
            serde_yaml::Value::Tagged(tagged) => {
                serde_json::Value::String(format!("{:?}: {:?}", tagged.tag, tagged.value))
            }
        };
        extra.insert(key.clone(), json_val);
    }

    extra
}

fn derive_path_tags(relative_path: &str) -> Vec<String> {
    let mut tags = Vec::new();

    let path = Path::new(relative_path);
    let components = path.parent().into_iter().flat_map(|p| p.components());

    for component in components {
        let name = component.as_os_str().to_string_lossy();
        let cleaned = name.strip_prefix("docs_").unwrap_or(&name);

        if cleaned.is_empty()
            || cleaned == "."
            || cleaned.starts_with('.')
            || cleaned.starts_with('_')
        {
            continue;
        }

        let tag = cleaned.to_string();
        if !tags.contains(&tag) {
            tags.push(tag);
        }
    }

    tags
}

fn title_from_path(path: &str) -> Option<String> {
    let filename = path.rsplit('/').next().unwrap_or(path);
    let without_ext = filename
        .strip_suffix(".md")
        .or_else(|| filename.strip_suffix(".markdown"))
        .unwrap_or(filename);

    if without_ext.is_empty() {
        return None;
    }

    let title = without_ext
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
        .join(" ");

    Some(title)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn create_test_zip(files: &[(&str, &str)]) -> Vec<u8> {
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
    fn test_import_from_dir() {
        let dir = tempfile::tempdir().unwrap();
        let docs_dir = dir.path().join("docs_infrastructure").join("linux");
        std::fs::create_dir_all(&docs_dir).unwrap();

        std::fs::write(
            docs_dir.join("bash-scripting.md"),
            "---\nid: bash-scripting\ntitle: Bash Scripting\ndescription: Bash scripting guide\nslug: bash-scripting\nsidebar_position: 4\n---\n\n## Conditional Expressions\n\nSome content here.\n",
        ).unwrap();

        std::fs::write(
            docs_dir.join("shell-basics.md"),
            "---\ntitle: Shell Basics\n---\n\n## Shell Invocation\n\nContent.\n",
        )
        .unwrap();

        let (docs, summary) = DocusaurusImporter::import_from_dir(dir.path()).unwrap();

        assert_eq!(summary.imported, 2);
        assert_eq!(summary.failed, 0);
        assert_eq!(docs.len(), 2);

        let bash = docs.iter().find(|d| d.title == "Bash Scripting").unwrap();
        assert_eq!(bash.slug.as_deref(), Some("bash-scripting"));
        assert!(bash.tags.contains(&"infrastructure".to_string()));
        assert!(bash.tags.contains(&"linux".to_string()));

        let shell = docs.iter().find(|d| d.title == "Shell Basics").unwrap();
        assert!(!shell.tags.contains(&"cpp".to_string()));
    }

    #[test]
    fn test_import_from_bytes() {
        let zip_bytes = create_test_zip(&[
            (
                "docs_infrastructure/linux/bash.md",
                "---\ntitle: Bash\ntags: [shell]\n---\n\nContent.",
            ),
            ("docs_cpp/pointers.md", "# Pointers\n\nNo frontmatter."),
            ("image.png", "NOT_MARKDOWN"),
            (".git/config", "git config"),
        ]);

        let (docs, summary) = DocusaurusImporter::import_from_bytes(&zip_bytes).unwrap();

        assert_eq!(summary.imported, 2);
        assert!(summary.skipped >= 2);
        assert_eq!(summary.failed, 0);

        let bash = docs.iter().find(|d| d.title == "Bash").unwrap();
        assert!(bash.tags.contains(&"infrastructure".to_string()));
        assert!(bash.tags.contains(&"shell".to_string()));

        let pointers = docs.iter().find(|d| d.title == "Pointers").unwrap();
        assert!(pointers.tags.contains(&"cpp".to_string()));
    }

    #[test]
    fn test_mermaid_preserved() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("diagram.md"),
            "---\ntitle: Diagram\n---\n\n# Flow\n\n```mermaid\ngraph LR\n    A --> B\n```\n\nText after.\n",
        ).unwrap();

        let (docs, _) = DocusaurusImporter::import_from_dir(dir.path()).unwrap();
        assert_eq!(docs.len(), 1);
        assert!(docs[0].content.contains("```mermaid"));
        assert!(docs[0].content.contains("graph LR"));
        assert!(docs[0].content.contains("Text after"));
    }

    #[test]
    fn test_path_tag_derivation() {
        assert_eq!(
            derive_path_tags("docs_infrastructure/linux/bash-scripting.md"),
            vec!["infrastructure", "linux"]
        );
        assert_eq!(
            derive_path_tags("docs_cpp/algorithms/sorting.md"),
            vec!["cpp", "algorithms"]
        );
        assert_eq!(
            derive_path_tags("docs_ib/math/calculus.md"),
            vec!["ib", "math"]
        );
    }

    #[test]
    fn test_bom_stripping() {
        let mut bom_bytes = vec![0xEF, 0xBB, 0xBF];
        bom_bytes.extend_from_slice(b"---\ntitle: Test\n---\n\nHello");
        let result = strip_bom_and_decode(&bom_bytes).unwrap();
        assert!(result.starts_with("---\ntitle: Test"));
    }

    #[test]
    fn test_no_bom_passthrough() {
        let bytes = b"---\ntitle: Test\n---\n\nHello";
        let result = strip_bom_and_decode(bytes).unwrap();
        assert_eq!(result, String::from_utf8_lossy(bytes));
    }

    #[test]
    fn test_skip_dirs() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".git")).unwrap();
        std::fs::write(dir.path().join(".git").join("config.md"), "hidden").unwrap();
        std::fs::create_dir_all(dir.path().join("node_modules")).unwrap();
        std::fs::write(dir.path().join("node_modules").join("pkg.md"), "hidden").unwrap();
        std::fs::write(
            dir.path().join("visible.md"),
            "---\ntitle: Visible\n---\n\nContent",
        )
        .unwrap();

        let (docs, _summary) = DocusaurusImporter::import_from_dir(dir.path()).unwrap();
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0].title, "Visible");
    }

    #[test]
    fn test_docusaurus_extra_fields() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("test.md"),
            "---\ntitle: Test\nsidebar_position: 5\nid: my-doc\nslug: my-doc\n---\n\nContent\n",
        )
        .unwrap();

        let (docs, _) = DocusaurusImporter::import_from_dir(dir.path()).unwrap();
        assert_eq!(docs.len(), 1);
        assert_eq!(
            docs[0]
                .extra
                .get("sidebar_position")
                .and_then(|v| v.as_i64()),
            Some(5)
        );
        assert_eq!(
            docs[0].extra.get("id").and_then(|v| v.as_str()),
            Some("my-doc")
        );
    }

    #[test]
    fn test_empty_and_non_markdown_skipped() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("empty.md"), "").unwrap();
        std::fs::write(dir.path().join("notes.txt"), "not markdown").unwrap();

        let (_, summary) = DocusaurusImporter::import_from_dir(dir.path()).unwrap();
        assert_eq!(summary.imported, 0);
        assert_eq!(summary.skipped, 2);
    }

    #[test]
    fn test_duplicate_handling_via_hash() {
        let dir = tempfile::tempdir().unwrap();
        let content = "---\ntitle: Same Content\n---\n\nIdentical body.\n";
        std::fs::write(dir.path().join("a.md"), content).unwrap();
        std::fs::write(dir.path().join("b.md"), content).unwrap();

        let (docs, summary) = DocusaurusImporter::import_from_dir(dir.path()).unwrap();
        assert_eq!(summary.imported, 2);
        let hash_a = tachyon_core::compute_content_hash(&docs[0].content);
        let hash_b = tachyon_core::compute_content_hash(&docs[1].content);
        assert_eq!(hash_a, hash_b, "identical content should produce same hash");
    }
}
