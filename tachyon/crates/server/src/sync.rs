use std::path::{Path, PathBuf};

use chrono::Utc;
use regex::Regex;
use tachyon_core::{
    compute_content_hash, generate_document_id, FileChangeEvent, FileChangeKind,
};
use tachyon_core::id::DocumentId;
use tachyon_database::{DatabasePool, DocumentMetadata, DocumentRepository};
use tachyon_renderer::{RenderConfig, Renderer};
use tracing::{debug, info, warn};

#[derive(Debug)]
pub enum SyncResult {
    Created { id: String, slug: String },
    Updated {
        id: String,
        slug: String,
        hash_changed: bool,
        conflict: bool,
    },
    Deleted { id: String, slug: String },
    Skipped { path: PathBuf, reason: String },
    Error { path: PathBuf, message: String },
}

#[derive(Debug, Clone)]
pub struct SyncConfig {
    pub default_author_id: String,
    pub default_visibility: String,
    pub render_html: bool,
    pub update_search_index: bool,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            default_author_id: generate_document_id().to_string(),
            default_visibility: "private".to_string(),
            render_html: true,
            update_search_index: true,
        }
    }
}

pub struct FileSyncService {
    pool: DatabasePool,
    config: SyncConfig,
}

impl FileSyncService {
    pub fn new(pool: DatabasePool, config: SyncConfig) -> Self {
        Self { pool, config }
    }

    pub async fn sync_file(&self, event: &FileChangeEvent) -> SyncResult {
        match event.kind {
            FileChangeKind::Created => self.handle_created(&event.path).await,
            FileChangeKind::Modified => self.handle_modified(&event.path).await,
            FileChangeKind::Deleted => self.handle_deleted(&event.path).await,
        }
    }

    async fn handle_created(&self, path: &Path) -> SyncResult {
        let raw = match tokio::fs::read_to_string(path).await {
            Ok(c) => c,
            Err(e) => {
                return SyncResult::Error {
                    path: path.to_path_buf(),
                    message: format!("Failed to read file: {}", e),
                };
            }
        };

        let hash = compute_content_hash(&raw);
        let repo = DocumentRepository::new(self.pool.clone());

        if let Ok(Some(existing)) = repo.get_by_content_hash(&hash).await {
            return SyncResult::Skipped {
                path: path.to_path_buf(),
                reason: format!("Duplicate content (hash matches document {})", existing.id),
            };
        }

        let slug = Self::derive_slug_from_path(path);
        let title = Self::derive_title_from_path(path);
        let (frontmatter, body) = Self::derive_frontmatter(&raw);
        let doc_id = generate_document_id();

        let (html, word_count, character_count) = if self.config.render_html {
            let renderer = Renderer::new(RenderConfig::default());
            match renderer.render(&body, None) {
                Ok(result) => (
                    Some(result.content),
                    result.metadata.word_count as i32,
                    result.metadata.char_count as i32,
                ),
                Err(e) => {
                    warn!("Render failed for {}: {}", path.display(), e);
                    (None, 0, 0)
                }
            }
        } else {
            (None, 0, 0)
        };

        let now = Utc::now();
        let metadata = DocumentMetadata {
            id: doc_id.to_string(),
            title: title.clone(),
            slug: Some(slug.clone()),
            author_id: self.config.default_author_id.clone(),
            description: None,
            tags: "[]".to_string(),
            frontmatter,
            project_id: None,
            visibility: self.config.default_visibility.clone(),
            status: "draft".to_string(),
            content_type: "markdown".to_string(),
            word_count,
            character_count,
            read_count: 0,
            edit_count: 1,
            content: Some(body.clone()),
            html,
            created_at: now,
            updated_at: now,
            published_at: None,
            content_hash: Some(hash),
            conflict_detected: Some(false),
        };

        if let Err(e) = repo.create(metadata).await {
            return SyncResult::Error {
                path: path.to_path_buf(),
                message: format!("Failed to create document: {}", e),
            };
        }

        if self.config.update_search_index {
            if let Err(e) = repo.update_search_index(&doc_id, &title, &body, &[]).await {
                warn!("Failed to update search index for {}: {}", doc_id, e);
            }
        }

        info!("Document created from file: {} -> {}", path.display(), slug);
        SyncResult::Created {
            id: doc_id.to_string(),
            slug,
        }
    }

