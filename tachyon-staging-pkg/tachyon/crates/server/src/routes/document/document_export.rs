//! Document export endpoints.
//!
//! Supports PDF, JSON, and Markdown ZIP export formats.

use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, HeaderName, HeaderValue},
    response::IntoResponse,
};
use serde::Deserialize;
use tracing::info;

use super::DocumentState;
use crate::error::ServerError;
use crate::middleware::AuthContext;

#[derive(Debug, Deserialize)]
pub struct ExportQuery {
    pub format: Option<String>,
}

/// Returns whether a caller may export a document.
///
/// Export is intentionally stricter than public document viewing: only the
/// owner or an administrator may receive an export, regardless of visibility.
pub(crate) fn can_export_document(
    caller_id: Option<&str>,
    author_id: &str,
    is_admin: bool,
) -> bool {
    is_admin || caller_id == Some(author_id)
}

fn export_headers(content_type: &str, filename: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("content-type"),
        HeaderValue::from_bytes(content_type.as_bytes())
            .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream")),
    );
    headers.insert(
        HeaderName::from_static("content-disposition"),
        HeaderValue::from_bytes(format!("attachment; filename=\"{}\"", filename).as_bytes())
            .unwrap_or_else(|_| HeaderValue::from_static("attachment")),
    );
    headers
}
/// Export a single document in the specified format.
pub async fn export_document(
    Path(document_id): Path<String>,
    Query(params): Query<ExportQuery>,
    State(state): State<DocumentState>,
    auth: Option<axum::Extension<AuthContext>>,
) -> Result<impl IntoResponse, ServerError> {
    let format = params.format.as_deref().unwrap_or("json");

    let mut conn = state
        .pool
        .acquire()
        .await
        .map_err(|e| ServerError::database(e.to_string()))?;

    let row: Option<(String, String, String, String)> = sqlx::query_as(
        "SELECT title, content, slug, author_id FROM documents WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(&document_id)
    .fetch_optional(&mut *conn)
    .await
    .map_err(|e| ServerError::database(e.to_string()))?;

    let (title, content, slug, author_id) =
        row.ok_or_else(|| ServerError::not_found("Document", &document_id))?;

    let caller_id = auth
        .as_ref()
        .map(|axum::Extension(ctx)| ctx.user_id.as_str());
    let is_admin = auth
        .as_ref()
        .is_some_and(|axum::Extension(ctx)| ctx.is_admin());
    if !can_export_document(caller_id, &author_id, is_admin) {
        return Err(ServerError::forbidden(
            "You do not have permission to export this document",
        ));
    }

    // DLP scan before export — block if content contains restricted data
    state.scan_content_dlp(&content)?;

    info!(
        document_id = %document_id,
        format = %format,
        "Exporting document"
    );

    match format {
        "json" => {
            let exporter = tachyon_import_export::JsonExporter::new().with_pretty_print();
            let doc = tachyon_import_export::ExportableDocument {
                id: document_id.clone(),
                title: title.clone(),
                content: content.clone(),
                slug: slug.clone(),
                tags: vec![],
                created_at: None,
                updated_at: None,
                metadata: None,
            };
            let bytes = exporter
                .export(vec![doc])
                .map_err(|e| ServerError::internal(e.to_string()))?;

            Ok((
                export_headers("application/json", &format!("{}.json", slug)),
                bytes,
            ))
        }
        "markdown" => {
            let bytes = tachyon_import_export::MarkdownZipExporter::export_to_bytes(&[(
                &title as &str,
                &content as &str,
                &format!("{}.md", slug) as &str,
            )])
            .map_err(|e| ServerError::internal(e.to_string()))?;

            Ok((
                export_headers("application/zip", &format!("{}.zip", slug)),
                bytes,
            ))
        }
        #[cfg(feature = "pdf-export")]
        "pdf" => {
            let doc = tachyon_import_export::PdfExportDocument {
                title: title.clone(),
                content: content.clone(),
                author: None,
                created_at: None,
            };
            let config = tachyon_import_export::PdfExportConfig::default();
            let bytes = tachyon_import_export::PdfExporter::export(&doc, &config)
                .map_err(|e| ServerError::internal(e.to_string()))?;

            Ok((
                export_headers("application/pdf", &format!("{}.pdf", slug)),
                bytes,
            ))
        }
        #[cfg(not(feature = "pdf-export"))]
        "pdf" => Err(ServerError::internal(
            "PDF export is not enabled. Rebuild with --features pdf-export.".to_string(),
        )),
        _ => Err(ServerError::bad_request(format!(
            "Unsupported export format: {}",
            format
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::{can_export_document, export_headers};

    #[test]
    fn export_denies_anonymous_and_cross_tenant_callers() {
        assert!(!can_export_document(None, "owner", false));
        assert!(!can_export_document(
            Some("other-tenant-user"),
            "owner",
            false
        ));
    }

    #[test]
    fn export_allows_owner_and_admin() {
        assert!(can_export_document(Some("owner"), "owner", false));
        assert!(can_export_document(Some("other-user"), "owner", true));
    }

    #[test]
    fn export_headers_sets_content_type_and_attachment() {
        let headers = export_headers("application/json", "document.json");
        assert_eq!(headers["content-type"], "application/json");
        assert_eq!(
            headers["content-disposition"],
            "attachment; filename=\"document.json\""
        );
    }
}
