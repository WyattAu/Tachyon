use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::collections::BTreeMap;
use tachyon_core::{
    compute_content_hash, generate_document_id, generate_user_id, slugify, Document,
    DocumentContent, DocumentId, DocumentVisibility, UserId,
};
use tachyon_renderer::{RenderConfig, Renderer};
use tachyon_search::{
    IndexManager, QueryEngine, ResultAggregator, SearchDocument, SearchRequest, SearchResponseItem,
};
use tempfile::TempDir;

fn generate_document_markdown(word_count: usize) -> String {
    let mut md = String::from("# Benchmark Document\n\n");
    md.push_str("An introductory paragraph for benchmarking purposes.\n\n");
    let mut words = 0;
    let mut section = 0;
    while words < word_count {
        section += 1;
        md.push_str(&format!("## Section {}\n\n", section));
        md.push_str("**Bold** and *italic* inline formatting test. `code span` included.\n\n");
        md.push_str("- List item one\n- List item two\n- List item three\n\n");
        md.push_str("| Col A | Col B |\n|-------|-------|\n| a | b |\n\n");
        md.push_str("```rust\nfn main() { println!(\"hello\"); }\n```\n\n");
        let para = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. \
                    Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. \
                    Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris. ";
        while words < word_count && words < (section * 100) {
            md.push_str(para);
            words += 30;
        }
        md.push_str("\n");
    }
    md
}

fn bench_health_check_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("api/health_check");

    group.bench_function("serialize_response", |b| {
        let response = serde_json::json!({
            "status": "healthy",
            "version": "0.1.0",
            "uptime_secs": 3600,
            "checks": {
                "database": { "status": "ok", "latency_ms": 2, "error": null },
                "redis": { "status": "disabled", "latency_ms": null, "error": null },
                "tantivy": { "status": "ok", "latency_ms": 5, "error": null },
                "smtp": { "status": "disabled", "latency_ms": null, "error": null },
            },
            "memory": { "rss_bytes": 52428800, "rss_mb": 50 },
        });
        b.iter(|| {
            let _ = black_box(serde_json::to_string(&response).unwrap());
        });
    });

    group.bench_function("deserialize_response", |b| {
        let raw = serde_json::json!({
            "status": "healthy",
            "version": "0.1.0",
            "uptime_secs": 3600,
            "checks": {
                "database": { "status": "ok", "latency_ms": 2, "error": null },
                "redis": { "status": "disabled", "latency_ms": null, "error": null },
                "tantivy": { "status": "ok", "latency_ms": 5, "error": null },
                "smtp": { "status": "disabled", "latency_ms": null, "error": null },
            },
            "memory": { "rss_bytes": 52428800, "rss_mb": 50 },
        });
        let serialized = serde_json::to_string(&raw).unwrap();
        b.iter(|| {
            let _: serde_json::Value = serde_json::from_str(black_box(&serialized)).unwrap();
        });
    });

    group.finish();
}

fn bench_document_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("api/document_creation");

    group.bench_function("construct_document", |b| {
        b.iter(|| {
            let id = generate_document_id();
            let author_id = generate_user_id();
            let content =
                DocumentContent::markdown("# Test Document\n\nContent here.\n".to_string());
            let doc = Document::new(
                id,
                "Benchmark Document Title".to_string(),
                author_id,
                content,
            );
            black_box(&doc);
        });
    });

    group.bench_function("construct_document_with_tags", |b| {
        b.iter(|| {
            let id = generate_document_id();
            let author_id = generate_user_id();
            let content = DocumentContent::markdown("# Test\n\nBody.\n".to_string());
            let mut doc = Document::new(id, "Tagged Document".to_string(), author_id, content);
            doc.metadata.add_tag("rust".to_string()).unwrap();
            doc.metadata.add_tag("benchmark".to_string()).unwrap();
            doc.metadata.add_tag("performance".to_string()).unwrap();
            doc.visibility = DocumentVisibility::Public;
            black_box(&doc);
        });
    });

    for wc in [100usize, 500, 2000] {
        group.bench_with_input(
            BenchmarkId::new("construct_large_document", wc),
            &wc,
            |b, &wc| {
                let md = generate_document_markdown(wc);
                b.iter(|| {
                    let id = generate_document_id();
                    let author_id = generate_user_id();
                    let content = DocumentContent::markdown(md.clone());
                    let doc = Document::new(id, "Large Document".to_string(), author_id, content);
                    black_box(&doc);
                });
            },
        );
    }

    group.bench_function("compute_content_hash_1kb", |b| {
        let content = "a".repeat(1000);
        b.iter(|| black_box(compute_content_hash(&content)));
    });

    group.bench_function("compute_content_hash_10kb", |b| {
        let content = "a".repeat(10_000);
        b.iter(|| black_box(compute_content_hash(&content)));
    });

    group.bench_function("slugify_title", |b| {
        b.iter(|| black_box(slugify("My Awesome Document Title (2025 Edition)")));
    });

    group.finish();
}