    async fn handle_modified(&self, path: &Path) -> SyncResult {
        let raw = match tokio::fs::read_to_string(path).await {
            Ok(c) => c,
            Err(e) => {
                return SyncResult::Error {
                    path: path.to_path_buf(),
                    message: format!("Failed to read file: {}", e),
                };
            }
        };

        let hash = compute_content_hash(&raw);
        let slug = Self::derive_slug_from_path(path);
        let repo = DocumentRepository::new(self.pool.clone());

        let existing = match repo.get_by_slug(&slug).await {
            Ok(Some(doc)) => doc,
            Ok(None) => {
                debug!(
                    "Modified file has no existing document (slug {}), treating as create",
                    slug
                );
                let event = FileChangeEvent {
                    path: path.to_path_buf(),
                    kind: FileChangeKind::Created,
                };
                return self.handle_created(&event.path).await;
            }
            Err(e) => {
                return SyncResult::Error {
                    path: path.to_path_buf(),
                    message: format!("Failed to query document by slug: {}", e),
                };
            }
        };

        if existing.content_hash.as_deref() == Some(&hash) {
            return SyncResult::Skipped {
                path: path.to_path_buf(),
                reason: "Content unchanged (hash matches)".to_string(),
            };
        }

        let file_mtime = tokio::fs::metadata(path)
            .await
            .ok()
            .and_then(|m| m.modified().ok())
            .map(chrono::DateTime::<Utc>::from);

        let conflict = matches!(file_mtime, Some(file_time) if existing.updated_at > file_time);

        if conflict {
            warn!(
                "Conflict detected: file {} was modified but DB record {} is newer",
                path.display(),
                existing.id
            );
        }

        let title = Self::derive_title_from_path(path);
        let (frontmatter, body) = Self::derive_frontmatter(&raw);

        let (html, word_count, character_count) = if self.config.render_html {
            let renderer = Renderer::new(RenderConfig::default());
            match renderer.render(&body, None) {
                Ok(result) => (
                    Some(result.content),
                    result.metadata.word_count as i32,
                    result.metadata.char_count as i32,
                ),
                Err(e) => {
                    warn!("Render failed for {}: {}", path.display(), e);
                    (existing.html.clone(), existing.word_count, existing.character_count)
                }
            }
        } else {
            (existing.html.clone(), existing.word_count, existing.character_count)
        };

        let mut metadata = existing;
        metadata.title = title.clone();
        metadata.content = Some(body.clone());
        metadata.content_hash = Some(hash);
        metadata.conflict_detected = Some(conflict);
        metadata.html = html;
        metadata.word_count = word_count;
        metadata.character_count = character_count;
        metadata.frontmatter = frontmatter;
        metadata.updated_at = Utc::now();
        metadata.edit_count += 1;

        let id = metadata.id.clone();
        if let Err(e) = repo.update(metadata).await {
            return SyncResult::Error {
                path: path.to_path_buf(),
                message: format!("Failed to update document: {}", e),
            };
        }

        if self.config.update_search_index {
            if let Ok(doc_id) = DocumentId::parse_str(&id) {
                if let Err(e) = repo.update_search_index(&doc_id, &title, &body, &[]).await {
                    warn!("Failed to update search index for {}: {}", doc_id, e);
                }
            }
        }

        info!(
            "Document updated from file: {} -> {} (conflict={})",
            path.display(),
            slug,
            conflict
        );
        SyncResult::Updated {
            id,
            slug,
            hash_changed: true,
            conflict,
        }
    }

