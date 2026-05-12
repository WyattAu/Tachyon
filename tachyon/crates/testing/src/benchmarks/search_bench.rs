// Search benchmarks - measure query throughput.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tachyon_search::types::{FieldDefinition, FieldType, QueryType, RangeValue, SortOrder};
use tachyon_search::SearchRequest;

fn bench_search_request_create(c: &mut Criterion) {
    c.bench_function("search_request_create", |b| {
        b.iter(|| {
            black_box(SearchRequest::new(black_box("benchmark search query")))
        })
    });
}

fn bench_search_request_validate(c: &mut Criterion) {
    let req = SearchRequest::new("valid query").with_page_size(20);
    c.bench_function("search_request_validate", |b| {
        b.iter(|| black_box(req.validate()))
    });
}

fn bench_search_request_serialize(c: &mut Criterion) {
    let req = SearchRequest::new("query with filters")
        .with_page_size(25)
        .with_pagination(0, 25);
    c.bench_function("search_request_serialize", |b| {
        b.iter(|| black_box(serde_json::to_string(black_box(&req)).unwrap()))
    });

    let json = serde_json::to_string(&req).unwrap();
    c.bench_function("search_request_deserialize", |b| {
        b.iter(|| {
            black_box(
                serde_json::from_str::<SearchRequest>(black_box(&json)).unwrap(),
            )
        })
    });
}

fn bench_query_type_serialize(c: &mut Criterion) {
    let queries = vec![
        QueryType::Term {
            field: "title".to_string(),
            value: "test".to_string(),
        },
        QueryType::Phrase {
            field: "content".to_string(),
            value: "hello world".to_string(),
            slop: 0,
        },
        QueryType::Range {
            field: "date".to_string(),
            from: Some(RangeValue::DateTime("2024-01-01".to_string())),
            to: Some(RangeValue::DateTime("2024-12-31".to_string())),
        },
    ];

    c.bench_function("query_type_serialize_roundtrip", |b| {
        b.iter(|| {
            for q in &queries {
                let json = serde_json::to_string(black_box(q)).unwrap();
                let _: QueryType = serde_json::from_str(&json).unwrap();
            }
        })
    });
}

criterion_group!(
    benches,
    bench_search_request_create,
    bench_search_request_validate,
    bench_search_request_serialize,
    bench_query_type_serialize,
);
criterion_main!(benches);
