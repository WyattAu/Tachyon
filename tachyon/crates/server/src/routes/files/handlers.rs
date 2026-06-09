#![allow(private_interfaces)]

use super::types::*;
use crate::audit::{AuditEvent, AuditEventType, AuditSeverity};
use crate::error::ServerError;
use axum::{
    extract::{Multipart, Query, State},
    http::{StatusCode, header},
    response::{IntoResponse, Json, Response},
};
use std::path::{Path, PathBuf};
use std::pin::Pin;
use tokio::fs;

const ALLOWED_EXTENSIONS: &[&str] = &[".md", ".txt", ".json", ".yaml", ".yml", ".toml"];

const UPLOAD_ALLOWED_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "webp", "svg", "pdf", "doc", "docx", "xls", "xlsx", "txt", "md",
    "csv", "json",
];

const MAX_UPLOAD_SIZE: usize = 50 * 1024 * 1024;

async fn resolve_path(root: &Path, relative: &str) -> Result<PathBuf, ServerError> {
    if let Err(e) = tachyon_core::validate_path(relative) {
        return Err(ServerError::bad_request(format!(
            "Path validation failed: {}",
            e
        )));
    }

    let joined = root.join(relative.strip_prefix('/').unwrap_or(relative));

    let canonical = tokio::fs::canonicalize(&joined).await.map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            ServerError::NotFound(format!("Path not found: {}", relative))
        } else {
            ServerError::bad_request(format!("Cannot resolve path: {}", e))
        }
    })?;

    let root_canonical = tokio::fs::canonicalize(root)
        .await
        .unwrap_or_else(|_| root.to_path_buf());
    if !canonical.starts_with(&root_canonical) {
        return Err(ServerError::forbidden(
            "Access denied: path is outside root directory",
        ));
    }

    Ok(canonical)
}

fn format_modified(metadata: &std::fs::Metadata) -> String {
    metadata
        .modified()
        .ok()
        .map(|t| {
            let dt: chrono::DateTime<chrono::Utc> = t.into();
            dt.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
        })
        .unwrap_or_default()
}

fn is_allowed_ext(name: &str) -> bool {
    let ext = std::path::Path::new(name)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| format!(".{}", e.to_lowercase()));
    match ext {
        Some(ref e) => ALLOWED_EXTENSIONS.contains(&e.as_str()),
        None => false,
    }
}

fn is_upload_allowed_ext(ext: &str) -> bool {
    UPLOAD_ALLOWED_EXTENSIONS.contains(&ext.to_lowercase().as_str())
}

fn is_binary_file(path: &Path) -> bool {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
    {
        Some(ref ext) => matches!(
            ext.as_str(),
            "png"
                | "jpg"
                | "jpeg"
                | "gif"
                | "webp"
                | "svg"
                | "pdf"
                | "doc"
                | "docx"
                | "xls"
                | "xlsx"
        ),
        None => false,
    }
}

fn content_type_for_ext(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
    {
        Some(ref ext) => match ext.as_str() {
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "webp" => "image/webp",
            "svg" => "image/svg+xml",
            "pdf" => "application/pdf",
            "doc" => "application/msword",
            "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            "xls" => "application/vnd.ms-excel",
            "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            "txt" => "text/plain",
            "md" => "text/markdown",
            "csv" => "text/csv",
            "json" => "application/json",
            "yaml" | "yml" => "application/x-yaml",
            "toml" => "application/x-toml",
            _ => "application/octet-stream",
        },
        None => "application/octet-stream",
    }
}

fn expected_content_type(ext: &str) -> &'static str {
    match ext {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "pdf" => "application/pdf",
        "doc" => "application/msword",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xls" => "application/vnd.ms-excel",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "txt" => "text/plain",
        "md" => "text/markdown",
        "csv" => "text/csv",
        "json" => "application/json",
        _ => "application/octet-stream",
    }
}

fn should_show(name: &str, is_dir: bool, show_all: bool) -> bool {
    if name.starts_with('.') {
        return false;
    }
    if is_dir || show_all {
        return true;
    }
    is_allowed_ext(name)
}

