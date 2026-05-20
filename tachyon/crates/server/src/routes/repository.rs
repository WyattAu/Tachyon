// Repository API routes
// Handles repository operations: init, clone, commit, push, status

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::error::ServerError;

/// Repository data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryData {
    /// Repository ID
    pub id: String,
    /// Repository name
    pub name: String,
    /// Repository description
    pub description: Option<String>,
    /// Repository metadata
    pub metadata: Option<serde_json::Value>,
    /// Current branch
    pub branch: String,
    /// Owner ID
    pub owner_id: String,
    /// Created at
    pub created_at: DateTime<Utc>,
    /// Updated at
    pub updated_at: DateTime<Utc>,
    /// Remote URL (if any)
    pub remote_url: Option<String>,
    /// Is initialized
    pub is_initialized: bool,
}

/// Application state for repository routes
#[derive(Clone)]
pub struct RepositoryState {
    /// In-memory repository store (for demo mode)
    pub repositories: Arc<RwLock<HashMap<String, RepositoryData>>>,
}

impl RepositoryState {
    /// Create a new repository state
    pub fn new() -> Self {
        Self {
            repositories: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a repository state with existing repositories
    pub fn with_repositories(repositories: HashMap<String, RepositoryData>) -> Self {
        Self {
            repositories: Arc::new(RwLock::new(repositories)),
        }
    }
}

impl Default for RepositoryState {
    fn default() -> Self {
        Self::new()
    }
}

/// Request to initialize a repository
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct InitRepositoryRequest {
    /// Repository name
    pub name: String,
    /// Repository description
    pub description: Option<String>,
    /// Repository metadata
    pub metadata: Option<serde_json::Value>,
    /// Owner ID (optional, for demo uses a default)
    pub owner_id: Option<String>,
}

/// Request to clone a repository
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CloneRepositoryRequest {
    /// Source repository URL or ID
    pub source: String,
    /// Destination name
    pub name: String,
}

/// Request to commit changes
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CommitRequest {
    /// Commit message
    pub message: String,
    /// Commit metadata
    pub metadata: Option<serde_json::Value>,
}

/// Request to push changes
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct PushRequest {
    /// Target branch
    pub branch: Option<String>,
}

/// Repository response
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct RepositoryResponse {
    /// Repository ID
    pub id: String,
    /// Repository name
    pub name: String,
    /// Repository description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Repository metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    /// Current branch
    pub branch: String,
    /// Owner ID
    pub owner_id: String,
    /// Created at
    pub created_at: String,
    /// Updated at
    pub updated_at: String,
    /// Remote URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote_url: Option<String>,
    /// Is initialized
    pub is_initialized: bool,
}

impl From<RepositoryData> for RepositoryResponse {
    fn from(repo: RepositoryData) -> Self {
        Self {
            id: repo.id,
            name: repo.name,
            description: repo.description,
            metadata: repo.metadata,
            branch: repo.branch,
            owner_id: repo.owner_id,
            created_at: repo.created_at.to_rfc3339(),
            updated_at: repo.updated_at.to_rfc3339(),
            remote_url: repo.remote_url,
            is_initialized: repo.is_initialized,
        }
    }
}

/// Repository status response
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct RepositoryStatus {
    /// Repository ID
    pub repository_id: String,
    /// Repository name
    pub name: String,
    /// Current branch
    pub branch: String,
    /// Status
    pub status: String,
    /// Uncommitted changes
    pub uncommitted_changes: usize,
    /// Unpushed commits
    pub unpushed_commits: usize,
    /// Last commit message
    pub last_commit: Option<String>,
}

/// Repository list response
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct RepositoryListResponse {
    /// List of repositories
    pub repositories: Vec<RepositoryResponse>,
    /// Total count
    pub total: usize,
}

/// Initialize a new repository
#[utoipa::path(
    post,
    path = "/repositories/init",
    request_body(content = InitRepositoryRequest, description = "Repository init request"),
    responses(
        (status = 200, description = "Repository initialized", body = RepositoryResponse),
    ),
    tag = "repositories",
    security(("bearer_auth" = [])),
)]
pub async fn init_repository(
    State(state): State<RepositoryState>,
    Json(req): Json<InitRepositoryRequest>,
) -> Result<Json<RepositoryResponse>, ServerError> {
    info!("Initializing repository: {}", req.name);

    let now = Utc::now();
    let owner_id = req.owner_id.unwrap_or_else(|| "default_user".to_string());

    let repo = RepositoryData {
        id: format!("repo_{}", Uuid::new_v4()),
        name: req.name.clone(),
        description: req.description,
        metadata: req.metadata,
        branch: "main".to_string(),
        owner_id,
        created_at: now,
        updated_at: now,
        remote_url: None,
        is_initialized: true,
    };

    let response = RepositoryResponse::from(repo.clone());

    // Store repository
    let mut repos = state.repositories.write().await;
    repos.insert(repo.id.clone(), repo);

    info!("Repository initialized: {}", response.id);

    Ok(Json(response))
}

