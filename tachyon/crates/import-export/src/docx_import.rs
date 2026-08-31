//! DOCX import: parse .docx files (ZIP of OOXML) into markdown.
//!
//! Reads `word/document.xml` from the DOCX ZIP, extracts paragraphs,
//! runs, text, headings, bold/italic, lists, and tables, then converts
//! to markdown producing a list of `ImportedDocument` values.

use crate::{
    ImportSummary, ImportedDocument,
    error::{ImportExportError, ImportExportResult},
    frontmatter::Frontmatter,
};
use quick_xml::events::Event;
use std::collections::HashSet;
use std::io::{Cursor, Read};

const DOCX_DOCUMENT_XML: &str = "word/document.xml";

#[derive(Debug, Clone)]
pub struct DocxImportOptions {
    pub convert_tables: bool,
    pub extract_images: bool,
}

impl Default for DocxImportOptions {
    fn default() -> Self {
        Self {
            convert_tables: true,
            extract_images: true,
        }
    }
}

pub struct DocxImporter;

impl DocxImporter {
    pub fn import_from_bytes(
        docx_bytes: &[u8],
    ) -> ImportExportResult<(Vec<ImportedDocument>, ImportSummary)> {
        Self::import_from_bytes_with_options(docx_bytes, &DocxImportOptions::default())
    }

    pub fn import_from_bytes_with_options(
        docx_bytes: &[u8],
        options: &DocxImportOptions,
    ) -> ImportExportResult<(Vec<ImportedDocument>, ImportSummary)> {
        let cursor = Cursor::new(docx_bytes);
        let mut archive =
            zip::ZipArchive::new(cursor).map_err(|e| ImportExportError::zip(e.to_string()))?;

        let mut summary = ImportSummary {
            total_files: archive.len(),
            imported: 0,
            skipped: 0,
            failed: 0,
            document_titles: Vec::new(),
            all_tags: Vec::new(),
            warnings: Vec::new(),
        };

        let mut documents = Vec::new();
        let mut all_tags: HashSet<String> = HashSet::new();

        let mut file = match archive.by_name(DOCX_DOCUMENT_XML) {
            Ok(f) => f,
            Err(_) => {
                return Err(ImportExportError::import(
                    "Invalid DOCX: word/document.xml not found",
                ));
            }
        };

        let mut xml_content = String::new();
        file.read_to_string(&mut xml_content).map_err(|e| {
            ImportExportError::import(format!("Failed to read document.xml: {}", e))
        })?;

        let markdown = parse_docx_xml(&xml_content, options);

        let title = extract_first_heading(&markdown).unwrap_or_else(|| "Untitled Document".into());

        for tag in &["docx", "import"] {
            all_tags.insert(tag.to_string());
        }

        summary.imported += 1;
        summary.document_titles.push(title.clone());

        documents.push(ImportedDocument {
            title,
            slug: None,
            content: markdown,
            frontmatter: Frontmatter::default(),
            tags: all_tags.iter().cloned().collect(),
            source_path: "document.docx".to_string(),
            created_at: None,
            updated_at: None,
            extra: std::collections::BTreeMap::new(),
        });

        summary.all_tags = all_tags.into_iter().collect();
        summary.all_tags.sort();
        summary.skipped = summary.total_files.saturating_sub(1);

        Ok((documents, summary))
    }

    pub fn import_from_bytes_multi(
        docx_bytes: &[u8],
    ) -> ImportExportResult<(Vec<ImportedDocument>, ImportSummary)> {
        Self::import_from_bytes(docx_bytes)
    }
}

fn extract_first_heading(markdown: &str) -> Option<String> {
    for line in markdown.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            return Some(trimmed.trim_start_matches('#').trim().to_string());
        }
    }
    if let Some(first_line) = markdown.lines().next() {
        let trimmed = first_line.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    None
}

#[derive(Debug, Clone, Default)]
struct Paragraph {
    text: String,
    heading_level: Option<u8>,
    is_bold: bool,
    is_italic: bool,
    is_list_item: bool,
    num_id: Option<String>,
    ilvl: Option<String>,
}

#[derive(Debug, Clone, Default)]
struct Run {
    text: String,
    bold: bool,
    italic: bool,
}

#[derive(Debug, Clone, Default)]
struct TableRow {
    cells: Vec<String>,
}

impl Paragraph {
    fn new() -> Self {
        Self::default()
    }
}

impl Run {
    fn new() -> Self {
        Self::default()
    }
}

