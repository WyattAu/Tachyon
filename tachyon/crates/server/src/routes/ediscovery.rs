//! eDiscovery export endpoint for legal review.
//!
//! Exports a filtered document set as a ZIP archive containing:
//! - Individual `.md` files with YAML frontmatter metadata
//! - `metadata.json` with export details (date, filters, document count, SHA-256 hashes)
//! - `index.csv` with document listing (id, title, author, date, tags, file_path)

use axum::{
    Json,
    extract::State,
    http::{HeaderMap, HeaderName, HeaderValue},
    response::IntoResponse,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::Row;
use std::io::{Cursor, Write};
use tracing::info;
use uuid::Uuid;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

use crate::error::ServerError;

/// State required by the eDiscovery endpoints.
#[derive(Clone)]
pub struct EdiscoveryState {
    pub pool: tachyon_database::DatabasePool,
}

/// Request body for eDiscovery export.
#[derive(Debug, Deserialize)]
pub struct EdiscoveryExportRequest {
    /// Filter by documents created/updated after this date (ISO 8601).
    pub date_from: Option<String>,
    /// Filter by documents created/updated before this date (ISO 8601).
    pub date_to: Option<String>,
    /// Filter by author user ID.
    pub author_id: Option<String>,
    /// Filter by tags (any match).
    pub tags: Option<Vec<String>>,
    /// Full-text content search query.
    pub content_search: Option<String>,
    /// Filter by project/space ID.
    pub space_id: Option<String>,
    /// Exporter name or identifier for chain-of-custody.
    pub exporter: Option<String>,
    /// Purpose/statement for chain-of-custody.
    pub purpose: Option<String>,
}

/// Chain-of-custody metadata included in the export.
#[derive(Debug, Serialize, Deserialize)]
pub struct ChainOfCustody {
    pub exporter: String,
    pub exported_at: String,
    pub purpose: String,
    pub export_id: String,
}

/// Export metadata written to `metadata.json`.
#[derive(Debug, Serialize, Deserialize)]
pub struct ExportMetadata {
    pub export_id: String,
    pub exported_at: String,
    pub chain_of_custody: ChainOfCustody,
    pub filters_applied: FiltersApplied,
    pub document_count: usize,
    pub document_hashes: Vec<DocumentHash>,
}

/// Record of which filters were applied during export.
#[derive(Debug, Serialize, Deserialize)]
pub struct FiltersApplied {
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub author_id: Option<String>,
    pub tags: Option<Vec<String>>,
    pub content_search: Option<String>,
    pub space_id: Option<String>,
}

/// SHA-256 hash of a document's content for integrity verification.
#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentHash {
    pub document_id: String,
    pub title: String,
    pub sha256: String,
    pub byte_size: usize,
}

/// A single row in the `index.csv`.
#[derive(Debug, Serialize)]
pub struct CsvIndexRow {
    pub id: String,
    pub title: String,
    pub author_id: String,
    pub created_at: String,
    pub updated_at: String,
    pub tags: String,
    pub file_path: String,
}

/// Create the eDiscovery router with all endpoints.
pub fn create_ediscovery_router() -> axum::Router<EdiscoveryState> {
    use axum::routing::post;
    axum::Router::new().route("/ediscovery/export", post(export_ediscovery))
}

