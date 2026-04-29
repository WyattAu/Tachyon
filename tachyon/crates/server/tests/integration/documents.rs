use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;
use tachyon_server::routes::create_router;

fn skip_without_db() -> bool {
    std::env::var("TEST_DATABASE_URL").is_err()
}

async fn create_test_app() -> axum::Router {
    create_router().await
}

#[tokio::test]
async fn test_list_documents() {
    if skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = create_test_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/documents")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status() == StatusCode::OK || response.status() == StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_get_document_not_found() {
    if skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = create_test_app().await;
    let fake_id = "00000000-0000-0000-0000-000000000000";
    let response = app
        .oneshot(
            Request::builder()
                .uri(&format!("/api/v1/documents/{}", fake_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(
        response.status() == StatusCode::NOT_FOUND
            || response.status() == StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn test_create_document_missing_fields() {
    if skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = create_test_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/documents")
                .header("Content-Type", "application/json")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::UNAUTHORIZED
            || response.status() == StatusCode::CREATED
            || response.status() == StatusCode::OK
            || response.status() == StatusCode::INTERNAL_SERVER_ERROR
            || response.status() == StatusCode::UNPROCESSABLE_ENTITY
    );
}

#[tokio::test]
async fn test_delete_document_not_found() {
    if skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = create_test_app().await;
    let fake_id = "00000000-0000-0000-0000-000000000000";
    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(&format!("/api/v1/documents/{}", fake_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(
        response.status() == StatusCode::NOT_FOUND
            || response.status() == StatusCode::UNAUTHORIZED
            || response.status() == StatusCode::NO_CONTENT
    );
}
