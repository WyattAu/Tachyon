use crate::error::ServerError;
use axum::{
    extract::{Path, State},
    response::Json,
};
use serde::{Deserialize, Serialize};

use super::DocumentState;

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct VersionResponse {
    pub id: String,
    pub document_id: String,
    pub version_number: i32,
    pub content: String,
    pub commit_message: Option<String>,
    pub created_at: String,
    pub created_by: String,
}

impl From<tachyon_database::DocumentVersion> for VersionResponse {
    fn from(v: tachyon_database::DocumentVersion) -> Self {
        Self {
            id: v.id,
            document_id: v.document_id,
            version_number: v.version_number,
            content: v.content,
            commit_message: v.commit_message,
            created_at: v.created_at.to_rfc3339(),
            created_by: v.created_by,
        }
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateVersionBody {
    pub content: String,
    pub commit_message: Option<String>,
}

/// List document versions.
///
/// `GET /api/v1/documents/{document_id}/versions`
///
/// Returns up to 50 most recent versions.
#[utoipa::path(
    get,
    path = "/api/v1/documents/{document_id}/versions",
    params(
        ("document_id" = String, Path, description = "Document ID"),
    ),
    responses(
        (status = 200, description = "List of versions", body = Vec<VersionResponse>),
        (status = 500, description = "Internal error"),
    ),
    tag = "versions",
)]
pub async fn list_versions(
    Path(document_id): Path<String>,
    State(state): State<DocumentState>,
) -> Result<Json<Vec<VersionResponse>>, ServerError> {
    let repo = tachyon_database::DocumentVersionRepository::new(state.pool.clone());
    let versions = repo
        .list_by_document(&document_id, Some(50))
        .await
        .map_err(|e| ServerError::database(format!("Failed to list versions: {}", e)))?;

    Ok(Json(
        versions.into_iter().map(VersionResponse::from).collect(),
    ))
}

/// Get a specific document version.
///
/// `GET /api/v1/documents/{document_id}/versions/{version_number}`
#[utoipa::path(
    get,
    path = "/api/v1/documents/{document_id}/versions/{version_number}",
    params(
        ("document_id" = String, Path, description = "Document ID"),
        ("version_number" = i32, Path, description = "Version number"),
    ),
    responses(
        (status = 200, description = "Version found", body = VersionResponse),
        (status = 404, description = "Version not found"),
    ),
    tag = "versions",
)]
pub async fn get_version(
    Path((document_id, version_number)): Path<(String, i32)>,
    State(state): State<DocumentState>,
) -> Result<Json<VersionResponse>, ServerError> {
    let repo = tachyon_database::DocumentVersionRepository::new(state.pool.clone());
    let version = repo
        .get_by_version_number(&document_id, version_number)
        .await
        .map_err(|e| ServerError::not_found("Version", &format!("{}: {}", version_number, e)))?;

    Ok(Json(VersionResponse::from(version)))
}