struct DocxParser {
    options: DocxImportOptions,
    paragraphs: Vec<Paragraph>,
    current_paragraph: Paragraph,
    current_run: Run,
    in_run: bool,
    in_text: bool,
    in_table: bool,
    table_rows: Vec<TableRow>,
    current_row: TableRow,
    current_cell: String,
    in_row: bool,
    in_cell: bool,
    current_heading_level: Option<u8>,
    is_list_item: bool,
    in_numpr: bool,
    num_id: String,
    ilvl: String,
    in_bookmark_start: bool,
}

fn get_attr_val(e: &quick_xml::events::BytesStart, attr_name: &[u8]) -> Option<String> {
    for attr_result in e.attributes() {
        if let Ok(attr) = attr_result
            && attr.key.local_name().as_ref() == attr_name
        {
            return Some(String::from_utf8_lossy(&attr.value).to_string());
        }
    }
    None
}

impl DocxParser {
    fn new(options: DocxImportOptions) -> Self {
        Self {
            options,
            paragraphs: Vec::new(),
            current_paragraph: Paragraph::new(),
            current_run: Run::new(),
            in_run: false,
            in_text: false,
            in_table: false,
            table_rows: Vec::new(),
            current_row: TableRow { cells: Vec::new() },
            current_cell: String::new(),
            in_row: false,
            in_cell: false,
            current_heading_level: None,
            is_list_item: false,
            in_numpr: false,
            num_id: String::new(),
            ilvl: String::new(),
            in_bookmark_start: false,
        }
    }

    fn process_event(&mut self, event: &Event) {
        match event {
            Event::Start(e) => self.handle_start(e),
            Event::Empty(e) => self.handle_empty(e),
            Event::End(e) => self.handle_end(e),
            Event::Text(e) => {
                if self.in_text {
                    let text = e.decode().unwrap_or_default().to_string();
                    self.current_run.text.push_str(&text);
                }
            }
            _ => {}
        }
    }

    fn handle_start(&mut self, e: &quick_xml::events::BytesStart) {
        let name = e.local_name();
        let name_ref: &[u8] = name.as_ref();

        match name_ref {
            b"p" => {
                self.current_paragraph = Paragraph::new();
                self.current_heading_level = None;
                self.is_list_item = false;
                self.num_id.clear();
                self.ilvl.clear();
            }
            b"r" => {
                self.current_run = Run::new();
                self.in_run = true;
            }
            b"t" => {
                self.in_text = true;
            }
            b"pStyle" => {
                if let Some(style) = get_attr_val(e, b"val") {
                    self.current_heading_level = heading_level_from_style(&style);
                }
            }
            b"b" | b"bCs" => {
                if self.in_run {
                    self.current_run.bold = true;
                }
            }
            b"i" | b"iCs" => {
                if self.in_run {
                    self.current_run.italic = true;
                }
            }
            b"tbl" => {
                self.in_table = true;
                self.table_rows.clear();
            }
            b"tr" => {
                self.in_row = true;
                self.current_row = TableRow { cells: Vec::new() };
            }
            b"tc" => {
                self.in_cell = true;
                self.current_cell = String::new();
            }
            b"numPr" => {
                self.in_numpr = true;
            }
            b"numId" => {
                if let Some(val) = get_attr_val(e, b"val") {
                    self.num_id = val;
                }
            }
            b"ilvl" => {
                if let Some(val) = get_attr_val(e, b"val") {
                    self.ilvl = val;
                }
            }
            b"bookmarkStart" => {
                self.in_bookmark_start = true;
            }
            _ => {}
        }
    }

    fn handle_empty(&mut self, e: &quick_xml::events::BytesStart) {
        let name = e.local_name();
        let name_ref: &[u8] = name.as_ref();

        match name_ref {
            b"pStyle" => {
                if let Some(style) = get_attr_val(e, b"val") {
                    self.current_heading_level = heading_level_from_style(&style);
                }
            }
            b"numId" => {
                if let Some(val) = get_attr_val(e, b"val")
                    && val != "0"
                {
                    self.is_list_item = true;
                    self.num_id = val;
                }
            }
            b"ilvl" => {
                if let Some(val) = get_attr_val(e, b"val") {
                    self.ilvl = val;
                }
            }
            b"br" => {
                if self.in_run {
                    self.current_run.text.push('\n');
                }
            }
            _ => {}
        }
    }

