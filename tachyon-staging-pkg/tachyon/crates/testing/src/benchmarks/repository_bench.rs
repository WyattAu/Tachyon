// Repository benchmarks - measure CRUD operation throughput.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tachyon_core::types::repository::{
    Repository, RepositoryConfig, RepositoryStatus, RepositoryType, RepositoryVisibility,
};
use tachyon_core::{generate_repository_id, generate_user_id};

fn bench_repository_create(c: &mut Criterion) {
    c.bench_function("repository_create", |b| {
        b.iter(|| {
            black_box(Repository::new(
                black_box(generate_repository_id()),
                black_box("Bench Repository".to_string()),
                black_box(RepositoryType::Personal),
                black_box(generate_user_id()),
            ))
        })
    });
}

fn bench_repository_validate(c: &mut Criterion) {
    let repo = Repository::new(
        generate_repository_id(),
        "Valid Repository Name".to_string(),
        RepositoryType::Personal,
        generate_user_id(),
    );
    c.bench_function("repository_validate", |b| {
        b.iter(|| black_box(repo.validate()))
    });
}

fn bench_repository_serialize_roundtrip(c: &mut Criterion) {
    let repo = Repository::new(
        generate_repository_id(),
        "Bench Repo".to_string(),
        RepositoryType::Team,
        generate_user_id(),
    )
    .with_status(RepositoryStatus::Synced)
    .with_visibility(RepositoryVisibility::Public);

    c.bench_function("repository_serialize", |b| {
        b.iter(|| black_box(serde_json::to_string(black_box(&repo)).unwrap()))
    });

    let json = serde_json::to_string(&repo).unwrap();
    c.bench_function("repository_deserialize", |b| {
        b.iter(|| {
            black_box(
                serde_json::from_str::<Repository>(black_box(&json)).unwrap(),
            )
        })
    });
}

fn bench_repository_config_create(c: &mut Criterion) {
    c.bench_function("repository_config_create", |b| {
        b.iter(|| {
            black_box(
                RepositoryConfig::new()
                    .with_default_branch("main".to_string())
                    .with_remote_url("https://github.com/user/repo.git".to_string())
                    .with_auto_sync(true),
            )
        })
    });
}

criterion_group!(
    benches,
    bench_repository_create,
    bench_repository_validate,
    bench_repository_serialize_roundtrip,
    bench_repository_config_create,
);
criterion_main!(benches);