/// Create a new document version (snapshot).
///
/// `POST /api/v1/documents/{document_id}/versions`
///
/// Saves a snapshot of the document content with an optional commit message.
#[utoipa::path(
    post,
    path = "/api/v1/documents/{document_id}/versions",
    params(
        ("document_id" = String, Path, description = "Document ID"),
    ),
    request_body = CreateVersionBody,
    responses(
        (status = 200, description = "Version created", body = VersionResponse),
        (status = 500, description = "Internal error"),
    ),
    tag = "versions",
)]
pub async fn create_version(
    Path(document_id): Path<String>,
    State(state): State<DocumentState>,
    Json(body): Json<CreateVersionBody>,
) -> Result<Json<VersionResponse>, ServerError> {
    let user_id = tachyon_core::generate_user_id();
    let repo = tachyon_database::DocumentVersionRepository::new(state.pool.clone());

    let version = repo
        .create(tachyon_database::CreateVersionRequest {
            document_id: document_id.clone(),
            content: body.content,
            commit_message: body.commit_message,
            created_by: user_id.to_string(),
        })
        .await
        .map_err(|e| ServerError::database(format!("Failed to create version: {}", e)))?;

    tracing::info!(
        "Created version {} for document {}",
        version.version_number,
        document_id
    );
    Ok(Json(VersionResponse::from(version)))
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct DiffLine {
    pub content: String,
    pub line_type: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct DocumentDiffResponse {
    pub old_lines: Vec<DiffLine>,
    pub new_lines: Vec<DiffLine>,
    pub stats: DiffStats,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct DiffStats {
    pub added: usize,
    pub removed: usize,
    pub unchanged: usize,
}

/// Compute a line-level diff between two document versions.
///
/// `GET /api/v1/documents/{document_id}/versions/{v1}/diff/{v2}`
///
/// Returns old/new line arrays with `added`, `removed`, or `unchanged` type annotations
/// and diff statistics.
#[utoipa::path(
    get,
    path = "/api/v1/documents/{document_id}/versions/{v1}/diff/{v2}",
    params(
        ("document_id" = String, Path, description = "Document ID"),
        ("v1" = i32, Path, description = "First version number"),
        ("v2" = i32, Path, description = "Second version number"),
    ),
    responses(
        (status = 200, description = "Version diff", body = DocumentDiffResponse),
        (status = 404, description = "Version not found"),
    ),
    tag = "versions",
)]
pub async fn diff_versions(
    Path((document_id, v1, v2)): Path<(String, i32, i32)>,
    State(state): State<DocumentState>,
) -> Result<Json<DocumentDiffResponse>, ServerError> {
    let repo = tachyon_database::DocumentVersionRepository::new(state.pool.clone());

    let ver1 = repo
        .get_by_version_number(&document_id, v1)
        .await
        .map_err(|e| ServerError::not_found("Version", &format!("{}: {}", v1, e)))?;

    let ver2 = repo
        .get_by_version_number(&document_id, v2)
        .await
        .map_err(|e| ServerError::not_found("Version", &format!("{}: {}", v2, e)))?;

    let diff = compute_line_diff(&ver1.content, &ver2.content);
    Ok(Json(diff))
}

fn compute_line_diff(old_content: &str, new_content: &str) -> DocumentDiffResponse {
    let old_lines: Vec<&str> = old_content.lines().collect();
    let new_lines: Vec<&str> = new_content.lines().collect();

    let lcs = longest_common_subsequence(&old_lines, &new_lines);

    let mut old_idx = 0usize;
    let mut new_idx = 0usize;
    let mut lcs_idx = 0usize;
    let mut result_old = Vec::new();
    let mut result_new = Vec::new();
    let mut added = 0usize;
    let mut removed = 0usize;
    let mut unchanged = 0usize;

    while old_idx < old_lines.len() || new_idx < new_lines.len() {
        if lcs_idx < lcs.len() {
            let lcs_line = lcs[lcs_idx];

            while old_idx < old_lines.len() && old_lines[old_idx] != lcs_line {
                result_old.push(DiffLine {
                    content: old_lines[old_idx].to_string(),
                    line_type: "removed".to_string(),
                });
                result_new.push(DiffLine {
                    content: String::new(),
                    line_type: "unchanged".to_string(),
                });
                removed += 1;
                old_idx += 1;
            }

            while new_idx < new_lines.len() && new_lines[new_idx] != lcs_line {
                result_old.push(DiffLine {
                    content: String::new(),
                    line_type: "unchanged".to_string(),
                });
                result_new.push(DiffLine {
                    content: new_lines[new_idx].to_string(),
                    line_type: "added".to_string(),
                });
                added += 1;
                new_idx += 1;
            }

            if old_idx < old_lines.len() && new_idx < new_lines.len() {
                result_old.push(DiffLine {
                    content: old_lines[old_idx].to_string(),
                    line_type: "unchanged".to_string(),
                });
                result_new.push(DiffLine {
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
                result_old.push(DiffLine {
                    content: old_lines[old_idx].to_string(),
                    line_type: "removed".to_string(),
                });
                result_new.push(DiffLine {
                    content: String::new(),
                    line_type: "unchanged".to_string(),
                });
                removed += 1;
                old_idx += 1;
            }
            while new_idx < new_lines.len() {
                result_old.push(DiffLine {
                    content: String::new(),
                    line_type: "unchanged".to_string(),
                });
                result_new.push(DiffLine {
                    content: new_lines[new_idx].to_string(),
                    line_type: "added".to_string(),
                });
                added += 1;
                new_idx += 1;
            }
        }
    }

    DocumentDiffResponse {
        old_lines: result_old,
        new_lines: result_new,
        stats: DiffStats {
            added,
            removed,
            unchanged,
        },
    }
}

fn longest_common_subsequence<'a>(old: &[&'a str], new: &[&'a str]) -> Vec<&'a str> {
    let m = old.len();
    let n = new.len();
    if m == 0 || n == 0 {
        return Vec::new();
    }

    let mut dp = vec![vec![0usize; n + 1]; m + 1];
    for i in 1..=m {
        for j in 1..=n {
            if old[i - 1] == new[j - 1] {
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
        if old[i - 1] == new[j - 1] {
            result.push(old[i - 1]);
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
