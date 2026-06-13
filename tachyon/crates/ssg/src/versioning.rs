use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::{SsgError, SsgResult};
use crate::manifest::SsgDocument;

/// Status of a documentation version.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum VersionStatus {
    Draft,
    Published,
    Archived,
}

impl Default for VersionStatus {
    fn default() -> Self {
        Self::Draft
    }
}

impl std::fmt::Display for VersionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VersionStatus::Draft => write!(f, "draft"),
            VersionStatus::Published => write!(f, "published"),
            VersionStatus::Archived => write!(f, "archived"),
        }
    }
}

/// A documentation version branch, analogous to a git branch for docs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocVersion {
    /// Unique version identifier (e.g., "1.0", "2.0-beta", "main")
    pub id: String,
    /// Display name for the version
    pub name: String,
    /// Optional description of what this version represents
    pub description: Option<String>,
    /// Current status of this version
    pub status: VersionStatus,
    /// The parent version this was branched from (if any)
    pub parent_id: Option<String>,
    /// Snapshot of documents at the time of version creation
    pub documents: Vec<SsgDocument>,
    /// When this version was created
    pub created_at: DateTime<Utc>,
    /// When this version was last updated
    pub updated_at: DateTime<Utc>,
    /// Whether this is the latest/recommended version
    pub is_latest: bool,
    /// Custom metadata for this version
    pub metadata: HashMap<String, String>,
}

/// Represents a line-level diff between two document versions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionDiffLine {
    pub content: String,
    pub line_type: String,
}

/// Diff result between two versions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionDiff {
    pub old_lines: Vec<VersionDiffLine>,
    pub new_lines: Vec<VersionDiffLine>,
    pub stats: VersionDiffStats,
}

/// Statistics for a version diff.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionDiffStats {
    pub added: usize,
    pub removed: usize,
    pub unchanged: usize,
}

/// Request to create a new version from current state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVersionRequest {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub parent_id: Option<String>,
}

/// In-memory version store for documentation versions.
pub struct VersionStore {
    versions: HashMap<String, DocVersion>,
}

impl VersionStore {
    pub fn new() -> Self {
        Self {
            versions: HashMap::new(),
        }
    }

    /// Create a new version from a snapshot of documents.
    pub fn create_version(
        &mut self,
        request: CreateVersionRequest,
        current_documents: &[SsgDocument],
    ) -> SsgResult<DocVersion> {
        if self.versions.contains_key(&request.id) {
            return Err(SsgError::Config(format!(
                "Version '{}' already exists",
                request.id
            )));
        }

        let now = Utc::now();
        let version = DocVersion {
            id: request.id.clone(),
            name: request.name,
            description: request.description,
            status: VersionStatus::Draft,
            parent_id: request.parent_id,
            documents: current_documents.to_vec(),
            created_at: now,
            updated_at: now,
            is_latest: false,
            metadata: HashMap::new(),
        };

        self.versions.insert(request.id, version.clone());
        Ok(version)
    }

    /// Get a version by ID.
    pub fn get_version(&self, version_id: &str) -> SsgResult<DocVersion> {
        self.versions
            .get(version_id)
            .cloned()
            .ok_or_else(|| SsgError::Config(format!("Version '{}' not found", version_id)))
    }

    /// List all versions.
    pub fn list_versions(&self) -> Vec<DocVersion> {
        let mut versions: Vec<DocVersion> = self.versions.values().cloned().collect();
        versions.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        versions
    }

    /// Publish a version (mark as published).
    pub fn publish_version(&mut self, version_id: &str) -> SsgResult<DocVersion> {
        let version = self
            .versions
            .get_mut(version_id)
            .ok_or_else(|| SsgError::Config(format!("Version '{}' not found", version_id)))?;

        version.status = VersionStatus::Published;
        version.updated_at = Utc::now();
        Ok(version.clone())
    }

    /// Rollback to a previous version — restores its documents as the current working set.
    pub fn rollback_to_version(&mut self, version_id: &str) -> SsgResult<Vec<SsgDocument>> {
        let version = self
            .versions
            .get(version_id)
            .ok_or_else(|| SsgError::Config(format!("Version '{}' not found", version_id)))?;

        Ok(version.documents.clone())
    }