/// List files and directories.
///
/// `GET /api/v1/files/list?path=...&all=...`
///
/// Returns entries sorted with directories first. Supports `path` and `all` query parameters.
#[utoipa::path(
    get,
    path = "/files/list",
    params(
        ListQuery,
    ),
    responses(
        (status = 200, description = "Directory listing", body = ListResponse),
        (status = 400, description = "Invalid path"),
        (status = 404, description = "Not found"),
    ),
    tag = "files",
    security(("bearer_auth" = [])),
)]
pub async fn list_directory(
    State(state): State<FilesState>,
    Query(params): Query<ListQuery>,
) -> Result<Json<ListResponse>, ServerError> {
    let relative = params.path.as_deref().unwrap_or("/");
    let show_all = params.all.unwrap_or(false);
    let target = resolve_path(&state.root_path, relative).await?;

    let metadata = fs::metadata(&target).await.map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            ServerError::NotFound(format!("Directory not found: {}", relative))
        } else {
            ServerError::internal(format!("Failed to read directory: {}", e))
        }
    })?;

    if !metadata.is_dir() {
        return Err(ServerError::bad_request("Path is not a directory"));
    }

    let mut entries = fs::read_dir(&target)
        .await
        .map_err(|e| ServerError::internal(format!("Failed to read directory: {}", e)))?;

    let mut result = Vec::new();

    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|e| ServerError::internal(format!("Failed to read directory entry: {}", e)))?
    {
        let name = entry.file_name().to_string_lossy().to_string();
        let meta = entry
            .metadata()
            .await
            .map_err(|e| ServerError::internal(format!("Failed to read metadata: {}", e)))?;
        let is_dir = meta.is_dir();

        if !should_show(&name, is_dir, show_all) {
            continue;
        }

        result.push(EntryInfo {
            name,
            is_dir,
            size: meta.len(),
            modified_at: format_modified(&meta),
            extension: entry
                .path()
                .extension()
                .and_then(|e| e.to_str())
                .map(|s| s.to_string()),
        });
    }

    result.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.cmp(&b.name),
    });

    Ok(Json(ListResponse {
        path: relative.to_string(),
        entries: result,
    }))
}

fn parse_frontmatter(content: &str) -> (Option<serde_json::Value>, &str) {
    let (fm, body) = tachyon_import_export::Frontmatter::parse(content);
    // Frontmatter::parse returns default struct when no frontmatter found.
    // If body == content, no frontmatter was actually parsed.
    let has_frontmatter = body != content;
    let value = if has_frontmatter {
        serde_json::to_value(&fm).ok().filter(|v| !v.is_null())
    } else {
        None
    };
    (value, body)
}

/// Read a file's contents.
///
/// `GET /api/v1/files/read?path=...`
///
/// Returns JSON with content, size, encoding, and parsed YAML frontmatter for text files.
/// Returns raw binary content for image/PDF files.
#[utoipa::path(
    get,
    path = "/files/read",
    params(
        ReadQuery,
    ),
    responses(
        (status = 200, description = "File content", body = serde_json::Value),
        (status = 400, description = "Invalid path or is directory"),
        (status = 404, description = "Not found"),
    ),
    tag = "files",
    security(("bearer_auth" = [])),
)]
pub async fn read_file(
    State(state): State<FilesState>,
    Query(params): Query<ReadQuery>,
) -> Result<Response, ServerError> {
    let target = resolve_path(&state.root_path, &params.path).await?;

    let metadata = fs::metadata(&target).await.map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            ServerError::NotFound(format!("File not found: {}", params.path))
        } else {
            ServerError::internal(format!("Failed to read file metadata: {}", e))
        }
    })?;

    if metadata.is_dir() {
        return Err(ServerError::bad_request("Path is a directory, not a file"));
    }

    if is_binary_file(&target) {
        let data = fs::read(&target)
            .await
            .map_err(|e| ServerError::internal(format!("Failed to read file: {}", e)))?;
        let content_type = content_type_for_ext(&target);
        let size = data.len();

        let response = Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, content_type)
            .header(header::CONTENT_LENGTH, size.to_string())
            .body(axum::body::Body::from(data))
            .map_err(|e| ServerError::internal(format!("Failed to build response: {}", e)))?;
        return Ok(response);
    }

    let content = fs::read_to_string(&target)
        .await
        .map_err(|e| ServerError::internal(format!("Failed to read file: {}", e)))?;

    let size = metadata.len();
    let (frontmatter, body) = parse_frontmatter(&content);

    Ok(Json(ReadResponse {
        path: params.path.clone(),
        content: body.to_string(),
        size,
        encoding: "utf-8".to_string(),
        frontmatter,
    })
    .into_response())
}

