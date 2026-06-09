// Import/Export Tauri commands
// Wires tachyon-import-export into the Tauri IPC layer

use serde::{Deserialize, Serialize};
use std::path::Path;
use tachyon_core::TachyonError;
use tauri::AppHandle;

use crate::events::EventEmitter;

/// Result of an import operation (serializable for the WebView).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    /// Total files found in the source
    pub total_files: usize,
    /// Files successfully parsed as documents
    pub imported: usize,
    /// Files skipped (non-markdown, empty, etc.)
    pub skipped: usize,
    /// Files that failed to parse
    pub failed: usize,
    /// Titles of imported documents
    pub document_titles: Vec<String>,
    /// Tags found across all documents
    pub all_tags: Vec<String>,
    /// Warnings encountered during import
    pub warnings: Vec<String>,
    /// Imported documents (title, content, tags, source_path)
    pub documents: Vec<ImportedDocumentDto>,
}

/// Lightweight DTO for an imported document sent to the WebView.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportedDocumentDto {
    pub title: String,
    pub slug: String,
    pub content: String,
    pub tags: Vec<String>,
    pub source_path: String,
}

/// Result of an export operation (serializable for the WebView).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResult {
    /// Total documents exported
    pub exported: usize,
    /// Export format used
    pub format: String,
    /// Output file size in bytes
    pub file_size_bytes: u64,
    /// Warnings encountered during export
    pub warnings: Vec<String>,
    /// Output file path (where the ZIP was saved)
    pub output_path: String,
}

/// Import request for Obsidian vault.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsidianImportRequest {
    /// Path to the Obsidian vault directory or ZIP file
    pub path: String,
}

/// Import request for Markdown ZIP.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkdownZipImportRequest {
    /// Path to the ZIP file
    pub path: String,
}

/// Export request for Markdown ZIP.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct MarkdownZipExportRequest {
    /// Output file path for the ZIP
    pub output_path: String,
    /// Documents to export
    pub documents: Vec<ExportDocumentDto>,
}

/// Export request for HTML ZIP.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HtmlExportRequest {
    /// Output file path for the ZIP
    pub output_path: String,
    /// Site title for the HTML pages
    pub site_title: Option<String>,
    /// Site description
    pub site_description: Option<String>,
    /// Documents to export
    pub documents: Vec<HtmlExportDocumentDto>,
}

/// A document to export as Markdown.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportDocumentDto {
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub description: Option<String>,
    pub path: String,
    pub created_at: Option<String>, // ISO 8601
    pub updated_at: Option<String>, // ISO 8601
}

/// A document to export as HTML.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HtmlExportDocumentDto {
    pub title: String,
    pub content: String,
    pub slug: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub created_at: Option<String>, // ISO 8601
    pub updated_at: Option<String>, // ISO 8601
}

/// Import an Obsidian vault from a directory or ZIP file.
///
/// If `path` ends with `.zip`, imports from the ZIP archive.
/// Otherwise, imports from the directory on disk.
#[tauri::command]
pub async fn import_obsidian_vault(
    request: ObsidianImportRequest,
    app: AppHandle,
) -> Result<ImportResult, String> {
    let path = Path::new(&request.path);
    let emitter = EventEmitter::new(app);

    if !path.exists() {
        let msg = format!("Path does not exist: {}", request.path);
        let _ = emitter.emit_error(&TachyonError::not_found(&msg));
        return Err(msg);
    }

    // Run on blocking thread since ObsidianImporter reads files
    let path_owned = request.path.clone();
    let result = tokio::task::spawn_blocking(move || {
        if path_owned.ends_with(".zip") {
            // Import from ZIP
            let zip_bytes = std::fs::read(&path_owned)
                .map_err(|e| format!("Failed to read ZIP file: {}", e))?;
            let (documents, summary) =
                tachyon_import_export::ObsidianImporter::import_from_bytes(&zip_bytes)
                    .map_err(|e| e.to_string())?;
            Ok::<_, String>((documents, summary))
        } else {
            // Import from directory
            let dir_path = Path::new(&path_owned);
            let (documents, summary) =
                tachyon_import_export::ObsidianImporter::import_from_dir(dir_path)
                    .map_err(|e| e.to_string())?;
            Ok::<_, String>((documents, summary))
        }
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))??;

    let (documents, summary) = result;
    let import_result = ImportResult {
        total_files: summary.total_files,
        imported: summary.imported,
        skipped: summary.skipped,
        failed: summary.failed,
        document_titles: summary.document_titles,
        all_tags: summary.all_tags,
        warnings: summary.warnings,
        documents: documents
            .into_iter()
            .map(|d| {
                let slug = d.effective_slug();
                ImportedDocumentDto {
                    title: d.title,
                    slug,
                    content: d.content,
                    tags: d.tags,
                    source_path: d.source_path,
                }
            })
            .collect(),
    };

    let _ = emitter.emit_notification(
        crate::events::NotificationLevel::Info,
        "Import Complete",
        format!(
            "Imported {} documents from Obsidian vault",
            import_result.imported
        ),
    );

    Ok(import_result)
}

