//! Performance benchmarks for search operations
//!
//! Benchmarks search error handling and type operations.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use tachyon_core::{DocumentId, RepositoryId};

fn bench_id_operations(c: &mut Criterion) {
    c.bench_function("document_id_generation", |b| {
        b.iter(|| {
            let _ = black_box(DocumentId::new());
        });
    });

    c.bench_function("repository_id_generation", |b| {
        b.iter(|| {
            let _ = black_box(RepositoryId::new());
        });
    });
}

fn bench_search_types(c: &mut Criterion) {
    c.bench_function("search_error_creation", |b| {
        b.iter(|| {
            let _ = black_box(tachyon_search::SearchError::index(
                "TEST_ERROR",
                "test error message",
            ));
        });
    });
}

criterion_group!(benches, bench_id_operations, bench_search_types);

criterion_main!(benches);
