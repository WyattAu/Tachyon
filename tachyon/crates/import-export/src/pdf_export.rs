//! PDF export for documents.
//!
//! Generates PDF files from markdown content using Rust-native `genpdf`.
//! Requires the `pdf-export` feature flag.

#[cfg(feature = "pdf-export")]
use genpdf::{Document, Margins, Mm, SimplePageDecorator, elements, fonts};
#[cfg(feature = "pdf-export")]
use printpdf;

use crate::error::{ImportExportError, ImportExportResult};

/// A document suitable for PDF export.
#[derive(Debug, Clone)]
pub struct PdfExportDocument {
    pub title: String,
    pub content: String,
    pub author: Option<String>,
    pub created_at: Option<String>,
}

/// PDF export configuration.
#[derive(Debug, Clone)]
pub struct PdfExportConfig {
    pub title: String,
    pub author: Option<String>,
    /// Page margin in millimeters (default: 20).
    pub margin: u16,
}

impl Default for PdfExportConfig {
    fn default() -> Self {
        Self {
            title: String::from("Exported Document"),
            author: None,
            margin: 20,
        }
    }
}

fn markdown_to_plaintext(md: &str) -> String {
    let mut result = String::with_capacity(md.len());
    let mut in_code_block = false;

    for line in md.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            if !in_code_block {
                result.push('\n');
            }
            continue;
        }

        if in_code_block {
            result.push_str(line);
            result.push('\n');
            continue;
        }

        if trimmed.starts_with("<!--") || trimmed.starts_with("---") || trimmed.starts_with("===") {
            continue;
        }

        let clean = stripped_line(trimmed);
        if !clean.is_empty() {
            result.push_str(&clean);
            result.push('\n');
        } else {
            result.push('\n');
        }
    }

    result
}

fn stripped_line(line: &str) -> String {
    let mut result = String::with_capacity(line.len());
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        match chars[i] {
            '#' if i + 1 < chars.len() && chars[i + 1].is_whitespace() => {
                i += 1;
                while i < chars.len() && chars[i] == '#' {
                    i += 1;
                }
                while i < chars.len() && chars[i] == ' ' {
                    i += 1;
                }
            }
            '*' | '_' => {
                let marker = chars[i];
                if i + 1 < chars.len() && chars[i + 1] == marker {
                    // Double marker (bold or italic depending on char): skip both
                    i += 2;
                } else if is_formatting_marker(&chars, i) {
                    i += 1;
                } else {
                    result.push(chars[i]);
                    i += 1;
                }
            }
            '`' => {
                i += 1;
                if i < chars.len() && chars[i] == '`' {
                    i += 1;
                }
            }
            '[' => {
                i += 1;
                while i < chars.len() && chars[i] != ']' {
                    i += 1;
                }
                i += 1;
                if i < chars.len() && chars[i] == '(' {
                    i += 1;
                    while i < chars.len() && chars[i] != ')' {
                        i += 1;
                    }
                    i += 1;
                }
            }
            '!' if i + 1 < chars.len() && chars[i + 1] == '[' => {
                i += 2;
                while i < chars.len() && chars[i] != ']' {
                    i += 1;
                }
                i += 1;
                if i < chars.len() && chars[i] == '(' {
                    i += 1;
                    while i < chars.len() && chars[i] != ')' {
                        i += 1;
                    }
                    i += 1;
                }
            }
            '-' | '+' => {
                let is_list = i + 1 < chars.len()
                    && (chars[i + 1] == ' ')
                    && (i == 0 || chars[..i].iter().all(|c| *c == ' '));
                if is_list {
                    result.push_str("- ");
                    i += 1;
                    while i < chars.len() && chars[i] == ' ' {
                        i += 1;
                    }
                } else {
                    result.push(chars[i]);
                    i += 1;
                }
            }
            '>' if i + 1 < chars.len() && chars[i + 1] == ' ' => {
                i += 2;
            }
            c => {
                result.push(c);
                i += 1;
            }
        }
    }

    result.trim().to_string()
}

fn is_formatting_marker(chars: &[char], pos: usize) -> bool {
    if pos + 1 >= chars.len() {
        return false;
    }
    chars[pos + 1] != ' '
        && chars[pos + 1] != '\n'
        && (pos == 0 || chars[pos - 1] == ' ' || chars[pos - 1] == '\n')
}

/// PDF exporter.
pub struct PdfExporter;

impl PdfExporter {
    /// Export a single document to PDF bytes.
    #[cfg(feature = "pdf-export")]
    pub fn export(
        document: &PdfExportDocument,
        config: &PdfExportConfig,
    ) -> ImportExportResult<Vec<u8>> {
        use std::io::Cursor;

        let plain = markdown_to_plaintext(&document.content);

        let font_family = load_builtin_font_family()?;

        let mut doc = Document::new(font_family);
        doc.set_title(&config.title);
        doc.set_paper_size(genpdf::PaperSize::A4);

        let mut decorator = SimplePageDecorator::new();
        decorator.set_margins(Margins::all(Mm::from(config.margin)));
        doc.set_page_decorator(decorator);

        let mut paragraphs: Vec<String> = Vec::new();
        let mut current = String::new();
        for line in plain.lines() {
            if line.trim().is_empty() {
                if !current.trim().is_empty() {
                    paragraphs.push(std::mem::take(&mut current));
                }
                current.clear();
            } else {
                if !current.is_empty() {
                    current.push('\n');
                }
                current.push_str(line);
            }
        }
        if !current.trim().is_empty() {
            paragraphs.push(current);
        }

        for para in &paragraphs {
            doc.push(elements::Paragraph::new(para.as_str()));
        }

        let mut buf = Cursor::new(Vec::new());
        doc.render(&mut buf)
            .map_err(|e| ImportExportError::export(format!("PDF render error: {}", e)))?;

        Ok(buf.into_inner())
    }

