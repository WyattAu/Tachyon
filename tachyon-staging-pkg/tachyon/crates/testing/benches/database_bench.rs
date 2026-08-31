//! Performance benchmarks for database operations
//!
//! Benchmarks database error handling and type operations.

use criterion::{Criterion, black_box, criterion_group, criterion_main};

fn bench_database_types(c: &mut Criterion) {
    c.bench_function("database_error_not_found", |b| {
        b.iter(|| {
            let _ = black_box(tachyon_database::DatabaseError::not_found(
                "document", "doc123",
            ));
        });
    });

    c.bench_function("database_error_validation", |b| {
        b.iter(|| {
            let _ = black_box(tachyon_database::DatabaseError::validation_error(
                "invalid input",
            ));
        });
    });
}

criterion_group!(benches, bench_database_types);

criterion_main!(benches);