/// Search files by name (recursive).
///
/// `GET /api/v1/files/search?query=...&path=...`
///
/// Performs a case-insensitive filename search under the specified path (defaults to root).
#[utoipa::path(
    get,
    path = "/files/search",
    params(
        SearchQuery,
    ),
    responses(
        (status = 200, description = "File search results", body = SearchResponse),
        (status = 400, description = "Not a directory"),
    ),
    tag = "files",
    security(("bearer_auth" = [])),
)]
pub async fn search_files(
    State(state): State<FilesState>,
    Query(params): Query<SearchQuery>,
) -> Result<Json<SearchResponse>, ServerError> {
    let query_lower = params.query.to_lowercase();
    let base = resolve_path(&state.root_path, params.path.as_deref().unwrap_or("/")).await?;

    if !base.is_dir() {
        return Err(ServerError::bad_request(
            "Specified path is not a directory",
        ));
    }

    let root_canonical = tokio::fs::canonicalize(&state.root_path)
        .await
        .unwrap_or_else(|_| state.root_path.clone());
    let mut results = Vec::new();
    Box::pin(walk_dir_search(
        &base,
        &root_canonical,
        &query_lower,
        &mut results,
    ))
    .await?;

    results.sort_by_key(|a| a.path.clone());

    Ok(Json(SearchResponse { files: results }))
}

fn walk_dir_search<'a>(
    dir: &'a PathBuf,
    root: &'a PathBuf,
    query: &'a str,
    results: &'a mut Vec<SearchResultEntry>,
) -> Pin<Box<dyn std::future::Future<Output = Result<(), ServerError>> + Send + 'a>> {
    Box::pin(async move {
        let mut entries = fs::read_dir(dir)
            .await
            .map_err(|e| ServerError::internal(format!("Failed to read directory: {}", e)))?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| ServerError::internal(format!("Failed to read directory entry: {}", e)))?
        {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            if name.starts_with('.') {
                continue;
            }

            let meta = entry
                .metadata()
                .await
                .map_err(|e| ServerError::internal(format!("Failed to read metadata: {}", e)))?;

            let relative = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .to_string();

            if name.to_lowercase().contains(query) {
                results.push(SearchResultEntry {
                    path: format!("/{}", relative),
                    name,
                    is_dir: meta.is_dir(),
                    modified_at: format_modified(&meta),
                });
            }

            if meta.is_dir() {
                Box::pin(walk_dir_search(&path, root, query, results)).await?;
            }
        }

        Ok(())
    })
}

/// Get a recursive directory tree.
///
/// `GET /api/v1/files/tree?path=...&depth=...`
///
/// Returns a nested tree structure. Maximum depth is 5 (defaults to 2).
#[utoipa::path(
    get,
    path = "/files/tree",
    params(
        TreeQuery,
    ),
    responses(
        (status = 200, description = "Directory tree", body = TreeResponse),
        (status = 400, description = "Invalid path"),
        (status = 404, description = "Not found"),
    ),
    tag = "files",
    security(("bearer_auth" = [])),
)]
pub async fn get_tree(
    State(state): State<FilesState>,
    Query(params): Query<TreeQuery>,
) -> Result<Json<TreeResponse>, ServerError> {
    let relative = params.path.as_deref().unwrap_or("/");
    let max_depth = params.depth.unwrap_or(2).min(5);
    let target = resolve_path(&state.root_path, relative).await?;

    let metadata = fs::metadata(&target).await.map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            ServerError::NotFound(format!("Directory not found: {}", relative))
        } else {
            ServerError::internal(format!("Failed to read metadata: {}", e))
        }
    })?;

    if !metadata.is_dir() {
        return Err(ServerError::bad_request("Path is not a directory"));
    }

    let root_canonical = tokio::fs::canonicalize(&state.root_path)
        .await
        .unwrap_or_else(|_| state.root_path.clone());
    let children = Box::pin(build_tree(&target, &root_canonical, 0, max_depth)).await?;

    Ok(Json(TreeResponse {
        path: relative.to_string(),
        tree: children,
    }))
}

