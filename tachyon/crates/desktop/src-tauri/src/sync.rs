// Auto-sync commits implementation
// Handles automatic Git commits and synchronization

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;
use tachyon_core::{ErrorResult, TachyonError, ErrorCategory};
use git2::{Repository, Signature, Time, Oid};
use chrono::{DateTime, Utc};
use std::collections::VecDeque;
use tokio::task::JoinHandle;

/// Commit queue entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitQueueEntry {
    /// Entry ID
    pub id: String,
    /// File path that changed
    pub path: String,
    /// Commit message
    pub message: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Committed flag
    pub committed: bool,
}

/// Sync status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncStatus {
    /// Not syncing
    Idle,
    /// Sync in progress
    Syncing,
    /// Sync completed successfully
    Success,
    /// Sync failed
    Failed,
}

/// Sync result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    /// Sync status
    pub status: SyncStatus,
    /// Number of commits made
    pub commits_made: u32,
    /// Number of files synced
    pub files_synced: u32,
    /// Error message if failed
    pub error: Option<String>,
}

/// Sync configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    /// Auto-sync enabled
    pub auto_sync_enabled: bool,
    /// Sync interval in seconds
    pub sync_interval_seconds: u64,
    /// Maximum commit queue size
    pub max_queue_size: usize,
    /// Commit message template
    pub commit_message_template: String,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            auto_sync_enabled: true,
            sync_interval_seconds: 30,
            max_queue_size: 100,
            commit_message_template: "Auto-sync: Update {filename}".to_string(),
        }
    }
}

/// Auto-sync manager
pub struct AutoSyncManager {
    repository_path: Option<PathBuf>,
    config: SyncConfig,
    commit_queue: Arc<RwLock<VecDeque<CommitQueueEntry>>>,
    sync_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
    is_syncing: Arc<RwLock<bool>>,
}

impl AutoSyncManager {
    /// Create a new auto-sync manager
    ///
    /// # Arguments
    /// * `config` - Sync configuration
    pub fn new(config: SyncConfig) -> Self {
        Self {
            repository_path: None,
            config,
            commit_queue: Arc::new(RwLock::new(VecDeque::new())),
            sync_handle: Arc::new(Mutex::new(None)),
            is_syncing: Arc::new(RwLock::new(false)),
        }
    }

    /// Set the repository path
    ///
    /// # Arguments
    /// * `path` - Repository path
    pub fn set_repository_path(&mut self, path: PathBuf) {
        self.repository_path = Some(path);
    }

    /// Initialize the Git repository
    ///
    /// # Returns
    /// Error if initialization fails
    pub fn initialize_repository(&self) -> ErrorResult<()> {
        let path = self.repository_path
            .as_ref()
            .ok_or_else(|| TachyonError::validation("NO_REPOSITORY", "Repository path not set"))?;

        // Check if repository already exists
        if Repository::open(path).is_ok() {
            return Ok(());
        }

        // Initialize new repository
        Repository::init(path)
            .map_err(|e| TachyonError::git("INIT_ERROR", format!("Failed to initialize repository: {}", e)))?;

        Ok(())
    }

    /// Add a file change to the commit queue
    ///
    /// # Arguments
    /// * `path` - File path that changed
    pub async fn queue_file_change(&self, path: impl AsRef<str>) -> ErrorResult<()> {
        let path_ref = path.as_ref();

        // Check queue size
        {
            let mut queue = self.commit_queue.write().await;
            if queue.len() >= self.config.max_queue_size {
                queue.pop_front();
            }

            let entry = CommitQueueEntry {
                id: uuid::Uuid::new_v4().to_string(),
                path: path_ref.to_string(),
                message: self.generate_commit_message(path_ref),
                timestamp: Utc::now(),
                committed: false,
            };

            queue.push_back(entry);
        }

        Ok(())
    }

