use axum::{
    Extension,
    extract::{Multipart, Query, State},
    response::Json,
    response::{IntoResponse, Redirect, Response},
};
use serde::{Deserialize, Serialize};
use std::io::Read;
use std::time::Instant;
use tachyon_core::id::DocumentId;
use tachyon_database::DocumentRepository;

use crate::error::ServerError;
use crate::middleware::AuthContext;
use tracing::{debug, warn};

#[derive(Clone)]
pub struct ImportState {
    pub pool: tachyon_database::DatabasePool,
    pub last_import: std::sync::Arc<std::sync::Mutex<std::time::Instant>>,
}

/// Notion OAuth configuration from environment/config.
#[derive(Debug, Clone)]
pub struct NotionOAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub base_url: String,
}

/// Pending Notion OAuth sessions (CSRF state -> config).
pub type NotionOAuthSessions =
    std::sync::Arc<dashmap::DashMap<String, (String, NotionOAuthConfig, String)>>;

/// State for the Notion API import flow.
#[derive(Clone)]
pub struct NotionImportState {
    pub pool: tachyon_database::DatabasePool,
    pub http_client: reqwest::Client,
    /// Pending OAuth sessions (state nonce -> (access_token_or_state, config, user_id)).
    pub oauth_sessions: NotionOAuthSessions,
    /// Notion OAuth config from env.
    pub notion_config: Option<NotionOAuthConfig>,
}

#[derive(Debug, Serialize)]
pub struct ImportResponse {
    pub imported: usize,
    pub skipped: usize,
    pub failed: usize,
    pub total: usize,
    pub warnings: Vec<String>,
    pub tags: Vec<String>,
    pub elapsed_ms: u64,
}

fn check_rate_limit(state: &ImportState) -> Result<(), ServerError> {
    let mut last = state
        .last_import
        .lock()
        .map_err(|_| ServerError::internal("Failed to acquire rate limit lock"))?;
    let elapsed = last.elapsed();
    if elapsed < std::time::Duration::from_secs(60) {
        let remaining = 60 - elapsed.as_secs();
        return Err(ServerError::rate_limited(remaining));
    }
    *last = std::time::Instant::now();
    Ok(())
}

/// Helper to extract file bytes from multipart upload, enforcing size limits.
async fn extract_upload_bytes(
    multipart: &mut Multipart,
    max_size: usize,
) -> Result<(String, Vec<u8>), ServerError> {
    let field = multipart
        .next_field()
        .await
        .map_err(|e| ServerError::bad_request(format!("Failed to parse multipart: {}", e)))?
        .ok_or_else(|| ServerError::bad_request("No file provided"))?;

    let filename = field.file_name().unwrap_or("upload").to_string();

    let bytes = field
        .bytes()
        .await
        .map_err(|e| ServerError::bad_request(format!("Failed to read file: {}", e)))?;

    if bytes.len() > max_size {
        return Err(ServerError::bad_request(format!(
            "File exceeds {}MB limit",
            max_size / (1024 * 1024)
        )));
    }

    Ok((filename, bytes.to_vec()))
}

