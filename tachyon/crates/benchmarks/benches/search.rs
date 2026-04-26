use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::collections::BTreeMap;
use tachyon_core::id::{DocumentId, RepositoryId, UserId};
use tachyon_search::{IndexManager, SearchDocument};
use tempfile::TempDir;

fn generate_documents(count: usize) -> Vec<SearchDocument> {
    (0..count)
        .map(|i| {
            SearchDocument {
                id: DocumentId::new(),
                title: format!("Document {} with a descriptive title for search indexing", i),
                content: format!(
                    "This is the content of document {}. It contains searchable text \
                     about various topics including Rust programming, web development, \
                     databases, and knowledge management systems. The content is varied \
                     to test full-text search performance across different query patterns.",
                    i
                ),
                author_id: UserId::new(),
                repository_id: Some(RepositoryId::new()),
                tags: vec!["benchmark".to_string(), format!("tag-{}", i % 10)],
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                custom_fields: BTreeMap::new(),
            }
        })
        .collect()
}

fn bench_batch_indexing(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("search_batch_indexing");

    for size in [100, 1000, 10000] {
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            b.to_async(&rt).iter(|| async {
                let temp_dir = TempDir::new().unwrap();
                let index_path = temp_dir.path().to_path_buf();
                let index_manager = IndexManager::new(index_path).await.unwrap();
                let documents = generate_documents(size);
                let _ = black_box(index_manager.batch_index(&documents).await);
            });
        });
    }
    group.finish();
}

fn bench_single_document_indexing(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("search_single_indexing");

    for size in [100, 1000, 10000] {
        group.bench_with_input(BenchmarkId::new("total_docs", size), &size, |b, &size| {
            b.to_async(&rt).iter(|| async {
                let temp_dir = TempDir::new().unwrap();
                let index_path = temp_dir.path().to_path_buf();
                let index_manager = IndexManager::new(index_path).await.unwrap();
                for i in 0..size {
                    let doc = SearchDocument::new(
                        DocumentId::new(),
                        format!("Document {}", i),
                        format!("Content for document {}", i),
                        UserId::new(),
                    );
                    let _ = black_box(index_manager.index_document(&doc).await);
                }
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_batch_indexing, bench_single_document_indexing);
criterion_main!(benches);