fn build_tree<'a>(
    dir: &'a PathBuf,
    root: &'a PathBuf,
    current_depth: usize,
    max_depth: usize,
) -> Pin<Box<dyn std::future::Future<Output = Result<Vec<TreeNode>, ServerError>> + Send + 'a>> {
    Box::pin(async move {
        if current_depth >= max_depth {
            return Ok(Vec::new());
        }

        let mut entries = fs::read_dir(dir)
            .await
            .map_err(|e| ServerError::internal(format!("Failed to read directory: {}", e)))?;

        let mut nodes = Vec::new();

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| ServerError::internal(format!("Failed to read directory entry: {}", e)))?
        {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            if name.starts_with('.') {
                continue;
            }

            let meta = entry
                .metadata()
                .await
                .map_err(|e| ServerError::internal(format!("Failed to read metadata: {}", e)))?;

            let is_dir = meta.is_dir();
            let relative = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .to_string();

            if !is_dir && !is_allowed_ext(&name) {
                continue;
            }

            let children = if is_dir {
                Some(Box::pin(build_tree(&path, root, current_depth + 1, max_depth)).await?)
            } else {
                None
            };

            nodes.push(TreeNode {
                name,
                path: format!("/{}", relative),
                is_dir,
                children,
            });
        }

        nodes.sort_by(|a, b| match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.cmp(&b.name),
        });

        Ok(nodes)
    })
}

/// Get file system statistics.
///
/// `GET /api/v1/files/stats`
///
/// Returns total files, directories, size, file type breakdown, and top 10 largest files.
#[utoipa::path(
    get,
    path = "/files/stats",
    responses(
        (status = 200, description = "File system statistics", body = StatsResponse),
        (status = 500, description = "Internal error"),
    ),
    tag = "files",
    security(("bearer_auth" = [])),
)]
pub async fn get_stats(
    State(state): State<FilesState>,
) -> Result<Json<StatsResponse>, ServerError> {
    let root = tokio::fs::canonicalize(&state.root_path)
        .await
        .unwrap_or_else(|_| state.root_path.clone());

    let mut total_files = 0usize;
    let mut total_dirs = 0usize;
    let mut total_size = 0u64;
    let mut file_types = std::collections::BTreeMap::new();
    let mut largest_files: Vec<LargestFileEntry> = Vec::new();

    Box::pin(walk_dir_stats(
        &root,
        &root,
        &mut total_files,
        &mut total_dirs,
        &mut total_size,
        &mut file_types,
        &mut largest_files,
    ))
    .await?;

    largest_files.sort_by_key(|b| std::cmp::Reverse(b.size));
    largest_files.truncate(10);

    Ok(Json(StatsResponse {
        total_files,
        total_dirs,
        total_size_bytes: total_size,
        file_types,
        largest_files,
    }))
}

fn walk_dir_stats<'a>(
    dir: &'a PathBuf,
    root: &'a PathBuf,
    total_files: &'a mut usize,
    total_dirs: &'a mut usize,
    total_size: &'a mut u64,
    file_types: &'a mut std::collections::BTreeMap<String, usize>,
    largest_files: &'a mut Vec<LargestFileEntry>,
) -> Pin<Box<dyn std::future::Future<Output = Result<(), ServerError>> + Send + 'a>> {
    Box::pin(async move {
        let mut entries = fs::read_dir(dir)
            .await
            .map_err(|e| ServerError::internal(format!("Failed to read directory: {}", e)))?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| ServerError::internal(format!("Failed to read directory entry: {}", e)))?
        {
            let name = entry.file_name().to_string_lossy().to_string();

            if name.starts_with('.') {
                continue;
            }

            let meta = entry
                .metadata()
                .await
                .map_err(|e| ServerError::internal(format!("Failed to read metadata: {}", e)))?;

            if meta.is_dir() {
                *total_dirs += 1;
                Box::pin(walk_dir_stats(
                    &entry.path(),
                    root,
                    total_files,
                    total_dirs,
                    total_size,
                    file_types,
                    largest_files,
                ))
                .await?;
            } else {
                *total_files += 1;
                let size = meta.len();
                *total_size += size;

                let ext = entry
                    .path()
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|e| format!(".{}", e.to_lowercase()))
                    .unwrap_or_else(|| ".other".to_string());
                *file_types.entry(ext).or_insert(0) += 1;

                let relative = entry
                    .path()
                    .strip_prefix(root)
                    .unwrap_or(&entry.path())
                    .to_string_lossy()
                    .to_string();

                largest_files.push(LargestFileEntry {
                    path: format!("/{}", relative),
                    name,
                    size,
                });
            }
        }

        Ok(())
    })
}

