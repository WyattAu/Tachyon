// Repository type definitions
// Represents knowledge repositories with Git integration

use crate::id::RepositoryId;
use crate::id::UserId;
use crate::types::error::TachyonError;
use crate::util::slugify;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ============================================================================
// Repository Type
// ============================================================================

/// Repository type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RepositoryType {
    /// Personal repository
    #[serde(rename = "personal")]
    Personal,
    /// Team repository
    #[serde(rename = "team")]
    Team,
    /// Organization repository
    #[serde(rename = "organization")]
    Organization,
    /// Public repository
    #[serde(rename = "public")]
    Public,
}

// ============================================================================
// Repository Status
// ============================================================================

/// Repository lifecycle status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RepositoryStatus {
    /// Repository is initialized but not cloned
    #[serde(rename = "init")]
    Init,
    /// Repository is cloned and ready
    #[serde(rename = "cloned")]
    Cloned,
    /// Repository has uncommitted changes
    #[serde(rename = "dirty")]
    Dirty,
    /// Repository is synced with remote
    #[serde(rename = "synced")]
    Synced,
    /// Repository has merge conflicts
    #[serde(rename = "conflict")]
    Conflict,
}

impl RepositoryStatus {
    /// Check if repository needs sync
    pub fn needs_sync(&self) -> bool {
        matches!(self, Self::Dirty | Self::Conflict)
    }

    /// Check if repository is ready for operations
    pub fn is_ready(&self) -> bool {
        matches!(self, Self::Cloned | Self::Synced | Self::Dirty)
    }
}

// ============================================================================
// Repository Config
// ============================================================================

/// Repository configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryConfig {
    /// Default branch name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_branch: Option<String>,
    /// Auto-sync enabled
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_sync: Option<bool>,
    /// Sync interval in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sync_interval_seconds: Option<u64>,
    /// File watching enabled
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_watching_enabled: Option<bool>,
    /// Git remote URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote_url: Option<String>,
}

impl RepositoryConfig {
    /// Create new repository config
    pub fn new() -> Self {
        Self {
            default_branch: Some("main".to_string()),
            auto_sync: Some(false),
            sync_interval_seconds: Some(300),
            file_watching_enabled: Some(true),
            remote_url: None,
        }
    }

    /// Set default branch
    pub fn with_default_branch(mut self, branch: String) -> Self {
        self.default_branch = Some(branch);
        self
    }

    /// Set remote URL
    pub fn with_remote_url(mut self, url: String) -> Self {
        self.remote_url = Some(url);
        self
    }

    /// Enable auto-sync
    pub fn with_auto_sync(mut self, enabled: bool) -> Self {
        self.auto_sync = Some(enabled);
        self
    }
}

impl Default for RepositoryConfig {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Git Operations Wrapper
// ============================================================================

/// Git operations wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitOperations {
    /// Last commit hash
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_commit_hash: Option<String>,
    /// Current branch
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_branch: Option<String>,
    /// Number of commits ahead
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commits_ahead: Option<usize>,
    /// Number of commits behind
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commits_behind: Option<usize>,
}

impl GitOperations {
    /// Create new git operations
    pub fn new() -> Self {
        Self {
            last_commit_hash: None,
            current_branch: None,
            commits_ahead: None,
            commits_behind: None,
        }
    }
}

impl Default for GitOperations {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Repository Metadata
// ============================================================================

/// Repository metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryMetadata {
    /// Repository name
    pub name: String,
    /// Repository slug (URL-friendly identifier)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    /// Repository description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Repository type
    pub repository_type: RepositoryType,
    /// Owner user ID
    pub owner_id: UserId,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
    /// Repository configuration
    pub config: RepositoryConfig,
    /// Git operations state
    pub git_operations: GitOperations,
}

