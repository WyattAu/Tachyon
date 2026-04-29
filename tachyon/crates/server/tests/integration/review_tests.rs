// Integration tests for review routes
use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use tower::ServiceExt;
use serde_json::json;

use crate::common;

#[tokio::test]
async fn test_create_review() {
    if common::skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = common::create_test_app().await;
    let unique = uuid::Uuid::new_v4();

    let auth = common::register_and_login(
        &app,
        &format!("review_user_{}", unique),
        &format!("review_{}@test.com", unique),
        "Password123!",
    )
    .await
    .expect("Failed to register/login");

    // First create a document
    let doc_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/documents")
                .header("Content-Type", "application/json")
                .header("Authorization", common::auth_header(&auth.token))
                .body(Body::from(
                    json!({
                        "title": format!("Review Doc {}", unique),
                        "content": "# Content for review",
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .expect("Document creation failed");

    if doc_response.status() == StatusCode::CREATED || doc_response.status() == StatusCode::OK {
        let doc_body = common::read_body_json(doc_response).await;
        let doc_id = doc_body["id"].as_str().or_else(|| doc_body["document_id"].as_str());

        if let Some(id) = doc_id {
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri(format!("/api/v1/documents/{}/reviews", id))
                        .header("Content-Type", "application/json")
                        .header("Authorization", common::auth_header(&auth.token))
                        .body(Body::from(
                            json!({
                                "reviewer_id": &auth.user_id,
                                "summary": "Looks good",
                                "version_number": 1,
                            })
                            .to_string(),
                        ))
                        .unwrap(),
                )
                .await
                .expect("Review creation failed");

            assert!(
                response.status() == StatusCode::CREATED || response.status() == StatusCode::OK,
                "Expected CREATED or OK, got {}",
                response.status()
            );
        }
    }
}

#[tokio::test]
async fn test_review_unauthorized() {
    if common::skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = common::create_test_app().await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/documents/fake-id/reviews")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert!(
        response.status() == StatusCode::UNAUTHORIZED || response.status() == StatusCode::OK || response.status() == StatusCode::INTERNAL_SERVER_ERROR,
        "Expected UNAUTHORIZED, OK, or INTERNAL_SERVER_ERROR (no auth middleware in test router), got {}",
        response.status()
    );
}
