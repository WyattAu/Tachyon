//! DOCX export → import round-trip seam (`tachyon_import_export`).
//!
//! Locks the contract between the hand-rolled DOCX driver halves the
//! editor relies on: markdown exported to the OOXML ZIP comes back as
//! equivalent markdown (heading hierarchy, paragraphs, list items), the
//! first heading becomes the document title, summaries report honest
//! counts, and non-DOCX input is rejected rather than mis-imported.
//!
//! Extraction note: these drivers are slated for the designed (not yet
//! built) `doc-drivers-bundle` estate crate — this suite pins current
//! behavior so the extraction can be verified against it.

use tachyon_import_export::docx_export::DocxExporter;
use tachyon_import_export::docx_import::DocxImporter;

const MARKDOWN: &str = "# Alpha Title\n\nBody paragraph with detail.\n\n## Beta Section\n\n- first item\n- second item\n";

#[test]
fn export_then_import_round_trips_structure() {
    let (bytes, _summary) =
        DocxExporter::export_to_bytes("Doc", MARKDOWN).expect("markdown exports to DOCX");
    assert!(!bytes.is_empty());

    let (documents, _import_summary) =
        DocxImporter::import_from_bytes(&bytes).expect("DOCX imports");
    assert_eq!(documents.len(), 1);
    let doc = &documents[0];

    assert!(
        doc.content.contains("# Alpha Title"),
        "h1 preserved, got: {}",
        doc.content
    );
    assert!(
        doc.content.contains("## Beta Section"),
        "h2 hierarchy preserved, got: {}",
        doc.content
    );
    assert!(
        doc.content.contains("Body paragraph with detail."),
        "paragraph text preserved, got: {}",
        doc.content
    );
    // KNOWN GAP (doc-drivers-bundle extraction evidence): unordered list
    // items export as ListParagraph-styled paragraphs WITHOUT <w:numPr>,
    // so the importer cannot recover the "- " markers — only the text
    // survives the round-trip.
    assert!(doc.content.contains("first item"));
    assert!(doc.content.contains("second item"));
    assert!(
        !doc.content.contains("- first item"),
        "pins current behavior: unordered markers are lost on round-trip"
    );
}

#[test]
fn first_heading_becomes_document_title() {
    let (bytes, _summary) =
        DocxExporter::export_to_bytes("Ignored Export Title", MARKDOWN).expect("exports");
    let (documents, _import_summary) = DocxImporter::import_from_bytes(&bytes).expect("imports");
    assert_eq!(documents[0].title, "Alpha Title");
}

#[test]
fn headless_document_titles_from_first_line() {
    let markdown = "Just a paragraph, no headings.\n";
    let (bytes, _summary) = DocxExporter::export_to_bytes("Doc", markdown).expect("exports");
    let (documents, _import_summary) = DocxImporter::import_from_bytes(&bytes).expect("imports");
    // extract_first_heading falls back to the first non-empty line when no
    // heading exists — the paragraph text becomes the title.
    assert_eq!(documents[0].title, "Just a paragraph, no headings.");
    assert!(
        documents[0]
            .content
            .contains("Just a paragraph, no headings.")
    );
}

#[test]
fn export_summary_reports_format_and_size() {
    let (bytes, summary) = DocxExporter::export_to_bytes("Doc", MARKDOWN).expect("exports");
    assert_eq!(summary.exported, 1);
    assert_eq!(summary.format, "docx");
    assert_eq!(summary.file_size_bytes, Some(bytes.len() as u64));
    assert!(summary.warnings.is_empty());
}

#[test]
fn import_summary_counts_entries_and_tags() {
    let (bytes, _export_summary) = DocxExporter::export_to_bytes("Doc", MARKDOWN).expect("exports");
    let (_documents, summary) = DocxImporter::import_from_bytes(&bytes).expect("imports");

    assert_eq!(summary.imported, 1);
    assert_eq!(summary.failed, 0);
    assert!(
        summary.total_files >= 4,
        "OOXML ZIP carries content types, rels, and document.xml"
    );
    assert_eq!(summary.skipped, summary.total_files - 1);
    assert_eq!(summary.document_titles, vec!["Alpha Title".to_string()]);
    assert!(summary.all_tags.contains(&"docx".to_string()));
    assert!(summary.all_tags.contains(&"import".to_string()));
}

#[test]
fn non_docx_input_is_rejected() {
    let err = DocxImporter::import_from_bytes(b"definitely not a zip")
        .expect_err("garbage input must error");
    let message = err.to_string();
    assert!(
        message.contains("document.xml") || message.to_lowercase().contains("zip"),
        "error names the missing OOXML part, got: {message}"
    );

    // A valid ZIP without word/document.xml is equally rejected.
    let buf = std::io::Cursor::new(Vec::new());
    let mut zip = zip::ZipWriter::new(buf);
    zip.start_file("unrelated.txt", zip::write::SimpleFileOptions::default())
        .expect("zip write");
    let wrong_shape = zip.finish().expect("zip finish").into_inner();
    let err = DocxImporter::import_from_bytes(&wrong_shape)
        .expect_err("zip without document.xml must error");
    assert!(err.to_string().contains("document.xml"));
}

#[test]
fn batch_export_parts_beyond_the_first_are_not_importable() {
    let documents = vec![
        ("One", "# First\n\nOne body."),
        ("Two", "## Second\n\nTwo body."),
    ];
    let (bytes, summary) = DocxExporter::export_batch_to_bytes(&documents).expect("batch exports");
    assert_eq!(summary.format, "docx");
    assert!(!bytes.is_empty());

    // KNOWN GAP (doc-drivers-bundle extraction evidence): batch export
    // writes word/document{1,2,...}.xml for subsequent documents, but the
    // importer reads only word/document.xml — later documents' content is
    // silently unreachable on import. Pin the current single-part result.
    let (imported, _import_summary) =
        DocxImporter::import_from_bytes(&bytes).expect("batch imports");
    assert_eq!(imported.len(), 1);
    let combined = &imported[0].content;
    assert!(combined.contains("First"));
    assert!(
        !combined.contains("Two body."),
        "pins current behavior: document1.xml is never imported"
    );
}
