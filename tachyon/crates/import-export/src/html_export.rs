//! HTML export for documents.
//!
//! Renders markdown documents to HTML pages using the Tachyon renderer.
//! Supports full-page rendering with metadata (suitable for Confluence
//! import, static hosting, or archival).

use crate::{error::ImportExportResult, ExportSummary, ImportExportError};
use std::io::{Cursor, Write};
use tachyon_renderer::{RenderConfig, Renderer};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

/// Export documents as rendered HTML pages in a ZIP archive.
pub struct HtmlExporter;

/// Configuration for HTML export.
pub struct HtmlExportConfig {
    /// Site title for the HTML pages.
    pub site_title: String,
    /// Site description.
    pub site_description: String,
    /// Base URL for canonical links.
    pub base_url: String,
    /// Include navigation/sidebar.
    pub include_navigation: bool,
    /// Enable syntax highlighting.
    pub syntax_highlight: bool,
}

impl Default for HtmlExportConfig {
    fn default() -> Self {
        Self {
            site_title: "Tachyon Export".to_string(),
            site_description: "Exported from Tachyon".to_string(),
            base_url: "/".to_string(),
            include_navigation: true,
            syntax_highlight: true,
        }
    }
}

/// A document to export as HTML.
pub struct HtmlExportDocument {
    pub title: String,
    pub content: String,
    pub slug: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl HtmlExporter {
    /// Create a new renderer configured for HTML export.
    fn create_renderer(_config: &HtmlExportConfig) -> Renderer {
        let render_config = RenderConfig {
            format: tachyon_renderer::OutputFormat::Html,
            markdown: tachyon_renderer::MarkdownOptions {
                enable_gfm: true,
                enable_footnotes: true,
                enable_tables: true,
                enable_task_lists: true,
                enable_strikethrough: true,
                enable_autolinks: true,
                enable_smart_punctuation: true,
                enable_heading_attributes: true,
            },
            ..Default::default()
        };
        Renderer::new(render_config)
    }

    /// Render a single markdown document to a full HTML page string.
    pub fn render_document(
        markdown_content: &str,
        title: &str,
        config: &HtmlExportConfig,
    ) -> ImportExportResult<String> {
        let renderer = Self::create_renderer(config);
        let result = renderer
            .render(markdown_content, None)
            .map_err(|e| ImportExportError::Render(e.to_string()))?;

        let html_body = result.content;

        // Build full HTML page
        let description = config.site_description.clone();
        let site_title = &config.site_title;
        let page_title = title;

        let html = format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{page_title} - {site_title}</title>
    <meta name="description" content="{description}">
    <meta name="generator" content="Tachyon Knowledge Management System">
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
            max-width: 800px;
            margin: 0 auto;
            padding: 2rem;
            line-height: 1.6;
            color: #1a1a1a;
        }}
        @media (prefers-color-scheme: dark) {{
            body {{ color: #e0e0e0; background: #1a1a1a; }}
            a {{ color: #60a5fa; }}
            code {{ background: #2d2d2d; }}
            pre {{ background: #2d2d2d; }}
            blockquote {{ border-color: #4a4a4a; color: #b0b0b0; }}
        }}
        pre {{ background: #f5f5f5; padding: 1rem; border-radius: 0.5rem; overflow-x: auto; }}
        code {{ background: #f0f0f0; padding: 0.2rem 0.4rem; border-radius: 0.25rem; font-size: 0.9em; }}
        pre > code {{ background: none; padding: 0; }}
        blockquote {{ border-left: 4px solid #ddd; margin: 1rem 0; padding: 0.5rem 1rem; color: #666; }}
        table {{ border-collapse: collapse; width: 100%; }}
        th, td {{ border: 1px solid #ddd; padding: 0.5rem 1rem; text-align: left; }}
        th {{ background: #f5f5f5; }}
        img {{ max-width: 100%; height: auto; }}
        a {{ color: #2563eb; text-decoration: none; }}
        a:hover {{ text-decoration: underline; }}
        hr {{ border: none; border-top: 1px solid #ddd; margin: 2rem 0; }}
    </style>
</head>
<body>
{html_body}
</body>
</html>"#,
            page_title = html_escape(page_title),
            site_title = html_escape(site_title),
            description = html_escape(&description),
            html_body = html_body,
        );

        Ok(html)
    }

    /// Export multiple documents to a ZIP archive of HTML files.
    pub fn export_to_zip(
        documents: &[HtmlExportDocument],
        config: &HtmlExportConfig,
    ) -> ImportExportResult<(Vec<u8>, ExportSummary)> {
        let buf = Cursor::new(Vec::new());
        let mut zip = ZipWriter::new(buf);
        let options = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .compression_level(Some(6));

        // Generate an index page
        let mut index_html = format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    <meta name="description" content="{}">
    <meta name="generator" content="Tachyon Knowledge Management System">
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; max-width: 800px; margin: 0 auto; padding: 2rem; line-height: 1.6; }}
        a {{ color: #2563eb; text-decoration: none; }}
        a:hover {{ text-decoration: underline; }}
        ul {{ list-style: none; padding: 0; }}
        li {{ padding: 0.5rem 0; border-bottom: 1px solid #eee; }}
        .tags {{ color: #666; font-size: 0.85em; }}
    </style>
</head>
<body>
<h1>{}</h1>
<p>{}</p>
<ul>
"#,
            html_escape(&config.site_title),
            html_escape(&config.site_description),
            html_escape(&config.site_title),
            html_escape(&config.site_description),
        );

        let mut warnings = Vec::new();

        for doc in documents {
            // Render the document
            let html = match Self::render_document(&doc.content, &doc.title, config) {
                Ok(h) => h,
                Err(e) => {
                    warnings.push(format!("Failed to render '{}': {}", doc.title, e));
                    continue;
                }
            };

            // Write to ZIP
            let path = format!("{}.html", doc.slug);
            zip.start_file(&path, options.clone())
                .map_err(|e| ImportExportError::zip(e.to_string()))?;
            zip.write_all(html.as_bytes())
                .map_err(|e| ImportExportError::zip(e.to_string()))?;

            // Add to index
            let tags_str = if doc.tags.is_empty() {
                String::new()
            } else {
                format!(
                    r#" <span class="tags">[{}]</span>"#,
                    doc.tags
                        .iter()
                        .map(|t| t.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            };
            index_html.push_str(&format!(
                r#"<li><a href="{}.html">{}</a>{}</li>"#,
                html_escape(&doc.slug),
                html_escape(&doc.title),
                tags_str,
            ));
        }

        index_html.push_str("</ul>\n</body>\n</html>");

        // Write index
        zip.start_file("index.html", options)
            .map_err(|e| ImportExportError::zip(e.to_string()))?;
        zip.write_all(index_html.as_bytes())
            .map_err(|e| ImportExportError::zip(e.to_string()))?;

        let buf = zip
            .finish()
            .map_err(|e| ImportExportError::zip(e.to_string()))?;
        let bytes = buf.into_inner();

        let summary = ExportSummary {
            exported: documents.len(),
            format: "html-zip".to_string(),
            file_size_bytes: Some(bytes.len() as u64),
            warnings,
        };

        Ok((bytes, summary))
    }

    /// Export a single document as an HTML string (no ZIP wrapping).
    /// Useful for the Confluence export path.
    pub fn export_single_html(
        markdown_content: &str,
        title: &str,
        config: &HtmlExportConfig,
    ) -> ImportExportResult<String> {
        Self::render_document(markdown_content, title, config)
    }
}

/// Simple HTML entity escaping.
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_simple_document() {
        let config = HtmlExportConfig::default();
        let html = HtmlExporter::render_document("# Hello\n\nWorld", "Test Doc", &config).unwrap();

        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<title>Test Doc - Tachyon Export</title>"));
        assert!(html.contains("<h1>"));
        assert!(html.contains("Hello"));
        assert!(html.contains("<p>World</p>"));
    }

    #[test]
    fn test_render_with_gfm() {
        let config = HtmlExportConfig::default();
        let content = "| Header | Value |\n|--------|-------|\n| A | B |\n\n- [x] Done\n- [ ] Todo";
        let html = HtmlExporter::render_document(content, "GFM Test", &config).unwrap();

        assert!(html.contains("<table>"));
        assert!(html.contains("Done")); // Task list
    }

    #[test]
    fn test_export_to_zip() {
        let docs = vec![
            HtmlExportDocument {
                title: "First Doc".to_string(),
                content: "# First\n\nContent 1".to_string(),
                slug: "first-doc".to_string(),
                description: Some("First document".to_string()),
                tags: vec!["test".to_string()],
                created_at: None,
                updated_at: None,
            },
            HtmlExportDocument {
                title: "Second Doc".to_string(),
                content: "## Second\n\nContent 2".to_string(),
                slug: "second-doc".to_string(),
                description: None,
                tags: vec![],
                created_at: None,
                updated_at: None,
            },
        ];

        let config = HtmlExportConfig::default();
        let (bytes, summary) = HtmlExporter::export_to_zip(&docs, &config).unwrap();

        assert_eq!(summary.exported, 2);
        assert!(summary.file_size_bytes.unwrap() > 0);

        // Verify ZIP contains 3 files (2 docs + index)
        let cursor = Cursor::new(bytes);
        let archive = zip::ZipArchive::new(cursor).unwrap();
        assert_eq!(archive.len(), 3);
        assert!(archive.file_names().any(|n| n == "index.html"));
        assert!(archive.file_names().any(|n| n == "first-doc.html"));
        assert!(archive.file_names().any(|n| n == "second-doc.html"));
    }

    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("a < b & c > d"), "a &lt; b &amp; c &gt; d");
        assert_eq!(html_escape("quote\"test"), "quote&quot;test");
    }

    #[test]
    fn test_dark_mode_css() {
        let config = HtmlExportConfig::default();
        let html = HtmlExporter::render_document("# Test", "Test", &config).unwrap();
        assert!(html.contains("prefers-color-scheme: dark"));
    }
}
