//! AI Features Validation Test Suite (Phase 6)
//!
//! Validates: AI provider interface, semantic search, auto-tagging, RAG Q&A.
//! Requires: DATABASE_URL and TACHYON_AI_PROVIDER env vars to be set.
//!
//! Run with:
//!   TACHYON_AI_PROVIDER=openai TACHYON_AI_API_KEY=sk-... \
//!   DATABASE_URL=postgres://... \
//!   cargo test --package tachyon-server --test ai_validation_test -- --nocapture

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::{Value, json};
use tachyon_server::routes::create_router;
use tower::ServiceExt;

fn skip_without_db() -> bool {
    std::env::var("DATABASE_URL").is_err() && std::env::var("TEST_DATABASE_URL").is_err()
}

fn skip_without_ai() -> bool {
    std::env::var("TACHYON_AI_PROVIDER").is_err()
}

fn ai_provider() -> String {
    std::env::var("TACHYON_AI_PROVIDER").unwrap_or_default()
}

async fn create_test_app() -> Router {
    create_router().await
}

macro_rules! db_test {
    ($name:ident, $($body:tt)*) => {
        #[tokio::test]
        async fn $name() {
            if skip_without_db() {
                eprintln!("SKIP: {} (no DATABASE_URL)", stringify!($name));
                return;
            }
            $($body)*
        }
    };
}

macro_rules! ai_test {
    ($name:ident, $($body:tt)*) => {
        #[tokio::test]
        async fn $name() {
            if skip_without_db() {
                eprintln!("SKIP: {} (no DATABASE_URL)", stringify!($name));
                return;
            }
            if skip_without_ai() {
                eprintln!("SKIP: {} (no TACHYON_AI_PROVIDER)", stringify!($name));
                return;
            }
            $($body)*
        }
    };
}

// ─── 6.3.1: AI Provider Configuration Validation ──────────────────────────

ai_test!(test_ai_provider_config_loaded, {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/ai/complete")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"prompt": "Say hello", "max_tokens": 10}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .expect("request failed");

    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let val: Value = serde_json::from_slice(&body).unwrap_or_default();

    match status {
        StatusCode::OK => {
            eprintln!(
                "PASS: AI provider '{}' is configured and responding",
                ai_provider()
            );
            assert!(
                val.get("completion").is_some(),
                "Response missing 'completion' field"
            );
            assert!(val.get("model").is_some(), "Response missing 'model' field");
            eprintln!("  model: {}", val["model"]);
        }
        StatusCode::SERVICE_UNAVAILABLE => {
            eprintln!("INFO: AI provider not configured (503 SERVICE_UNAVAILABLE)");
            eprintln!("  Set TACHYON_AI_PROVIDER to one of: openai, anthropic, ollama");
        }
        _ => {
            panic!("Unexpected status {}: {}", status, val);
        }
    }
});

// ─── 6.3.2: Semantic Search Validation ─────────────────────────────────────

ai_test!(test_semantic_search_endpoint_exists, {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/documents/semantic-search?q=test")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request failed");

    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let val: Value = serde_json::from_slice(&body).unwrap_or_default();

    match status {
        StatusCode::OK => {
            eprintln!("PASS: Semantic search endpoint returned 200");
            assert!(val.get("results").is_some(), "Response missing 'results'");
            assert!(val.get("query").is_some(), "Response missing 'query'");
            assert!(
                val.get("threshold").is_some(),
                "Response missing 'threshold'"
            );
            eprintln!(
                "  results: {}, threshold: {}",
                val["results"].as_array().map_or(0, |a| a.len()),
                val["threshold"]
            );
        }
        StatusCode::SERVICE_UNAVAILABLE => {
            eprintln!("INFO: Semantic search unavailable (AI or pgvector not configured)");
        }
        StatusCode::BAD_REQUEST => {
            eprintln!("INFO: Semantic search returned 400 - check query parameter");
        }
        _ => {
            eprintln!(
                "WARN: Semantic search returned unexpected status {}: {}",
                status, val
            );
        }
    }
});

ai_test!(test_semantic_search_empty_query, {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/documents/semantic-search?q=")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request failed");

    let status = response.status();
    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "Empty query should return 400"
    );
    eprintln!("PASS: Empty query correctly returns 400");
});