/// Import markdown documents from a ZIP archive.
#[tauri::command]
pub async fn import_markdown_zip(
    request: MarkdownZipImportRequest,
    app: AppHandle,
) -> Result<ImportResult, String> {
    let emitter = EventEmitter::new(app);

    if !Path::new(&request.path).exists() {
        let msg = format!("ZIP file does not exist: {}", request.path);
        let _ = emitter.emit_error(&TachyonError::not_found(&msg));
        return Err(msg);
    }

    let path_owned = request.path.clone();
    let result = tokio::task::spawn_blocking(move || {
        let zip_bytes =
            std::fs::read(&path_owned).map_err(|e| format!("Failed to read ZIP file: {}", e))?;
        let (documents, summary) =
            tachyon_import_export::MarkdownZipImporter::import_documents_from_bytes(&zip_bytes)
                .map_err(|e| e.to_string())?;
        Ok::<_, String>((documents, summary))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))??;

    let (documents, summary) = result;
    let import_result = ImportResult {
        total_files: summary.total_files,
        imported: summary.imported,
        skipped: summary.skipped,
        failed: summary.failed,
        document_titles: summary.document_titles,
        all_tags: summary.all_tags,
        warnings: summary.warnings,
        documents: documents
            .into_iter()
            .map(|d| {
                let slug = d.effective_slug();
                ImportedDocumentDto {
                    title: d.title,
                    slug,
                    content: d.content,
                    tags: d.tags,
                    source_path: d.source_path,
                }
            })
            .collect(),
    };

    let _ = emitter.emit_notification(
        crate::events::NotificationLevel::Info,
        "Import Complete",
        format!(
            "Imported {} documents from Markdown ZIP",
            import_result.imported
        ),
    );

    Ok(import_result)
}

/// Export documents as a Markdown ZIP archive.
#[tauri::command]
#[allow(dead_code)]
pub async fn export_markdown_zip(
    request: MarkdownZipExportRequest,
    app: AppHandle,
) -> Result<ExportResult, String> {
    let emitter = EventEmitter::new(app);

    if request.documents.is_empty() {
        let msg = "No documents to export".to_string();
        let _ = emitter.emit_error(&TachyonError::validation("EMPTY_EXPORT", &msg));
        return Err(msg);
    }

    let export_docs: Vec<tachyon_import_export::ExportDocument> = request
        .documents
        .into_iter()
        .map(|d| tachyon_import_export::ExportDocument {
            title: d.title,
            content: d.content,
            tags: d.tags,
            description: d.description,
            path: d.path,
            created_at: d
                .created_at
                .and_then(|s| tachyon_import_export::parse_date(&s)),
            updated_at: d
                .updated_at
                .and_then(|s| tachyon_import_export::parse_date(&s)),
        })
        .collect();

    let output_path = request.output_path.clone();
    let result = tokio::task::spawn_blocking(move || {
        let (bytes, summary) =
            tachyon_import_export::MarkdownZipExporter::export_with_metadata(&export_docs)
                .map_err(|e| e.to_string())?;

        // Ensure parent directory exists
        if let Some(parent) = Path::new(&output_path).parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create output directory: {}", e))?;
        }

        std::fs::write(&output_path, &bytes)
            .map_err(|e| format!("Failed to write ZIP file: {}", e))?;

        Ok::<_, String>((summary, bytes.len() as u64))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))??;

    let (summary, file_size) = result;
    let export_result = ExportResult {
        exported: summary.exported,
        format: summary.format,
        file_size_bytes: file_size,
        warnings: summary.warnings,
        output_path: request.output_path.clone(),
    };

    let _ = emitter.emit_notification(
        crate::events::NotificationLevel::Info,
        "Export Complete",
        format!(
            "Exported {} documents to {}",
            export_result.exported, request.output_path
        ),
    );

    Ok(export_result)
}

