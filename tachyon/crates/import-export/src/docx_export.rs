//! DOCX export: convert markdown content to .docx (ZIP of OOXML).
//!
//! Parses markdown into blocks (headings, paragraphs, lists, tables,
//! code blocks) and generates the DOCX ZIP structure with proper
//! OOXML markup.

use crate::{
    ExportSummary,
    error::{ImportExportError, ImportExportResult},
};
use std::io::{Cursor, Write};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

#[derive(Debug, Clone)]
pub struct DocxExportOptions {
    pub include_metadata: bool,
    pub page_size: PageSize,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum PageSize {
    #[default]
    A4,
    Letter,
}

impl PageSize {
    fn width_inches(&self) -> f64 {
        match self {
            Self::A4 => 8.27,
            Self::Letter => 8.5,
        }
    }

    fn height_inches(&self) -> f64 {
        match self {
            Self::A4 => 11.69,
            Self::Letter => 11.0,
        }
    }

    fn width_twips(&self) -> i64 {
        (self.width_inches() * 1440.0) as i64
    }

    fn height_twips(&self) -> i64 {
        (self.height_inches() * 1440.0) as i64
    }
}

impl Default for DocxExportOptions {
    fn default() -> Self {
        Self {
            include_metadata: true,
            page_size: PageSize::default(),
        }
    }
}

pub struct DocxExporter;

impl DocxExporter {
    pub fn export_to_bytes(
        title: &str,
        markdown_content: &str,
    ) -> ImportExportResult<(Vec<u8>, ExportSummary)> {
        Self::export_to_bytes_with_options(title, markdown_content, &DocxExportOptions::default())
    }

    pub fn export_to_bytes_with_options(
        title: &str,
        markdown_content: &str,
        options: &DocxExportOptions,
    ) -> ImportExportResult<(Vec<u8>, ExportSummary)> {
        let blocks = parse_markdown_blocks(markdown_content);
        let document_xml = build_document_xml(title, &blocks, options);

        let content_types = build_content_types_xml();
        let rels = build_rels_xml();
        let word_rels = build_word_rels_xml();

        let buf = Cursor::new(Vec::new());
        let mut zip = ZipWriter::new(buf);
        let zip_options = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .compression_level(Some(6));

        zip.start_file("[Content_Types].xml", zip_options)
            .map_err(|e| ImportExportError::zip(e.to_string()))?;
        zip.write_all(content_types.as_bytes())
            .map_err(|e| ImportExportError::zip(e.to_string()))?;

        zip.start_file("_rels/.rels", zip_options)
            .map_err(|e| ImportExportError::zip(e.to_string()))?;
        zip.write_all(rels.as_bytes())
            .map_err(|e| ImportExportError::zip(e.to_string()))?;

        zip.start_file("word/_rels/document.xml.rels", zip_options)
            .map_err(|e| ImportExportError::zip(e.to_string()))?;
        zip.write_all(word_rels.as_bytes())
            .map_err(|e| ImportExportError::zip(e.to_string()))?;

        zip.start_file("word/document.xml", zip_options)
            .map_err(|e| ImportExportError::zip(e.to_string()))?;
        zip.write_all(document_xml.as_bytes())
            .map_err(|e| ImportExportError::zip(e.to_string()))?;

        let buf = zip
            .finish()
            .map_err(|e| ImportExportError::zip(e.to_string()))?;
        let bytes = buf.into_inner();

        let summary = ExportSummary {
            exported: 1,
            format: "docx".to_string(),
            file_size_bytes: Some(bytes.len() as u64),
            warnings: Vec::new(),
        };

        Ok((bytes, summary))
    }

    pub fn export_batch_to_bytes(
        documents: &[(&str, &str)],
    ) -> ImportExportResult<(Vec<u8>, ExportSummary)> {
        Self::export_batch_to_bytes_with_options(documents, &DocxExportOptions::default())
    }