    fn handle_end(&mut self, e: &quick_xml::events::BytesEnd) {
        let name = e.local_name();
        let name_ref: &[u8] = name.as_ref();

        match name_ref {
            b"t" => {
                self.in_text = false;
            }
            b"r" => {
                self.in_run = false;
                if !self.current_run.text.is_empty() {
                    self.current_paragraph.text.push_str(&self.current_run.text);
                    if self.current_run.bold {
                        self.current_paragraph.is_bold = true;
                    }
                    if self.current_run.italic {
                        self.current_paragraph.is_italic = true;
                    }
                }
            }
            b"p" => {
                self.current_paragraph.heading_level = self.current_heading_level.take();
                self.current_paragraph.is_list_item = self.is_list_item;
                if !self.num_id.is_empty() {
                    self.current_paragraph.num_id = Some(self.num_id.clone());
                }
                if !self.ilvl.is_empty() {
                    self.current_paragraph.ilvl = Some(self.ilvl.clone());
                }

                if self.in_table && (self.in_cell || self.in_row) {
                    if !self.current_cell.is_empty() {
                        self.current_cell.push('\n');
                    }
                    self.current_cell.push_str(&self.current_paragraph.text);
                } else {
                    self.paragraphs
                        .push(std::mem::take(&mut self.current_paragraph));
                }
            }
            b"tc" => {
                self.in_cell = false;
                if !self.current_cell.is_empty() {
                    self.current_row
                        .cells
                        .push(std::mem::take(&mut self.current_cell));
                }
            }
            b"tr" => {
                self.in_row = false;
                if !self.current_row.cells.is_empty() {
                    self.table_rows.push(std::mem::take(&mut self.current_row));
                }
            }
            b"tbl" => {
                self.in_table = false;
            }
            b"numPr" => {
                self.in_numpr = false;
            }
            b"bookmarkStart" => {
                self.in_bookmark_start = false;
            }
            _ => {}
        }
    }

    fn to_markdown(&self) -> String {
        let mut md = String::new();

        for para in &self.paragraphs {
            if let Some(level) = para.heading_level {
                md.push_str(&"#".repeat(level as usize));
                md.push(' ');
                md.push_str(&para.text);
                md.push('\n');
            } else if para.is_list_item {
                let indent = para
                    .ilvl
                    .as_deref()
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(0);
                md.push_str(&"  ".repeat(indent));
                md.push_str("- ");
                md.push_str(&para.text);
                md.push('\n');
            } else if para.text.trim().is_empty() {
                md.push('\n');
            } else {
                md.push_str(&para.text);
                md.push('\n');
            }
        }

        if self.options.convert_tables && !self.table_rows.is_empty() {
            md.push('\n');
            if let Some((first_row, rest)) = self.table_rows.split_first() {
                md.push('|');
                for cell in &first_row.cells {
                    md.push_str(&xml_to_md_inline(cell));
                    md.push('|');
                }
                md.push('\n');

                md.push('|');
                for _ in &first_row.cells {
                    md.push_str("---|");
                }
                md.push('\n');

                for row in rest {
                    md.push('|');
                    for cell in &row.cells {
                        md.push_str(&xml_to_md_inline(cell));
                        md.push('|');
                    }
                    md.push('\n');
                }
                md.push('\n');
            }
        }

        while md.ends_with("\n\n\n") {
            md.pop();
        }
        while md.starts_with("\n") {
            md.remove(0);
        }

        md
    }
}

fn heading_level_from_style(style: &str) -> Option<u8> {
    let lower = style.to_lowercase();
    if lower == "title" {
        return Some(1);
    }
    if lower.contains("heading") {
        let digits: String = lower.chars().filter(|c| c.is_ascii_digit()).collect();
        return digits.parse::<u8>().ok();
    }
    None
}

fn xml_to_md_inline(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut bold_open = false;
    let mut italic_open = false;
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if i + 2 < chars.len() && chars[i] == '<' {
            let end = match chars[i + 1..].iter().position(|&c| c == '>') {
                Some(pos) => i + 1 + pos,
                None => {
                    result.push(chars[i]);
                    i += 1;
                    continue;
                }
            };
            let tag: String = chars[i + 1..end].iter().collect();

            match tag.as_str() {
                "b" | "strong" if !bold_open => {
                    result.push_str("**");
                    bold_open = true;
                }
                "/b" | "/strong" if bold_open => {
                    result.push_str("**");
                    bold_open = false;
                }
                "i" | "em" if !italic_open => {
                    result.push('*');
                    italic_open = true;
                }
                "/i" | "/em" if italic_open => {
                    result.push('*');
                    italic_open = false;
                }
                "br" | "/br" => {
                    result.push('\n');
                }
                _ => {}
            }
            i = end + 1;
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }

    if bold_open {
        result.push_str("**");
    }
    if italic_open {
        result.push('*');
    }

    result
}