fn bench_document_listing(c: &mut Criterion) {
    let mut group = c.benchmark_group("api/document_listing");

    for count in [20usize, 100, 500] {
        let responses: Vec<serde_json::Value> = (0..count)
            .map(|i| {
                serde_json::json!({
                    "id": format!("doc-{}", i),
                    "title": format!("Document {}", i),
                    "slug": format!("document-{}", i),
                    "status": "published",
                    "visibility": "public",
                    "tags": ["rust", "benchmark"],
                    "author_id": "user-1",
                    "word_count": 500,
                    "character_count": 3000,
                    "created_at": "2025-01-01T00:00:00Z",
                    "updated_at": "2025-01-01T00:00:00Z",
                })
            })
            .collect();

        group.bench_with_input(
            BenchmarkId::new("serialize_list_response", count),
            &responses,
            |b, responses| {
                let list_response = serde_json::json!({
                    "results": responses,
                    "total": responses.len(),
                    "page": 1,
                    "page_size": responses.len(),
                });
                b.iter(|| {
                    let _ = black_box(serde_json::to_string(&list_response).unwrap());
                });
            },
        );
    }

    group.finish();
}

fn bench_markdown_rendering(c: &mut Criterion) {
    let renderer = Renderer::new(RenderConfig::default());
    let mut group = c.benchmark_group("api/markdown_rendering");

    let small = "# Title\n\nParagraph with **bold** and *italic* text.\n\n- item 1\n- item 2\n";
    group.bench_function("small_50_words", |b| {
        b.iter(|| renderer.render(black_box(small), None));
    });

    let medium = generate_document_markdown(500);
    group.bench_function("medium_500_words", |b| {
        b.iter(|| renderer.render(black_box(&medium), None));
    });

    let large = generate_document_markdown(2000);
    group.bench_function("large_2000_words", |b| {
        b.iter(|| renderer.render(black_box(&large), None));
    });

    group.finish();
}

fn bench_search_query(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("api/search_query");
    group.measurement_time(std::time::Duration::from_secs(30));

    let doc_count = 1000;
    let documents: Vec<SearchDocument> = (0..doc_count)
        .map(|i| SearchDocument {
            id: DocumentId::new(),
            title: format!(
                "Document {} with a descriptive title for search indexing",
                i
            ),
            content: format!(
                "This is the content of document {}. It covers topics including \
                 Rust programming, web development, databases, knowledge management, \
                 and full-text search performance benchmarking across different query patterns.",
                i
            ),
            author_id: UserId::new(),
            repository_id: Some(tachyon_core::id::RepositoryId::new()),
            tags: vec!["benchmark".to_string(), format!("tag-{}", i % 10)],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            custom_fields: BTreeMap::new(),
        })
        .collect();

    let temp_dir = TempDir::new().unwrap();
    let index_path = temp_dir.path().to_path_buf();
    let index_manager = rt.block_on(async {
        let mgr = IndexManager::new(index_path).await.unwrap();
        mgr.batch_index(&documents).await.unwrap();
        mgr
    });

    let queries = [
        ("single_term", "Rust"),
        ("multi_term", "web development databases"),
        ("phrase_match", "full-text search performance"),
        ("tag_filter", "tag-5"),
    ];

    for (name, query_str) in &queries {
        group.bench_with_input(
            BenchmarkId::new("tantivy_search", name),
            query_str,
            |b, query_str| {
                b.to_async(&rt).iter(|| async {
                    let engine = QueryEngine::new(index_manager.clone());
                    let request = SearchRequest::new(query_str).with_pagination(1, 20);
                    let _ = black_box(engine.search(&request).await);
                });
            },
        );
    }

    group.bench_function("result_aggregation_100", |b| {
        let items: Vec<SearchResponseItem> = (0..100)
            .map(|i| SearchResponseItem {
                document_id: DocumentId::new(),
                title: format!("Result {}", i),
                snippet: format!("Snippet for result {}", i),
                score: 1.0 - (i as f32) * 0.01,
                highlights: vec![format!("highlight {}", i)],
                author_id: UserId::new(),
                repository_id: None,
                tags: vec!["tag".to_string()],
                created_at: chrono::Utc::now(),
            })
            .collect();

        let aggregator = ResultAggregator::default();
        b.iter(|| {
            let _ = black_box(aggregator.fuse_results(vec![items.clone()]));
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_health_check_serialization,
    bench_document_creation,
    bench_document_listing,
    bench_markdown_rendering,
    bench_search_query,
);
criterion_main!(benches);
