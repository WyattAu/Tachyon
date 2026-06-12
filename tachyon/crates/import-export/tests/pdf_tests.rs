//! Integration tests for PDF export functionality.

#[cfg(feature = "pdf-export")]
mod pdf_export_tests {
    use tachyon_import_export::{
        PdfExportConfig, PdfExportDocument, PdfExporter, PdfHeaderFooter, PdfMargins, PdfPageSize,
    };

    fn simple_doc() -> PdfExportDocument {
        PdfExportDocument {
            title: "Simple Document".to_string(),
            content: "# Hello World\n\nThis is a simple test document.\n\n## Section 1\n\nSome content here.".to_string(),
            author: Some("Test Author".to_string()),
            created_at: None,
        }
    }

    fn code_block_doc() -> PdfExportDocument {
        PdfExportDocument {
            title: "Code Document".to_string(),
            content: "# Code Example\n\nHere is some Rust code:\n\n```rust\nfn main() {\n    println!(\"Hello, world!\");\n}\n```\n\nAnd a Python example:\n\n```python\ndef greet(name):\n    return f\"Hello, {name}!\"\n```\n\nEnd of code.".to_string(),
            author: None,
            created_at: None,
        }
    }

    fn image_doc() -> PdfExportDocument {
        PdfExportDocument {
            title: "Document with Images".to_string(),
            content: "# Image Test\n\nHere is an image:\n\n![Alt text](image.png)\n\nAnd another:\n\n![Logo](logo.jpg)\n\nText after images.".to_string(),
            author: None,
            created_at: None,
        }
    }

    fn utf8_doc() -> PdfExportDocument {
        PdfExportDocument {
            title: "UTF-8 Document".to_string(),
            content: "# International Content\n\nFrench: café, résumé, naïve\n\nGerman: über, Straße, Müller\n\nJapanese: こんにちは世界\n\nEmoji: 🎉🚀📝\n\nMathematical: ∑ ∏ ∫ α β γ".to_string(),
            author: None,
            created_at: None,
        }
    }

    #[test]
    fn test_simple_document_export() {
        let doc = simple_doc();
        let config = PdfExportConfig::default();
        let result = PdfExporter::export(&doc, &config).expect("PDF export should succeed");

        assert!(!result.bytes.is_empty(), "PDF bytes should not be empty");
        assert!(result.page_count >= 1, "Should have at least 1 page");
        // Verify PDF magic bytes
        assert_eq!(&result.bytes[..4], b"%PDF", "Should start with %PDF header");
    }

    #[test]
    fn test_code_block_document_export() {
        let doc = code_block_doc();
        let config = PdfExportConfig::default();
        let result = PdfExporter::export(&doc, &config).expect("PDF export should succeed");

        assert!(!result.bytes.is_empty());
        assert_eq!(&result.bytes[..4], b"%PDF");
        assert!(result.page_count >= 1);
    }

    #[test]
    fn test_image_document_export() {
        let doc = image_doc();
        let config = PdfExportConfig::default();
        let result = PdfExporter::export(&doc, &config).expect("PDF export should succeed");

        assert!(!result.bytes.is_empty());
        assert_eq!(&result.bytes[..4], b"%PDF");
    }

    #[test]
    fn test_utf8_characters_export() {
        let doc = utf8_doc();
        let config = PdfExportConfig::default();
        let result = PdfExporter::export(&doc, &config).expect("PDF export should succeed");

        assert!(!result.bytes.is_empty());
        assert_eq!(&result.bytes[..4], b"%PDF");
    }

    #[test]
    fn test_page_size_a4() {
        let doc = simple_doc();
        let config = PdfExportConfig {
            page_size: PdfPageSize::A4,
            ..Default::default()
        };
        let result = PdfExporter::export(&doc, &config).expect("A4 export should succeed");
        assert!(!result.bytes.is_empty());
    }

    #[test]
    fn test_page_size_letter() {
        let doc = simple_doc();
        let config = PdfExportConfig {
            page_size: PdfPageSize::Letter,
            ..Default::default()
        };
        let result = PdfExporter::export(&doc, &config).expect("Letter export should succeed");
        assert!(!result.bytes.is_empty());
    }

    #[test]
    fn test_page_size_legal() {
        let doc = simple_doc();
        let config = PdfExportConfig {
            page_size: PdfPageSize::Legal,
            ..Default::default()
        };
        let result = PdfExporter::export(&doc, &config).expect("Legal export should succeed");
        assert!(!result.bytes.is_empty());
    }

