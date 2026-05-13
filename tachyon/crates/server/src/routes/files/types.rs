use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone)]
pub struct FilesState {
    pub root_path: PathBuf,
    pub uploads_dir: PathBuf,
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
#[serde(rename_all = "snake_case")]
pub struct ListQuery {
    pub path: Option<String>,
    pub all: Option<bool>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ListResponse {
    pub path: String,
    pub entries: Vec<EntryInfo>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct EntryInfo {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified_at: String,
    pub extension: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct ReadQuery {
    pub path: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ReadResponse {
    pub path: String,
    pub content: String,
    pub size: u64,
    pub encoding: String,
    pub frontmatter: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct SearchQuery {
    pub query: String,
    pub path: Option<String>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SearchResponse {
    pub files: Vec<SearchResultEntry>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SearchResultEntry {
    pub path: String,
    pub name: String,
    pub is_dir: bool,
    pub modified_at: String,
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct TreeQuery {
    pub path: Option<String>,
    pub depth: Option<usize>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct TreeResponse {
    pub path: String,
    #[schema(value_type = Vec<Object>)]
    pub tree: Vec<TreeNode>,
}

/// Tree node for directory listing. Not registered with utoipa ToSchema
/// because self-referential types require utoipa's `#[schema(recursive)]`
/// which is incompatible with the derive macro in utoipa 5.x.
/// Instead, the OpenAPI schema is defined manually in api_docs.rs.
#[derive(Debug, Serialize)]
pub struct TreeNode {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub children: Option<Vec<TreeNode>>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct StatsResponse {
    pub total_files: usize,
    pub total_dirs: usize,
    pub total_size_bytes: u64,
    pub file_types: std::collections::BTreeMap<String, usize>,
    pub largest_files: Vec<LargestFileEntry>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct LargestFileEntry {
    pub path: String,
    pub name: String,
    pub size: u64,
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct RecentQuery {
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct RecentResponse {
    pub files: Vec<RecentFileEntry>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct RecentFileEntry {
    pub path: String,
    pub name: String,
    pub modified_at: String,
    pub size: u64,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct UploadResponse {
    pub id: String,
    pub filename: String,
    pub url: String,
    pub size: u64,
    pub content_type: String,
    pub uploaded_at: String,
}
