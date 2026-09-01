//! Editor intelligence primitives shared by external editor adapters.
//!
//! This module intentionally contains no transport dependency. VS Code, Neovim,
//! and other clients can use the same indexing and validation behavior through
//! an LSP transport adapter.

use regex::Regex;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorDocument {
    pub path: PathBuf,
    pub title: String,
    pub slug: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorDiagnostic {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Default)]
pub struct VaultIndex {
    documents: BTreeMap<String, EditorDocument>,
}

impl VaultIndex {
    pub fn from_directory(root: &Path) -> std::io::Result<Self> {
        let mut index = Self::default();
        for entry in walkdir::WalkDir::new(root)
            .into_iter()
            .filter_map(Result::ok)
        {
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }
            let slug = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or_default()
                .to_string();
            if slug.is_empty() {
                continue;
            }
            let title = std::fs::read_to_string(path)
                .ok()
                .and_then(|text| {
                    text.lines().find_map(|line| {
                        line.strip_prefix("title:")
                            .map(|v| v.trim().trim_matches('"').to_string())
                    })
                })
                .filter(|v| !v.is_empty())
                .unwrap_or_else(|| slug.replace(['-', '_'], " "));
            index.documents.insert(
                slug.clone(),
                EditorDocument {
                    path: path.to_path_buf(),
                    title,
                    slug,
                },
            );
        }
        Ok(index)
    }

    pub fn completions(&self, prefix: &str) -> Vec<EditorDocument> {
        self.documents
            .values()
            .filter(|doc| {
                doc.slug.starts_with(prefix)
                    || doc.title.to_lowercase().starts_with(&prefix.to_lowercase())
            })
            .cloned()
            .collect()
    }

    pub fn definition(&self, target: &str) -> Option<&EditorDocument> {
        let target = target.split('#').next().unwrap_or(target).trim();
        self.documents.get(target).or_else(|| {
            self.documents
                .values()
                .find(|d| d.title.eq_ignore_ascii_case(target))
        })
    }

    pub fn diagnostics(&self, content: &str) -> Vec<EditorDiagnostic> {
        let re = Regex::new(r"\[\[([^\]|#]+)(?:#[^\]|]+)?(?:\|[^\]]+)?\]\]")
            .expect("static wiki-link regex");
        let mut diagnostics = Vec::new();
        for (line, text) in content.lines().enumerate() {
            for capture in re.captures_iter(text) {
                let target = capture
                    .get(1)
                    .map(|m| m.as_str().trim())
                    .unwrap_or_default();
                if !target.is_empty() && self.definition(target).is_none() {
                    diagnostics.push(EditorDiagnostic {
                        message: format!("Tachyon document not found: {target}"),
                        line,
                        column: capture.get(0).map(|m| m.start()).unwrap_or(0),
                    });
                }
            }
        }
        diagnostics
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn indexes_documents_and_reports_broken_links() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("guide.md"), "---\ntitle: Guide\n---\nBody").unwrap();
        let index = VaultIndex::from_directory(dir.path()).unwrap();
        assert_eq!(index.completions("gui").len(), 1);
        assert!(index.definition("Guide").is_some());
        assert_eq!(index.diagnostics("See [[Guide]] and [[Missing]]").len(), 1);
    }
}
