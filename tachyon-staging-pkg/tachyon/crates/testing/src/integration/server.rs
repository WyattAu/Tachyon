//! Integration tests for server operations
//!
//! Tests using Axum's body utilities to verify the server can build
//! a router and respond to requests.

use axum::{Router, routing::get};
#[allow(unused_imports)]
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
#[allow(unused_imports)]
use http_body_util::BodyExt;
#[allow(unused_imports)]
use tower::ServiceExt;

async fn health_handler() -> &'static str {
    "OK"
}

async fn version_handler() -> &'static str {
    "1.0.0"
}

fn build_test_app() -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/version", get(version_handler))
}

#[tokio::test]
async fn test_health_endpoint() {
    let app = build_test_app();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(&body[..], b"OK");
}

#[tokio::test]
async fn test_version_endpoint() {
    let app = build_test_app();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/version")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(&body[..], b"1.0.0");
}

#[tokio::test]
async fn test_not_found_endpoint() {
    let app = build_test_app();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/nonexistent")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_method_not_allowed() {
    let app = build_test_app();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
}