/// Clone a repository
#[utoipa::path(
    post,
    path = "/repositories/clone",
    request_body(content = CloneRepositoryRequest, description = "Repository clone request"),
    responses(
        (status = 200, description = "Repository cloned", body = RepositoryResponse),
        (status = 404, description = "Source repository not found"),
    ),
    tag = "repositories",
    security(("bearer_auth" = [])),
)]
pub async fn clone_repository(
    State(state): State<RepositoryState>,
    Json(req): Json<CloneRepositoryRequest>,
) -> Result<Json<RepositoryResponse>, ServerError> {
    info!("Cloning repository from: {}", req.source);

    // Check if source exists
    let source_repo = {
        let repos = state.repositories.read().await;
        repos.get(&req.source).cloned()
    };

    let repo = match source_repo {
        Some(source) => {
            let now = Utc::now();
            RepositoryData {
                id: format!("repo_{}", Uuid::new_v4()),
                name: req.name,
                description: source.description.clone(),
                metadata: source.metadata.clone(),
                branch: source.branch.clone(),
                owner_id: source.owner_id.clone(),
                created_at: now,
                updated_at: now,
                remote_url: source.remote_url.clone(),
                is_initialized: source.is_initialized,
            }
        }
        None => {
            warn!("Source repository not found: {}", req.source);
            return Err(ServerError::not_found("repository", &req.source));
        }
    };

    let response = RepositoryResponse::from(repo.clone());

    // Store repository
    let mut repos = state.repositories.write().await;
    repos.insert(repo.id.clone(), repo);

    info!("Repository cloned: {}", response.id);

    Ok(Json(response))
}

/// Commit changes to a repository
#[utoipa::path(
    post,
    path = "/repositories/{repository_id}/commit",
    params(
        ("repository_id" = String, Path, description = "Repository ID"),
    ),
    request_body(content = CommitRequest, description = "Commit request"),
    responses(
        (status = 200, description = "Commit created", body = serde_json::Value),
        (status = 404, description = "Repository not found"),
    ),
    tag = "repositories",
    security(("bearer_auth" = [])),
)]
pub async fn commit(
    Path(repository_id): Path<String>,
    State(state): State<RepositoryState>,
    Json(req): Json<CommitRequest>,
) -> Result<Json<serde_json::Value>, ServerError> {
    info!("Committing to repository: {}", repository_id);

    let mut repos = state.repositories.write().await;

    match repos.get_mut(&repository_id) {
        Some(repo) => {
            repo.updated_at = Utc::now();

            let commit_id = format!("commit_{}", Uuid::new_v4());
            info!("Commit created: {}", commit_id);

            Ok(Json(serde_json::json!({
                "success": true,
                "commit_id": commit_id,
                "repository_id": repository_id,
                "branch": repo.branch,
                "message": req.message,
                "timestamp": Utc::now().to_rfc3339()
            })))
        }
        None => {
            debug!("Repository not found: {}", repository_id);
            Err(ServerError::not_found("repository", &repository_id))
        }
    }
}