impl RepositoryMetadata {
    /// Create new repository metadata
    ///
    /// # Arguments
    /// * `name` - Repository name
    /// * `repository_type` - Repository type
    /// * `owner_id` - Owner's user ID
    pub fn new(name: String, repository_type: RepositoryType, owner_id: UserId) -> Self {
        let now = Utc::now();
        let slug = slugify(&name);
        Self {
            name,
            slug: Some(slug),
            description: None,
            repository_type,
            owner_id,
            created_at: now,
            updated_at: now,
            config: RepositoryConfig::new(),
            git_operations: GitOperations::new(),
        }
    }

    /// Update timestamp
    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    /// Set description
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// Set config
    pub fn with_config(mut self, config: RepositoryConfig) -> Self {
        self.config = config;
        self
    }
}

// ============================================================================
// Repository Statistics
// ============================================================================

/// Repository usage statistics
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct RepositoryStats {
    /// Number of documents
    pub document_count: usize,
    /// Total storage used in bytes
    pub total_storage_bytes: u64,
    /// Number of members
    pub member_count: usize,
}

impl RepositoryStats {
    /// Create new repository stats
    pub fn new() -> Self {
        Self {
            document_count: 0,
            total_storage_bytes: 0,
            member_count: 1,
        }
    }

    /// Increment document count
    pub fn increment_document_count(&mut self) {
        self.document_count += 1;
    }

    /// Update storage used
    pub fn update_storage(&mut self, bytes: u64) {
        self.total_storage_bytes = bytes;
    }
}

// ============================================================================
// Repository
// ============================================================================

/// Knowledge repository with Git integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    /// Unique repository identifier
    pub id: RepositoryId,
    /// Repository metadata
    pub metadata: RepositoryMetadata,
    /// Repository status
    pub status: RepositoryStatus,
    /// Repository statistics
    pub stats: RepositoryStats,
    /// Repository visibility
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<RepositoryVisibility>,
    /// Local repository path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_path: Option<PathBuf>,
}

/// Repository visibility settings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RepositoryVisibility {
    /// Repository is publicly visible
    #[serde(rename = "public")]
    Public,
    /// Repository is private
    #[serde(rename = "private")]
    Private,
}

impl Repository {
    /// Create a new repository
    ///
    /// # Arguments
    /// * `id` - Repository ID
    /// * `name` - Repository name
    /// * `repository_type` - Repository type
    /// * `owner_id` - Owner's user ID
    pub fn new(
        id: RepositoryId,
        name: String,
        repository_type: RepositoryType,
        owner_id: UserId,
    ) -> Self {
        let metadata = RepositoryMetadata::new(name, repository_type, owner_id);
        let stats = RepositoryStats::new();
        Self {
            id,
            metadata,
            status: RepositoryStatus::Init,
            stats,
            visibility: Some(RepositoryVisibility::Private),
            local_path: None,
        }
    }

    /// Set repository visibility
    pub fn with_visibility(mut self, visibility: RepositoryVisibility) -> Self {
        self.visibility = Some(visibility);
        self
    }

    /// Set local path
    pub fn with_local_path(mut self, path: PathBuf) -> Self {
        self.local_path = Some(path);
        self
    }

    /// Set repository status
    pub fn with_status(mut self, status: RepositoryStatus) -> Self {
        self.status = status;
        self
    }

    /// Update repository metadata
    pub fn update_metadata(&mut self, name: String, description: Option<String>) {
        self.metadata.name = name.clone();
        self.metadata.slug = Some(slugify(&name));
        self.metadata.description = description;
        self.metadata.touch();
    }

    /// Mark repository as dirty (has uncommitted changes)
    pub fn mark_dirty(&mut self) {
        if self.status.is_ready() {
            self.status = RepositoryStatus::Dirty;
            self.metadata.touch();
        }
    }

    /// Mark repository as synced
    pub fn mark_synced(&mut self) {
        self.status = RepositoryStatus::Synced;
        self.metadata.touch();
    }

    /// Mark repository as in conflict
    pub fn mark_conflict(&mut self) {
        self.status = RepositoryStatus::Conflict;
        self.metadata.touch();
    }

