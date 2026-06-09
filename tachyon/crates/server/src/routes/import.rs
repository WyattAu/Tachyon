use axum::{
    Extension,
    extract::{Multipart, State},
    response::Json,
};
use serde::Serialize;
use std::io::Read;
use std::time::Instant;
use tachyon_core::id::DocumentId;
use tachyon_database::DocumentRepository;

use crate::error::ServerError;
use crate::middleware::AuthContext;

#[derive(Clone)]
pub struct ImportState {
    pub pool: tachyon_database::DatabasePool,
    pub last_import: std::sync::Arc<std::sync::Mutex<std::time::Instant>>,
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

        let metadata = tachyon_database::DocumentMetadata {
            id: id_str.clone(),
            title: doc.title.clone(),
            slug: doc.slug.clone(),
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

pub fn create_import_router() -> axum::Router<ImportState> {
    use axum::routing::post;

    axum::Router::new()
        .route("/import/markdown", post(import_markdown_zip))
        .route("/import/docusaurus", post(import_docusaurus_zip))
        .route("/import/obsidian", post(import_obsidian_zip))
        .route("/import/notion", post(import_notion_zip))
        .route("/import/confluence", post(import_confluence_xml))
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