/// Push changes to remote
#[utoipa::path(
    post,
    path = "/repositories/{repository_id}/push",
    params(
        ("repository_id" = String, Path, description = "Repository ID"),
    ),
    request_body(content = PushRequest, description = "Push request"),
    responses(
        (status = 200, description = "Push successful", body = serde_json::Value),
        (status = 404, description = "Repository not found"),
        (status = 412, description = "Repository not initialized"),
    ),
    tag = "repositories",
    security(("bearer_auth" = [])),
)]
pub async fn push(
    Path(repository_id): Path<String>,
    State(state): State<RepositoryState>,
    Json(req): Json<PushRequest>,
) -> Result<Json<serde_json::Value>, ServerError> {
    info!("Pushing to repository: {}", repository_id);

    let branch = req.branch.unwrap_or_else(|| "main".to_string());

    let repos = state.repositories.read().await;

    match repos.get(&repository_id) {
        Some(repo) => {
            if !repo.is_initialized {
                return Err(ServerError::bad_request("Repository is not initialized"));
            }

            info!(
                "Pushed to repository: {} (branch: {})",
                repository_id, branch
            );

            Ok(Json(serde_json::json!({
                "success": true,
                "repository_id": repository_id,
                "branch": branch,
                "message": "Pushed successfully",
                "timestamp": Utc::now().to_rfc3339()
            })))
        }
        None => {
            debug!("Repository not found: {}", repository_id);
            Err(ServerError::not_found("repository", &repository_id))
        }
    }
}

/// Get repository status
#[utoipa::path(
    get,
    path = "/repositories/{repository_id}/status",
    params(
        ("repository_id" = String, Path, description = "Repository ID"),
    ),
    responses(
        (status = 200, description = "Repository status", body = RepositoryStatus),
        (status = 404, description = "Repository not found"),
    ),
    tag = "repositories",
    security(("bearer_auth" = [])),
)]
pub async fn status(
    Path(repository_id): Path<String>,
    State(state): State<RepositoryState>,
) -> Result<Json<RepositoryStatus>, ServerError> {
    debug!("Getting repository status: {}", repository_id);

    let repos = state.repositories.read().await;

    match repos.get(&repository_id) {
        Some(repo) => Ok(Json(RepositoryStatus {
            repository_id: repo.id.clone(),
            name: repo.name.clone(),
            branch: repo.branch.clone(),
            status: if repo.is_initialized {
                "clean".to_string()
            } else {
                "not_initialized".to_string()
            },
            uncommitted_changes: 0,
            unpushed_commits: 0,
            last_commit: None,
        })),
        None => Err(ServerError::not_found("repository", &repository_id)),
    }
}

/// List all repositories
#[utoipa::path(
    get,
    path = "/repositories",
    responses(
        (status = 200, description = "List of repositories", body = RepositoryListResponse),
    ),
    tag = "repositories",
    security(("bearer_auth" = [])),
)]
pub async fn list_repositories(
    State(state): State<RepositoryState>,
) -> Result<Json<RepositoryListResponse>, ServerError> {
    debug!("Listing repositories");

    let repos = state.repositories.read().await;

    let mut repo_list: Vec<RepositoryResponse> = repos
        .values()
        .map(|r| RepositoryResponse::from(r.clone()))
        .collect();

    repo_list.sort_by(|a, b| a.name.cmp(&b.name));

    let total = repo_list.len();

    Ok(Json(RepositoryListResponse {
        repositories: repo_list,
        total,
    }))
}

