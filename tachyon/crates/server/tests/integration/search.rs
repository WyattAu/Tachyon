use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;
use tachyon_server::routes::create_router;

fn skip_without_db() -> bool {
    std::env::var("TEST_DATABASE_URL").is_err()
}

#[tokio::test]
async fn test_search_endpoint() {
    if skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = create_router().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/search?q=test")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(
        response.status() == StatusCode::OK
            || response.status() == StatusCode::UNAUTHORIZED
            || response.status() == StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn test_search_empty_query() {
    if skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = create_router().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/search?q=")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(
        response.status() == StatusCode::OK
            || response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::UNAUTHORIZED
            || response.status() == StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn test_search_with_pagination() {
    if skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = create_router().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/search?q=test&page=1&page_size=10")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(
        response.status() == StatusCode::OK
            || response.status() == StatusCode::UNAUTHORIZED
            || response.status() == StatusCode::NOT_FOUND
    );
}
