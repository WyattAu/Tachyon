use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tachyon_renderer::types::OutputFormat;
use tachyon_renderer::MarkdownParser;

fn generate_medium_document() -> String {
    let mut content = String::with_capacity(50_000);
    content.push_str("# Large Document\n\n");
    content.push_str("This is the introduction to a large document used for benchmarking.\n\n");
    for i in 1..=100 {
        content.push_str(&format!("## Section {}\n\n", i));
        content.push_str(&format!(
            "Paragraph {} with **bold** and *italic* text. Some `inline code` here.\n\n",
            i
        ));
        content.push_str(&format!(
            "- List item {}-a\n- List item {}-b\n- List item {}-c\n\n",
            i, i, i
        ));
        if i % 5 == 0 {
            content.push_str(&format!(
                "```rust\nfn example_{}() {{\n    println!(\"Hello from section {}\");\n}}\n```\n\n",
                i, i
            ));
        }
    }
    content
}

fn bench_small_document(c: &mut Criterion) {
    let content = "# Hello\n\nThis is a **test** document with *formatting*.\n\n- Item 1\n- Item 2\n- Item 3\n";
    c.bench_function("parse_html_small", |b| {
        let parser = MarkdownParser::new();
        b.iter(|| {
            parser
                .parse(black_box(content), OutputFormat::Html)
                .unwrap()
        });
    });
}

fn bench_medium_document(c: &mut Criterion) {
    let content = generate_medium_document();
    c.bench_function("parse_html_medium", |b| {
        let parser = MarkdownParser::new();
        b.iter(|| {
            parser
                .parse(black_box(&content), OutputFormat::Html)
                .unwrap()
        });
    });
}

fn bench_wikilink_extraction(c: &mut Criterion) {
    let content = "Check [[Document A]] and [[Document B|the second doc]] for details.\n";
    c.bench_function("extract_wikilinks", |b| {
        b.iter(|| MarkdownParser::extract_wikilinks(black_box(content)));
    });
}

fn bench_metadata_extraction(c: &mut Criterion) {
    let content =
        "# Title\n\nSome content here with **bold** and *italic* text.\n\nMore paragraphs.\n";
    c.bench_function("parse_with_metadata", |b| {
        let parser = MarkdownParser::new();
        b.iter(|| {
            parser
                .parse(black_box(content), OutputFormat::Html)
                .unwrap()
        });
    });
}

fn bench_plain_text(c: &mut Criterion) {
    let content = "# Hello\n\nThis is a **test** document with *formatting*.\n\n- Item 1\n- Item 2\n- Item 3\n";
    c.bench_function("parse_plain_text", |b| {
        let parser = MarkdownParser::new();
        b.iter(|| {
            parser
                .parse(black_box(content), OutputFormat::PlainText)
                .unwrap()
        });
    });
}

criterion_group!(
    benches,
    bench_small_document,
    bench_medium_document,
    bench_wikilink_extraction,
    bench_metadata_extraction,
    bench_plain_text,
);
criterion_main!(benches);
