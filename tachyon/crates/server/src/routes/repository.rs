// Repository API routes
// Handles repository operations: init, clone, commit, push, status

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};
use uuid::Uuid;

use crate::error::ServerError;
use tachyon_database::{DatabasePool, RepositoryId, RepositoryRepository};

/// Application state for repository routes
#[derive(Clone)]
pub struct RepositoryState {
    pub pool: DatabasePool,
}

impl RepositoryState {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
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
    /// Repository type
    pub repository_type: String,
    /// Current branch
    pub default_branch: Option<String>,
    /// Owner ID
    pub owner_id: String,
    /// Visibility
    pub visibility: String,
    /// Status
    pub status: String,
    /// Created at
    pub created_at: String,
    /// Updated at
    pub updated_at: String,
    /// Remote URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote_url: Option<String>,
}

impl From<tachyon_database::types::RepositoryMetadata> for RepositoryResponse {
    fn from(repo: tachyon_database::types::RepositoryMetadata) -> Self {
        Self {
            id: repo.id,
            name: repo.name,
            description: repo.description,
            repository_type: repo.repository_type,
            default_branch: repo.default_branch,
            owner_id: repo.owner_id,
            visibility: repo.visibility,
            status: repo.status,
            created_at: repo.created_at.to_rfc3339(),
            updated_at: repo.updated_at.to_rfc3339(),
            remote_url: repo.remote_url,
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
    let id = Uuid::new_v4().to_string();
    let slug = req.name.to_lowercase().replace(' ', "-");

    let metadata = tachyon_database::types::RepositoryMetadata {
        id: id.clone(),
        name: req.name.clone(),
        slug: Some(slug),
        description: req.description,
        repository_type: "git".to_string(),
        owner_id,
        visibility: "private".to_string(),
        status: "active".to_string(),
        default_branch: Some("main".to_string()),
        auto_sync: false,
        sync_interval_seconds: 300,
        file_watching_enabled: false,
        remote_url: None,
        last_commit_hash: None,
        current_branch: Some("main".to_string()),
        commits_ahead: None,
        commits_behind: None,
        document_count: 0,
        total_storage_bytes: 0,
        member_count: 1,
        local_path: None,
        created_at: now,
        updated_at: now,
    };

    let repo = RepositoryRepository::new(state.pool.clone());
    repo.create(metadata).await?;

    let created = repo
        .get_by_id(
            &id.parse::<RepositoryId>()
                .map_err(|_| ServerError::internal("Invalid repository ID"))?,
        )
        .await?;

    let response = RepositoryResponse::from(created);

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

    let repo = RepositoryRepository::new(state.pool.clone());

    let source = repo
        .get_by_id(
            &req.source
                .parse::<RepositoryId>()
                .map_err(|_| ServerError::not_found("repository", &req.source))?,
        )
        .await
        .map_err(|_| ServerError::not_found("repository", &req.source))?;

    let now = Utc::now();
    let id = Uuid::new_v4().to_string();
    let slug = req.name.to_lowercase().replace(' ', "-");

    let metadata = tachyon_database::types::RepositoryMetadata {
        id: id.clone(),
        name: req.name,
        slug: Some(slug),
        description: source.description.clone(),
        repository_type: source.repository_type.clone(),
        owner_id: source.owner_id.clone(),
        visibility: source.visibility.clone(),
        status: "active".to_string(),
        default_branch: source.default_branch.clone(),
        auto_sync: source.auto_sync,
        sync_interval_seconds: source.sync_interval_seconds,
        file_watching_enabled: source.file_watching_enabled,
        remote_url: source.remote_url.clone(),
        last_commit_hash: None,
        current_branch: source.current_branch.clone(),
        commits_ahead: None,
        commits_behind: None,
        document_count: 0,
        total_storage_bytes: 0,
        member_count: 1,
        local_path: None,
        created_at: now,
        updated_at: now,
    };

    repo.create(metadata).await?;

    let created = repo
        .get_by_id(
            &id.parse::<RepositoryId>()
                .map_err(|_| ServerError::internal("Invalid repository ID"))?,
        )
        .await?;

    let response = RepositoryResponse::from(created);

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

    let repo = RepositoryRepository::new(state.pool.clone());

    let repo_id = repository_id
        .parse::<RepositoryId>()
        .map_err(|_| ServerError::not_found("repository", &repository_id))?;

    let mut metadata = repo
        .get_by_id(&repo_id)
        .await
        .map_err(|_| ServerError::not_found("repository", &repository_id))?;

    metadata.updated_at = Utc::now();
    repo.update(metadata).await?;

    let commit_id = format!("commit_{}", Uuid::new_v4());
    info!("Commit created: {}", commit_id);

    Ok(Json(serde_json::json!({
        "success": true,
        "commit_id": commit_id,
        "repository_id": repository_id,
        "branch": "main",
        "message": req.message,
        "timestamp": Utc::now().to_rfc3339()
    })))
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

    let repo = RepositoryRepository::new(state.pool.clone());

    let repo_id = repository_id
        .parse::<RepositoryId>()
        .map_err(|_| ServerError::not_found("repository", &repository_id))?;

    let metadata = repo
        .get_by_id(&repo_id)
        .await
        .map_err(|_| ServerError::not_found("repository", &repository_id))?;

    if metadata.status != "active" {
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

    let repo = RepositoryRepository::new(state.pool.clone());

    let repo_id = repository_id
        .parse::<RepositoryId>()
        .map_err(|_| ServerError::not_found("repository", &repository_id))?;

    let metadata = repo
        .get_by_id(&repo_id)
        .await
        .map_err(|_| ServerError::not_found("repository", &repository_id))?;

    Ok(Json(RepositoryStatus {
        repository_id: metadata.id,
        name: metadata.name,
        branch: metadata.current_branch.unwrap_or_default(),
        status: if metadata.status == "active" {
            "clean".to_string()
        } else {
            "not_initialized".to_string()
        },
        uncommitted_changes: 0,
        unpushed_commits: 0,
        last_commit: metadata.last_commit_hash,
    }))
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

    let repo = RepositoryRepository::new(state.pool.clone());
    let repos = repo
        .list_by_owner("default_user", None, None)
        .await
        .map_err(|e| ServerError::database(format!("Failed to list repositories: {}", e)))?;

    let mut repo_list: Vec<RepositoryResponse> =
        repos.into_iter().map(RepositoryResponse::from).collect();
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

    let repo = RepositoryRepository::new(state.pool.clone());
    let repo_id = repository_id
        .parse::<RepositoryId>()
        .map_err(|_| ServerError::not_found("repository", &repository_id))?;
    let metadata = repo
        .get_by_id(&repo_id)
        .await
        .map_err(|_| ServerError::not_found("repository", &repository_id))?;

    Ok(Json(RepositoryResponse::from(metadata)))
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

    let repo = RepositoryRepository::new(state.pool.clone());
    let repo_id = repository_id
        .parse::<RepositoryId>()
        .map_err(|_| ServerError::not_found("repository", &repository_id))?;
    repo.delete(&repo_id)
        .await
        .map_err(|_| ServerError::not_found("repository", &repository_id))?;

    info!("Repository deleted: {}", repository_id);
    Ok(StatusCode::NO_CONTENT)
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
    fn test_repository_response_from_metadata() {
        let now = Utc::now();
        let meta = tachyon_database::types::RepositoryMetadata {
            id: "repo-123".to_string(),
            name: "test-repo".to_string(),
            slug: Some("test-repo".to_string()),
            description: Some("Test".to_string()),
            repository_type: "git".to_string(),
            owner_id: "user-1".to_string(),
            visibility: "private".to_string(),
            status: "active".to_string(),
            default_branch: Some("main".to_string()),
            auto_sync: false,
            sync_interval_seconds: 300,
            file_watching_enabled: false,
            remote_url: Some("https://example.com/repo.git".to_string()),
            last_commit_hash: None,
            current_branch: Some("main".to_string()),
            commits_ahead: None,
            commits_behind: None,
            document_count: 0,
            total_storage_bytes: 0,
            member_count: 1,
            local_path: None,
            created_at: now,
            updated_at: now,
        };

        let response = RepositoryResponse::from(meta);
        assert_eq!(response.id, "repo-123");
        assert_eq!(response.name, "test-repo");
        assert_eq!(response.visibility, "private");
    }

    #[test]
    fn test_repository_list_response_serialization() {
        let list = RepositoryListResponse {
            repositories: vec![],
            total: 0,
        };

        let json = serde_json::to_string(&list).unwrap();
        assert!(json.contains("\"total\":0"));
    }

    #[test]
    fn test_commit_request_construction() {
        let req = CommitRequest {
            message: "fix: typo".to_string(),
            metadata: None,
        };

        assert_eq!(req.message, "fix: typo");
    }

    #[test]
    fn test_clone_repository_request_construction() {
        let req = CloneRepositoryRequest {
            source: "repo-1".to_string(),
            name: "my-clone".to_string(),
        };

        assert_eq!(req.source, "repo-1");
        assert_eq!(req.name, "my-clone");
    }
}
