use chrono::Utc;
use criterion::{Criterion, black_box, criterion_group, criterion_main};
use std::collections::BTreeMap;
use tachyon_core::id::{DocumentId, UserId};
use tachyon_search::types::SearchDocument;

fn generate_search_doc(i: usize) -> SearchDocument {
    SearchDocument {
        id: DocumentId::new(),
        title: format!("Benchmark Document {}", i),
        content: format!(
            "This is the content of benchmark document {}. It contains some searchable text about topic {} \
             with keywords like performance, testing, benchmark, and evaluation. \
             The quick brown fox jumps over the lazy dog in paragraph {}.",
            i,
            i % 10,
            i
        ),
        author_id: UserId::new(),
        repository_id: None,
        tags: vec!["benchmark".to_string(), format!("topic-{}", i % 10)],
        created_at: Utc::now(),
        updated_at: Utc::now(),
        custom_fields: BTreeMap::new(),
    }
}

fn bench_search_document_creation(c: &mut Criterion) {
    c.bench_function("search_document_creation", |b| {
        b.iter(|| {
            black_box(generate_search_doc(black_box(42)));
        });
    });
}

fn bench_search_document_serialization(c: &mut Criterion) {
    let doc = generate_search_doc(0);
    c.bench_function("search_document_serialization", |b| {
        b.iter(|| {
            black_box(serde_json::to_string(&doc).unwrap());
        });
    });
}

fn bench_wikilink_extraction_search(c: &mut Criterion) {
    let content =
        "Check [[Document A]] and [[Document B|the second doc]] for details about [[C]].\n";
    c.bench_function("search_extract_wikilinks", |b| {
        b.iter(|| {
            tachyon_renderer::MarkdownParser::extract_wikilinks(black_box(content));
        });
    });
}

criterion_group!(
    benches,
    bench_search_document_creation,
    bench_search_document_serialization,
    bench_wikilink_extraction_search,
);
criterion_main!(benches);