    pub fn export_batch_to_bytes_with_options(
        documents: &[(&str, &str)],
        options: &DocxExportOptions,
    ) -> ImportExportResult<(Vec<u8>, ExportSummary)> {
        let buf = Cursor::new(Vec::new());
        let mut zip = ZipWriter::new(buf);
        let zip_options = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .compression_level(Some(6));

        let content_types = build_content_types_xml();
        let rels = build_rels_xml();

        zip.start_file("[Content_Types].xml", zip_options)
            .map_err(|e| ImportExportError::zip(e.to_string()))?;
        zip.write_all(content_types.as_bytes())
            .map_err(|e| ImportExportError::zip(e.to_string()))?;

        zip.start_file("_rels/.rels", zip_options)
            .map_err(|e| ImportExportError::zip(e.to_string()))?;
        zip.write_all(rels.as_bytes())
            .map_err(|e| ImportExportError::zip(e.to_string()))?;

        let mut warnings = Vec::new();

        for (i, (title, content)) in documents.iter().enumerate() {
            let path = if i == 0 {
                "word/document.xml".to_string()
            } else {
                format!("word/document{}.xml", i)
            };

            let blocks = parse_markdown_blocks(content);
            let doc_xml = build_document_xml(title, &blocks, options);

            zip.start_file(&path, zip_options)
                .map_err(|e| ImportExportError::zip(e.to_string()))?;
            zip.write_all(doc_xml.as_bytes())
                .map_err(|e| ImportExportError::zip(e.to_string()))?;

            if content.len() > 1_000_000 {
                warnings.push(format!(
                    "Large document: {} ({} chars)",
                    title,
                    content.len()
                ));
            }
        }

        let word_rels = build_word_rels_xml();
        zip.start_file("word/_rels/document.xml.rels", zip_options)
            .map_err(|e| ImportExportError::zip(e.to_string()))?;
        zip.write_all(word_rels.as_bytes())
            .map_err(|e| ImportExportError::zip(e.to_string()))?;

        let buf = zip
            .finish()
            .map_err(|e| ImportExportError::zip(e.to_string()))?;
        let bytes = buf.into_inner();

        let summary = ExportSummary {
            exported: documents.len(),
            format: "docx".to_string(),
            file_size_bytes: Some(bytes.len() as u64),
            warnings,
        };

        Ok((bytes, summary))
    }
}

#[derive(Debug, Clone, PartialEq)]
enum MdBlock {
    Heading {
        level: u8,
        text: String,
    },
    Paragraph {
        text: String,
    },
    CodeBlock {
        language: Option<String>,
        code: String,
    },
    UnorderedItem {
        indent: usize,
        text: String,
    },
    OrderedItem {
        indent: usize,
        number: usize,
        text: String,
    },
    Table {
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
    },
    HorizontalRule,
}

fn parse_markdown_blocks(md: &str) -> Vec<MdBlock> {
    let mut blocks = Vec::new();
    let mut lines = md.lines().peekable();
    let mut in_code_block = false;
    let mut code_lang = None;
    let mut code_lines = Vec::new();

    while let Some(line) = lines.next() {
        let trimmed = line.trim();

        if in_code_block {
            if trimmed.starts_with("```") {
                blocks.push(MdBlock::CodeBlock {
                    language: code_lang.take(),
                    code: code_lines.join("\n"),
                });
                code_lines.clear();
                in_code_block = false;
            } else {
                code_lines.push(line.to_string());
            }
            continue;
        }

        if trimmed.starts_with("```") {
            in_code_block = true;
            code_lang = trimmed.strip_prefix("```").map(|s| s.trim().to_string());
            code_lang = code_lang.filter(|s| !s.is_empty());
            continue;
        }

        if trimmed.is_empty() {
            continue;
        }

        if trimmed == "---" || trimmed == "***" || trimmed == "___" {
            blocks.push(MdBlock::HorizontalRule);
            continue;
        }

        if let Some(heading) = parse_heading(trimmed) {
            blocks.push(heading);
            continue;
        }

        if let Some(item) = parse_unordered_item(trimmed) {
            blocks.push(item);
            continue;
        }

        if let Some(item) = parse_ordered_item(trimmed) {
            blocks.push(item);
            continue;
        }

        if trimmed.starts_with('|') {
            let mut table_lines = vec![line.to_string()];
            while let Some(next) = lines.peek() {
                let next_trimmed = next.trim();
                if next_trimmed.starts_with('|') {
                    table_lines.push(lines.next().unwrap().to_string());
                } else {
                    break;
                }
            }
            if let Some(table) = parse_table(&table_lines) {
                blocks.push(table);
            }
            continue;
        }

        blocks.push(MdBlock::Paragraph {
            text: line.to_string(),
        });
    }

    if in_code_block {
        blocks.push(MdBlock::CodeBlock {
            language: code_lang,
            code: code_lines.join("\n"),
        });
    }

    blocks
}

fn parse_heading(line: &str) -> Option<MdBlock> {
    let count = line.chars().take_while(|&c| c == '#').count();
    if !(1..=6).contains(&count) {
        return None;
    }
    let rest = line[count..].trim();
    if rest.is_empty() {
        return None;
    }
    Some(MdBlock::Heading {
        level: count as u8,
        text: rest.to_string(),
    })
}

fn parse_unordered_item(line: &str) -> Option<MdBlock> {
    let trimmed = line.trim_start();
    let indent = line.len() - trimmed.len();
    let rest = trimmed
        .strip_prefix("- ")
        .or_else(|| trimmed.strip_prefix("* "))
        .or_else(|| trimmed.strip_prefix("+ "))?;
    Some(MdBlock::UnorderedItem {
        indent: indent / 2,
        text: rest.to_string(),
    })
}

fn parse_ordered_item(line: &str) -> Option<MdBlock> {
    let trimmed = line.trim_start();
    let indent = line.len() - trimmed.len();
    let rest = trimmed
        .strip_prefix("1. ")
        .or_else(|| trimmed.strip_prefix("2. "))
        .or_else(|| trimmed.strip_prefix("3. "))
        .or_else(|| trimmed.strip_prefix("4. "))
        .or_else(|| trimmed.strip_prefix("5. "))
        .or_else(|| trimmed.strip_prefix("6. "))
        .or_else(|| trimmed.strip_prefix("7. "))
        .or_else(|| trimmed.strip_prefix("8. "))
        .or_else(|| trimmed.strip_prefix("9. "));

    if let Some(text) = rest {
        let num: usize = trimmed
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect::<String>()
            .parse()
            .unwrap_or(1);
        return Some(MdBlock::OrderedItem {
            indent: indent / 2,
            number: num,
            text: text.to_string(),
        });
    }
    None
}

fn parse_table(lines: &[String]) -> Option<MdBlock> {
    if lines.len() < 2 {
        return None;
    }

    let parse_row = |line: &str| -> Vec<String> {
        line.split('|')
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.trim().to_string())
            .collect()
    };