/// Helper to persist imported documents to the database.
async fn persist_documents(
    pool: &tachyon_database::DatabasePool,
    auth: &AuthContext,
    documents: &[tachyon_import_export::ImportedDocument],
    mut warnings: Vec<String>,
) -> (usize, Vec<String>) {
    let repo = DocumentRepository::new(pool.clone());
    let mut actually_imported = 0usize;
    let mut imported_ids: Vec<(DocumentId, String, String, Vec<String>)> = Vec::new();

    for doc in documents {
        let id = DocumentId::new();
        let id_str = id.as_str();
        let tags_json = serde_json::to_string(&doc.tags).unwrap_or_else(|_| "[]".to_string());
        let frontmatter_json = serde_json::to_value(&doc.frontmatter)
            .ok()
            .and_then(|v| serde_json::to_string(&v).ok());
        let now = chrono::Utc::now();
        let created = doc.created_at.unwrap_or(now);
        let updated = doc.updated_at.unwrap_or(now);
        let content = &doc.content;
        let word_count = content.split_whitespace().count() as i32;
        let character_count = content.len() as i32;

        // Generate slug from title if not provided
        let slug = doc.slug.clone().unwrap_or_else(|| {
            let base: String = doc
                .title
                .to_lowercase()
                .chars()
                .filter_map(|c| {
                    if c.is_alphanumeric() || c == '-' || c == '_' {
                        Some(c)
                    } else if c.is_whitespace() {
                        Some('-')
                    } else {
                        None
                    }
                })
                .collect();
            // Collapse multiple hyphens and trim
            let base = base
                .split('-')
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join("-");
            if base.is_empty() {
                format!("doc-{}", &id_str[..8])
            } else {
                base
            }
        });

        let metadata = tachyon_database::DocumentMetadata {
            id: id_str.clone(),
            title: doc.title.clone(),
            slug: Some(slug),
            author_id: auth.user_id.clone(),
            description: doc.frontmatter.description.clone(),
            tags: tags_json,
            frontmatter: frontmatter_json,
            project_id: None,
            visibility: "private".to_string(),
            status: "draft".to_string(),
            content_type: "markdown".to_string(),
            word_count,
            character_count,
            read_count: 0,
            edit_count: 0,
            content: Some(content.clone()),
            html: None,
            created_at: created,
            updated_at: updated,
            published_at: None,
            content_hash: None,
            conflict_detected: Some(false),
        };

        match repo.create(metadata).await {
            Ok(()) => {
                actually_imported += 1;
                imported_ids.push((id, doc.title.clone(), doc.content.clone(), doc.tags.clone()));
            }
            Err(e) if format!("{}", e).contains("duplicate") => {
                warnings.push(format!("Skipped duplicate: {}", doc.title));
            }
            Err(e) => {
                warnings.push(format!("Failed to save '{}': {}", doc.title, e));
            }
        }
    }

    for (doc_id, title, content, tags) in &imported_ids {
        if let Err(e) = repo.update_search_index(doc_id, title, content, tags).await {
            tracing::warn!("Failed to update search index for {}: {}", doc_id, e);
        }
    }

    // Extract knowledge graph edges from imported documents
    {
        use crate::graph_extractor::{ExtractionConfig, GraphExtractor};
        let extractor = GraphExtractor::new(pool.clone(), ExtractionConfig::default());
        for (doc_id, _, _, _) in &imported_ids {
            let id_str = doc_id.as_str().to_string();
            match extractor.extract_document(&id_str).await {
                Ok(result) => {
                    debug!(
                        "Graph extraction for imported doc {}: {} nodes, {} edges",
                        doc_id, result.nodes_created, result.edges_created
                    );
                }
                Err(e) => {
                    warn!("Failed to extract graph for imported doc {}: {}", doc_id, e);
                }
            }
        }
    }

    (actually_imported, warnings)
}

/// POST /api/v1/import/markdown
///
/// Accepts a multipart form upload of a ZIP file containing markdown files.
/// Uses the MarkdownZipImporter to parse and create documents.
pub async fn import_markdown_zip(
    State(state): State<ImportState>,
    mut multipart: Multipart,
) -> Result<Json<ImportResponse>, ServerError> {
    check_rate_limit(&state)?;

    let start = Instant::now();

    let (_filename, bytes) = extract_upload_bytes(&mut multipart, 100 * 1024 * 1024).await?;

    let zip_bytes: &[u8] = &bytes;
    let (_documents, summary) =
        tachyon_import_export::MarkdownZipImporter::import_documents_from_bytes(zip_bytes)
            .map_err(|e| ServerError::bad_request(format!("Failed to import: {}", e)))?;

    let elapsed_ms = start.elapsed().as_millis() as u64;

    Ok(Json(ImportResponse {
        imported: summary.imported,
        skipped: summary.skipped,
        failed: summary.failed,
        total: summary.total_files,
        warnings: summary.warnings,
        tags: summary.all_tags,
        elapsed_ms,
    }))
}

