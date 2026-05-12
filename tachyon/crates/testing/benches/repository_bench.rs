//! Performance benchmarks for repository operations
//!
//! Benchmarks repository ID and user ID generation.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use tachyon_core::{RepositoryId, UserId};

fn bench_id_generation(c: &mut Criterion) {
    c.bench_function("repository_id_generation", |b| {
        b.iter(|| {
            let _ = black_box(RepositoryId::new());
        });
    });

    c.bench_function("user_id_generation", |b| {
        b.iter(|| {
            let _ = black_box(UserId::new());
        });
    });
}

fn bench_id_operations(c: &mut Criterion) {
    let repo_id = RepositoryId::new();
    let user_id = UserId::new();

    c.bench_function("repository_id_to_string", |b| {
        b.iter(|| {
            let _ = black_box(repo_id.to_string());
        });
    });

    c.bench_function("user_id_to_string", |b| {
        b.iter(|| {
            let _ = black_box(user_id.to_string());
        });
    });
}

criterion_group!(benches, bench_id_generation, bench_id_operations);

criterion_main!(benches);