/// Get recently modified files.
///
/// `GET /api/v1/files/recent?limit=...`
///
/// Returns files sorted by modification time. Default limit is 20, max 100.
#[utoipa::path(
    get,
    path = "/files/recent",
    params(
        RecentQuery,
    ),
    responses(
        (status = 200, description = "Recently modified files", body = RecentResponse),
        (status = 500, description = "Internal error"),
    ),
    tag = "files",
    security(("bearer_auth" = [])),
)]
pub async fn get_recent_files(
    State(state): State<FilesState>,
    Query(params): Query<RecentQuery>,
) -> Result<Json<RecentResponse>, ServerError> {
    let limit = params.limit.unwrap_or(20).min(100);
    let root = tokio::fs::canonicalize(&state.root_path)
        .await
        .unwrap_or_else(|_| state.root_path.clone());
    let mut all_files = Vec::new();

    Box::pin(walk_dir_recent(&root, &root, &mut all_files, 10_000)).await?;

    all_files.sort_by(|a, b| b.modified_at.cmp(&a.modified_at));
    all_files.truncate(limit);

    Ok(Json(RecentResponse { files: all_files }))
}

fn walk_dir_recent<'a>(
    dir: &'a PathBuf,
    root: &'a PathBuf,
    files: &'a mut Vec<RecentFileEntry>,
    max_files: usize,
) -> Pin<Box<dyn std::future::Future<Output = Result<(), ServerError>> + Send + 'a>> {
    Box::pin(async move {
        let mut entries = fs::read_dir(dir)
            .await
            .map_err(|e| ServerError::internal(format!("Failed to read directory: {}", e)))?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| ServerError::internal(format!("Failed to read directory entry: {}", e)))?
        {
            let name = entry.file_name().to_string_lossy().to_string();

            if name.starts_with('.') {
                continue;
            }

            let meta = entry
                .metadata()
                .await
                .map_err(|e| ServerError::internal(format!("Failed to read metadata: {}", e)))?;

            if meta.is_dir() {
                Box::pin(walk_dir_recent(&entry.path(), root, files, max_files)).await?;
                if files.len() >= max_files {
                    return Ok(());
                }
            } else {
                if !is_allowed_ext(&name) {
                    continue;
                }

                let relative = entry
                    .path()
                    .strip_prefix(root)
                    .unwrap_or(&entry.path())
                    .to_string_lossy()
                    .to_string();

                files.push(RecentFileEntry {
                    path: format!("/{}", relative),
                    name,
                    modified_at: format_modified(&meta),
                    size: meta.len(),
                });

                if files.len() >= max_files {
                    return Ok(());
                }
            }
        }

        Ok(())
    })
}