    let headers = parse_row(&lines[0]);
    if headers.is_empty() {
        return None;
    }

    let is_separator = |line: &str| -> bool {
        let trimmed = line.trim();
        trimmed
            .chars()
            .all(|c| c == '|' || c == '-' || c == ':' || c == ' ')
    };

    let rows: Vec<Vec<String>> = lines
        .iter()
        .skip(1)
        .filter(|l| !is_separator(l))
        .map(|l| parse_row(l))
        .collect();

    Some(MdBlock::Table { headers, rows })
}

fn build_document_xml(_title: &str, blocks: &[MdBlock], options: &DocxExportOptions) -> String {
    let mut body = String::new();

    for block in blocks {
        match block {
            MdBlock::Heading { level, text } => {
                let style = format!("Heading{}", level);
                body.push_str(&format!(
                    r#"    <w:p>
      <w:pPr>
        <w:pStyle w:val="{}"/>
      </w:pPr>
      <w:r><w:t>{}</w:t></w:r>
    </w:p>"#,
                    style,
                    xml_escape(text)
                ));
            }
            MdBlock::Paragraph { text } => {
                body.push_str(&format!(
                    r#"
    <w:p>
      <w:r><w:t>{}</w:t></w:r>
    </w:p>"#,
                    xml_escape(text)
                ));
            }
            MdBlock::CodeBlock { code, .. } => {
                for line in code.lines() {
                    body.push_str(&format!(
                        r#"
    <w:p>
      <w:pPr>
        <w:pStyle w:val="Code"/>
        <w:rPr><w:rFonts w:ascii="Courier New" w:hAnsi="Courier New"/></w:rPr>
      </w:pPr>
      <w:r><w:rPr><w:rFonts w:ascii="Courier New" w:hAnsi="Courier New"/></w:rPr><w:t xml:space="preserve">{}</w:t></w:r>
    </w:p>"#,
                        xml_escape(line)
                    ));
                }
            }
            MdBlock::UnorderedItem { indent, text } => {
                let indent_twips = (*indent as i64 * 720).to_string();
                body.push_str(&format!(
                    r#"
    <w:p>
      <w:pPr>
        <w:pStyle w:val="ListParagraph"/>
        <w:ind w:left="{}"/>
      </w:pPr>
      <w:r><w:t>{}</w:t></w:r>
    </w:p>"#,
                    indent_twips,
                    xml_escape(text)
                ));
            }
            MdBlock::OrderedItem {
                indent,
                number,
                text,
            } => {
                let indent_twips = (*indent as i64 * 720).to_string();
                body.push_str(&format!(
                    r#"
    <w:p>
      <w:pPr>
        <w:pStyle w:val="ListParagraph"/>
        <w:ind w:left="{}"/>
        <w:numPr>
          <w:ilvl w:val="0"/>
          <w:numId w:val="{}"/>
        </w:numPr>
      </w:pPr>
      <w:r><w:t>{}</w:t></w:r>
    </w:p>"#,
                    indent_twips,
                    *number + 1,
                    xml_escape(text)
                ));
            }
            MdBlock::Table { headers, rows } => {
                body.push_str("\n    <w:tbl>\n      <w:tblPr>\n        <w:tblBorders>\n");
                body.push_str(
                    r#"          <w:top w:val="single" w:sz="4" w:space="0" w:color="auto"/>
          <w:left w:val="single" w:sz="4" w:space="0" w:color="auto"/>
          <w:bottom w:val="single" w:sz="4" w:space="0" w:color="auto"/>
          <w:right w:val="single" w:sz="4" w:space="0" w:color="auto"/>
          <w:insideH w:val="single" w:sz="4" w:space="0" w:color="auto"/>
          <w:insideV w:val="single" w:sz="4" w:space="0" w:color="auto"/>
"#,
                );
                body.push_str("        </w:tblPr>\n      </w:tblPr>\n");

                if !headers.is_empty() {
                    body.push_str("      <w:tr>\n");
                    for header in headers {
                        body.push_str(&format!(
                            r#"        <w:tc>
          <w:tcPr><w:pStyle w:val="Heading3"/></w:tcPr>
          <w:p><w:r><w:rPr><w:b/></w:rPr><w:t>{}</w:t></w:r></w:p>
        </w:tc>"#,
                            xml_escape(header)
                        ));
                    }
                    body.push_str("\n      </w:tr>\n");
                }

                for row in rows {
                    body.push_str("      <w:tr>\n");
                    for cell in row {
                        body.push_str(&format!(
                            r#"        <w:tc>
          <w:p><w:r><w:t>{}</w:t></w:r></w:p>
        </w:tc>"#,
                            xml_escape(cell)
                        ));
                    }
                    body.push_str("\n      </w:tr>\n");
                }
                body.push_str("    </w:tbl>\n");
            }
            MdBlock::HorizontalRule => {
                body.push_str(
                    r#"
    <w:p>
      <w:pPr>
        <w:pBdr>
          <w:bottom w:val="single" w:sz="6" w:space="1" w:color="auto"/>
        </w:pBdr>
      </w:pPr>
    </w:p>"#,
                );
            }
        }
        body.push('\n');
    }

    let page_w = options.page_size.width_twips();
    let page_h = options.page_size.height_twips();
    let margin = 1440;

    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
            xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <w:body>
    <w:sectPr>
      <w:pgSz w:w="{}" w:h="{}"/>
      <w:pgMar w:top="{}" w:right="{}" w:bottom="{}" w:left="{}" w:header="720" w:footer="720" w:gutter="0"/>
    </w:sectPr>
{}
  </w:body>
</w:document>"#,
        page_w, page_h, margin, margin, margin, margin, body
    )
}