    /// Validate repository
    pub fn validate(&self) -> Result<(), TachyonError> {
        if self.metadata.name.is_empty() {
            return Err(TachyonError::field_validation(
                "name",
                "Repository name cannot be empty",
            ));
        }

        if self.metadata.name.len() > 100 {
            return Err(TachyonError::field_validation(
                "name",
                "Repository name too long (max 100 characters)",
            ));
        }

        if let Some(ref path) = self.local_path {
            if path.as_os_str().is_empty() {
                return Err(TachyonError::field_validation(
                    "local_path",
                    "Local path cannot be empty",
                ));
            }
        }

        Ok(())
    }

    /// Check if repository needs sync
    pub fn needs_sync(&self) -> bool {
        self.status.needs_sync()
    }

    /// Check if repository is ready for operations
    pub fn is_ready(&self) -> bool {
        self.status.is_ready()
    }
}

// ============================================================================
// RepositoryBuilder for fluent construction
// ============================================================================

/// Builder for creating Repository instances
pub struct RepositoryBuilder {
    id: Option<RepositoryId>,
    name: String,
    repository_type: RepositoryType,
    owner_id: UserId,
    description: Option<String>,
    visibility: Option<RepositoryVisibility>,
    local_path: Option<PathBuf>,
    remote_url: Option<String>,
    default_branch: Option<String>,
}

impl RepositoryBuilder {
    /// Create a new RepositoryBuilder
    ///
    /// # Arguments
    /// * `name` - Repository name
    /// * `repository_type` - Repository type
    /// * `owner_id` - Owner's user ID
    pub fn new(name: String, repository_type: RepositoryType, owner_id: UserId) -> Self {
        Self {
            id: None,
            name,
            repository_type,
            owner_id,
            description: None,
            visibility: None,
            local_path: None,
            remote_url: None,
            default_branch: None,
        }
    }

    /// Set repository ID
    pub fn id(mut self, id: RepositoryId) -> Self {
        self.id = Some(id);
        self
    }

    /// Set description
    pub fn description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// Set visibility
    pub fn visibility(mut self, visibility: RepositoryVisibility) -> Self {
        self.visibility = Some(visibility);
        self
    }

    /// Set local path
    pub fn local_path(mut self, path: PathBuf) -> Self {
        self.local_path = Some(path);
        self
    }

    /// Set remote URL
    pub fn remote_url(mut self, url: String) -> Self {
        self.remote_url = Some(url);
        self
    }

    /// Set default branch
    pub fn default_branch(mut self, branch: String) -> Self {
        self.default_branch = Some(branch);
        self
    }

