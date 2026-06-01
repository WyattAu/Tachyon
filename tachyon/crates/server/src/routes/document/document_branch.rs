use crate::error::ServerError;
use axum::{
    extract::{Extension, Path, State},
    response::Json,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::info;

use super::DocumentState;

#[derive(Debug, Deserialize)]
pub struct CreateBranchRequest {
    pub branch_name: String,
}

#[derive(Debug, Serialize)]
pub struct BranchResponse {
    pub id: String,
    pub document_id: String,
    pub branch_name: String,
    pub source_content_hash: String,
    pub source_version: i32,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct MergeBranchRequest {
    pub content: Option<String>,
    pub strategy: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MergeResponse {
    pub merged: bool,
    pub document_id: String,
    pub branch_id: String,
    pub new_version: i32,
    pub conflicts: Option<Vec<MergeConflict>>,
}

#[derive(Debug, Serialize)]
pub struct MergeConflict {
    pub position: usize,
    pub ours: String,
    pub theirs: String,
    pub description: String,
}

#[derive(Debug, Serialize)]
pub struct BranchListResponse {
    pub branches: Vec<BranchResponse>,
    pub total: i64,
}

#[derive(Debug, Deserialize)]
pub struct UpdateBranchRequest {
    pub content: String,
}

pub async fn list_branches(
    Path(document_id): Path<String>,
    State(state): State<DocumentState>,
) -> Result<Json<BranchListResponse>, ServerError> {
    let mut conn = state
        .pool
        .acquire()
        .await
        .map_err(|e| ServerError::database(e.to_string()))?;

    #[allow(clippy::type_complexity)]
    let rows: Vec<(
        String,
        String,
        String,
        String,
        i32,
        String,
        chrono::DateTime<chrono::Utc>,
    )> = sqlx::query_as(
        r#"SELECT id, document_id, branch_name, source_content_hash, source_version, status, created_at
               FROM document_branches
               WHERE document_id = $1
               ORDER BY created_at DESC"#,
    )
    .bind(&document_id)
    .fetch_all(&mut *conn)
    .await
    .map_err(|e| ServerError::database(e.to_string()))?;

    let total = rows.len() as i64;
    let branches = rows
        .into_iter()
        .map(|r| BranchResponse {
            id: r.0,
            document_id: r.1,
            branch_name: r.2,
            source_content_hash: r.3,
            source_version: r.4,
            status: r.5,
            created_at: r.6.to_rfc3339(),
        })
        .collect();

    Ok(Json(BranchListResponse { branches, total }))
}

pub async fn create_branch(
    Extension(auth): Extension<crate::middleware::AuthContext>,
    Path(document_id): Path<String>,
    State(state): State<DocumentState>,
    Json(req): Json<CreateBranchRequest>,
) -> Result<Json<BranchResponse>, ServerError> {
    let user_id = auth.user_id.clone();

    if req.branch_name.trim().is_empty() {
        return Err(ServerError::bad_request("Branch name is required"));
    }

    if !req
        .branch_name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err(ServerError::bad_request(
            "Branch name must be alphanumeric (hyphens and underscores allowed)",
        ));
    }

    let mut conn = state
        .pool
        .acquire()
        .await
        .map_err(|e| ServerError::database(e.to_string()))?;

    let doc: Option<(String, i32)> =
        sqlx::query_as("SELECT content, COALESCE(edit_count, 0) FROM documents WHERE id = $1 AND deleted_at IS NULL")
            .bind(&document_id)
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| ServerError::database(e.to_string()))?;

    let (content, version) = doc.ok_or_else(|| ServerError::not_found("Document", &document_id))?;

    let hash = hex::encode(Sha256::digest(content.as_bytes()));

    let existing: Option<(String,)> = sqlx::query_as(
        "SELECT id FROM document_branches WHERE document_id = $1 AND branch_name = $2 AND status = 'open'",
    )
    .bind(&document_id)
    .bind(&req.branch_name)
    .fetch_optional(&mut *conn)
    .await
    .map_err(|e| ServerError::database(e.to_string()))?;

    if existing.is_some() {
        return Err(ServerError::conflict(format!(
            "Branch '{}' already exists",
            req.branch_name
        )));
    }

    let row: (
        String,
        String,
        String,
        String,
        i32,
        String,
        chrono::DateTime<chrono::Utc>,
    ) = sqlx::query_as(
        r#"INSERT INTO document_branches (document_id, branch_name, source_content, source_content_hash, source_version, branched_by)
               VALUES ($1, $2, $3, $4, $5, $6)
               RETURNING id, document_id, branch_name, source_content_hash, source_version, status, created_at"#,
    )
    .bind(&document_id)
    .bind(&req.branch_name)
    .bind(&content)
    .bind(&hash)
    .bind(version)
    .bind(&user_id)
    .fetch_one(&mut *conn)
    .await
    .map_err(|e| ServerError::database(e.to_string()))?;

    info!(
        document_id = %document_id,
        branch = %req.branch_name,
        user_id = %user_id,
        "Branch created"
    );

    Ok(Json(BranchResponse {
        id: row.0,
        document_id: row.1,
        branch_name: row.2,
        source_content_hash: row.3,
        source_version: row.4,
        status: row.5,
        created_at: row.6.to_rfc3339(),
    }))
}

pub async fn update_branch(
    Extension(_auth): Extension<crate::middleware::AuthContext>,
    Path((document_id, branch_id)): Path<(String, String)>,
    State(state): State<DocumentState>,
    Json(req): Json<UpdateBranchRequest>,
) -> Result<Json<serde_json::Value>, ServerError> {
    let mut conn = state
        .pool
        .acquire()
        .await
        .map_err(|e| ServerError::database(e.to_string()))?;

    let branch: Option<(String, String)> = sqlx::query_as(
        "SELECT id, branch_name FROM document_branches WHERE id = $1 AND document_id = $2 AND status = 'open'",
    )
    .bind(&branch_id)
    .bind(&document_id)
    .fetch_optional(&mut *conn)
    .await
    .map_err(|e| ServerError::database(e.to_string()))?;

    branch.ok_or_else(|| ServerError::not_found("Branch", &branch_id))?;

    let hash = hex::encode(Sha256::digest(req.content.as_bytes()));

    sqlx::query(
        "UPDATE document_branches SET source_content = $1, source_content_hash = $2, updated_at = NOW() WHERE id = $3",
    )
    .bind(&req.content)
    .bind(&hash)
    .bind(&branch_id)
    .execute(&mut *conn)
    .await
    .map_err(|e| ServerError::database(e.to_string()))?;

    Ok(Json(
        serde_json::json!({ "updated": true, "content_hash": hash }),
    ))
}

pub async fn merge_branch(
    Extension(auth): Extension<crate::middleware::AuthContext>,
    Path((document_id, branch_id)): Path<(String, String)>,
    State(state): State<DocumentState>,
    Json(req): Json<MergeBranchRequest>,
) -> Result<Json<MergeResponse>, ServerError> {
    let user_id = auth.user_id.clone();
    let strategy = req.strategy.as_deref().unwrap_or("auto");

    let mut conn = state
        .pool
        .acquire()
        .await
        .map_err(|e| ServerError::database(e.to_string()))?;

    let branch: Option<(String, String, String, i32)> = sqlx::query_as(
        "SELECT id, source_content, source_content_hash, source_version FROM document_branches WHERE id = $1 AND document_id = $2 AND status = 'open'",
    )
    .bind(&branch_id)
    .bind(&document_id)
    .fetch_optional(&mut *conn)
    .await
    .map_err(|e| ServerError::database(e.to_string()))?;

    let (bid, branch_content, branch_hash, _source_version) =
        branch.ok_or_else(|| ServerError::not_found("Branch", &branch_id))?;

    let doc: Option<(String, i32)> =
        sqlx::query_as("SELECT content, COALESCE(edit_count, 0) + 1 FROM documents WHERE id = $1 AND deleted_at IS NULL")
            .bind(&document_id)
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| ServerError::database(e.to_string()))?;

    let (current_content, new_version) =
        doc.ok_or_else(|| ServerError::not_found("Document", &document_id))?;

    let merge_content = req.content.as_deref().unwrap_or(&branch_content);

    let current_hash = hex::encode(Sha256::digest(current_content.as_bytes()));

    let (merged_content, conflicts) = if current_hash == branch_hash || strategy == "force" {
        (merge_content.to_string(), None)
    } else {
        let merge_result = simple_three_way_merge(&branch_content, &current_content, merge_content);
        (
            merge_result.merged_content,
            if merge_result.has_conflicts {
                Some(merge_result.conflicts)
            } else {
                None
            },
        )
    };

    let content_hash = hex::encode(Sha256::digest(merged_content.as_bytes()));

    sqlx::query(
        "UPDATE documents SET content = $1, content_hash = $2, edit_count = $3, updated_at = NOW(), conflict_detected = $4 WHERE id = $5",
    )
    .bind(&merged_content)
    .bind(&content_hash)
    .bind(new_version)
    .bind(conflicts.is_some())
    .bind(&document_id)
    .execute(&mut *conn)
    .await
    .map_err(|e| ServerError::database(e.to_string()))?;

    sqlx::query(
        "UPDATE document_branches SET status = 'merged', merged_at = NOW(), merged_by = $1, updated_at = NOW() WHERE id = $2",
    )
    .bind(&user_id)
    .bind(&bid)
    .execute(&mut *conn)
    .await
    .map_err(|e| ServerError::database(e.to_string()))?;

    let has_conflicts = conflicts.as_ref().is_some_and(|c| !c.is_empty());

    info!(
        document_id = %document_id,
        branch_id = %bid,
        version = new_version,
        conflicts = has_conflicts,
        "Branch merged"
    );

    Ok(Json(MergeResponse {
        merged: true,
        document_id: document_id.clone(),
        branch_id: bid,
        new_version,
        conflicts,
    }))
}