    #[test]
    fn test_margin_options() {
        let doc = simple_doc();
        let config = PdfExportConfig {
            margins: PdfMargins::all(40),
            ..Default::default()
        };
        let result =
            PdfExporter::export(&doc, &config).expect("Custom margins export should succeed");
        assert!(!result.bytes.is_empty());
        assert_eq!(&result.bytes[..4], b"%PDF");
    }

    #[test]
    fn test_asymmetric_margins() {
        let doc = simple_doc();
        let config = PdfExportConfig {
            margins: PdfMargins {
                top: 10,
                bottom: 30,
                left: 15,
                right: 25,
            },
            ..Default::default()
        };
        let result = PdfExporter::export(&doc, &config).expect("Asymmetric margins should succeed");
        assert!(!result.bytes.is_empty());
    }

    #[test]
    fn test_header_and_footer() {
        let doc = simple_doc();
        let config = PdfExportConfig {
            header_footer: PdfHeaderFooter {
                header_text: Some("My Document Header".to_string()),
                footer_text: Some("Confidential".to_string()),
                include_page_numbers: true,
            },
            ..Default::default()
        };
        let result =
            PdfExporter::export(&doc, &config).expect("Header/footer export should succeed");
        assert!(!result.bytes.is_empty());
        assert_eq!(&result.bytes[..4], b"%PDF");
    }

    #[test]
    fn test_export_config_with_all_options() {
        let doc = PdfExportDocument {
            title: "Full Options Doc".to_string(),
            content: "# Full Test\n\nContent with **bold** and *italic*.".to_string(),
            author: Some("Jane Doe".to_string()),
            created_at: Some("2026-01-01".to_string()),
        };
        let config = PdfExportConfig {
            title: "Full Options Doc".to_string(),
            author: Some("Jane Doe".to_string()),
            page_size: PdfPageSize::Letter,
            margins: PdfMargins {
                top: 25,
                bottom: 25,
                left: 30,
                right: 30,
            },
            header_footer: PdfHeaderFooter {
                header_text: Some("Generated by Tachyon".to_string()),
                footer_text: Some("Internal Use Only".to_string()),
                include_page_numbers: true,
            },
        };
        let result =
            PdfExporter::export(&doc, &config).expect("Full options export should succeed");
        assert!(!result.bytes.is_empty());
        assert_eq!(&result.bytes[..4], b"%PDF");
        assert!(result.page_count >= 1);
    }

    #[test]
    fn test_batch_export() {
        let docs = vec![
            PdfExportDocument {
                title: "First".to_string(),
                content: "# First\n\nContent 1.".to_string(),
                author: None,
                created_at: None,
            },
            PdfExportDocument {
                title: "Second".to_string(),
                content: "# Second\n\nContent 2.".to_string(),
                author: None,
                created_at: None,
            },
        ];
        let config = PdfExportConfig::default();
        let result =
            PdfExporter::export_batch(&docs, &config).expect("Batch export should succeed");
        assert!(!result.bytes.is_empty());
        assert_eq!(&result.bytes[..4], b"%PDF");
    }

    #[test]
    fn test_empty_document_export() {
        let doc = PdfExportDocument {
            title: "Empty".to_string(),
            content: String::new(),
            author: None,
            created_at: None,
        };
        let config = PdfExportConfig::default();
        let result = PdfExporter::export(&doc, &config).expect("Empty doc export should succeed");
        assert!(!result.bytes.is_empty());
        assert_eq!(&result.bytes[..4], b"%PDF");
    }

    #[test]
    fn test_large_document_export() {
        let mut content = String::from("# Large Document\n\n");
        for i in 0..100 {
            content.push_str(&format!(
                "## Section {}\n\nThis is paragraph {} with some filler text to make it longer. ",
                i, i
            ));
            content.push_str("Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.\n\n");
        }
        let doc = PdfExportDocument {
            title: "Large Doc".to_string(),
            content,
            author: None,
            created_at: None,
        };
        let config = PdfExportConfig::default();
        let result = PdfExporter::export(&doc, &config).expect("Large doc export should succeed");
        assert!(!result.bytes.is_empty());
        assert!(
            result.page_count > 1,
            "Large doc should have multiple pages"
        );
    }
}
