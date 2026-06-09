//! Confluence space export importer.
//!
//! Imports a Confluence XML space export, handling:
//! - `pages.xml` file with page titles, bodies, labels, and hierarchy
//! - Confluence storage format (XHTML) converted to Markdown
//! - Page tree structure preserved via parent-child relationships
//! - Labels mapped to Tachyon tags
//!
//! Confluence XML export structure:
//! ```xml
//! <confluence-data>
//!   <page>
//!     <title>Page Title</title>
//!     <body>Confluence storage format (XHTML)</body>
//!     <space>SPACE</space>
//!     <labels><label>tag1</label><label>tag2</label></labels>
//!     <ancestors><ancestor id="123"/></ancestors>
//!   </page>
//! </confluence-data>
//! ```

use crate::{
    ImportExportError, ImportSummary, ImportedDocument, error::ImportExportResult,
    frontmatter::Frontmatter,
};
use quick_xml::Reader;
use quick_xml::events::Event;
use std::collections::{BTreeMap, HashSet};
use std::path::Path;

/// Import Confluence space exports.
pub struct ConfluenceImporter;

/// A parsed Confluence page before conversion to ImportedDocument.
#[derive(Debug, Default)]
struct ConfluencePage {
    id: String,
    title: String,
    body: String,
    space: String,
    labels: Vec<String>,
    parent_id: Option<String>,
}

impl ConfluenceImporter {
    /// Import all pages from a Confluence XML export file.
    ///
    /// The `xml_bytes` should contain the contents of the `pages.xml` file
    /// from a Confluence space export.
    pub fn import_from_bytes(
        xml_bytes: &[u8],
    ) -> ImportExportResult<(Vec<ImportedDocument>, ImportSummary)> {
        let pages = parse_confluence_xml(xml_bytes)?;

        let mut summary = ImportSummary {
            total_files: pages.len(),
            imported: 0,
            skipped: 0,
            failed: 0,
            document_titles: Vec::new(),
            all_tags: Vec::new(),
            warnings: Vec::new(),
        };

        let mut documents = Vec::new();
        let mut all_tags: HashSet<String> = HashSet::new();

        for page in &pages {
            if page.title.is_empty() {
                summary.skipped += 1;
                continue;
            }

            if page.body.is_empty() {
                summary.skipped += 1;
                summary
                    .warnings
                    .push(format!("Skipped empty page: {}", page.title));
                continue;
            }

            let markdown_body = confluence_storage_to_markdown(&page.body);

            let mut tags: Vec<String> = page.labels.clone();

            // Add space as a tag
            if !page.space.is_empty() {
                tags.push(page.space.to_lowercase());
            }

            tags.sort();
            tags.dedup();

            for tag in &tags {
                all_tags.insert(tag.clone());
            }

            // Build source_path from hierarchy
            let source_path = if page.space.is_empty() {
                format!("{}.md", slugify_title(&page.title))
            } else {
                format!("{}/{}.md", page.space, slugify_title(&page.title))
            };

            summary.imported += 1;
            summary.document_titles.push(page.title.clone());

            let doc = ImportedDocument {
                title: page.title.clone(),
                slug: None,
                content: markdown_body,
                frontmatter: Frontmatter::default(),
                tags,
                source_path,
                created_at: None,
                updated_at: None,
                extra: {
                    let mut extra = BTreeMap::new();
                    if let Some(ref parent_id) = page.parent_id {
                        extra.insert(
                            "parent_id".to_string(),
                            serde_json::Value::String(parent_id.clone()),
                        );
                    }
                    extra.insert(
                        "confluence_id".to_string(),
                        serde_json::Value::String(page.id.clone()),
                    );
                    extra
                },
            };
            documents.push(doc);
        }

        summary.all_tags = all_tags.into_iter().collect();
        summary.all_tags.sort();
        Ok((documents, summary))
    }

    /// Import from a file path on disk.
    pub fn import_from_path(
        xml_path: &Path,
    ) -> ImportExportResult<(Vec<ImportedDocument>, ImportSummary)> {
        let bytes = std::fs::read(xml_path).map_err(|e| {
            ImportExportError::io(xml_path, format!("Failed to read XML file: {}", e))
        })?;
        Self::import_from_bytes(&bytes)
    }
}