fn build_content_types_xml() -> String {
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
</Types>"#
        .to_string()
}

fn build_rels_xml() -> String {
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
</Relationships>"#
        .to_string()
}

fn build_word_rels_xml() -> String {
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
</Relationships>"#
        .to_string()
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read as _;

    #[test]
    fn test_export_simple_document() {
        let (bytes, summary) =
            DocxExporter::export_to_bytes("Test Doc", "# Hello\n\nWorld").unwrap();
        assert_eq!(summary.exported, 1);
        assert_eq!(summary.format, "docx");
        assert!(summary.file_size_bytes.unwrap() > 0);
        assert!(!bytes.is_empty());

        verify_zip_structure(&bytes);
    }

    #[test]
    fn test_export_headings() {
        let md = "# H1\n\n## H2\n\n### H3\n\n#### H4";
        let (bytes, _) = DocxExporter::export_to_bytes("Headings", md).unwrap();
        let xml = extract_document_xml(&bytes);
        assert!(xml.contains("Heading1"));
        assert!(xml.contains("Heading2"));
        assert!(xml.contains("Heading3"));
        assert!(xml.contains("Heading4"));
    }

    #[test]
    fn test_export_paragraphs() {
        let md = "First paragraph.\n\nSecond paragraph.";
        let (bytes, _) = DocxExporter::export_to_bytes("Paragraphs", md).unwrap();
        let xml = extract_document_xml(&bytes);
        assert!(xml.contains("First paragraph"));
        assert!(xml.contains("Second paragraph"));
    }

    #[test]
    fn test_export_code_block() {
        let md = "```rust\nfn main() {}\n```";
        let (bytes, _) = DocxExporter::export_to_bytes("Code", md).unwrap();
        let xml = extract_document_xml(&bytes);
        assert!(xml.contains("Courier New"));
        assert!(xml.contains("fn main()"));
    }

    #[test]
    fn test_export_unordered_list() {
        let md = "- item 1\n- item 2\n  - nested";
        let (bytes, _) = DocxExporter::export_to_bytes("List", md).unwrap();
        let xml = extract_document_xml(&bytes);
        assert!(xml.contains("item 1"));
        assert!(xml.contains("item 2"));
        assert!(xml.contains("nested"));
        assert!(xml.contains("ListParagraph"));
    }

    #[test]
    fn test_export_table() {
        let md = "| Name | Value |\n|------|-------|\n| A | B |\n| C | D |";
        let (bytes, _) = DocxExporter::export_to_bytes("Table", md).unwrap();
        let xml = extract_document_xml(&bytes);
        assert!(xml.contains("<w:tbl>"));
        assert!(xml.contains("<w:tblBorders>"));
        assert!(xml.contains("Name"));
        assert!(xml.contains("Value"));
        assert!(xml.contains("A"));
        assert!(xml.contains("B"));
    }

    #[test]
    fn test_export_horizontal_rule() {
        let md = "before\n\n---\n\nafter";
        let (bytes, _) = DocxExporter::export_to_bytes("HR", md).unwrap();
        let xml = extract_document_xml(&bytes);
        assert!(xml.contains("<w:pBdr>"));
        assert!(xml.contains("before"));
        assert!(xml.contains("after"));
    }

    #[test]
    fn test_export_xml_escaping() {
        let md = "Text with <special> & \"quotes\" 'apostrophes'";
        let (bytes, _) = DocxExporter::export_to_bytes("Escape", md).unwrap();
        let xml = extract_document_xml(&bytes);
        assert!(!xml.contains("<special>"));
        assert!(xml.contains("&lt;special&gt;"));
        assert!(xml.contains("&amp;"));
        assert!(xml.contains("&quot;"));
        assert!(xml.contains("&apos;"));
    }

    #[test]
    fn test_export_page_size_a4() {
        let options = DocxExportOptions {
            include_metadata: true,
            page_size: PageSize::A4,
        };
        let (bytes, _) =
            DocxExporter::export_to_bytes_with_options("A4", "content", &options).unwrap();
        let xml = extract_document_xml(&bytes);
        assert!(xml.contains("w:w=\"11908\""));
        assert!(xml.contains("w:h=\"16833\""));
    }

    #[test]
    fn test_export_page_size_letter() {
        let options = DocxExportOptions {
            include_metadata: true,
            page_size: PageSize::Letter,
        };
        let (bytes, _) =
            DocxExporter::export_to_bytes_with_options("Letter", "content", &options).unwrap();
        let xml = extract_document_xml(&bytes);
        assert!(xml.contains("w:w=\"12240\""));
        assert!(xml.contains("w:h=\"15840\""));
    }

    #[test]
    fn test_export_batch() {
        let docs = vec![
            ("Doc One", "# First\n\nContent 1"),
            ("Doc Two", "# Second\n\nContent 2"),
        ];
        let (bytes, summary) = DocxExporter::export_batch_to_bytes(&docs).unwrap();
        assert_eq!(summary.exported, 2);
        assert!(summary.file_size_bytes.unwrap() > 0);

        verify_zip_structure(&bytes);
    }

    #[test]
    fn test_export_roundtrip_with_import() {
        let original = "# Title\n\nA paragraph with **bold** text.\n\n- list item 1\n- list item 2";
        let (docx_bytes, _) = DocxExporter::export_to_bytes("Roundtrip", original).unwrap();

        let (imported, _) =
            crate::docx_import::DocxImporter::import_from_bytes(&docx_bytes).unwrap();
        assert_eq!(imported.len(), 1);
        assert!(imported[0].content.contains("Title"));
        assert!(imported[0].content.contains("A paragraph"));
        assert!(imported[0].content.contains("list item 1"));
    }

    fn verify_zip_structure(bytes: &[u8]) {
        let cursor = Cursor::new(bytes);
        let mut archive = zip::ZipArchive::new(cursor).unwrap();
        assert!(
            archive.by_name("[Content_Types].xml").is_ok(),
            "Missing [Content_Types].xml"
        );
        assert!(
            archive.by_name("_rels/.rels").is_ok(),
            "Missing _rels/.rels"
        );
        assert!(
            archive.by_name("word/document.xml").is_ok(),
            "Missing word/document.xml"
        );
        assert!(
            archive.by_name("word/_rels/document.xml.rels").is_ok(),
            "Missing word/_rels/document.xml.rels"
        );
    }

    fn extract_document_xml(bytes: &[u8]) -> String {
        let cursor = Cursor::new(bytes);
        let mut archive = zip::ZipArchive::new(cursor).unwrap();
        let mut file = archive.by_name("word/document.xml").unwrap();
        let mut xml = String::new();
        file.read_to_string(&mut xml).unwrap();
        xml
    }

    #[test]
    fn test_xml_escape() {
        assert_eq!(xml_escape("a < b & c > d"), "a &lt; b &amp; c &gt; d");
        assert_eq!(xml_escape("quote\"test"), "quote&quot;test");
        assert_eq!(xml_escape("apos'test"), "apos&apos;test");
    }

    #[test]
    fn test_parse_heading() {
        assert_eq!(
            parse_heading("# Hello"),
            Some(MdBlock::Heading {
                level: 1,
                text: "Hello".to_string()
            })
        );
        assert_eq!(
            parse_heading("## World"),
            Some(MdBlock::Heading {
                level: 2,
                text: "World".to_string()
            })
        );
        assert_eq!(parse_heading("Not a heading"), None);
        assert_eq!(parse_heading("#"), None);
        assert_eq!(
            parse_heading("###### Deep"),
            Some(MdBlock::Heading {
                level: 6,
                text: "Deep".to_string()
            })
        );
    }

    #[test]
    fn test_parse_unordered_item() {
        assert_eq!(
            parse_unordered_item("- item"),
            Some(MdBlock::UnorderedItem {
                indent: 0,
                text: "item".to_string()
            })
        );
        assert_eq!(
            parse_unordered_item("* item"),
            Some(MdBlock::UnorderedItem {
                indent: 0,
                text: "item".to_string()
            })
        );
        assert_eq!(
            parse_unordered_item("  - nested"),
            Some(MdBlock::UnorderedItem {
                indent: 1,
                text: "nested".to_string()
            })
        );
    }

    #[test]
    fn test_parse_table() {
        let lines = vec![
            "| A | B |".to_string(),
            "|---|---|".to_string(),
            "| 1 | 2 |".to_string(),
        ];
        let table = parse_table(&lines).unwrap();
        if let MdBlock::Table { headers, rows } = table {
            assert_eq!(headers, vec!["A", "B"]);
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0], vec!["1", "2"]);
        } else {
            panic!("Expected table block");
        }
    }

    #[test]
    fn test_page_size_values() {
        assert_eq!(PageSize::A4.width_twips(), 11908);
        assert_eq!(PageSize::Letter.width_twips(), 12240);
    }
}
