//! SSG (Static Site Generator) API routes
//! Generate static sites from Tachyon documents.

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde::{Deserialize, Serialize};
use tachyon_ssg::{SiteConfig as SsgSiteConfig, SiteGenerator, SsgDocument, BuildResult};
use tracing::{info, warn};

/// State for SSG routes
#[derive(Clone)]
pub struct SsgState {
    pub pool: tachyon_database::DatabasePool,
}

impl SsgState {
    pub fn new(pool: tachyon_database::DatabasePool) -> Self {
        Self { pool }
    }
}

// ============================================================================
// Request/Response types
// ============================================================================

/// SSG site configuration request
#[derive(Debug, Deserialize)]
pub struct SsgBuildRequest {
    #[serde(alias = "site_title")]
    pub title: Option<String>,
    pub description: Option<String>,
    pub base_url: Option<String>,
    pub theme: Option<String>,
    pub custom_css: Option<String>,
    pub nav_links: Option<Vec<SsgNavLink>>,
    pub group_by_tag: Option<bool>,
    pub project_id: Option<String>,
    pub limit: Option<usize>,
}

/// SSG configuration response
#[derive(Debug, Serialize)]
pub struct SsgConfigResponse {
    pub site_title: String,
    pub site_description: String,
    pub base_url: String,
    pub theme: String,
    pub nav_links: Vec<SsgNavLink>,
}

/// Navigation link in SSG config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SsgNavLink {
    pub label: String,
    pub href: String,
}

/// SSG build response
#[derive(Debug, Serialize)]
pub struct SsgBuildResponse {
    pub success: bool,
    pub result: SsgBuildResultWrapper,
}

/// Wrapper for BuildResult to use in JSON
#[derive(Debug, Serialize)]
pub struct SsgBuildResultWrapper {
    pub pages: usize,
    pub category_pages: usize,
    pub total_files: usize,
    pub build_time_ms: u64,
    pub output_size_bytes: u64,
    pub generated_pages: Vec<String>,
}

impl From<BuildResult> for SsgBuildResultWrapper {
    fn from(r: BuildResult) -> Self {
        Self {
            pages: r.pages,
            category_pages: r.category_pages,
            total_files: r.total_files,
            build_time_ms: r.build_time_ms,
            output_size_bytes: r.output_size_bytes,
            generated_pages: r.generated_pages,
        }
    }
}

/// Error response
#[derive(Debug, Serialize)]
pub struct SsgErrorResponse {
    pub code: String,
    pub message: String,
}

// ============================================================================
// Route handlers
// ============================================================================

/// GET /api/v1/ssg/config — Return current SSG configuration
pub async fn get_ssg_config() -> Json<SsgConfigResponse> {
    Json(SsgConfigResponse {
        site_title: "Tachyon".to_string(),
        site_description: "A deterministic, high-performance knowledge management system".to_string(),
        base_url: "http://localhost:8080".to_string(),
        theme: "auto".to_string(),
        nav_links: vec![],
    })
}

/// POST /api/v1/ssg/build — Generate static site as ZIP
pub async fn build_site(
    State(state): State<SsgState>,
    Json(req): Json<SsgBuildRequest>,
) -> Result<Json<SsgBuildResponse>, (StatusCode, Json<SsgErrorResponse>)> {
    info!("SSG build request received");

    let documents = fetch_documents_for_ssg(&state.pool, req.project_id.as_deref(), req.limit.unwrap_or(0))
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(SsgErrorResponse {
                    code: "FETCH_ERROR".to_string(),
                    message: format!("Failed to fetch documents: {}", e),
                }),
            )
        })?;

    if documents.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(SsgErrorResponse {
                code: "NO_DOCUMENTS".to_string(),
                message: "No documents found to generate site".to_string(),
            }),
        ));
    }

    let ssg_config = SsgSiteConfig {
        title: req.title.unwrap_or_else(|| "Tachyon Docs".to_string()),
        description: req.description.unwrap_or_else(|| "A knowledge base built with Tachyon".to_string()),
        base_url: req.base_url.unwrap_or_else(|| "https://docs.example.com".to_string()),
        theme: req.theme.unwrap_or_else(|| "auto".to_string()),
        custom_css: req.custom_css,
        nav_links: req
            .nav_links
            .unwrap_or_default()
            .into_iter()
            .map(|l| tachyon_ssg::NavLink { label: l.label, href: l.href })
            .collect(),
        group_by_tag: req.group_by_tag.unwrap_or(false),
        ..Default::default()
    };

    let generator = SiteGenerator::new(ssg_config);

    let result = tokio::task::spawn_blocking(move || generator.build_to_zip(&documents))
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(SsgErrorResponse {
                    code: "BUILD_ERROR".to_string(),
                    message: format!("Task join error: {}", e),
                }),
            )
        })?
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(SsgErrorResponse {
                    code: "BUILD_ERROR".to_string(),
                    message: format!("SSG build failed: {}", e),
                }),
            )
        })?;

    let (_zip_bytes, build_result) = result;

    info!(
        "SSG build complete: {} pages, {}ms",
        build_result.pages,
        build_result.build_time_ms,
    );

    Ok(Json(SsgBuildResponse {
        success: true,
        result: build_result.into(),
    }))
}

