use criterion::{criterion_group, criterion_main, Criterion};

fn bench_health_check(c: &mut Criterion) {
    c.bench_function("health_check_parse", |b| {
        b.iter(|| {
            let json = serde_json::json!({
                "status": "ok",
                "version": "0.1.0",
                "uptime_seconds": 12345,
                "database": "connected",
                "checks": {
                    "database": "ok",
                    "redis": "ok"
                }
            });
            let serialized = serde_json::to_string(&json).unwrap();
            let _parsed: serde_json::Value = serde_json::from_str(&serialized).unwrap();
        })
    });
}

fn bench_jwt_encode_decode(c: &mut Criterion) {
    use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};

    c.bench_function("jwt_encode_decode", |b| {
        let claims = serde_json::json!({
            "sub": "user-123",
            "exp": 9999999999u64,
            "role": "admin"
        });
        let key = EncodingKey::from_secret("test-secret-key-must-be-at-least-32-chars");
        b.iter(|| {
            let token = encode(&Header::default(), &claims, &key).unwrap();
            let _ = decode::<serde_json::Value>(
                &token,
                &DecodingKey::from_secret("test-secret-key-must-be-at-least-32-chars"),
                &Validation::new(Algorithm::HS256),
            );
        })
    });
}

criterion_group!(benches, bench_health_check, bench_jwt_encode_decode);
criterion_main!(benches);