    /// Generate a commit message for a file change
    ///
    /// # Arguments
    /// * `path` - File path
    fn generate_commit_message(&self, path: &str) -> String {
        let filename = Path::new(path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("file");

        self.config.commit_message_template
            .replace("{filename}", filename)
            .replace("{path}", path)
    }

    /// Commit pending changes
    ///
    /// # Returns
    /// Sync result with commit information
    pub async fn commit_pending(&self) -> ErrorResult<SyncResult> {
        let path = self.repository_path
            .as_ref()
            .ok_or_else(|| TachyonError::validation("NO_REPOSITORY", "Repository path not set"))?;

        let repo = Repository::open(path)
            .map_err(|e| TachyonError::git("OPEN_ERROR", format!("Failed to open repository: {}", e)))?;

        let mut commits_made = 0;
        let mut files_synced = 0;

        // Process commit queue
        {
            let mut queue = self.commit_queue.write().await;
            
            while let Some(entry) = queue.pop_front() {
                match self.commit_file(&repo, &entry).await {
                    Ok(_) => {
                        commits_made += 1;
                        files_synced += 1;
                    }
                    Err(e) => {
                        return Ok(SyncResult {
                            status: SyncStatus::Failed,
                            commits_made,
                            files_synced,
                            error: Some(e.to_string()),
                        });
                    }
                }
            }
        }

        Ok(SyncResult {
            status: SyncStatus::Success,
            commits_made,
            files_synced,
            error: None,
        })
    }

    /// Commit a single file
    ///
    /// # Arguments
    /// * `repo` - Git repository
    /// * `entry` - Commit queue entry
    async fn commit_file(&self, repo: &Repository, entry: &CommitQueueEntry) -> ErrorResult<()> {
        let path = Path::new(&entry.path);

        // Check if file exists
        if !path.exists() {
            return Err(TachyonError::not_found(format!("File: {}", entry.path)));
        }

        // Get repository relative path
        let repo_path = repo.path().parent()
            .ok_or_else(|| TachyonError::internal("REPO_PATH_ERROR", "Failed to get repository path"))?;
        
        let relative_path = path.strip_prefix(repo_path)
            .map_err(|_| TachyonError::validation("PATH_ERROR", "File is not in repository"))?;

        // Get index
        let mut index = repo.index()
            .map_err(|e| TachyonError::git("INDEX_ERROR", format!("Failed to get index: {}", e)))?;

        // Add file to index
        index.add_path(relative_path)
            .map_err(|e| TachyonError::git("ADD_ERROR", format!("Failed to add file to index: {}", e)))?;

        // Write index
        index.write()
            .map_err(|e| TachyonError::git("WRITE_INDEX_ERROR", format!("Failed to write index: {}", e)))?;

        // Get tree ID
        let tree_id = index.write_tree()
            .map_err(|e| TachyonError::git("TREE_ERROR", format!("Failed to write tree: {}", e)))?;

        let tree = repo.find_tree(tree_id)
            .map_err(|e| TachyonError::git("FIND_TREE_ERROR", format!("Failed to find tree: {}", e)))?;

        // Get HEAD commit if exists
        let parent_commit = match repo.head() {
            Ok(head) => {
                let commit = head.peel_to_commit()
                    .map_err(|e| TachyonError::git("PEEL_ERROR", format!("Failed to peel to commit: {}", e)))?;
                Some(commit)
            }
            Err(_) => None,
        };

        // Create signature
        let time = Time::new(
            entry.timestamp.timestamp(),
            entry.timestamp.timestamp_subsec_nanos() as i32,  // git2 expects i32
        );
        let signature = Signature::new("Tachyon Auto-Sync", "auto-sync@tachyon.io", &time)
            .map_err(|e| TachyonError::git("SIGNATURE_ERROR", format!("Failed to create signature: {}", e)))?;

        // Create commit
        let mut parents = Vec::new();
        if let Some(ref parent) = parent_commit {
            parents.push(parent);
        }

        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            &entry.message,
            &tree,
            parents.as_slice(),
        )
        .map_err(|e| TachyonError::git("COMMIT_ERROR", format!("Failed to commit: {}", e)))?;

        Ok(())
    }

    /// Push commits to remote repository
    ///
    /// # Returns
    /// Sync result with push information
    pub async fn push_to_remote(&self, remote_name: &str, branch_name: &str) -> ErrorResult<SyncResult> {
        let path = self.repository_path
            .as_ref()
            .ok_or_else(|| TachyonError::validation("NO_REPOSITORY", "Repository path not set"))?;

        let repo = Repository::open(path)
            .map_err(|e| TachyonError::git("OPEN_ERROR", format!("Failed to open repository: {}", e)))?;

        // Find remote
        let mut remote = repo.find_remote(remote_name)
            .map_err(|e| TachyonError::git("REMOTE_ERROR", format!("Failed to find remote: {}", e)))?;

        // Get HEAD reference
        let head = repo.head()
            .map_err(|e| TachyonError::git("HEAD_ERROR", format!("Failed to get HEAD: {}", e)))?;

        // Push to remote
        remote.push(&[format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name)], None)
            .map_err(|e| TachyonError::git("PUSH_ERROR", format!("Failed to push: {}", e)))?;