/// The POST /api/v1/ediscovery/export endpoint.
///
/// Queries the database for documents matching the filters, builds a ZIP
/// archive, and returns it as a binary download.
pub async fn export_ediscovery(
    State(state): State<EdiscoveryState>,
    Json(request): Json<EdiscoveryExportRequest>,
) -> Result<impl IntoResponse, ServerError> {
    let export_id = Uuid::new_v4().to_string();
    let exported_at = Utc::now().to_rfc3339();
    let exporter_name = request.exporter.as_deref().unwrap_or("system");
    let purpose = request
        .purpose
        .as_deref()
        .unwrap_or("eDiscovery legal review");

    info!(
        export_id = %export_id,
        "Starting eDiscovery export"
    );

    let mut conn = state
        .pool
        .acquire()
        .await
        .map_err(|e| ServerError::database(e.to_string()))?;

    // Build dynamic SQL query based on filters
    let mut conditions = vec!["deleted_at IS NULL".to_string()];

    if request.date_from.is_some() {
        conditions.push(format!("created_at >= ${}", conditions.len()));
    }
    if request.date_to.is_some() {
        conditions.push(format!("created_at <= ${}", conditions.len()));
    }
    if request.author_id.is_some() {
        conditions.push(format!("author_id::text = ${}", conditions.len()));
    }
    if request.space_id.is_some() {
        conditions.push(format!("project_id::text = ${}", conditions.len()));
    }
    if request.content_search.is_some() {
        conditions.push(format!(
            "to_tsvector('english', COALESCE(content, '')) @@ to_tsquery('english', ${})",
            conditions.len()
        ));
    }
    if let Some(ref tags) = request.tags
        && !tags.is_empty()
    {
        let tag_conditions: Vec<String> = tags
            .iter()
            .enumerate()
            .map(|(i, _)| format!("tags::jsonb @> ${}::jsonb", conditions.len() + 1 + i))
            .collect();
        conditions.push(tag_conditions.join(" OR "));
    }

    let where_clause = conditions.join(" AND ");

    let sql = format!(
        r#"
        SELECT
            id::text as id,
            title,
            slug,
            author_id::text as author_id,
            tags::text as tags,
            content,
            created_at,
            updated_at
        FROM documents
        WHERE {}
        ORDER BY created_at DESC
        "#,
        where_clause
    );

    let mut query = sqlx::query(&sql);

    if let Some(ref date_from) = request.date_from {
        query = query.bind(date_from);
    }
    if let Some(ref date_to) = request.date_to {
        query = query.bind(date_to);
    }
    if let Some(ref author_id) = request.author_id {
        query = query.bind(author_id);
    }
    if let Some(ref space_id) = request.space_id {
        query = query.bind(space_id);
    }
    if let Some(ref content_search) = request.content_search {
        query = query.bind(content_search);
    }
    if let Some(ref tags) = request.tags {
        for tag in tags {
            let tag_json = serde_json::to_string(&vec![tag]).unwrap_or_else(|_| "[]".to_string());
            query = query.bind(tag_json);
        }
    }

    let rows = query
        .fetch_all(&mut *conn)
        .await
        .map_err(|e| ServerError::database(e.to_string()))?;

    // Collect document data from rows
    let mut documents: Vec<ExportedDoc> = Vec::new();
    for row in &rows {
        let id: String = row.get("id");
        let title: String = row.get("title");
        let content: Option<String> = row.get("content");
        let author_id: String = row.get("author_id");
        let tags: Option<String> = row.get("tags");
        let created_at: Option<DateTime<Utc>> = row.get("created_at");
        let updated_at: Option<DateTime<Utc>> = row.get("updated_at");
        let slug: Option<String> = row.get("slug");

        documents.push(ExportedDoc {
            id,
            title,
            content: content.unwrap_or_default(),
            author_id,
            tags: tags.unwrap_or_else(|| "[]".to_string()),
            created_at: created_at.map(|dt| dt.to_rfc3339()).unwrap_or_default(),
            updated_at: updated_at.map(|dt| dt.to_rfc3339()).unwrap_or_default(),
            slug,
        });
    }

    let document_count = documents.len();
    info!(
        export_id = %export_id,
        document_count = document_count,
        "eDiscovery export found documents"
    );

    // Build ZIP archive
    let zip_bytes = build_ediscovery_zip(
        &documents,
        &export_id,
        &exported_at,
        exporter_name,
        purpose,
        &request,
    )?;

    // Build response headers for ZIP download
    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("content-type"),
        HeaderValue::from_static("application/zip"),
    );
    headers.insert(
        HeaderName::from_static("content-disposition"),
        HeaderValue::from_bytes(
            format!(
                "attachment; filename=\"ediscovery-{}.zip\"",
                &export_id[..8]
            )
            .as_bytes(),
        )
        .unwrap_or_else(|_| HeaderValue::from_static("attachment")),
    );

    Ok((headers, zip_bytes))
}

