use super::*;
use serde::{Deserialize, Serialize};

/// SSG version management types for the frontend API.
///
/// A documentation version as represented by the API.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DocVersion {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub parent_id: Option<String>,
    pub document_count: usize,
    pub is_latest: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Request body for creating a new documentation version.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDocVersionRequest {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub parent_id: Option<String>,
}

/// Diff line in a version comparison.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionDiffLine {
    pub content: String,
    pub line_type: String,
}

/// Diff statistics for a version comparison.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionDiffStats {
    pub added: usize,
    pub removed: usize,
    pub unchanged: usize,
}

/// Full diff result between two versions for a document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionDiffResponse {
    pub document_slug: String,
    pub old_lines: Vec<VersionDiffLine>,
    pub new_lines: Vec<VersionDiffLine>,
    pub stats: VersionDiffStats,
}

/// SSG version management API methods.
impl ApiClient {
    /// List all documentation versions.
    pub async fn list_doc_versions(&self) -> Result<Vec<DocVersion>, ApiError> {
        let url = format!("{}/versions", self.base_url);
        self.get(&url).await
    }

    /// Get a specific documentation version by ID.
    pub async fn get_doc_version(&self, version_id: &str) -> Result<DocVersion, ApiError> {
        let url = format!("{}/versions/{}", self.base_url, version_id);
        self.get(&url).await
    }

    /// Create a new documentation version.
    pub async fn create_doc_version(
        &self,
        request: &CreateDocVersionRequest,
    ) -> Result<DocVersion, ApiError> {
        let url = format!("{}/versions", self.base_url);
        self.post(&url, request).await
    }

    /// Publish a documentation version.
    pub async fn publish_doc_version(&self, version_id: &str) -> Result<DocVersion, ApiError> {
        let url = format!("{}/versions/{}/publish", self.base_url, version_id);
        self.post_empty_json(&url).await
    }

    /// Rollback to a documentation version.
    pub async fn rollback_doc_version(&self, version_id: &str) -> Result<DocVersion, ApiError> {
        let url = format!("{}/versions/{}/rollback", self.base_url, version_id);
        self.post_empty_json(&url).await
    }

    /// Compare two documentation versions for a specific document.
    pub async fn diff_doc_versions(
        &self,
        version_a_id: &str,
        version_b_id: &str,
        document_slug: &str,
    ) -> Result<VersionDiffResponse, ApiError> {
        let url = format!(
            "{}/versions/{}/diff/{}?document={}",
            self.base_url, version_a_id, version_b_id, document_slug
        );
        self.get(&url).await
    }
}