    /// Edit a version independently — update its document snapshot.
    pub fn edit_version(
        &mut self,
        version_id: &str,
        documents: Vec<SsgDocument>,
    ) -> SsgResult<DocVersion> {
        let version = self
            .versions
            .get_mut(version_id)
            .ok_or_else(|| SsgError::Config(format!("Version '{}' not found", version_id)))?;

        version.documents = documents;
        version.updated_at = Utc::now();
        Ok(version.clone())
    }

    /// Compare two versions and produce a diff for a specific document.
    pub fn compare_versions(
        &self,
        version_a_id: &str,
        version_b_id: &str,
        document_slug: &str,
    ) -> SsgResult<VersionDiff> {
        let version_a = self.get_version(version_a_id)?;
        let version_b = self.get_version(version_b_id)?;

        let doc_a = version_a.documents.iter().find(|d| d.slug == document_slug);
        let doc_b = version_b.documents.iter().find(|d| d.slug == document_slug);

        let content_a = doc_a.map(|d| d.content.as_str()).unwrap_or("");
        let content_b = doc_b.map(|d| d.content.as_str()).unwrap_or("");

        Ok(compute_line_diff(content_a, content_b))
    }

    /// Delete a version.
    pub fn delete_version(&mut self, version_id: &str) -> SsgResult<()> {
        self.versions
            .remove(version_id)
            .ok_or_else(|| SsgError::Config(format!("Version '{}' not found", version_id)))?;
        Ok(())
    }
}

/// Compute a line-by-line diff between two text contents.
pub fn compute_line_diff(old_content: &str, new_content: &str) -> VersionDiff {
    let old_lines: Vec<&str> = old_content.lines().collect();
    let new_lines: Vec<&str> = new_content.lines().collect();

    let lcs = longest_common_subsequence(&old_lines, &new_lines);

    let mut result_old = Vec::new();
    let mut result_new = Vec::new();
    let mut added = 0usize;
    let mut removed = 0usize;
    let mut unchanged = 0usize;

    let mut old_idx = 0;
    let mut new_idx = 0;
    let mut lcs_idx = 0;

    while old_idx < old_lines.len() || new_idx < new_lines.len() {
        if lcs_idx < lcs.len() {
            let lcs_line = lcs[lcs_idx];

            while old_idx < old_lines.len() && old_lines[old_idx] != lcs_line {
                result_old.push(VersionDiffLine {
                    content: old_lines[old_idx].to_string(),
                    line_type: "removed".to_string(),
                });
                result_new.push(VersionDiffLine {
                    content: String::new(),
                    line_type: "unchanged".to_string(),
                });
                removed += 1;
                old_idx += 1;
            }

            while new_idx < new_lines.len() && new_lines[new_idx] != lcs_line {
                result_old.push(VersionDiffLine {
                    content: String::new(),
                    line_type: "unchanged".to_string(),
                });
                result_new.push(VersionDiffLine {
                    content: new_lines[new_idx].to_string(),
                    line_type: "added".to_string(),
                });
                added += 1;
                new_idx += 1;
            }

            if old_idx < old_lines.len() && new_idx < new_lines.len() {
                result_old.push(VersionDiffLine {
                    content: old_lines[old_idx].to_string(),
                    line_type: "unchanged".to_string(),
                });
                result_new.push(VersionDiffLine {
                    content: new_lines[new_idx].to_string(),
                    line_type: "unchanged".to_string(),
                });
                unchanged += 1;
                old_idx += 1;
                new_idx += 1;
                lcs_idx += 1;
            }
        } else {
            while old_idx < old_lines.len() {
                result_old.push(VersionDiffLine {
                    content: old_lines[old_idx].to_string(),
                    line_type: "removed".to_string(),
                });
                result_new.push(VersionDiffLine {
                    content: String::new(),
                    line_type: "unchanged".to_string(),
                });
                removed += 1;
                old_idx += 1;
            }

            while new_idx < new_lines.len() {
                result_old.push(VersionDiffLine {
                    content: String::new(),
                    line_type: "unchanged".to_string(),
                });
                result_new.push(VersionDiffLine {
                    content: new_lines[new_idx].to_string(),
                    line_type: "added".to_string(),
                });
                added += 1;
                new_idx += 1;
            }
        }
    }

    VersionDiff {
        old_lines: result_old,
        new_lines: result_new,
        stats: VersionDiffStats {
            added,
            removed,
            unchanged,
        },
    }
}