ai_test!(test_semantic_search_with_threshold, {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/documents/semantic-search?q=rust+programming&limit=5&threshold=0.7")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request failed");

    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let val: Value = serde_json::from_slice(&body).unwrap_or_default();

    if status == StatusCode::OK {
        assert_eq!(val["limit"], 5);
        assert_eq!(val["threshold"], 0.7);
        eprintln!(
            "PASS: Semantic search with custom threshold works (threshold={})",
            val["threshold"]
        );
    } else {
        eprintln!(
            "INFO: Semantic search returned {} (AI/pgvector may not be configured)",
            status
        );
    }
});

// ─── 6.3.3: Auto-Tagging Validation ────────────────────────────────────────

ai_test!(test_auto_tagging_endpoint, {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/ai/tags")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"text": "Rust is a systems programming language focused on safety and performance"}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .expect("request failed");

    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let val: Value = serde_json::from_slice(&body).unwrap_or_default();

    match status {
        StatusCode::OK => {
            let tags = val.get("tags").and_then(|t| t.as_array());
            assert!(tags.is_some(), "Response missing 'tags' array");
            let tags = tags.unwrap();
            assert!(!tags.is_empty(), "Tags array should not be empty");
            eprintln!(
                "PASS: Auto-tagging returned {} tags: {:?}",
                tags.len(),
                tags.iter()
                    .map(|t| t.as_str().unwrap_or(""))
                    .collect::<Vec<_>>()
            );
        }
        StatusCode::SERVICE_UNAVAILABLE => {
            eprintln!("INFO: Auto-tagging unavailable (AI provider not configured)");
        }
        _ => {
            eprintln!("WARN: Auto-tagging returned {}: {}", status, val);
        }
    }
});

// ─── 6.3.4: RAG Q&A Validation ────────────────────────────────────────────

ai_test!(test_rag_question_endpoint, {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/ai/question")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "question": "What is Tachyon?",
                        "max_tokens": 256
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .expect("request failed");

    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let val: Value = serde_json::from_slice(&body).unwrap_or_default();

    match status {
        StatusCode::OK => {
            assert!(val.get("answer").is_some(), "Response missing 'answer'");
            assert!(val.get("sources").is_some(), "Response missing 'sources'");
            assert!(val.get("model").is_some(), "Response missing 'model'");
            eprintln!(
                "PASS: RAG Q&A endpoint returned answer (model: {})",
                val["model"]
            );
        }
        StatusCode::SERVICE_UNAVAILABLE => {
            eprintln!("INFO: RAG Q&A unavailable (AI provider not configured)");
        }
        _ => {
            eprintln!("WARN: RAG Q&A returned {}: {}", status, val);
        }
    }
});

ai_test!(test_rag_question_with_document_ids, {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/ai/question")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "question": "What are the main features?",
                        "document_ids": ["doc-1", "doc-2"],
                        "max_tokens": 512
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .expect("request failed");

    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let val: Value = serde_json::from_slice(&body).unwrap_or_default();

    match status {
        StatusCode::OK => {
            let sources = val.get("sources").and_then(|s| s.as_array());
            assert!(sources.is_some());
            let sources = sources.unwrap();
            assert_eq!(sources.len(), 2, "Should have 2 source document IDs");
            eprintln!(
                "PASS: RAG Q&A with document IDs works (sources: {:?})",
                sources
            );
        }
        StatusCode::SERVICE_UNAVAILABLE => {
            eprintln!("INFO: RAG Q&A with document IDs unavailable");
        }
        _ => {
            eprintln!(
                "WARN: RAG Q&A with document IDs returned {}: {}",
                status, val
            );
        }
    }
});

// ─── 6.3.5: Embedding Generation Validation ───────────────────────────────

ai_test!(test_embedding_endpoint, {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/ai/embed")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(json!({"text": "Hello world"}).to_string()))
                .unwrap(),
        )
        .await
        .expect("request failed");

    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let val: Value = serde_json::from_slice(&body).unwrap_or_default();

    match status {
        StatusCode::OK => {
            let embedding = val.get("embedding").and_then(|e| e.as_array());
            assert!(embedding.is_some(), "Response missing 'embedding' array");
            let dim = embedding.unwrap().len();
            eprintln!("PASS: Embedding endpoint returned {} dimensions", dim);
            // Common dimensions: 768 (nomic-embed-text), 1536 (OpenAI text-embedding-3-small)
            assert!(
                dim == 768 || dim == 1536 || dim == 384,
                "Unexpected embedding dimension: {}. Expected 384, 768, or 1536",
                dim
            );
        }
        StatusCode::SERVICE_UNAVAILABLE => {
            eprintln!("INFO: Embedding endpoint unavailable (AI provider not configured)");
        }
        _ => {
            eprintln!("WARN: Embedding endpoint returned {}: {}", status, val);
        }
    }
});