struct ThreeWayMergeResult {
    merged_content: String,
    conflicts: Vec<MergeConflict>,
    has_conflicts: bool,
}

fn simple_three_way_merge(base: &str, current: &str, branch: &str) -> ThreeWayMergeResult {
    let base_lines: Vec<&str> = base.lines().collect();
    let current_lines: Vec<&str> = current.lines().collect();
    let branch_lines: Vec<&str> = branch.lines().collect();

    let mut conflicts = Vec::new();
    let mut merged = Vec::new();

    let mut current_idx = 0;
    let mut branch_idx = 0;

    while current_idx < current_lines.len() || branch_idx < branch_lines.len() {
        let current_line = current_lines.get(current_idx).copied().unwrap_or("");
        let branch_line = branch_lines.get(branch_idx).copied().unwrap_or("");
        let base_line = base_lines
            .get(current_idx.min(branch_idx))
            .copied()
            .unwrap_or("");

        if current_line == branch_line {
            merged.push(current_line.to_string());
            current_idx += 1;
            branch_idx += 1;
        } else if current_line == base_line {
            merged.push(branch_line.to_string());
            current_idx += 1;
            branch_idx += 1;
        } else if branch_line == base_line {
            merged.push(current_line.to_string());
            current_idx += 1;
            branch_idx += 1;
        } else {
            let pos = merged.len();
            conflicts.push(MergeConflict {
                position: pos,
                ours: current_line.to_string(),
                theirs: branch_line.to_string(),
                description: format!(
                    "Line {}: both versions changed from '{}'",
                    pos + 1,
                    base_line
                ),
            });
            merged.push(current_line.to_string());
            current_idx += 1;
            branch_idx += 1;
        }
    }

    let has_conflicts = !conflicts.is_empty();

    ThreeWayMergeResult {
        merged_content: merged.join("\n"),
        conflicts,
        has_conflicts,
    }
}

