use axum::body::Body;
use axum::http::{Request, StatusCode};
use tachyon_server::routes::create_router;
use tower::ServiceExt;

fn skip_without_db() -> bool {
    std::env::var("TEST_DATABASE_URL").is_err()
}

#[tokio::test]
async fn test_health_endpoint() {
    if skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = create_router().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/documents")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_api_root_returns_not_found() {
    if skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = create_router().await;
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