    async fn handle_deleted(&self, path: &Path) -> SyncResult {
        let slug = Self::derive_slug_from_path(path);
        let repo = DocumentRepository::new(self.pool.clone());

        let existing = match repo.get_by_slug(&slug).await {
            Ok(Some(doc)) => doc,
            Ok(None) => {
                return SyncResult::Skipped {
                    path: path.to_path_buf(),
                    reason: format!("No existing document with slug '{}'", slug),
                };
            }
            Err(e) => {
                return SyncResult::Error {
                    path: path.to_path_buf(),
                    message: format!("Failed to query document: {}", e),
                };
            }
        };

        let id = existing.id.clone();
        let doc_id = match DocumentId::parse_str(&id) {
            Ok(did) => did,
            Err(e) => {
                return SyncResult::Error {
                    path: path.to_path_buf(),
                    message: format!("Invalid document ID '{}': {}", id, e),
                };
            }
        };

        if let Err(e) = repo.delete(&doc_id).await {
            return SyncResult::Error {
                path: path.to_path_buf(),
                message: format!("Failed to delete document: {}", e),
            };
        }

        info!("Document deleted (file removed): {} -> {}", path.display(), slug);
        SyncResult::Deleted { id, slug }
    }

    pub fn derive_slug_from_path(path: &Path) -> String {
        let mut parts: Vec<&str> = Vec::new();

        if let Some(parent) = path.parent() {
            for component in parent.components() {
                if let std::path::Component::Normal(name) = component {
                    if let Some(s) = name.to_str() {
                        if !s.is_empty() && !s.starts_with('.') {
                            parts.push(s);
                        }
                    }
                }
            }
        }

        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            parts.push(stem);
        }

        let slug: String = parts
            .join(" ")
            .to_lowercase()
            .chars()
            .map(|c| {
                if c.is_alphanumeric() {
                    c
                } else if c.is_whitespace() || c == '-' || c == '_' {
                    '-'
                } else {
                    '\0'
                }
            })
            .filter(|c| *c != '\0')
            .collect();

        let re = Regex::new(r"-+").unwrap();
        let collapsed = re.replace_all(&slug, "-");
        let trimmed = collapsed.trim_matches('-').to_string();

        if trimmed.len() > 200 {
            trimmed[..200].to_string()
        } else {
            trimmed
        }
    }

    pub fn derive_title_from_path(path: &Path) -> String {
        path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Untitled")
            .split(['-', '_'])
            .filter(|s| !s.is_empty())
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => {
                        let mut result = String::new();
                        for c in first.to_uppercase() {
                            result.push(c);
                        }
                        result.push_str(chars.as_str());
                        result
                    }
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub fn derive_frontmatter(content: &str) -> (Option<String>, String) {
        if !content.starts_with("---") {
            return (None, content.to_string());
        }

        let rest = &content[3..];
        if let Some(pos) = rest.find("\n---") {
            let fm = rest[..pos].trim().to_string();
            let body = rest[pos + 4..].trim_start().to_string();
            return (Some(fm), body);
        }

        (None, content.to_string())
    }

    pub async fn sync_directory(&self, dir: &Path) -> Vec<SyncResult> {
        let mut results = Vec::new();
        let mut stack = vec![dir.to_path_buf()];

        while let Some(current) = stack.pop() {
            let mut dir = match tokio::fs::read_dir(&current).await {
                Ok(d) => d,
                Err(e) => {
                    results.push(SyncResult::Error {
                        path: current,
                        message: format!("Failed to read directory: {}", e),
                    });
                    continue;
                }
            };

            while let Some(entry) = dir.next_entry().await.unwrap_or_else(|e| {
                results.push(SyncResult::Error {
                    path: current.clone(),
                    message: format!("Failed to read directory entry: {}", e),
                });
                None
            }) {
                let path = entry.path();
                let is_dir = tokio::fs::metadata(&path)
                    .await
                    .map(|m| m.is_dir())
                    .unwrap_or(false);
                let is_file = tokio::fs::metadata(&path)
                    .await
                    .map(|m| m.is_file())
                    .unwrap_or(false);
                if is_dir {
                    stack.push(path);
                } else if is_file && is_markdown_file(&path) {
                    let event = FileChangeEvent {
                        path: path.clone(),
                        kind: FileChangeKind::Created,
                    };
                    results.push(self.sync_file(&event).await);
                }
            }
        }

        info!(
            "Directory sync complete: {} files processed in {}",
            results.len(),
            dir.display()
        );
        results
    }
}