#[derive(Debug, Serialize)]
pub struct BranchDiffResponse {
    pub branch_content: String,
    pub document_content: String,
    pub added_lines: Vec<String>,
    pub removed_lines: Vec<String>,
}

pub async fn diff_branch(
    Path((document_id, branch_id)): Path<(String, String)>,
    State(state): State<DocumentState>,
) -> Result<Json<BranchDiffResponse>, ServerError> {
    let mut conn = state
        .pool
        .acquire()
        .await
        .map_err(|e| ServerError::database(e.to_string()))?;

    let branch: Option<(String, String)> = sqlx::query_as(
        "SELECT id, source_content FROM document_branches WHERE id = $1 AND document_id = $2 AND status = 'open'",
    )
    .bind(&branch_id)
    .bind(&document_id)
    .fetch_optional(&mut *conn)
    .await
    .map_err(|e| ServerError::database(e.to_string()))?;

    let (_bid, branch_content) =
        branch.ok_or_else(|| ServerError::not_found("Branch", &branch_id))?;

    let doc: Option<(String,)> =
        sqlx::query_as("SELECT content FROM documents WHERE id = $1 AND deleted_at IS NULL")
            .bind(&document_id)
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| ServerError::database(e.to_string()))?;

    let (doc_row,) = doc.ok_or_else(|| ServerError::not_found("Document", &document_id))?;
    let document_content = doc_row;

    let branch_lines: std::collections::HashSet<&str> = branch_content.lines().collect();
    let doc_lines: std::collections::HashSet<&str> = document_content.lines().collect();

    let added_lines: Vec<String> = branch_lines
        .difference(&doc_lines)
        .map(|s| s.to_string())
        .collect();
    let removed_lines: Vec<String> = doc_lines
        .difference(&branch_lines)
        .map(|s| s.to_string())
        .collect();

    Ok(Json(BranchDiffResponse {
        branch_content,
        document_content,
        added_lines,
        removed_lines,
    }))
}

pub async fn delete_branch(
    Extension(_auth): Extension<crate::middleware::AuthContext>,
    Path((document_id, branch_id)): Path<(String, String)>,
    State(state): State<DocumentState>,
) -> Result<Json<serde_json::Value>, ServerError> {
    let mut conn = state
        .pool
        .acquire()
        .await
        .map_err(|e| ServerError::database(e.to_string()))?;

    sqlx::query(
        "UPDATE document_branches SET status = 'abandoned', updated_at = NOW() WHERE id = $1 AND document_id = $2 AND status = 'open'",
    )
    .bind(&branch_id)
    .bind(&document_id)
    .execute(&mut *conn)
    .await
    .map_err(|e| ServerError::database(e.to_string()))?;

    Ok(Json(serde_json::json!({ "deleted": true })))
}