/// POST /api/v1/import/docusaurus
///
/// Accepts a multipart form upload of a ZIP file from a Docusaurus site.
/// Uses the DocusaurusImporter to parse, preserving frontmatter metadata.
pub async fn import_docusaurus_zip(
    State(state): State<ImportState>,
    Extension(auth): Extension<AuthContext>,
    mut multipart: Multipart,
) -> Result<Json<ImportResponse>, ServerError> {
    check_rate_limit(&state)?;

    let start = Instant::now();

    let (_filename, bytes) = extract_upload_bytes(&mut multipart, 100 * 1024 * 1024).await?;

    let zip_bytes: &[u8] = &bytes;
    let (documents, summary) =
        tachyon_import_export::DocusaurusImporter::import_from_bytes(zip_bytes)
            .map_err(|e| ServerError::bad_request(format!("Failed to import: {}", e)))?;

    let warnings = summary.warnings.clone();
    let (actually_imported, warnings) =
        persist_documents(&state.pool, &auth, &documents, warnings).await;

    let elapsed_ms = start.elapsed().as_millis() as u64;

    Ok(Json(ImportResponse {
        imported: actually_imported,
        skipped: summary.skipped,
        failed: documents.len() - actually_imported + summary.failed,
        total: summary.total_files,
        warnings,
        tags: summary.all_tags,
        elapsed_ms,
    }))
}

/// POST /api/v1/import/obsidian
///
/// Accepts a multipart form upload of an Obsidian vault ZIP archive.
/// Handles wiki-links, frontmatter, inline tags, and path-derived tags.
pub async fn import_obsidian_zip(
    State(state): State<ImportState>,
    Extension(auth): Extension<AuthContext>,
    mut multipart: Multipart,
) -> Result<Json<ImportResponse>, ServerError> {
    check_rate_limit(&state)?;

    let start = Instant::now();

    let (_filename, bytes) = extract_upload_bytes(&mut multipart, 100 * 1024 * 1024).await?;

    let (documents, summary) =
        tachyon_import_export::ObsidianImporter::import_from_bytes(&bytes)
            .map_err(|e| ServerError::bad_request(format!("Failed to import: {}", e)))?;

    let warnings = summary.warnings.clone();
    let (actually_imported, warnings) =
        persist_documents(&state.pool, &auth, &documents, warnings).await;

    let elapsed_ms = start.elapsed().as_millis() as u64;

    Ok(Json(ImportResponse {
        imported: actually_imported,
        skipped: summary.skipped,
        failed: documents.len() - actually_imported + summary.failed,
        total: summary.total_files,
        warnings,
        tags: summary.all_tags,
        elapsed_ms,
    }))
}

/// POST /api/v1/import/notion
///
/// Accepts a multipart form upload of a Notion workspace export ZIP.
/// Extracts pages, maps database properties to tags, preserves page hierarchy.
pub async fn import_notion_zip(
    State(state): State<ImportState>,
    Extension(auth): Extension<AuthContext>,
    mut multipart: Multipart,
) -> Result<Json<ImportResponse>, ServerError> {
    check_rate_limit(&state)?;

    let start = Instant::now();

    let (_filename, bytes) = extract_upload_bytes(&mut multipart, 100 * 1024 * 1024).await?;

    let (documents, summary) = tachyon_import_export::NotionImporter::import_from_bytes(&bytes)
        .map_err(|e| ServerError::bad_request(format!("Failed to import: {}", e)))?;

    let warnings = summary.warnings.clone();
    let (actually_imported, warnings) =
        persist_documents(&state.pool, &auth, &documents, warnings).await;

    let elapsed_ms = start.elapsed().as_millis() as u64;

    Ok(Json(ImportResponse {
        imported: actually_imported,
        skipped: summary.skipped,
        failed: documents.len() - actually_imported + summary.failed,
        total: summary.total_files,
        warnings,
        tags: summary.all_tags,
        elapsed_ms,
    }))
}