/// Parse Confluence XML export into a list of pages.
fn parse_confluence_xml(xml_bytes: &[u8]) -> ImportExportResult<Vec<ConfluencePage>> {
    let mut reader = Reader::from_reader(xml_bytes);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut pages = Vec::new();
    let mut current_page: Option<ConfluencePage> = None;
    let mut in_body = false;
    let mut in_labels = false;
    let mut current_tag = String::new();

    let mut in_ancestors = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match tag_name.as_str() {
                    "page" => {
                        current_page = Some(ConfluencePage::default());
                    }
                    "body" => {
                        in_body = true;
                        if let Some(ref mut page) = current_page {
                            page.body.clear();
                        }
                    }
                    "labels" => {
                        in_labels = true;
                    }
                    "ancestors" => {
                        in_ancestors = true;
                    }
                    "ancestor" => {
                        // Extract the id attribute from <ancestor id="12345"/>
                        if in_ancestors
                            && let Some(id) = get_attr(e, "id")
                                && let Some(ref mut page) = current_page {
                                    page.parent_id = Some(id);
                                }
                    }
                    _ => {
                        if !in_ancestors {
                            current_tag = tag_name;
                        }
                    }
                }
            }
            Ok(Event::Text(ref e)) => {
                let text = e
                    .unescape()
                    .map_err(|e| ImportExportError::import(format!("XML decode error: {}", e)))?;
                if let Some(ref mut page) = current_page {
                    if in_body {
                        page.body.push_str(&text);
                    } else if in_labels {
                        let label = text.trim().to_string();
                        if !label.is_empty() {
                            page.labels.push(label);
                        }
                    } else {
                        match current_tag.as_str() {
                            "id" => page.id = text.to_string(),
                            "title" => page.title = text.to_string(),
                            "space" => page.space = text.to_string(),
                            "parentId" => {
                                page.parent_id = Some(text.to_string());
                            }
                            _ => {}
                        }
                    }
                }
            }
            Ok(Event::CData(ref e)) => {
                let text = String::from_utf8_lossy(e.as_ref()).to_string();
                if let Some(ref mut page) = current_page
                    && in_body {
                        page.body.push_str(&text);
                    }
            }
            Ok(Event::End(ref e)) => {
                let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match tag_name.as_str() {
                    "page" => {
                        if let Some(page) = current_page.take() {
                            pages.push(page);
                        }
                    }
                    "body" => {
                        in_body = false;
                    }
                    "labels" => {
                        in_labels = false;
                    }
                    "ancestors" => {
                        in_ancestors = false;
                    }
                    _ => {}
                }
                if !in_ancestors {
                    current_tag.clear();
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(ImportExportError::import(format!(
                    "XML parse error at position {}: {}",
                    reader.buffer_position(),
                    e
                )));
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(pages)
}

/// Convert Confluence storage format (XHTML) to Markdown.
///
/// Handles common Confluence markup:
/// - `<p>` -> paragraphs
/// - `<h1>`-`<h6>` -> `#`-`######`
/// - `<strong>`, `<em>` -> `**`, `*`
/// - `<code>` -> `` ` ``
/// - `<pre><code>` -> ` ``` `
/// - `<a href="url">` -> `[text](url)`
/// - `<img src="url">` -> `![alt](url)`
/// - `<ul>`, `<ol>`, `<li>` -> lists
/// - `<table>` -> basic table syntax
/// - `<ac:structured-macro>` callouts -> blockquotes
/// - `<br>` -> newline
pub fn confluence_storage_to_markdown(html: &str) -> String {
    let mut md = String::with_capacity(html.len());
    let mut reader = Reader::from_str(html);
    // Don't trim text - we need to preserve spaces between inline elements
    let mut buf = Vec::new();
    let mut list_stack: Vec<char> = Vec::new(); // 'u' for ul, 'o' for ol
    let mut in_pre = false;
    let mut in_code_block = false;
    let mut code_content = String::new();
    let mut in_macro = false;
    let mut tag_stack: Vec<String> = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();

                match tag_name.as_str() {
                    "p" | "div" | "section" => {
                        if !in_pre && !in_code_block {
                            md.push('\n');
                        }
                    }
                    "br" => md.push('\n'),
                    "h1" => {
                        md.push_str("\n# ");
                        tag_stack.push("h1".into());
                    }
                    "h2" => {
                        md.push_str("\n## ");
                        tag_stack.push("h2".into());
                    }
                    "h3" => {
                        md.push_str("\n### ");
                        tag_stack.push("h3".into());
                    }
                    "h4" => {
                        md.push_str("\n#### ");
                        tag_stack.push("h4".into());
                    }
                    "h5" => {
                        md.push_str("\n##### ");
                        tag_stack.push("h5".into());
                    }
                    "h6" => {
                        md.push_str("\n###### ");
                        tag_stack.push("h6".into());
                    }
                    "strong" | "b" => {
                        md.push_str("**");
                        tag_stack.push("strong".into());
                    }
                    "em" | "i" => {
                        md.push('*');
                        tag_stack.push("em".into());
                    }
                    "code" => {
                        if in_pre {
                            code_content.clear();
                            in_code_block = true;
                        } else {
                            md.push('`');
                            tag_stack.push("code".into());
                        }
                    }
                    "pre" => {
                        in_pre = true;
                        tag_stack.push("pre".into());
                    }
                    "a" => {
                        if let Some(href) = get_attr(e, "href") {
                            md.push('[');
                            tag_stack.push(format!("a|{}", href));
                        }
                    }
                    "img" => {
                        let src = get_attr(e, "src");
                        let alt = get_attr(e, "alt").unwrap_or_default();
                        if let Some(src) = src {
                            md.push_str(&format!("![{}]({})", alt, src));
                        }
                    }
                    "ul" => {
                        list_stack.push('u');
                        tag_stack.push("ul".into());
                    }
                    "ol" => {
                        list_stack.push('o');
                        tag_stack.push("ol".into());
                    }
                    "li" => {
                        let prefix = match list_stack.last() {
                            Some('o') => "1. ",
                            _ => "- ",
                        };
                        let indent = "  ".repeat(list_stack.len().saturating_sub(1));
                        md.push_str(&format!("\n{}{}", indent, prefix));
                    }
                    "table" => {
                        tag_stack.push("table".into());
                    }
                    "tr" => {
                        md.push('\n');
                    }
                    "td" | "th" => {
                        md.push_str("| ");
                        tag_stack.push("td".into());
                    }
                    "ac:structured-macro" => {
                        in_macro = true;
                        let name = get_attr(e, "ac:name").unwrap_or_default();
                        tag_stack.push(format!("macro|{}", name));
                    }
                    "ac:plain-text-body" => {
                        // Callout body content
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(ref e)) => {
                let text = e.unescape().unwrap_or_default();
                if in_code_block {
                    code_content.push_str(&text);
                } else if in_macro {
                    // Callout content rendered as blockquote
                    for line in text.lines() {
                        md.push_str("> ");
                        md.push_str(line.trim());
                        md.push('\n');
                    }
                } else {
                    md.push_str(&text);
                }
            }
            Ok(Event::End(ref e)) => {
                let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match tag_name.as_str() {
                    "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                        md.push('\n');
                        tag_stack.pop();
                    }
                    "strong" | "b" => {
                        md.push_str("**");
                        tag_stack.pop();
                    }
                    "em" | "i" => {
                        md.push('*');
                        tag_stack.pop();
                    }
                    "code" => {
                        if in_code_block {
                            md.push_str("```\n");
                            md.push_str(&code_content);
                            md.push_str("\n```\n");
                            in_code_block = false;
                        } else {
                            md.push('`');
                            tag_stack.pop();
                        }
                    }
                    "pre" => {
                        in_pre = false;
                        tag_stack.pop();
                    }
                    "a" => {
                        if let Some(last) = tag_stack.pop()
                            && let Some(href) = last.strip_prefix("a|") {
                                md.push_str(&format!("]({})", href));
                            }
                    }
                    "ul" | "ol" => {
                        list_stack.pop();
                        md.push('\n');
                        tag_stack.pop();
                    }
                    "table" => {
                        md.push('\n');
                        tag_stack.pop();
                    }
                    "td" | "th" => {
                        md.push(' ');
                        tag_stack.pop();
                    }
                    "ac:structured-macro" => {
                        in_macro = false;
                        tag_stack.pop();
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                tracing::warn!("XML parse error: {}", e);
                break;
            }
            _ => {}
        }
        buf.clear();
    }

    // Clean up excessive newlines
    let mut result = md.trim().to_string();
    while result.contains("\n\n\n") {
        result = result.replace("\n\n\n", "\n\n");
    }
    result
}

/// Extract an attribute value from an XML element by name.
fn get_attr(e: &quick_xml::events::BytesStart<'_>, name: &str) -> Option<String> {
    for attr in e.attributes().flatten() {
        if attr.key.as_ref() == name.as_bytes() {
            return Some(String::from_utf8_lossy(&attr.value).to_string());
        }
    }
    None
}

/// Generate a URL-friendly slug from a title.
fn slugify_title(title: &str) -> String {
    tachyon_core::util::slugify(title)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confluence_storage_to_markdown() {
        let html = r#"
            <p>Hello <strong>world</strong></p>
            <h2>Section</h2>
            <p>Some <em>italic</em> text with <code>inline code</code>.</p>
            <pre><code>fn main() {
    println!("hello");
}</code></pre>
            <ul>
                <li>Item 1</li>
                <li>Item 2</li>
            </ul>
            <a href="https://example.com">Link</a>
            <img src="image.png" alt="Photo" />
        "#;

        let md = confluence_storage_to_markdown(html);
        assert!(md.contains("Hello **world**"));
        assert!(md.contains("## Section"));
        assert!(md.contains("*italic*"));
        assert!(md.contains("`inline code`"));
        assert!(md.contains("```"));
        assert!(md.contains("fn main()"));
        assert!(md.contains("- Item 1"));
        assert!(md.contains("- Item 2"));
        assert!(md.contains("[Link](https://example.com)"));
        assert!(md.contains("![Photo](image.png)"));
    }

    #[test]
    fn test_parse_confluence_xml() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
            <confluence-data>
                <page>
                    <id>12345</id>
                    <title>Getting Started</title>
                    <space>DEV</space>
                    <body><![CDATA[<p>Welcome to the <strong>dev</strong> space.</p>]]></body>
                    <labels>
                        <label>onboarding</label>
                        <label>guide</label>
                    </labels>
                </page>
                <page>
                    <id>12346</id>
                    <title>API Reference</title>
                    <space>DEV</space>
                    <body><![CDATA[<h2>Endpoints</h2><p>List of API endpoints.</p>]]></body>
                    <labels>
                        <label>api</label>
                    </labels>
                    <ancestors>
                        <ancestor id="12345"/>
                    </ancestors>
                </page>
            </confluence-data>"#;

        let pages = parse_confluence_xml(xml.as_bytes()).unwrap();
        assert_eq!(pages.len(), 2);
        assert_eq!(pages[0].title, "Getting Started");
        assert_eq!(pages[0].space, "DEV");
        assert!(pages[0].labels.contains(&"onboarding".to_string()));
        assert!(pages[0].labels.contains(&"guide".to_string()));
        assert!(pages[0].body.contains("Welcome"));
        assert_eq!(pages[1].title, "API Reference");
        assert!(pages[1].parent_id.is_some());
    }

    #[test]
    fn test_import_confluence_xml() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
            <confluence-data>
                <page>
                    <id>1</id>
                    <title>Home</title>
                    <space>DOCS</space>
                    <body><![CDATA[<p>Welcome to docs.</p>]]></body>
                    <labels><label>home</label></labels>
                </page>
                <page>
                    <id>2</id>
                    <title></title>
                    <body><![CDATA[<p>Empty title page.</p>]]></body>
                </page>
                <page>
                    <id>3</id>
                    <title>Empty Body</title>
                    <body><![CDATA[]]></body>
                </page>
            </confluence-data>"#;

        let (docs, summary) = ConfluenceImporter::import_from_bytes(xml.as_bytes()).unwrap();

        assert_eq!(summary.imported, 1);
        assert_eq!(summary.skipped, 2); // empty title + empty body
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0].title, "Home");
        assert!(docs[0].tags.contains(&"home".to_string()));
        assert!(docs[0].tags.contains(&"docs".to_string()));
    }

    #[test]
    fn test_slugify_title() {
        assert_eq!(slugify_title("Hello World"), "hello-world");
        assert_eq!(slugify_title("API Reference"), "api-reference");
    }
}