fn parse_docx_xml(xml: &str, options: &DocxImportOptions) -> String {
    let mut parser = DocxParser::new(options.clone());
    let mut reader = quick_xml::Reader::from_str(xml);

    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Eof) => break,
            Ok(event) => {
                parser.process_event(&event);
            }
            Err(e) => {
                tracing::warn!("DOCX XML read error: {}", e);
                continue;
            }
        }
        buf.clear();
    }

    parser.to_markdown()
}

#[cfg(test)]
fn create_minimal_docx_bytes(document_xml: &str) -> Vec<u8> {
    use std::io::Write;
    use zip::write::SimpleFileOptions;
    use zip::{CompressionMethod, ZipWriter};

    let buf = Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(buf);
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .compression_level(Some(6));

    let content_types = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
</Types>"#;

    let rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
</Relationships>"#;

    let word_rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
</Relationships>"#;

    zip.start_file("[Content_Types].xml", options).unwrap();
    zip.write_all(content_types.as_bytes()).unwrap();

    zip.start_file("_rels/.rels", options).unwrap();
    zip.write_all(rels.as_bytes()).unwrap();

    zip.start_file("word/_rels/document.xml.rels", options)
        .unwrap();
    zip.write_all(word_rels.as_bytes()).unwrap();

    zip.start_file("word/document.xml", options).unwrap();
    zip.write_all(document_xml.as_bytes()).unwrap();

    let buf = zip.finish().unwrap();
    buf.into_inner()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_docx(document_xml: &str) -> Vec<u8> {
        create_minimal_docx_bytes(document_xml)
    }

    fn simple_doc_xml(content: &str) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
            xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <w:body>
    {}
  </w:body>
</w:document>"#,
            content
        )
    }

    #[test]
    fn test_import_simple_paragraph() {
        let xml = simple_doc_xml(r#"<w:p><w:r><w:t>Hello World</w:t></w:r></w:p>"#);
        let bytes = make_docx(&xml);
        let (docs, summary) = DocxImporter::import_from_bytes(&bytes).unwrap();
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0].content.trim(), "Hello World");
        assert_eq!(summary.imported, 1);
    }

    #[test]
    fn test_import_heading_detection() {
        let xml = simple_doc_xml(
            r#"<w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t>Title</w:t></w:r></w:p>
<w:p><w:pPr><w:pStyle w:val="Heading2"/></w:pPr><w:r><w:t>Subtitle</w:t></w:r></w:p>
<w:p><w:r><w:t>Body</w:t></w:r></w:p>"#,
        );
        let bytes = make_docx(&xml);
        let (docs, _) = DocxImporter::import_from_bytes(&bytes).unwrap();
        let md = &docs[0].content;
        assert!(md.contains("# Title"));
        assert!(md.contains("## Subtitle"));
        assert!(md.contains("Body"));
    }

    #[test]
    fn test_import_bold_italic() {
        let xml = simple_doc_xml(
            r#"<w:p>
  <w:r><w:rPr><w:b/></w:rPr><w:t>bold text</w:t></w:r>
  <w:r><w:rPr><w:i/></w:rPr><w:t> italic text</w:t></w:r>
  <w:r><w:rPr><w:b/><w:i/></w:rPr><w:t> bolditalic</w:t></w:r>
</w:p>"#,
        );
        let bytes = make_docx(&xml);
        let (docs, _) = DocxImporter::import_from_bytes(&bytes).unwrap();
        let md = &docs[0].content;
        assert!(md.contains("bold text"));
        assert!(md.contains("italic text"));
        assert!(md.contains("bolditalic"));
    }

    #[test]
    fn test_import_title_style() {
        let xml = simple_doc_xml(
            r#"<w:p><w:pPr><w:pStyle w:val="Title"/></w:pPr><w:r><w:t>My Title</w:t></w:r></w:p>"#,
        );
        let bytes = make_docx(&xml);
        let (docs, _) = DocxImporter::import_from_bytes(&bytes).unwrap();
        assert!(docs[0].content.contains("# My Title"));
    }

    #[test]
    fn test_import_multiple_paragraphs() {
        let xml = simple_doc_xml(
            r#"<w:p><w:r><w:t>First paragraph.</w:t></w:r></w:p>
<w:p><w:r><w:t>Second paragraph.</w:t></w:r></w:p>
<w:p><w:r><w:t>Third paragraph.</w:t></w:r></w:p>"#,
        );
        let bytes = make_docx(&xml);
        let (docs, _) = DocxImporter::import_from_bytes(&bytes).unwrap();
        let md = &docs[0].content;
        assert!(md.contains("First paragraph."));
        assert!(md.contains("Second paragraph."));
        assert!(md.contains("Third paragraph."));
    }

    #[test]
    fn test_import_empty_docx_fails() {
        let result = DocxImporter::import_from_bytes(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_import_invalid_zip_fails() {
        let result = DocxImporter::import_from_bytes(b"not a zip file");
        assert!(result.is_err());
    }

    #[test]
    fn test_import_missing_document_xml_fails() {
        let buf = Cursor::new(Vec::new());
        let mut zip = zip::ZipWriter::new(buf);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        zip.start_file("not_document.xml", options).unwrap();
        use std::io::Write;
        zip.write_all(b"nope").unwrap();
        let buf = zip.finish().unwrap();
        let bytes = buf.into_inner();

        let result = DocxImporter::import_from_bytes(&bytes);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("word/document.xml not found")
        );
    }

    #[test]
    fn test_import_table() {
        let xml = simple_doc_xml(
            r#"<w:p><w:r><w:t>Before table</w:t></w:r></w:p>
<w:tbl>
  <w:tr>
    <w:tc><w:p><w:r><w:t>Name</w:t></w:r></w:p></w:tc>
    <w:tc><w:p><w:r><w:t>Value</w:t></w:r></w:p></w:tc>
  </w:tr>
  <w:tr>
    <w:tc><w:p><w:r><w:t>Foo</w:t></w:r></w:p></w:tc>
    <w:tc><w:p><w:r><w:t>Bar</w:t></w:r></w:p></w:tc>
  </w:tr>
</w:tbl>"#,
        );
        let bytes = make_docx(&xml);
        let (docs, _) = DocxImporter::import_from_bytes(&bytes).unwrap();
        let md = &docs[0].content;
        assert!(md.contains("|"));
        assert!(md.contains("Name"));
        assert!(md.contains("Value"));
        assert!(md.contains("Foo"));
        assert!(md.contains("Bar"));
    }

    #[test]
    fn test_import_with_options() {
        let xml = simple_doc_xml(r#"<w:p><w:r><w:t>Test content</w:t></w:r></w:p>"#);
        let bytes = make_docx(&xml);
        let options = DocxImportOptions {
            convert_tables: false,
            extract_images: false,
        };
        let (docs, _) = DocxImporter::import_from_bytes_with_options(&bytes, &options).unwrap();
        assert_eq!(docs[0].content.trim(), "Test content");
    }

    #[test]
    fn test_extract_first_heading() {
        assert_eq!(
            extract_first_heading("# Hello\n\nWorld"),
            Some("Hello".to_string())
        );
        assert_eq!(
            extract_first_heading("Just text"),
            Some("Just text".to_string())
        );
        assert_eq!(extract_first_heading(""), None);
    }

    #[test]
    fn test_heading_level_from_style() {
        assert_eq!(heading_level_from_style("Heading1"), Some(1));
        assert_eq!(heading_level_from_style("Heading2"), Some(2));
        assert_eq!(heading_level_from_style("heading3"), Some(3));
        assert_eq!(heading_level_from_style("heading4"), Some(4));
        assert_eq!(heading_level_from_style("Title"), Some(1));
        assert_eq!(heading_level_from_style("Normal"), None);
        assert_eq!(heading_level_from_style("Quote"), None);
    }

    #[test]
    fn test_xml_to_md_inline() {
        assert_eq!(xml_to_md_inline("normal text"), "normal text");
        assert_eq!(xml_to_md_inline("<b>bold</b> text"), "**bold** text");
        assert_eq!(xml_to_md_inline("<i>italic</i> text"), "*italic* text");
        assert_eq!(
            xml_to_md_inline("<b>bold</b> and <i>italic</i>"),
            "**bold** and *italic*"
        );
    }

    #[test]
    fn test_create_minimal_docx_bytes_roundtrip() {
        let xml = simple_doc_xml(r#"<w:p><w:r><w:t>Roundtrip test</w:t></w:r></w:p>"#);
        let bytes = create_minimal_docx_bytes(&xml);
        let cursor = Cursor::new(&bytes);
        let mut archive = zip::ZipArchive::new(cursor).unwrap();
        assert!(archive.by_name("word/document.xml").is_ok());
        assert!(archive.by_name("[Content_Types].xml").is_ok());
        assert!(archive.by_name("_rels/.rels").is_ok());
    }
}