/// GET /api/v1/ssg/download — Download the generated site as ZIP
pub async fn download_site(
    State(state): State<SsgState>,
) -> Response {
    let documents = fetch_documents_for_ssg(&state.pool, None, 0)
        .await
        .unwrap_or_else(|_| {
            warn!("Failed to fetch documents for SSG download");
            vec![]
        });

    if documents.is_empty() {
        return (
            StatusCode::NO_CONTENT,
            [(axum::http::header::CONTENT_TYPE, "text/plain")],
            "No documents found",
        )
            .into_response();
    }

    let ssg_config = SsgSiteConfig::default();
    let generator = SiteGenerator::new(ssg_config);

    match tokio::task::spawn_blocking(move || generator.build_to_zip(&documents)).await {
        Ok(Ok((zip_bytes, _result))) => {
            let headers = [
                (axum::http::header::CONTENT_TYPE, "application/zip".to_string()),
                (
                    axum::http::header::CONTENT_DISPOSITION,
                    "attachment; filename=\"tachyon-site.zip\"".to_string(),
                ),
            ];
            (StatusCode::OK, headers, zip_bytes).into_response()
        }
        Ok(Err(e)) => {
            warn!("SSG download build failed: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Build failed").into_response()
        }
        Err(e) => {
            warn!("SSG download task failed: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Task error").into_response()
        }
    }
}

// ============================================================================
// Helper: Fetch documents from database
// ============================================================================

/// Row type for fetching documents from the database.
#[derive(Debug, sqlx::FromRow)]
struct DocRow {
    #[allow(dead_code)]
    id: String,
    title: String,
    slug: String,
    content: Option<String>,
    description: Option<String>,
    tags: Option<serde_json::Value>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
    updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Fetch documents from the database for SSG generation.
async fn fetch_documents_for_ssg(
    pool: &tachyon_database::DatabasePool,
    project_id: Option<&str>,
    limit: usize,
) -> Result<Vec<SsgDocument>, String> {
    let limit_val = if limit > 0 { limit as i64 } else { 10000i64 };

    let (sql, _params): (String, Vec<String>) = if let Some(pid) = project_id {
        (
            format!(
                "SELECT id::text, title, slug, content, description, tags, created_at, updated_at \
                 FROM documents \
                 WHERE status = 'published' AND project_id = '{}'::uuid \
                 ORDER BY updated_at DESC \
                 LIMIT {}",
                pid, limit_val,
            ),
            vec![],
        )
    } else {
        (
            format!(
                "SELECT id::text, title, slug, content, description, tags, created_at, updated_at \
                 FROM documents \
                 WHERE status = 'published' \
                 ORDER BY updated_at DESC \
                 LIMIT {}",
                limit_val,
            ),
            vec![],
        )
    };

    let mut conn = pool.acquire().await.map_err(|e| format!("Failed to acquire connection: {}", e))?;

    let rows: Vec<DocRow> = sqlx::query_as(&sql)
        .fetch_all(&mut *conn)
        .await
        .map_err(|e| format!("Query failed: {}", e))?;

    let mut docs = Vec::new();
    let mut order = 0i32;

    for row in rows {
        let tags: Vec<String> = row
            .tags
            .and_then(|t| {
                if let Some(arr) = t.as_array() {
                    Some(
                        arr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect(),
                    )
                } else {
                    None
                }
            })
            .unwrap_or_default();

        let created_at = row.created_at.unwrap_or_else(chrono::Utc::now);
        let updated_at = row.updated_at.unwrap_or_else(chrono::Utc::now);

        docs.push(SsgDocument {
            slug: row.slug,
            title: row.title,
            content: row.content.unwrap_or_default(),
            description: row.description,
            author: None,
            tags,
            created_at,
            updated_at,
            order,
            language: "en".to_string(),
        });
        order += 1;
    }

    Ok(docs)
}

// ============================================================================
// Router
// ============================================================================

/// Create the SSG router
pub fn create_ssg_router() -> axum::Router<SsgState> {
    use axum::routing::{get, post};

    axum::Router::new()
        .route("/ssg/config", get(get_ssg_config))
        .route("/ssg/build", post(build_site))
        .route("/ssg/download", get(download_site))
}