    /// Export without pdf-export feature.
    #[cfg(not(feature = "pdf-export"))]
    pub fn export(
        _document: &PdfExportDocument,
        _config: &PdfExportConfig,
    ) -> ImportExportResult<Vec<u8>> {
        Err(ImportExportError::export(
            "PDF export requires the 'pdf-export' feature. Rebuild with: cargo build -p tachyon-import-export --features pdf-export".to_string(),
        ))
    }

    /// Export multiple documents to a single PDF.
    #[cfg(feature = "pdf-export")]
    pub fn export_batch(
        documents: &[PdfExportDocument],
        config: &PdfExportConfig,
    ) -> ImportExportResult<Vec<u8>> {
        let mut combined = String::new();
        for (i, doc) in documents.iter().enumerate() {
            if i > 0 {
                combined.push_str("\n\n---\n\n");
            }
            combined.push_str(&format!("# {}\n\n", doc.title));
            combined.push_str(&doc.content);
        }
        let combined_doc = PdfExportDocument {
            title: config.title.clone(),
            content: combined,
            author: config.author.clone(),
            created_at: None,
        };
        Self::export(&combined_doc, config)
    }

    #[cfg(not(feature = "pdf-export"))]
    pub fn export_batch(
        _documents: &[PdfExportDocument],
        _config: &PdfExportConfig,
    ) -> ImportExportResult<Vec<u8>> {
        Err(ImportExportError::export(
            "PDF export requires the 'pdf-export' feature.".to_string(),
        ))
    }
}

#[cfg(feature = "pdf-export")]
fn load_builtin_font_family() -> ImportExportResult<fonts::FontFamily<fonts::FontData>> {
    let font_bytes: &[u8] = include_bytes!("fonts/DejaVuSans.ttf");
    let regular = fonts::FontData::new(font_bytes.to_vec(), Some(printpdf::BuiltinFont::Helvetica))
        .map_err(|e| ImportExportError::export(format!("Failed to load font: {}", e)))?;

    Ok(fonts::FontFamily {
        regular: regular.clone(),
        bold: regular.clone(),
        italic: regular.clone(),
        bold_italic: regular,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_doc(title: &str, content: &str) -> PdfExportDocument {
        PdfExportDocument {
            title: title.to_string(),
            content: content.to_string(),
            author: None,
            created_at: None,
        }
    }

    #[test]
    fn test_markdown_to_plaintext() {
        let md = "# Hello World\n\nSome **bold** and *italic* text.\n\n- item 1\n- item 2\n\n[link](http://example.com)";
        let plain = markdown_to_plaintext(md);
        assert!(!plain.contains('#'));
        assert!(!plain.contains("**"));
        assert!(!plain.contains("http://"));
        assert!(plain.contains("Hello World"));
        assert!(plain.contains("item 1"));
    }

    #[test]
    fn test_markdown_code_block() {
        let md = "before\n```\ncode line\n```\nafter";
        let plain = markdown_to_plaintext(md);
        assert!(plain.contains("code line"));
        assert!(!plain.contains("```"));
    }

    #[test]
    fn test_markdown_strips_images() {
        let md = "see ![alt text](image.png)";
        let plain = markdown_to_plaintext(md);
        assert!(!plain.contains("!["));
        assert!(!plain.contains("image.png"));
    }

    #[test]
    fn test_markdown_strips_html_comments() {
        let md = "before\n<!-- comment -->\nafter";
        let plain = markdown_to_plaintext(md);
        assert!(!plain.contains("<!--"));
        assert!(plain.contains("before"));
        assert!(plain.contains("after"));
    }

    #[test]
    fn test_export_config_default() {
        let config = PdfExportConfig::default();
        assert_eq!(config.title, "Exported Document");
        assert!(config.author.is_none());
        assert_eq!(config.margin, 20);
    }

    #[test]
    #[cfg(not(feature = "pdf-export"))]
    fn test_export_without_feature_returns_error() {
        let doc = test_doc("Test", "Hello");
        let config = PdfExportConfig::default();
        let result = PdfExporter::export(&doc, &config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("pdf-export"));
    }

    #[test]
    #[cfg(not(feature = "pdf-export"))]
    fn test_export_batch_without_feature_returns_error() {
        let docs = vec![test_doc("Test", "Hello")];
        let config = PdfExportConfig::default();
        let result = PdfExporter::export_batch(&docs, &config);
        assert!(result.is_err());
    }
}