/// Internal representation of a document for ZIP export.
struct ExportedDoc {
    id: String,
    title: String,
    content: String,
    author_id: String,
    tags: String,
    created_at: String,
    updated_at: String,
    slug: Option<String>,
}

/// Build the eDiscovery ZIP archive from document data.
fn build_ediscovery_zip(
    documents: &[ExportedDoc],
    export_id: &str,
    exported_at: &str,
    exporter: &str,
    purpose: &str,
    request: &EdiscoveryExportRequest,
) -> Result<Vec<u8>, ServerError> {
    let buf = Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(buf);
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .compression_level(Some(6));

    let mut document_hashes = Vec::new();
    let mut csv_rows: Vec<CsvIndexRow> = Vec::new();

    // Write each document as a .md file with metadata header
    for doc in documents {
        let filename = format!(
            "{}.md",
            doc.slug
                .as_deref()
                .unwrap_or(&sanitize_filename(&doc.title))
        );

        // Compute SHA-256 hash of content
        let mut hasher = Sha256::new();
        hasher.update(doc.content.as_bytes());
        let hash = format!("{:x}", hasher.finalize());

        // Build markdown file with YAML frontmatter
        let md_content = format!(
            "---\ntitle: \"{}\"\ndocument_id: \"{}\"\nauthor_id: \"{}\"\ncreated_at: \"{}\"\nupdated_at: \"{}\"\ntags: {}\n---\n\n{}",
            escape_yaml_string(&doc.title),
            doc.id,
            doc.author_id,
            doc.created_at,
            doc.updated_at,
            doc.tags,
            doc.content
        );

        let byte_size = md_content.len();

        zip.start_file(&filename, options)
            .map_err(|e| ServerError::internal(format!("Failed to add file to ZIP: {}", e)))?;
        zip.write_all(md_content.as_bytes())
            .map_err(|e| ServerError::internal(format!("Failed to write file to ZIP: {}", e)))?;

        document_hashes.push(DocumentHash {
            document_id: doc.id.clone(),
            title: doc.title.clone(),
            sha256: hash,
            byte_size,
        });

        csv_rows.push(CsvIndexRow {
            id: doc.id.clone(),
            title: doc.title.clone(),
            author_id: doc.author_id.clone(),
            created_at: doc.created_at.clone(),
            updated_at: doc.updated_at.clone(),
            tags: doc.tags.clone(),
            file_path: filename.clone(),
        });
    }

    // Write metadata.json
    let export_metadata = ExportMetadata {
        export_id: export_id.to_string(),
        exported_at: exported_at.to_string(),
        chain_of_custody: ChainOfCustody {
            exporter: exporter.to_string(),
            exported_at: exported_at.to_string(),
            purpose: purpose.to_string(),
            export_id: export_id.to_string(),
        },
        filters_applied: FiltersApplied {
            date_from: request.date_from.clone(),
            date_to: request.date_to.clone(),
            author_id: request.author_id.clone(),
            tags: request.tags.clone(),
            content_search: request.content_search.clone(),
            space_id: request.space_id.clone(),
        },
        document_count: documents.len(),
        document_hashes,
    };

    let metadata_json = serde_json::to_string_pretty(&export_metadata)
        .map_err(|e| ServerError::internal(format!("Failed to serialize metadata: {}", e)))?;

    zip.start_file("metadata.json", options)
        .map_err(|e| ServerError::internal(format!("Failed to add metadata.json: {}", e)))?;
    zip.write_all(metadata_json.as_bytes())
        .map_err(|e| ServerError::internal(format!("Failed to write metadata.json: {}", e)))?;

    // Write index.csv
    let mut csv_content = String::from("id,title,author_id,created_at,updated_at,tags,file_path\n");
    for row in &csv_rows {
        csv_content.push_str(&format!(
            "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"\n",
            escape_csv_string(&row.id),
            escape_csv_string(&row.title),
            escape_csv_string(&row.author_id),
            escape_csv_string(&row.created_at),
            escape_csv_string(&row.updated_at),
            escape_csv_string(&row.tags),
            escape_csv_string(&row.file_path),
        ));
    }

    zip.start_file("index.csv", options)
        .map_err(|e| ServerError::internal(format!("Failed to add index.csv: {}", e)))?;
    zip.write_all(csv_content.as_bytes())
        .map_err(|e| ServerError::internal(format!("Failed to write index.csv: {}", e)))?;

    let buf = zip
        .finish()
        .map_err(|e| ServerError::internal(format!("Failed to finalize ZIP: {}", e)))?;

    Ok(buf.into_inner())
}