// ============================================================================
// Notion OAuth 2.0 Import Routes
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct NotionCallbackQuery {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

/// Response for Notion import status.
#[derive(Debug, Serialize)]
pub struct NotionImportResponse {
    pub imported: usize,
    pub skipped: usize,
    pub failed: usize,
    pub total: usize,
    pub warnings: Vec<String>,
    pub tags: Vec<String>,
    pub elapsed_ms: u64,
}

/// POST /api/v1/import/notion/start
///
/// Initiates the Notion OAuth 2.0 flow.
/// Returns a redirect URL to Notion's consent screen.
pub async fn notion_start_oauth(
    State(state): State<NotionImportState>,
    Extension(auth): Extension<AuthContext>,
) -> Result<Json<serde_json::Value>, ServerError> {
    let config = state.notion_config.as_ref().ok_or_else(|| {
        ServerError::internal(
            "Notion OAuth is not configured. Set NOTION_CLIENT_ID and NOTION_CLIENT_SECRET.",
        )
    })?;

    // Generate CSRF state nonce
    use rand::RngCore;
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    let state_nonce = hex::encode(bytes);

    // Store the session (state nonce -> (state, config, user_id))
    state.oauth_sessions.insert(
        state_nonce.clone(),
        (state_nonce.clone(), config.clone(), auth.user_id.clone()),
    );

    // Build authorization URL
    let auth_url = tachyon_import_export::notion_oauth::build_authorization_url(
        &tachyon_import_export::NotionOAuthConfig::new(
            config.client_id.clone(),
            config.client_secret.clone(),
            config.base_url.clone(),
        ),
        &state_nonce,
    );

    Ok(Json(serde_json::json!({
        "authorization_url": auth_url,
        "state": state_nonce,
    })))
}

/// GET /api/v1/import/notion/callback
///
/// Handles the OAuth 2.0 callback from Notion.
/// Exchanges the authorization code for an access token,
/// then stores the token for the subsequent import step.
pub async fn notion_oauth_callback(
    State(state): State<NotionImportState>,
    Query(query): Query<NotionCallbackQuery>,
) -> Response {
    // Check for OAuth error
    if let Some(error) = &query.error {
        return axum::Json(serde_json::json!({
            "error": error,
            "description": query.error_description,
        }))
        .into_response();
    }

    // Validate state parameter
    let returned_state = match &query.state {
        Some(s) if !s.is_empty() => s.clone(),
        _ => {
            return axum::Json(serde_json::json!({
                "error": "invalid_request",
                "message": "Missing or empty state parameter"
            }))
            .into_response();
        }
    };

    // Look up and consume the OAuth session
    let session = match state.oauth_sessions.remove(&returned_state) {
        Some((_, session_data)) => session_data,
        None => {
            return axum::Json(serde_json::json!({
                "error": "invalid_state",
                "message": "No matching OAuth session found. Possible CSRF attack."
            }))
            .into_response();
        }
    };

    let (_state_token, config, user_id) = session;

    // Exchange code for token
    let code = match &query.code {
        Some(c) => c.clone(),
        None => {
            return axum::Json(serde_json::json!({
                "error": "missing_code",
                "message": "No authorization code provided"
            }))
            .into_response();
        }
    };

    let notion_config = tachyon_import_export::NotionOAuthConfig::new(
        config.client_id.clone(),
        config.client_secret.clone(),
        config.base_url.clone(),
    );

    let token = match tachyon_import_export::notion_oauth::exchange_code(
        &state.http_client,
        &notion_config,
        &code,
    )
    .await
    {
        Ok(t) => t,
        Err(e) => {
            return axum::Json(serde_json::json!({
                "error": "token_exchange_failed",
                "message": format!("Failed to exchange code: {}", e)
            }))
            .into_response();
        }
    };

    // Store the token temporarily (keyed by user_id) for the import step
    let base_url = config.base_url.clone();
    state.oauth_sessions.insert(
        format!("token:{}", user_id),
        (token.access_token.clone(), config, user_id.clone()),
    );

    tracing::info!(
        user_id = %user_id,
        workspace_id = %token.workspace_id,
        "Notion OAuth completed successfully"
    );

    // Redirect to frontend with success
    let redirect_url = format!(
        "{}/settings/import?notion=connected&workspace={}",
        &base_url, token.workspace_id
    );

    Redirect::temporary(&redirect_url).into_response()
}

/// POST /api/v1/import/notion/import
///
/// Triggers the Notion import after OAuth authentication.
/// Fetches all pages from the Notion workspace and imports them.
#[axum::debug_handler(state = NotionImportState)]
pub async fn notion_import(
    State(state): State<NotionImportState>,
) -> Result<Json<NotionImportResponse>, ServerError> {
    let start = Instant::now();

    // Find any stored access token (simplified - in production, use session cookie)
    let session_entry = state.oauth_sessions.iter().next().ok_or_else(|| {
        ServerError::unauthorized("No Notion session found. Please authenticate first.")
    })?;
    let (_token_key, (access_token, _config, _user_id)) = session_entry.pair();
    // Create Notion client
    let client = tachyon_import_export::NotionClient::new(access_token.clone());

    // Verify token works
    client
        .verify_token()
        .await
        .map_err(|e| ServerError::bad_request(format!("Failed to verify Notion token: {}", e)))?;

    // Search for all pages
    let pages = client
        .search_all_pages()
        .await
        .map_err(|e| ServerError::bad_request(format!("Failed to search Notion pages: {}", e)))?;

    let total = pages.len();
    let mut documents = Vec::new();
    let mut warnings = Vec::new();
    let mut imported = 0usize;
    let mut failed = 0usize;

    for page in &pages {
        // Skip archived pages
        if page.archived {
            continue;
        }

        // Extract title from properties
        let title = tachyon_import_export::notion::extract_page_title(&page.properties);

        // Extract tags from properties
        let tags = tachyon_import_export::notion::extract_page_tags(&page.properties);

        // Get block children for page content
        match client.get_block_children_recursive(&page.id).await {
            Ok(blocks) => {
                let content = tachyon_import_export::notion::convert_blocks_to_markdown(&blocks);

                let created_at = page
                    .created_time
                    .as_deref()
                    .and_then(tachyon_import_export::parse_date);
                let updated_at = page
                    .last_edited_time
                    .as_deref()
                    .and_then(tachyon_import_export::parse_date);

                let mut extra = std::collections::BTreeMap::new();
                extra.insert(
                    "notion_page_id".to_string(),
                    serde_json::Value::String(page.id.clone()),
                );

                documents.push(tachyon_import_export::ImportedDocument {
                    title: title.clone(),
                    slug: None,
                    content,
                    frontmatter: tachyon_import_export::Frontmatter::default(),
                    tags,
                    source_path: format!("notion/{}.md", page.id),
                    created_at,
                    updated_at,
                    extra,
                });
                imported += 1;
            }
            Err(e) => {
                warnings.push(format!("Failed to fetch blocks for '{}': {}", title, e));
                failed += 1;
            }
        }
    }

    let skipped = total - imported - failed;

    let elapsed_ms = start.elapsed().as_millis() as u64;

    Ok(Json(NotionImportResponse {
        imported,
        skipped,
        failed,
        total,
        warnings,
        tags: Vec::new(),
        elapsed_ms,
    }))
}

/// POST /api/v1/import/confluence
///
/// Accepts a multipart form upload of a Confluence XML space export.
/// Parses pages.xml, converts Confluence storage format to Markdown,
/// preserves page tree structure and labels as tags.
pub async fn import_confluence_xml(
    State(state): State<ImportState>,
    Extension(auth): Extension<AuthContext>,
    mut multipart: Multipart,
) -> Result<Json<ImportResponse>, ServerError> {
    check_rate_limit(&state)?;

    let start = Instant::now();

    let (filename, bytes) = extract_upload_bytes(&mut multipart, 200 * 1024 * 1024).await?;

    // Accept both .xml and .zip (zipped XML export)
    let (documents, summary) = if filename.ends_with(".xml") {
        tachyon_import_export::ConfluenceImporter::import_from_bytes(&bytes)
            .map_err(|e| ServerError::bad_request(format!("Failed to import: {}", e)))?
    } else if filename.ends_with(".zip") {
        // Extract pages.xml from the ZIP
        use std::io::Cursor;
        use zip::ZipArchive;

        let cursor = Cursor::new(&bytes);
        let mut archive = ZipArchive::new(cursor)
            .map_err(|e| ServerError::bad_request(format!("Invalid ZIP: {}", e)))?;

        let mut xml_bytes = Vec::new();
        let mut found = false;
        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .map_err(|e| ServerError::bad_request(format!("ZIP read error: {}", e)))?;
            let name = file.name().to_string();
            if name.ends_with(".xml") {
                file.read_to_end(&mut xml_bytes)
                    .map_err(|e| ServerError::bad_request(format!("Failed to read XML: {}", e)))?;
                found = true;
                break;
            }
        }

        if !found {
            return Err(ServerError::bad_request("No XML file found in ZIP archive"));
        }

        tachyon_import_export::ConfluenceImporter::import_from_bytes(&xml_bytes)
            .map_err(|e| ServerError::bad_request(format!("Failed to import: {}", e)))?
    } else {
        return Err(ServerError::bad_request(
            "Expected .xml or .zip file for Confluence export",
        ));
    };

    let warnings = summary.warnings.clone();
    let (actually_imported, warnings) =
        persist_documents(&state.pool, &auth, &documents, warnings).await;

    let elapsed_ms = start.elapsed().as_millis() as u64;

    Ok(Json(ImportResponse {
        imported: actually_imported,
        skipped: summary.skipped,
        failed: documents.len() - actually_imported + summary.failed,
        total: summary.total_files,
        warnings,
        tags: summary.all_tags,
        elapsed_ms,
    }))
}