    /// Build the Repository
    ///
    /// # Returns
    /// Result containing Repository or error
    pub fn build(self) -> Result<Repository, TachyonError> {
        let id = self.id.unwrap_or_else(crate::id::generate_repository_id);
        let mut repo = Repository::new(id, self.name, self.repository_type, self.owner_id);

        if let Some(visibility) = self.visibility {
            repo = repo.with_visibility(visibility);
        }

        if let Some(local_path) = self.local_path {
            repo = repo.with_local_path(local_path);
        }

        if let Some(remote_url) = self.remote_url {
            repo.metadata.config = repo.metadata.config.with_remote_url(remote_url);
        }

        if let Some(default_branch) = self.default_branch {
            repo.metadata.config = repo.metadata.config.with_default_branch(default_branch);
        }

        if let Some(description) = self.description {
            repo.metadata.description = Some(description);
        }

        repo.validate()?;

        Ok(repo)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repository_status() {
        assert!(RepositoryStatus::Dirty.needs_sync());
        assert!(RepositoryStatus::Conflict.needs_sync());
        assert!(!RepositoryStatus::Synced.needs_sync());

        assert!(RepositoryStatus::Cloned.is_ready());
        assert!(RepositoryStatus::Synced.is_ready());
        assert!(!RepositoryStatus::Init.is_ready());
    }

    #[test]
    fn test_repository_config() {
        let config = RepositoryConfig::new();
        assert_eq!(config.default_branch, Some("main".to_string()));
        assert_eq!(config.auto_sync, Some(false));
        assert_eq!(config.sync_interval_seconds, Some(300));
        assert_eq!(config.file_watching_enabled, Some(true));
        assert!(config.remote_url.is_none());

        let config = config.with_remote_url("https://github.com/test/repo".to_string());
        assert_eq!(
            config.remote_url,
            Some("https://github.com/test/repo".to_string())
        );
    }

    #[test]
    fn test_git_operations() {
        let ops = GitOperations::new();
        assert!(ops.last_commit_hash.is_none());
        assert!(ops.current_branch.is_none());
        assert!(ops.commits_ahead.is_none());
        assert!(ops.commits_behind.is_none());
    }

    #[test]
    fn test_repository_creation() {
        let repo_id = crate::id::generate_repository_id();
        let user_id = crate::id::generate_user_id();

        let repo = Repository::new(
            repo_id,
            "Test Repo".to_string(),
            RepositoryType::Personal,
            user_id,
        );

        assert_eq!(repo.id, repo_id);
        assert_eq!(repo.metadata.name, "Test Repo");
        assert_eq!(repo.status, RepositoryStatus::Init);
        assert!(!repo.is_ready());
    }

    #[test]
    fn test_repository_mark_dirty() {
        let repo_id = crate::id::generate_repository_id();
        let user_id = crate::id::generate_user_id();

        let mut repo = Repository::new(
            repo_id,
            "Test Repo".to_string(),
            RepositoryType::Personal,
            user_id,
        );
        repo.status = RepositoryStatus::Synced;

        repo.mark_dirty();
        assert_eq!(repo.status, RepositoryStatus::Dirty);
        assert!(repo.needs_sync());
    }

    #[test]
    fn test_repository_validation() {
        let repo_id = crate::id::generate_repository_id();
        let user_id = crate::id::generate_user_id();

        // Valid repository
        let repo = Repository::new(
            repo_id,
            "Valid Repo Name".to_string(),
            RepositoryType::Personal,
            user_id,
        );
        assert!(repo.validate().is_ok());

        // Empty name
        let invalid_repo = Repository::new(
            crate::id::generate_repository_id(),
            "".to_string(),
            RepositoryType::Personal,
            user_id,
        );
        assert!(invalid_repo.validate().is_err());

        // Name too long
        let invalid_repo = Repository::new(
            crate::id::generate_repository_id(),
            "a".repeat(101),
            RepositoryType::Personal,
            user_id,
        );
        assert!(invalid_repo.validate().is_err());
    }

    #[test]
    fn test_repository_builder() {
        let user_id = crate::id::generate_user_id();
        let local_path = PathBuf::from("/tmp/test/repo");

        let repo =
            RepositoryBuilder::new("Test Repository".to_string(), RepositoryType::Team, user_id)
                .description("Test description".to_string())
                .visibility(RepositoryVisibility::Public)
                .local_path(local_path.clone())
                .remote_url("https://github.com/test/repo".to_string())
                .default_branch("develop".to_string())
                .build()
                .expect("Should build repository");

        assert_eq!(
            repo.metadata.description,
            Some("Test description".to_string())
        );
        assert_eq!(repo.visibility, Some(RepositoryVisibility::Public));
        assert_eq!(repo.local_path, Some(local_path));
        assert_eq!(
            repo.metadata.config.remote_url,
            Some("https://github.com/test/repo".to_string())
        );
        assert_eq!(
            repo.metadata.config.default_branch,
            Some("develop".to_string())
        );
    }

    #[test]
    fn test_repository_stats() {
        let mut stats = RepositoryStats::new();
        assert_eq!(stats.document_count, 0);
        assert_eq!(stats.total_storage_bytes, 0);
        assert_eq!(stats.member_count, 1);

        stats.increment_document_count();
        assert_eq!(stats.document_count, 1);

        stats.update_storage(1024);
        assert_eq!(stats.total_storage_bytes, 1024);
    }
}