/// Compute the longest common subsequence of two string slices.
fn longest_common_subsequence<'a>(a: &[&'a str], b: &[&'a str]) -> Vec<&'a str> {
    let m = a.len();
    let n = b.len();

    if m == 0 || n == 0 {
        return Vec::new();
    }

    let mut dp = vec![vec![0u32; n + 1]; m + 1];

    for i in 1..=m {
        for j in 1..=n {
            if a[i - 1] == b[j - 1] {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else {
                dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
            }
        }
    }

    let mut result = Vec::new();
    let mut i = m;
    let mut j = n;

    while i > 0 && j > 0 {
        if a[i - 1] == b[j - 1] {
            result.push(a[i - 1]);
            i -= 1;
            j -= 1;
        } else if dp[i - 1][j] > dp[i][j - 1] {
            i -= 1;
        } else {
            j -= 1;
        }
    }

    result.reverse();
    result
}

impl Default for VersionStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_documents() -> Vec<SsgDocument> {
        vec![
            SsgDocument {
                slug: "intro".to_string(),
                title: "Introduction".to_string(),
                content: "# Introduction\n\nWelcome to Tachyon.".to_string(),
                ..Default::default()
            },
            SsgDocument {
                slug: "setup".to_string(),
                title: "Setup".to_string(),
                content: "# Setup\n\nRun `cargo build`.".to_string(),
                ..Default::default()
            },
        ]
    }

    #[test]
    fn test_version_store_create() {
        let mut store = VersionStore::new();
        let req = CreateVersionRequest {
            id: "1.0".to_string(),
            name: "v1.0".to_string(),
            description: Some("First release".to_string()),
            parent_id: None,
        };
        let docs = sample_documents();
        let version = store.create_version(req, &docs).unwrap();
        assert_eq!(version.id, "1.0");
        assert_eq!(version.status, VersionStatus::Draft);
        assert_eq!(version.documents.len(), 2);
    }

    #[test]
    fn test_version_store_create_duplicate() {
        let mut store = VersionStore::new();
        let req = CreateVersionRequest {
            id: "1.0".to_string(),
            name: "v1.0".to_string(),
            description: None,
            parent_id: None,
        };
        store.create_version(req.clone(), &[]).unwrap();
        let result = store.create_version(req, &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_version_store_get() {
        let mut store = VersionStore::new();
        let req = CreateVersionRequest {
            id: "1.0".to_string(),
            name: "v1.0".to_string(),
            description: None,
            parent_id: None,
        };
        store.create_version(req, &[]).unwrap();
        let v = store.get_version("1.0").unwrap();
        assert_eq!(v.id, "1.0");
    }

    #[test]
    fn test_version_store_get_not_found() {
        let store = VersionStore::new();
        let result = store.get_version("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_version_store_list() {
        let mut store = VersionStore::new();
        store
            .create_version(
                CreateVersionRequest {
                    id: "1.0".to_string(),
                    name: "v1.0".to_string(),
                    description: None,
                    parent_id: None,
                },
                &[],
            )
            .unwrap();
        store
            .create_version(
                CreateVersionRequest {
                    id: "2.0".to_string(),
                    name: "v2.0".to_string(),
                    description: None,
                    parent_id: None,
                },
                &[],
            )
            .unwrap();
        let versions = store.list_versions();
        assert_eq!(versions.len(), 2);
    }

    #[test]
    fn test_version_store_publish() {
        let mut store = VersionStore::new();
        store
            .create_version(
                CreateVersionRequest {
                    id: "1.0".to_string(),
                    name: "v1.0".to_string(),
                    description: None,
                    parent_id: None,
                },
                &[],
            )
            .unwrap();
        let v = store.publish_version("1.0").unwrap();
        assert_eq!(v.status, VersionStatus::Published);
    }

    #[test]
    fn test_version_store_publish_not_found() {
        let mut store = VersionStore::new();
        let result = store.publish_version("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_version_store_rollback() {
        let mut store = VersionStore::new();
        let docs = sample_documents();
        store
            .create_version(
                CreateVersionRequest {
                    id: "1.0".to_string(),
                    name: "v1.0".to_string(),
                    description: None,
                    parent_id: None,
                },
                &docs,
            )
            .unwrap();
        let restored = store.rollback_to_version("1.0").unwrap();
        assert_eq!(restored.len(), 2);
        assert_eq!(restored[0].slug, "intro");
    }

    #[test]
    fn test_version_store_edit() {
        let mut store = VersionStore::new();
        store
            .create_version(
                CreateVersionRequest {
                    id: "1.0".to_string(),
                    name: "v1.0".to_string(),
                    description: None,
                    parent_id: None,
                },
                &[],
            )
            .unwrap();
        let new_docs = vec![SsgDocument {
            slug: "new".to_string(),
            title: "New".to_string(),
            content: "New content".to_string(),
            ..Default::default()
        }];
        let v = store.edit_version("1.0", new_docs).unwrap();
        assert_eq!(v.documents.len(), 1);
        assert_eq!(v.documents[0].slug, "new");
    }

    #[test]
    fn test_version_store_delete() {
        let mut store = VersionStore::new();
        store
            .create_version(
                CreateVersionRequest {
                    id: "1.0".to_string(),
                    name: "v1.0".to_string(),
                    description: None,
                    parent_id: None,
                },
                &[],
            )
            .unwrap();
        store.delete_version("1.0").unwrap();
        assert!(store.get_version("1.0").is_err());
    }

    #[test]
    fn test_version_store_delete_not_found() {
        let mut store = VersionStore::new();
        let result = store.delete_version("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_compare_versions_same() {
        let mut store = VersionStore::new();
        let docs = sample_documents();
        store
            .create_version(
                CreateVersionRequest {
                    id: "1.0".to_string(),
                    name: "v1.0".to_string(),
                    description: None,
                    parent_id: None,
                },
                &docs,
            )
            .unwrap();
        store
            .create_version(
                CreateVersionRequest {
                    id: "2.0".to_string(),
                    name: "v2.0".to_string(),
                    description: None,
                    parent_id: Some("1.0".to_string()),
                },
                &docs,
            )
            .unwrap();
        let diff = store.compare_versions("1.0", "2.0", "intro").unwrap();
        assert_eq!(diff.stats.added, 0);
        assert_eq!(diff.stats.removed, 0);
        assert!(diff.stats.unchanged > 0);
    }

    #[test]
    fn test_compare_versions_different() {
        let mut store = VersionStore::new();
        let docs_a = vec![SsgDocument {
            slug: "intro".to_string(),
            title: "Intro".to_string(),
            content: "Hello".to_string(),
            ..Default::default()
        }];
        let docs_b = vec![SsgDocument {
            slug: "intro".to_string(),
            title: "Intro".to_string(),
            content: "World".to_string(),
            ..Default::default()
        }];
        store
            .create_version(
                CreateVersionRequest {
                    id: "1.0".to_string(),
                    name: "v1.0".to_string(),
                    description: None,
                    parent_id: None,
                },
                &docs_a,
            )
            .unwrap();
        store
            .create_version(
                CreateVersionRequest {
                    id: "2.0".to_string(),
                    name: "v2.0".to_string(),
                    description: None,
                    parent_id: Some("1.0".to_string()),
                },
                &docs_b,
            )
            .unwrap();
        let diff = store.compare_versions("1.0", "2.0", "intro").unwrap();
        assert!(diff.stats.added > 0 || diff.stats.removed > 0);
    }

    #[test]
    fn test_compare_versions_missing_doc() {
        let mut store = VersionStore::new();
        let docs_a = vec![SsgDocument {
            slug: "intro".to_string(),
            title: "Intro".to_string(),
            content: "Hello".to_string(),
            ..Default::default()
        }];
        store
            .create_version(
                CreateVersionRequest {
                    id: "1.0".to_string(),
                    name: "v1.0".to_string(),
                    description: None,
                    parent_id: None,
                },
                &docs_a,
            )
            .unwrap();
        store
            .create_version(
                CreateVersionRequest {
                    id: "2.0".to_string(),
                    name: "v2.0".to_string(),
                    description: None,
                    parent_id: Some("1.0".to_string()),
                },
                &[],
            )
            .unwrap();
        let diff = store.compare_versions("1.0", "2.0", "intro").unwrap();
        assert_eq!(diff.stats.removed, 1);
        assert_eq!(diff.stats.added, 0);
    }

    #[test]
    fn test_compute_line_diff_identical() {
        let diff = compute_line_diff("a\nb\nc", "a\nb\nc");
        assert_eq!(diff.stats.added, 0);
        assert_eq!(diff.stats.removed, 0);
        assert_eq!(diff.stats.unchanged, 3);
    }

    #[test]
    fn test_compute_line_diff_empty_old() {
        let diff = compute_line_diff("", "new line");
        assert_eq!(diff.stats.added, 1);
        assert_eq!(diff.stats.removed, 0);
    }

    #[test]
    fn test_compute_line_diff_empty_new() {
        let diff = compute_line_diff("old line", "");
        assert_eq!(diff.stats.added, 0);
        assert_eq!(diff.stats.removed, 1);
    }

    #[test]
    fn test_compute_line_diff_both_empty() {
        let diff = compute_line_diff("", "");
        assert_eq!(diff.stats.added, 0);
        assert_eq!(diff.stats.removed, 0);
        assert_eq!(diff.stats.unchanged, 0);
    }

    #[test]
    fn test_compute_line_diff_added_lines() {
        let diff = compute_line_diff("a", "a\nb\nc");
        assert_eq!(diff.stats.added, 2);
        assert_eq!(diff.stats.unchanged, 1);
    }

    #[test]
    fn test_compute_line_diff_removed_lines() {
        let diff = compute_line_diff("a\nb\nc", "a");
        assert_eq!(diff.stats.removed, 2);
        assert_eq!(diff.stats.unchanged, 1);
    }

    #[test]
    fn test_lcs_empty() {
        assert!(longest_common_subsequence(&[], &[]).is_empty());
        assert!(longest_common_subsequence(&["a"], &[]).is_empty());
        assert!(longest_common_subsequence(&[], &["a"]).is_empty());
    }

    #[test]
    fn test_lcs_identical() {
        let lines = vec!["a", "b", "c"];
        assert_eq!(
            longest_common_subsequence(&lines, &lines),
            vec!["a", "b", "c"]
        );
    }

    #[test]
    fn test_lcs_no_common() {
        let a = vec!["a", "b", "c"];
        let b = vec!["x", "y", "z"];
        assert!(longest_common_subsequence(&a, &b).is_empty());
    }

    #[test]
    fn test_lcs_partial() {
        let a = vec!["a", "b", "c", "d"];
        let b = vec!["b", "c", "x"];
        assert_eq!(longest_common_subsequence(&a, &b), vec!["b", "c"]);
    }

    #[test]
    fn test_version_status_display() {
        assert_eq!(VersionStatus::Draft.to_string(), "draft");
        assert_eq!(VersionStatus::Published.to_string(), "published");
        assert_eq!(VersionStatus::Archived.to_string(), "archived");
    }

    #[test]
    fn test_version_status_default() {
        assert_eq!(VersionStatus::default(), VersionStatus::Draft);
    }

    #[test]
    fn test_version_store_parent_id() {
        let mut store = VersionStore::new();
        store
            .create_version(
                CreateVersionRequest {
                    id: "1.0".to_string(),
                    name: "v1.0".to_string(),
                    description: None,
                    parent_id: None,
                },
                &[],
            )
            .unwrap();
        let v = store
            .create_version(
                CreateVersionRequest {
                    id: "2.0".to_string(),
                    name: "v2.0".to_string(),
                    description: None,
                    parent_id: Some("1.0".to_string()),
                },
                &[],
            )
            .unwrap();
        assert_eq!(v.parent_id.as_deref(), Some("1.0"));
    }

    #[test]
    fn test_version_store_metadata() {
        let mut store = VersionStore::new();
        let mut v = store
            .create_version(
                CreateVersionRequest {
                    id: "1.0".to_string(),
                    name: "v1.0".to_string(),
                    description: None,
                    parent_id: None,
                },
                &[],
            )
            .unwrap();
        v.metadata.insert("author".to_string(), "team".to_string());
        store.versions.insert("1.0".to_string(), v.clone());
        let retrieved = store.get_version("1.0").unwrap();
        assert_eq!(
            retrieved.metadata.get("author").map(|s| s.as_str()),
            Some("team")
        );
    }
}