/// Export documents as an HTML ZIP archive.
#[tauri::command]
pub async fn export_html(
    request: HtmlExportRequest,
    app: AppHandle,
) -> Result<ExportResult, String> {
    let emitter = EventEmitter::new(app);

    if request.documents.is_empty() {
        let msg = "No documents to export".to_string();
        let _ = emitter.emit_error(&TachyonError::validation("EMPTY_EXPORT", &msg));
        return Err(msg);
    }

    let config = tachyon_import_export::HtmlExportConfig {
        site_title: request
            .site_title
            .unwrap_or_else(|| "Tachyon Export".to_string()),
        site_description: request
            .site_description
            .unwrap_or_else(|| "Exported from Tachyon".to_string()),
        ..Default::default()
    };

    let export_docs: Vec<tachyon_import_export::HtmlExportDocument> = request
        .documents
        .into_iter()
        .map(|d| tachyon_import_export::HtmlExportDocument {
            title: d.title,
            content: d.content,
            slug: d.slug,
            description: d.description,
            tags: d.tags,
            created_at: d
                .created_at
                .and_then(|s| tachyon_import_export::parse_date(&s)),
            updated_at: d
                .updated_at
                .and_then(|s| tachyon_import_export::parse_date(&s)),
        })
        .collect();

    let output_path = request.output_path.clone();
    let result = tokio::task::spawn_blocking(move || {
        let (bytes, summary) =
            tachyon_import_export::HtmlExporter::export_to_zip(&export_docs, &config)
                .map_err(|e| e.to_string())?;

        // Ensure parent directory exists
        if let Some(parent) = Path::new(&output_path).parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create output directory: {}", e))?;
        }

        std::fs::write(&output_path, &bytes)
            .map_err(|e| format!("Failed to write ZIP file: {}", e))?;

        Ok::<_, String>((summary, bytes.len() as u64))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))??;

    let (summary, file_size) = result;
    let export_result = ExportResult {
        exported: summary.exported,
        format: summary.format,
        file_size_bytes: file_size,
        warnings: summary.warnings,
        output_path: request.output_path.clone(),
    };

    let _ = emitter.emit_notification(
        crate::events::NotificationLevel::Info,
        "Export Complete",
        format!(
            "Exported {} HTML documents to {}",
            export_result.exported, request.output_path
        ),
    );

    Ok(export_result)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_imported_document_dto() {
        let dto = ImportedDocumentDto {
            title: "Test Doc".to_string(),
            slug: "test-doc".to_string(),
            content: "# Hello\n\nWorld".to_string(),
            tags: vec!["test".to_string()],
            source_path: "test-doc.md".to_string(),
        };
        assert_eq!(dto.title, "Test Doc");
        assert_eq!(dto.slug, "test-doc");
        assert_eq!(dto.tags.len(), 1);
    }

    #[test]
    fn test_export_document_dto() {
        let dto = ExportDocumentDto {
            title: "Export Doc".to_string(),
            content: "# Export\n\nContent".to_string(),
            tags: vec![],
            description: Some("A doc".to_string()),
            path: "export.md".to_string(),
            created_at: Some("2024-01-15T10:30:00Z".to_string()),
            updated_at: None,
        };
        assert_eq!(dto.title, "Export Doc");
        assert_eq!(dto.path, "export.md");
    }

    #[test]
    fn test_import_result() {
        let result = ImportResult {
            total_files: 10,
            imported: 8,
            skipped: 1,
            failed: 1,
            document_titles: vec!["Doc 1".to_string()],
            all_tags: vec!["test".to_string()],
            warnings: vec![],
            documents: vec![],
        };
        assert_eq!(result.total_files, 10);
        assert_eq!(result.imported, 8);
    }

    #[test]
    fn test_export_result() {
        let result = ExportResult {
            exported: 5,
            format: "markdown-zip".to_string(),
            file_size_bytes: 1024,
            warnings: vec![],
            output_path: "/tmp/export.zip".to_string(),
        };
        assert_eq!(result.exported, 5);
        assert_eq!(result.file_size_bytes, 1024);
    }

    #[test]
    fn test_obsidian_import_request() {
        let req = ObsidianImportRequest {
            path: "/vault".to_string(),
        };
        assert_eq!(req.path, "/vault");
    }

    #[test]
    fn test_html_export_request() {
        let req = HtmlExportRequest {
            output_path: "/tmp/html.zip".to_string(),
            site_title: Some("My Site".to_string()),
            site_description: None,
            documents: vec![HtmlExportDocumentDto {
                title: "Doc".to_string(),
                content: "# Test".to_string(),
                slug: "doc".to_string(),
                description: None,
                tags: vec![],
                created_at: None,
                updated_at: None,
            }],
        };
        assert_eq!(req.output_path, "/tmp/html.zip");
        assert_eq!(req.documents.len(), 1);
    }
}