        Ok(SyncResult {
            status: SyncStatus::Success,
            commits_made: 0,
            files_synced: 0,
            error: None,
        })
    }

    /// Pull changes from remote repository
    ///
    /// # Returns
    /// Sync result with pull information
    pub async fn pull_from_remote(&self, remote_name: &str, branch_name: &str) -> ErrorResult<SyncResult> {
        let path = self.repository_path
            .as_ref()
            .ok_or_else(|| TachyonError::validation("NO_REPOSITORY", "Repository path not set"))?;

        let repo = Repository::open(path)
            .map_err(|e| TachyonError::git("OPEN_ERROR", format!("Failed to open repository: {}", e)))?;

        // Find remote
        let mut remote = repo.find_remote(remote_name)
            .map_err(|e| TachyonError::git("REMOTE_ERROR", format!("Failed to find remote: {}", e)))?;

        // Fetch from remote
        remote.fetch(&[branch_name], None, None)
            .map_err(|e| TachyonError::git("FETCH_ERROR", format!("Failed to fetch: {}", e)))?;

        // Get fetch head
        let fetch_head = repo.find_reference(&format!("refs/remotes/{}/{}", remote_name, branch_name))
            .map_err(|e| TachyonError::git("FETCH_HEAD_ERROR", format!("Failed to find FETCH_HEAD: {}", e)))?;

        let fetch_commit = fetch_head.peel_to_commit()
            .map_err(|e| TachyonError::git("PEEL_ERROR", format!("Failed to peel to commit: {}", e)))?;

        // Merge into HEAD
        let head = repo.head()
            .map_err(|e| TachyonError::git("HEAD_ERROR", format!("Failed to get HEAD: {}", e)))?;

        let head_commit = head.peel_to_commit()
            .map_err(|e| TachyonError::git("PEEL_ERROR", format!("Failed to peel to commit: {}", e)))?;

        let annotated_head = repo.find_annotated_commit(head_commit.id())
            .map_err(|e| TachyonError::git("ANNOTATED_ERROR", format!("Failed to find annotated commit: {}", e)))?;

        let annotated_fetch = repo.find_annotated_commit(fetch_commit.id())
            .map_err(|e| TachyonError::git("ANNOTATED_ERROR", format!("Failed to find annotated commit: {}", e)))?;

        // Perform merge with the fetched commit
        let mut merge_opts = git2::MergeOptions::new();
        repo.merge(&[&annotated_fetch], Some(&mut merge_opts), None)
            .map_err(|e| TachyonError::git("MERGE_ERROR", format!("Failed to merge: {}", e)))?;

        Ok(SyncResult {
            status: SyncStatus::Success,
            commits_made: 0,
            files_synced: 0,
            error: None,
        })
    }

    /// Get the sync status
    ///
    /// # Returns
    /// Current sync status
    pub async fn get_sync_status(&self) -> SyncStatus {
        let is_syncing = *self.is_syncing.read().await;
        if is_syncing {
            SyncStatus::Syncing
        } else {
            SyncStatus::Idle
        }
    }

    /// Get the commit queue size
    ///
    /// # Returns
    /// Number of pending commits
    pub async fn get_queue_size(&self) -> usize {
        self.commit_queue.read().await.len()
    }

    /// Clear the commit queue
    pub async fn clear_queue(&self) {
        self.commit_queue.write().await.clear();
    }
}

impl Default for AutoSyncManager {
    fn default() -> Self {
        Self::new(SyncConfig::default())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_config_default() {
        let config = SyncConfig::default();
        assert!(config.auto_sync_enabled);
        assert_eq!(config.sync_interval_seconds, 30);
        assert_eq!(config.max_queue_size, 100);
        assert!(config.commit_message_template.contains("{filename}"));
    }

    #[test]
    fn test_commit_queue_entry() {
        let entry = CommitQueueEntry {
            id: "test-id".to_string(),
            path: "/path/to/file.txt".to_string(),
            message: "Test commit".to_string(),
            timestamp: Utc::now(),
            committed: false,
        };

        assert_eq!(entry.id, "test-id");
        assert_eq!(entry.path, "/path/to/file.txt");
        assert_eq!(entry.message, "Test commit");
        assert!(!entry.committed);
    }

    #[test]
    fn test_sync_result() {
        let result = SyncResult {
            status: SyncStatus::Success,
            commits_made: 5,
            files_synced: 5,
            error: None,
        };

        assert_eq!(result.status, SyncStatus::Success);
        assert_eq!(result.commits_made, 5);
        assert_eq!(result.files_synced, 5);
        assert!(result.error.is_none());
    }

    #[test]
    fn test_generate_commit_message() {
        let config = SyncConfig {
            commit_message_template: "Update {filename}".to_string(),
            ..Default::default()
        };

        let manager = AutoSyncManager::new(config);
        let message = manager.generate_commit_message("/path/to/file.txt");
        
        assert!(message.contains("file.txt"));
        assert!(message.contains("Update"));
    }

    #[test]
    fn test_auto_sync_manager_default() {
        let manager = AutoSyncManager::default();
        assert!(manager.repository_path.is_none());
        assert!(manager.config.auto_sync_enabled);
    }
}