/// Upload a file.
///
/// `POST /api/v1/files/upload`
///
/// Accepts multipart form data. Validates file extension and content type.
/// Maximum file size is 50 MB.
#[utoipa::path(
    post,
    path = "/files/upload",
    responses(
        (status = 200, description = "Upload successful", body = serde_json::Value),
        (status = 400, description = "Invalid file or missing fields"),
        (status = 500, description = "Internal error"),
    ),
    tag = "files",
    security(("bearer_auth" = [])),
)]
pub async fn upload_file(
    State(state): State<FilesState>,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, ServerError> {
    let field = multipart
        .next_field()
        .await
        .map_err(|e| ServerError::internal(format!("Failed to parse multipart: {}", e)))?
        .ok_or_else(|| ServerError::bad_request("No file provided in upload"))?;

    let filename = field
        .file_name()
        .ok_or_else(|| ServerError::bad_request("File has no filename"))?
        .to_string();

    if filename.is_empty() {
        return Err(ServerError::bad_request("Filename is empty"));
    }

    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        return Err(ServerError::bad_request(
            "Filename contains invalid characters",
        ));
    }

    let content_type = field
        .content_type()
        .ok_or_else(|| ServerError::bad_request("File has no content type"))?
        .to_string();

    let ext = Path::new(&filename)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .ok_or_else(|| ServerError::bad_request("File has no extension"))?;

    if !is_upload_allowed_ext(&ext) {
        return Err(ServerError::bad_request(format!(
            "File type .{} is not allowed",
            ext
        )));
    }

    let expected = expected_content_type(&ext);
    if !content_type.contains(expected.split('/').next().unwrap_or("")) {
        return Err(ServerError::bad_request(format!(
            "Content-Type {} does not match expected type for .{}",
            content_type, ext
        )));
    }

    let data = field
        .bytes()
        .await
        .map_err(|e| ServerError::internal(format!("Failed to read file data: {}", e)))?;

    if data.len() > MAX_UPLOAD_SIZE {
        return Err(ServerError::bad_request("File size exceeds 50MB limit"));
    }

    let safe_name = format!(
        "{}{}",
        uuid::Uuid::new_v4(),
        Path::new(&filename)
            .extension()
            .map(|e| format!(".{}", e.to_string_lossy().to_lowercase()))
            .unwrap_or_default()
    );

    if let Err(e) = fs::create_dir_all(&state.uploads_dir).await {
        return Err(ServerError::internal(format!(
            "Failed to create uploads directory: {}",
            e
        )));
    }

    let upload_path = state.uploads_dir.join(&safe_name);
    if let Err(e) = fs::write(&upload_path, &data).await {
        return Err(ServerError::internal(format!("Failed to save file: {}", e)));
    }

    let _ = state
        .audit_logger
        .log(
            AuditEvent::new(
                AuditEventType::FileUploaded,
                AuditSeverity::Low,
                "file_upload",
                format!("File '{}' uploaded ({} bytes)", filename, data.len()),
            )
            .with_metadata("filename", serde_json::json!(filename))
            .with_metadata("size", serde_json::json!(data.len())),
        )
        .await;

    Ok(Json(UploadResponse {
        id: uuid::Uuid::new_v4().to_string(),
        filename,
        url: format!("/files/uploads/{}", safe_name),
        size: data.len() as u64,
        content_type,
        uploaded_at: chrono::Utc::now().to_rfc3339(),
    }))
}