/// Request body for Confluence API import.
#[derive(Debug, Deserialize)]
pub struct ConfluenceApiImportRequest {
    /// Base URL of the Confluence instance
    pub base_url: String,
    /// Authentication method: "basic" or "token"
    pub auth_type: String,
    /// Username (for basic auth)
    pub username: Option<String>,
    /// Password or API token (for basic auth) or personal access token
    pub password: Option<String>,
    /// Space key to import
    pub space_key: String,
}

/// Response for Confluence API import.
#[derive(Debug, Serialize)]
pub struct ConfluenceApiImportResponse {
    pub imported: usize,
    pub skipped: usize,
    pub failed: usize,
    pub total: usize,
    pub warnings: Vec<String>,
    pub tags: Vec<String>,
    pub elapsed_ms: u64,
    pub space_key: String,
}

/// POST /api/v1/import/confluence-api
///
/// Accepts a JSON body with Confluence API credentials and space key.
/// Fetches the space via the Confluence REST API and imports all pages.
pub async fn import_confluence_api(
    State(state): State<ImportState>,
    Extension(auth): Extension<AuthContext>,
    Json(request): Json<ConfluenceApiImportRequest>,
) -> Result<Json<ConfluenceApiImportResponse>, ServerError> {
    check_rate_limit(&state)?;

    let start = Instant::now();

    let auth_method = match request.auth_type.as_str() {
        "basic" => {
            let username = request
                .username
                .ok_or_else(|| ServerError::bad_request("username required for basic auth"))?;
            let password = request
                .password
                .ok_or_else(|| ServerError::bad_request("password required for basic auth"))?;
            tachyon_import_export::ConfluenceAuth::Basic { username, password }
        }
        "token" => {
            let token = request
                .password
                .ok_or_else(|| ServerError::bad_request("token required for token auth"))?;
            tachyon_import_export::ConfluenceAuth::PersonalAccessToken(token)
        }
        _ => {
            return Err(ServerError::bad_request(
                "auth_type must be 'basic' or 'token'",
            ));
        }
    };

    let credentials = tachyon_import_export::ConfluenceCredentials {
        base_url: request.base_url,
        auth: auth_method,
    };

    let (documents, summary) =
        tachyon_import_export::ConfluenceImporter::import_from_api(credentials, &request.space_key)
            .await
            .map_err(|e| {
                ServerError::bad_request(format!("Confluence API import failed: {}", e))
            })?;

    let warnings = summary.warnings.clone();
    let (actually_imported, warnings) =
        persist_documents(&state.pool, &auth, &documents, warnings).await;

    let elapsed_ms = start.elapsed().as_millis() as u64;

    Ok(Json(ConfluenceApiImportResponse {
        imported: actually_imported,
        skipped: summary.skipped,
        failed: documents.len() - actually_imported + summary.failed,
        total: summary.total_files,
        warnings,
        tags: summary.all_tags,
        elapsed_ms,
        space_key: request.space_key,
    }))
}

pub fn create_import_router() -> axum::Router<ImportState> {
    use axum::routing::post;

    axum::Router::new()
        .route("/import/markdown", post(import_markdown_zip))
        .route("/import/docusaurus", post(import_docusaurus_zip))
        .route("/import/obsidian", post(import_obsidian_zip))
        .route("/import/notion", post(import_notion_zip))
        .route("/import/confluence", post(import_confluence_xml))
        .route("/import/confluence-api", post(import_confluence_api))
}

/// Create a router for Notion API import (requires separate state).
pub fn create_notion_import_router() -> axum::Router<NotionImportState> {
    use axum::routing::{get, post};

    axum::Router::new()
        .route("/import/notion/start", post(notion_start_oauth))
        .route("/import/notion/callback", get(notion_oauth_callback))
        .route("/import/notion/import", post(notion_import))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_import_response_serialization() {
        let resp = ImportResponse {
            imported: 10,
            skipped: 5,
            failed: 1,
            total: 16,
            warnings: vec!["test warning".to_string()],
            tags: vec!["rust".to_string()],
            elapsed_ms: 1234,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"imported\":10"));
        assert!(json.contains("\"elapsed_ms\":1234"));
    }
}
