//! Integration tests for PDF export functionality.

#[cfg(feature = "pdf-export")]
mod pdf_export_tests {
    use tachyon_import_export::{PdfExportConfig, PdfExportDocument, PdfExporter};

    fn document(content: &str) -> PdfExportDocument {
        PdfExportDocument {
            title: "Test Document".to_string(),
            content: content.to_string(),
            author: Some("Test Author".to_string()),
            created_at: None,
        }
    }

    fn assert_pdf(bytes: &[u8]) {
        assert!(!bytes.is_empty(), "PDF bytes should not be empty");
        assert!(bytes.len() >= 4);
        assert_eq!(&bytes[..4], b"%PDF", "should start with PDF magic bytes");
    }

    #[test]
    fn exports_markdown_document() {
        let result = PdfExporter::export(
            &document("# Hello\n\nThis is **bold** text."),
            &PdfExportConfig::default(),
        )
        .expect("PDF export should succeed");
        assert_pdf(&result);
    }

    #[test]
    fn exports_code_and_utf8_content() {
        let result = PdfExporter::export(
            &document("```rust\nfn main() {}\n```\n\nこんにちは世界 — café"),
            &PdfExportConfig::default(),
        )
        .expect("PDF export should succeed");
        assert_pdf(&result);
    }

    #[test]
    fn exports_empty_document() {
        let result = PdfExporter::export(&document(""), &PdfExportConfig::default())
            .expect("empty PDF export should succeed");
        assert_pdf(&result);
    }

    #[test]
    fn applies_title_author_and_margin_configuration() {
        let config = PdfExportConfig {
            title: "Configured title".to_string(),
            author: Some("Configured author".to_string()),
            margin: 40,
        };
        let result = PdfExporter::export(&document("Content"), &config)
            .expect("configured PDF export should succeed");
        assert_pdf(&result);
    }

    #[test]
    fn exports_batch() {
        let docs = vec![document("# First"), document("# Second")];
        let result = PdfExporter::export_batch(&docs, &PdfExportConfig::default())
            .expect("batch PDF export should succeed");
        assert_pdf(&result);
    }

    #[test]
    fn exports_large_document() {
        let content = (0..100)
            .map(|i| format!("## Section {i}\n\nLorem ipsum dolor sit amet.\n"))
            .collect::<Vec<_>>()
            .join("\n");
        let result = PdfExporter::export(&document(&content), &PdfExportConfig::default())
            .expect("large PDF export should succeed");
        assert_pdf(&result);
    }
}