pub fn create_files_router() -> axum::Router<FilesState> {
    use axum::routing::{get, post};

    axum::Router::new()
        .route("/files/list", get(list_directory))
        .route("/files/read", get(read_file))
        .route("/files/search", get(search_files))
        .route("/files/tree", get(get_tree))
        .route("/files/stats", get(get_stats))
        .route("/files/recent", get(get_recent_files))
        .route("/files/upload", post(upload_file))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_allowed_ext_md() {
        assert!(is_allowed_ext("test.md"));
        assert!(is_allowed_ext("test.MD"));
        assert!(is_allowed_ext("test.Txt"));
    }

    #[test]
    fn test_is_allowed_ext_json() {
        assert!(is_allowed_ext("config.json"));
        assert!(is_allowed_ext("data.JSON"));
    }

    #[test]
    fn test_is_allowed_ext_yaml() {
        assert!(is_allowed_ext("config.yaml"));
        assert!(is_allowed_ext("config.yml"));
    }

    #[test]
    fn test_is_allowed_ext_toml() {
        assert!(is_allowed_ext("Cargo.toml"));
    }

    #[test]
    fn test_is_allowed_ext_html() {
        assert!(!is_allowed_ext("index.html"));
    }

    #[test]
    fn test_is_allowed_ext_disallowed() {
        assert!(!is_allowed_ext("image.png"));
        assert!(!is_allowed_ext("script.js"));
        assert!(!is_allowed_ext("style.css"));
        assert!(!is_allowed_ext("data.csv"));
        assert!(!is_allowed_ext("archive.zip"));
    }

    #[test]
    fn test_is_allowed_ext_no_extension() {
        assert!(!is_allowed_ext("Makefile"));
        assert!(!is_allowed_ext("README"));
    }

    #[test]
    fn test_should_show_hidden_files() {
        assert!(!should_show(".gitignore", false, false));
        assert!(!should_show(".hidden", false, false));
        assert!(!should_show(".env", true, false));
    }

    #[test]
    fn test_should_show_directories() {
        assert!(should_show("src", true, false));
        assert!(should_show("docs", true, false));
    }

    #[test]
    fn test_should_show_allowed_files() {
        assert!(should_show("readme.md", false, false)); // .md is allowed
        assert!(should_show("readme.md", false, true)); // show_all also works
        assert!(!should_show("data.csv", false, false)); // .csv not allowed
    }

    #[test]
    fn test_should_show_all_flag() {
        assert!(should_show("data.csv", false, true));
        assert!(should_show("image.png", false, true));
    }

    #[test]
    fn test_parse_frontmatter_no_frontmatter() {
        let content = "# Hello\n\nSome content";
        let (fm, body) = parse_frontmatter(content);
        assert!(fm.is_none());
        assert_eq!(body, content);
    }

    #[test]
    fn test_parse_frontmatter_with_yaml() {
        let content = "---\ntitle: Test\ndate: 2024-01-01\n---\n\n# Hello\n\nContent here";
        let (fm, body) = parse_frontmatter(content);
        assert!(fm.is_some());
        let fm = fm.unwrap();
        assert_eq!(fm.get("title").and_then(|v| v.as_str()), Some("Test"));
        assert!(body.contains("# Hello"));
    }

    #[test]
    fn test_parse_frontmatter_incomplete() {
        let content = "---\ntitle: Test\nNo closing marker";
        let (fm, body) = parse_frontmatter(content);
        assert!(fm.is_none());
        assert_eq!(body, content);
    }

    #[test]
    fn test_parse_frontmatter_empty_yaml() {
        let content = "---\n---\n\nBody content";
        let (fm, body) = parse_frontmatter(content);
        assert!(fm.is_some());
        assert!(body.contains("Body content"));
    }

    #[test]
    fn test_parse_frontmatter_with_tags() {
        let content = "---\ntitle: Doc\ntags:\n  - test\n  - integration\n---\n\nContent";
        let (fm, body) = parse_frontmatter(content);
        assert!(fm.is_some());
        let fm = fm.unwrap();
        let tags = fm.get("tags").and_then(|v| v.as_array());
        assert!(tags.is_some());
        assert_eq!(tags.unwrap().len(), 2);
        assert!(body.contains("Content"));
    }

    #[test]
    fn test_list_query_deserialization() {
        let json = r#"{"path":"/src","all":true}"#;
        let query: ListQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.path.as_deref(), Some("/src"));
        assert_eq!(query.all, Some(true));
    }

    #[test]
    fn test_read_query_deserialization() {
        let json = r#"{"path":"/README.md"}"#;
        let query: ReadQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.path, "/README.md");
    }

    #[test]
    fn test_search_query_deserialization() {
        let json = r#"{"query":"test","path":"/docs"}"#;
        let query: SearchQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.query, "test");
        assert_eq!(query.path.as_deref(), Some("/docs"));
    }

    #[test]
    fn test_tree_query_deserialization() {
        let json = r#"{"path":"/","depth":3}"#;
        let query: TreeQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.path.as_deref(), Some("/"));
        assert_eq!(query.depth, Some(3));
    }

    #[test]
    fn test_recent_query_deserialization() {
        let json = r#"{"limit":50}"#;
        let query: RecentQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.limit, Some(50));
    }

    #[test]
    fn test_entry_info_serialization() {
        let entry = EntryInfo {
            name: "readme.md".to_string(),
            is_dir: false,
            size: 1024,
            modified_at: "2026-01-01T00:00:00+00:00".to_string(),
            extension: Some("md".to_string()),
        };
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("readme.md"));
        assert!(json.contains("\"is_dir\":false"));
        assert!(json.contains("\"size\":1024"));
    }

    #[test]
    fn test_list_response_serialization() {
        let resp = ListResponse {
            path: "/src".to_string(),
            entries: vec![EntryInfo {
                name: "main.rs".to_string(),
                is_dir: false,
                size: 512,
                modified_at: "2026-01-01T00:00:00+00:00".to_string(),
                extension: Some("rs".to_string()),
            }],
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("/src"));
        assert!(json.contains("main.rs"));
    }
}
