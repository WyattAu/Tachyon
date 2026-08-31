// Database benchmarks - measure connection pool and query throughput.
//
// Requires DATABASE_URL environment variable to be set.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;
use tachyon_core::{generate_document_id, generate_user_id};
use tachyon_database::{
    CreateDocumentRequest, DatabasePool, DocumentRepository, DocumentStatus, DocumentVisibility,
};
use tachyon_server::config::ServerConfig;

async fn setup_test_pool() -> DatabasePool {
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
                "postgres://tachyon:tachyon@localhost:5432/tachyon_test".to_string()
            })
        });

    PgPoolOptions::new()
        .max_connections(10)
        .min_connections(2)
        .acquire_timeout(Duration::from_secs(5))
        .idle_timeout(Duration::from_secs(600))
        .connect(&database_url)
        .await
        .expect("Failed to connect to test database for benchmarks")
}

fn bench_pool_acquire(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let pool = rt.block_on(setup_test_pool());

    c.bench_function("db_pool_acquire", |b| {
        b.iter(|| {
            rt.block_on(async {
                black_box(pool.acquire().await.ok())
            })
        })
    });
}

fn bench_document_insert_empty(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let pool = rt.block_on(setup_test_pool());
    let repo = DocumentRepository::new(pool.clone());

    c.bench_function("db_document_insert_empty", |b| {
        b.iter(|| {
            rt.block_on(async {
                let req = CreateDocumentRequest {
                    title: "bench".to_string(),
                    content: "bench content".to_string(),
                    slug: Some(format!("bench-{}", generate_document_id())),
                    tags: None,
                    status: Some(DocumentStatus::Draft),
                    visibility: Some(DocumentVisibility::Private),
                    author_id: Some(generate_user_id()),
                    parent_id: None,
                    space_id: None,
                    frontmatter: None,
                };
                black_box(repo.create(&req).await.ok())
            })
        })
    });
}

criterion_group!(benches, bench_pool_acquire, bench_document_insert_empty);
criterion_main!(benches);