/// Get repository by ID
#[utoipa::path(
    get,
    path = "/repositories/{repository_id}",
    params(
        ("repository_id" = String, Path, description = "Repository ID"),
    ),
    responses(
        (status = 200, description = "Repository details", body = RepositoryResponse),
        (status = 404, description = "Repository not found"),
    ),
    tag = "repositories",
    security(("bearer_auth" = [])),
)]
pub async fn get_repository(
    Path(repository_id): Path<String>,
    State(state): State<RepositoryState>,
) -> Result<Json<RepositoryResponse>, ServerError> {
    debug!("Getting repository: {}", repository_id);

    let repos = state.repositories.read().await;

    match repos.get(&repository_id) {
        Some(repo) => Ok(Json(RepositoryResponse::from(repo.clone()))),
        None => Err(ServerError::not_found("repository", &repository_id)),
    }
}

/// Delete a repository
#[utoipa::path(
    delete,
    path = "/repositories/{repository_id}",
    params(
        ("repository_id" = String, Path, description = "Repository ID"),
    ),
    responses(
        (status = 204, description = "Repository deleted"),
        (status = 404, description = "Repository not found"),
    ),
    tag = "repositories",
    security(("bearer_auth" = [])),
)]
pub async fn delete_repository(
    Path(repository_id): Path<String>,
    State(state): State<RepositoryState>,
) -> Result<StatusCode, ServerError> {
    debug!("Deleting repository: {}", repository_id);

    let mut repos = state.repositories.write().await;

    match repos.remove(&repository_id) {
        Some(_) => {
            info!("Repository deleted: {}", repository_id);
            Ok(StatusCode::NO_CONTENT)
        }
        None => {
            debug!("Repository not found for deletion: {}", repository_id);
            Err(ServerError::not_found("repository", &repository_id))
        }
    }
}

/// Create the repository router (without state - caller must use .with_state())
pub fn create_repository_router() -> axum::Router<RepositoryState> {
    use axum::routing::{delete, get, post};

    axum::Router::new()
        .route("/repositories/init", post(init_repository))
        .route("/repositories/clone", post(clone_repository))
        .route("/repositories/{repository_id}/commit", post(commit))
        .route("/repositories/{repository_id}/push", post(push))
        .route("/repositories/{repository_id}/status", get(status))
        .route("/repositories", get(list_repositories))
        .route("/repositories/{repository_id}", get(get_repository))
        .route("/repositories/{repository_id}", delete(delete_repository))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_repository_request_construction() {
        let req = InitRepositoryRequest {
            name: "test-repo".to_string(),
            description: Some("Test repository".to_string()),
            metadata: None,
            owner_id: None,
        };

        assert_eq!(req.name, "test-repo");
        assert_eq!(req.description, Some("Test repository".to_string()));
    }

    #[test]
    fn test_repository_status_serialization() {
        let status = RepositoryStatus {
            repository_id: "repo-1".to_string(),
            name: "test-repo".to_string(),
            branch: "main".to_string(),
            status: "clean".to_string(),
            uncommitted_changes: 0,
            unpushed_commits: 0,
            last_commit: None,
        };

        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("clean"));
    }

    #[test]
    fn test_repository_data_creation() {
        let now = Utc::now();
        let repo = RepositoryData {
            id: "repo-123".to_string(),
            name: "test-repo".to_string(),
            description: Some("Test".to_string()),
            metadata: None,
            branch: "main".to_string(),
            owner_id: "user-1".to_string(),
            created_at: now,
            updated_at: now,
            remote_url: None,
            is_initialized: true,
        };

        assert_eq!(repo.id, "repo-123");
        assert!(repo.is_initialized);
    }

    #[test]
    fn test_repository_response_from_data() {
        let now = Utc::now();
        let repo = RepositoryData {
            id: "repo-123".to_string(),
            name: "test-repo".to_string(),
            description: Some("Test".to_string()),
            metadata: None,
            branch: "main".to_string(),
            owner_id: "user-1".to_string(),
            created_at: now,
            updated_at: now,
            remote_url: Some("https://example.com/repo.git".to_string()),
            is_initialized: true,
        };

        let response = RepositoryResponse::from(repo);
        assert_eq!(response.id, "repo-123");
        assert_eq!(response.name, "test-repo");
        assert!(response.is_initialized);
    }
}
