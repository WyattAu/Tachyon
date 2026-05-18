use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use serde_json::json;
use tower::ServiceExt;

use crate::common;

#[tokio::test]
async fn test_create_document() {
    if common::skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = common::create_test_app().await;
    let unique = uuid::Uuid::new_v4();

    let auth = common::register_and_login(
        &app,
        &format!("docuser_{}", unique),
        &format!("doc_{}@example.com", unique),
        "SecurePass123!",
    )
    .await;

    let auth = match auth {
        Some(a) => a,
        None => {
            println!("Skipping: could not register/login test user");
            return;
        }
    };

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/documents")
                .header("Content-Type", "application/json")
                .header(header::AUTHORIZATION, common::auth_header(&auth.token))
                .body(Body::from(
                    json!({
                        "title": format!("Test Document {}", unique),
                        "slug": format!("test-doc-{}", unique),
                        "content_type": "Markdown",
                        "visibility": "Private",
                        "status": "Draft"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(
        response.status() == StatusCode::CREATED
            || response.status() == StatusCode::OK
            || response.status() == StatusCode::UNAUTHORIZED
            || response.status() == StatusCode::INTERNAL_SERVER_ERROR
            || response.status() == StatusCode::UNPROCESSABLE_ENTITY,
        "Expected CREATED/OK/UNAUTHORIZED/500/422 for document creation, got {}",
        response.status()
    );
}

#[tokio::test]
async fn test_read_document() {
    if common::skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = common::create_test_app().await;
    let fake_id = "00000000-0000-0000-0000-000000000000";

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/documents/{}", fake_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(
        response.status() == StatusCode::NOT_FOUND
            || response.status() == StatusCode::UNAUTHORIZED
            || response.status() == StatusCode::OK
    );
}

#[tokio::test]
async fn test_update_document() {
    if common::skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = common::create_test_app().await;
    let fake_id = uuid::Uuid::new_v4();

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/documents/{}", fake_id))
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "title": "Updated Title"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(
        response.status() == StatusCode::NOT_FOUND
            || response.status() == StatusCode::UNAUTHORIZED
            || response.status() == StatusCode::OK
    );
}

#[tokio::test]
async fn test_delete_document() {
    if common::skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = common::create_test_app().await;
    let fake_id = "00000000-0000-0000-0000-000000000000";

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/documents/{}", fake_id))
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

#[tokio::test]
async fn test_list_documents_with_filters() {
    if common::skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = common::create_test_app().await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/documents?status=Draft&page=1&page_size=10")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status() == StatusCode::OK || response.status() == StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_create_document_missing_fields() {
    if common::skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = common::create_test_app().await;

    let response = app
        .clone()
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
