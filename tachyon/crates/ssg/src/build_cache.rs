//! Incremental build cache — tracks content hashes to skip unchanged documents.

use std::collections::HashMap;
use std::path::Path;

use crate::manifest::SsgDocument;

/// Content hash (hex-encoded SHA-256 truncated to 16 chars for compactness).
type ContentHash = String;

/// Build cache mapping slug → (content_hash, output_path_relative).
#[derive(Debug, Clone, Default)]
pub struct BuildCache {
    entries: HashMap<String, ContentHash>,
    dirty: bool,
}

impl BuildCache {
    /// Load cache from a JSON file. Returns empty cache if file doesn't exist or is invalid.
    pub fn load(path: &Path) -> Self {
        if let Ok(data) = std::fs::read_to_string(path) {
            if let Ok(entries) = serde_json::from_str::<HashMap<String, ContentHash>>(&data) {
                return Self {
                    entries,
                    dirty: false,
                };
            }
        }
        Self::default()
    }

    /// Save cache to a JSON file.
    pub fn save(&self, path: &Path) {
        if self.dirty || !path.exists() {
            if let Ok(data) = serde_json::to_string_pretty(&self.entries) {
                let _ = std::fs::write(path, data);
            }
        }
    }

    /// Compute a content hash for a document.
    /// Uses title + content + tags + author + updated_at for change detection.
    pub fn hash_document(doc: &SsgDocument) -> ContentHash {
        use std::hash::{Hash, Hasher};
        // Use a simple deterministic hash (twox_hash would be ideal, but std is fine)
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        doc.title.hash(&mut hasher);
        doc.content.hash(&mut hasher);
        doc.tags.join(",").hash(&mut hasher);
        doc.author.as_deref().unwrap_or("").hash(&mut hasher);
        doc.updated_at.to_rfc3339().hash(&mut hasher);
        doc.description.as_deref().unwrap_or("").hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }

    /// Check if a document needs rebuilding.
    /// Returns true if: content hash changed OR output file doesn't exist.
    pub fn needs_rebuild(&self, doc: &SsgDocument, output_path: &Path) -> bool {
        let hash = Self::hash_document(doc);
        if let Some(cached_hash) = self.entries.get(&doc.slug) {
            if *cached_hash == hash && output_path.exists() {
                return false; // Unchanged and output exists
            }
        }
        true
    }

    /// Record that a document was built with its current content hash.
    pub fn record(&mut self, doc: &SsgDocument) {
        let hash = Self::hash_document(doc);
        self.entries.insert(doc.slug.clone(), hash);
        self.dirty = true;
    }

    /// Remove stale entries — slugs that are no longer in the document set.
    /// Returns the list of removed slugs.
    pub fn prune_stale(&mut self, active_slugs: &[&str]) -> Vec<String> {
        let active_set: std::collections::HashSet<&str> = active_slugs.iter().copied().collect();
        let stale: Vec<String> = self
            .entries
            .keys()
            .filter(|slug| !active_set.contains(slug.as_str()))
            .cloned()
            .collect();
        for slug in &stale {
            self.entries.remove(slug);
        }
        if !stale.is_empty() {
            self.dirty = true;
        }
        stale
    }

    /// Number of cached entries.
    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn make_doc(slug: &str, content: &str) -> SsgDocument {
        SsgDocument {
            slug: slug.to_string(),
            title: format!("Title {}", slug),
            content: content.to_string(),
            description: None,
            author: None,
            tags: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            order: 0,
            language: "en".to_string(),
            hide_breadcrumbs: false,
        }
    }

    #[test]
    fn test_hash_deterministic() {
        let doc = make_doc("test", "hello");
        let h1 = BuildCache::hash_document(&doc);
        let h2 = BuildCache::hash_document(&doc);
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_hash_changes_with_content() {
        let doc1 = make_doc("test", "hello");
        let doc2 = make_doc("test", "world");
        assert_ne!(
            BuildCache::hash_document(&doc1),
            BuildCache::hash_document(&doc2)
        );
    }

    #[test]
    fn test_needs_rebuild_new_doc() {
        let cache = BuildCache::default();
        let doc = make_doc("new", "content");
        let path = std::path::PathBuf::from("/tmp/nonexistent_test.html");
        assert!(cache.needs_rebuild(&doc, &path));
    }

    #[test]
    fn test_needs_rebuild_unchanged() {
        let mut cache = BuildCache::default();
        let doc = make_doc("test", "content");
        cache.record(&doc);
        // Output file must exist for "unchanged" detection
        let dir = std::env::temp_dir().join("tachyon_ssg_cache_test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.html");
        std::fs::write(&path, "test").unwrap();
        assert!(!cache.needs_rebuild(&doc, &path));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_needs_rebuild_changed_content() {
        let mut cache = BuildCache::default();
        let doc1 = make_doc("test", "v1");
        cache.record(&doc1);
        let doc2 = make_doc("test", "v2");
        let dir = std::env::temp_dir().join("tachyon_ssg_cache_test2");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.html");
        std::fs::write(&path, "test").unwrap();
        assert!(cache.needs_rebuild(&doc2, &path));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_prune_stale() {
        let mut cache = BuildCache::default();
        cache.record(&make_doc("a", "content"));
        cache.record(&make_doc("b", "content"));
        cache.record(&make_doc("c", "content"));
        let stale = cache.prune_stale(&["a", "c"]);
        assert_eq!(stale, vec!["b".to_string()]);
        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn test_save_and_load() {
        let dir = std::env::temp_dir().join("tachyon_ssg_cache_io");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(".build-cache.json");

        let mut cache = BuildCache::default();
        cache.record(&make_doc("test", "content"));
        cache.save(&path);

        let loaded = BuildCache::load(&path);
        assert_eq!(loaded.entries, cache.entries);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