/// Sanitize a string for use as a filename.
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Escape a string for safe inclusion in YAML.
fn escape_yaml_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

/// Escape a string for CSV fields.
fn escape_csv_string(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("Hello World!"), "hello-world");
        assert_eq!(sanitize_filename("API  Reference"), "api-reference");
        assert_eq!(sanitize_filename("C++ Guide"), "c-guide");
        assert_eq!(sanitize_filename(""), "");
    }

    #[test]
    fn test_escape_yaml_string() {
        assert_eq!(escape_yaml_string("hello"), "hello");
        assert_eq!(escape_yaml_string("say \"hi\""), "say \\\"hi\\\"");
        assert_eq!(escape_yaml_string("line1\nline2"), "line1\\nline2");
    }

    #[test]
    fn test_escape_csv_string() {
        assert_eq!(escape_csv_string("hello"), "hello");
        assert_eq!(escape_csv_string("hello,world"), "\"hello,world\"");
        assert_eq!(escape_csv_string("say \"hi\""), "\"say \"\"hi\"\"\"");
    }

    #[test]
    fn test_document_hash_structure() {
        let hash = DocumentHash {
            document_id: "doc-123".to_string(),
            title: "Test".to_string(),
            sha256: "abc123".to_string(),
            byte_size: 100,
        };
        assert_eq!(hash.document_id, "doc-123");
        assert_eq!(hash.byte_size, 100);
    }

    #[test]
    fn test_export_metadata_serialization() {
        let metadata = ExportMetadata {
            export_id: "test-export".to_string(),
            exported_at: "2026-01-01T00:00:00Z".to_string(),
            chain_of_custody: ChainOfCustody {
                exporter: "admin".to_string(),
                exported_at: "2026-01-01T00:00:00Z".to_string(),
                purpose: "Legal review".to_string(),
                export_id: "test-export".to_string(),
            },
            filters_applied: FiltersApplied {
                date_from: None,
                date_to: None,
                author_id: None,
                tags: None,
                content_search: None,
                space_id: None,
            },
            document_count: 0,
            document_hashes: vec![],
        };
        let json = serde_json::to_string(&metadata).unwrap();
        assert!(json.contains("export_id"));
        assert!(json.contains("chain_of_custody"));
    }

    #[test]
    fn test_csv_row_serialization() {
        let row = CsvIndexRow {
            id: "doc-1".to_string(),
            title: "Test Doc".to_string(),
            author_id: "user-1".to_string(),
            created_at: "2026-01-01".to_string(),
            updated_at: "2026-01-02".to_string(),
            tags: "[\"tag1\"]".to_string(),
            file_path: "test-doc.md".to_string(),
        };
        let json = serde_json::to_string(&row).unwrap();
        assert!(json.contains("Test Doc"));
    }
}