fn is_markdown_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext == "md" || ext == "markdown")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_slug_from_path_simple() {
        let path = Path::new("docs/API Reference.md");
        assert_eq!(FileSyncService::derive_slug_from_path(path), "docs-api-reference");
    }

    #[test]
    fn test_derive_slug_from_path_nested() {
        let path = Path::new("docs/guides/setup.md");
        assert_eq!(FileSyncService::derive_slug_from_path(path), "docs-guides-setup");
    }

    #[test]
    fn test_derive_slug_from_path_root_file() {
        let path = Path::new("README.md");
        assert_eq!(FileSyncService::derive_slug_from_path(path), "readme");
    }

    #[test]
    fn test_derive_slug_from_path_underscores() {
        let path = Path::new("notes/my_notes.md");
        assert_eq!(FileSyncService::derive_slug_from_path(path), "notes-my-notes");
    }

    #[test]
    fn test_derive_slug_from_path_multiple_hyphens() {
        let path = Path::new("some--weird---name.md");
        assert_eq!(FileSyncService::derive_slug_from_path(path), "some-weird-name");
    }

    #[test]
    fn test_derive_slug_from_path_hidden_dirs_excluded() {
        let path = Path::new(".git/refs/heads/main.md");
        assert_eq!(FileSyncService::derive_slug_from_path(path), "refs-heads-main");
    }

    #[test]
    fn test_derive_title_from_path() {
        assert_eq!(
            FileSyncService::derive_title_from_path(Path::new("api-reference.md")),
            "Api Reference"
        );
    }

    #[test]
    fn test_derive_title_from_path_underscores() {
        assert_eq!(
            FileSyncService::derive_title_from_path(Path::new("my_notes.md")),
            "My Notes"
        );
    }

    #[test]
    fn test_derive_title_from_path_no_extension() {
        assert_eq!(
            FileSyncService::derive_title_from_path(Path::new("Makefile")),
            "Makefile"
        );
    }

    #[test]
    fn test_derive_frontmatter_with_frontmatter() {
        let content = "---\ntitle: Hello\ntags: [a, b]\n---\n\nBody content";
        let (fm, body) = FileSyncService::derive_frontmatter(content);
        assert_eq!(fm, Some("title: Hello\ntags: [a, b]".to_string()));
        assert_eq!(body, "Body content");
    }

    #[test]
    fn test_derive_frontmatter_without_frontmatter() {
        let content = "# Hello\n\nBody content";
        let (fm, body) = FileSyncService::derive_frontmatter(content);
        assert!(fm.is_none());
        assert_eq!(body, content);
    }

    #[test]
    fn test_derive_frontmatter_empty() {
        let (fm, body) = FileSyncService::derive_frontmatter("");
        assert!(fm.is_none());
        assert_eq!(body, "");
    }

    #[test]
    fn test_derive_frontmatter_incomplete() {
        let content = "---\ntitle: Hello\nBody without closing";
        let (fm, body) = FileSyncService::derive_frontmatter(content);
        assert!(fm.is_none());
        assert_eq!(body, content);
    }

    #[test]
    fn test_sync_config_default() {
        let config = SyncConfig::default();
        assert!(!config.default_author_id.is_empty());
        assert_eq!(config.default_visibility, "private");
        assert!(config.render_html);
        assert!(config.update_search_index);
    }

    #[test]
    fn test_is_markdown_file() {
        assert!(is_markdown_file(Path::new("test.md")));
        assert!(is_markdown_file(Path::new("test.markdown")));
        assert!(!is_markdown_file(Path::new("test.txt")));
        assert!(!is_markdown_file(Path::new("Makefile")));
    }
}