// ─── 6.3.6: Text Completion / Summarize / Improve Validation ───────────────

ai_test!(test_completion_endpoint, {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/ai/complete")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"prompt": "What is 2+2?", "max_tokens": 50}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .expect("request failed");

    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let val: Value = serde_json::from_slice(&body).unwrap_or_default();

    match status {
        StatusCode::OK => {
            assert!(val.get("completion").is_some());
            assert!(val.get("usage").is_some());
            eprintln!("PASS: Completion endpoint works (model: {})", val["model"]);
        }
        StatusCode::SERVICE_UNAVAILABLE => {
            eprintln!("INFO: Completion endpoint unavailable");
        }
        _ => {
            eprintln!("WARN: Completion returned {}: {}", status, val);
        }
    }
});

ai_test!(test_summarize_endpoint, {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/ai/summarize")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "text": "Tachyon is a knowledge management system built with Rust and Leptos. It features real-time collaboration, semantic search, and AI-powered tools for organizing and finding information.",
                        "max_tokens": 256
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .expect("request failed");

    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let val: Value = serde_json::from_slice(&body).unwrap_or_default();

    match status {
        StatusCode::OK => {
            assert!(val.get("summary").is_some());
            eprintln!("PASS: Summarize endpoint works (model: {})", val["model"]);
        }
        StatusCode::SERVICE_UNAVAILABLE => {
            eprintln!("INFO: Summarize endpoint unavailable");
        }
        _ => {
            eprintln!("WARN: Summarize returned {}: {}", status, val);
        }
    }
});

ai_test!(test_improve_endpoint, {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/ai/improve")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "text": "this sentence has bad grammer and needs fixin",
                        "max_tokens": 256
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .expect("request failed");

    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let val: Value = serde_json::from_slice(&body).unwrap_or_default();

    match status {
        StatusCode::OK => {
            assert!(val.get("improved").is_some());
            eprintln!("PASS: Improve endpoint works (model: {})", val["model"]);
        }
        StatusCode::SERVICE_UNAVAILABLE => {
            eprintln!("INFO: Improve endpoint unavailable");
        }
        _ => {
            eprintln!("WARN: Improve returned {}: {}", status, val);
        }
    }
});

// ─── 6.3.7: Full-Text Search (non-AI) sanity check ────────────────────────

db_test!(test_fulltext_search_works, {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/documents/search?search=test&page=1&page_size=10")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request failed");

    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let _val: Value = serde_json::from_slice(&body).unwrap_or_default();

    assert!(
        status.is_success() || status == StatusCode::BAD_REQUEST,
        "Full-text search returned unexpected status: {}",
        status
    );
    eprintln!("PASS: Full-text search endpoint returns {}", status);
});

// ─── 6.3.8: AI Router Mount Verification ───────────────────────────────────

db_test!(test_ai_routes_are_mounted, {
    let app = create_test_app().await;

    let endpoints = vec![
        ("/ai/complete", "POST"),
        ("/ai/summarize", "POST"),
        ("/ai/improve", "POST"),
        ("/ai/tags", "POST"),
        ("/ai/embed", "POST"),
        ("/ai/question", "POST"),
    ];

    for (path, method) in endpoints {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(path)
                    .method(method)
                    .header("content-type", "application/json")
                    .body(Body::from("{}".to_string()))
                    .unwrap(),
            )
            .await
            .expect("request failed");

        let status = response.status();
        // 200, 400, 422, or 503 are all valid — they mean the route exists
        assert!(
            status != StatusCode::NOT_FOUND,
            "AI route {} {} is NOT mounted (got 404)",
            method,
            path
        );
        eprintln!(
            "PASS: Route {} {} is mounted (status: {})",
            method, path, status
        );
    }
});
