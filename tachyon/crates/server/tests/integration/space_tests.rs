// Integration tests for space routes
use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use tower::ServiceExt;
use serde_json::json;

use crate::common;

#[tokio::test]
async fn test_create_space() {
    if common::skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = common::create_test_app().await;
    let unique = uuid::Uuid::new_v4();

    let auth = common::register_and_login(
        &app,
        &format!("space_user_{}", unique),
        &format!("space_{}@test.com", unique),
        "Password123!",
    )
    .await
    .expect("Failed to register/login");

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/spaces")
                .header("Content-Type", "application/json")
                .header("Authorization", common::auth_header(&auth.token))
                .body(Body::from(
                    json!({
                        "name": format!("Test Space {}", unique),
                        "description": "A test space",
                        "visibility": "private",
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert!(
        response.status() == StatusCode::CREATED || response.status() == StatusCode::OK,
        "Expected CREATED or OK, got {}",
        response.status()
    );
}

#[tokio::test]
async fn test_list_spaces() {
    if common::skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = common::create_test_app().await;
    let unique = uuid::Uuid::new_v4();

    let auth = common::register_and_login(
        &app,
        &format!("spacelist_{}", unique),
        &format!("spacelist_{}@test.com", unique),
        "Password123!",
    )
    .await
    .expect("Failed to register/login");

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/spaces")
                .header("Authorization", common::auth_header(&auth.token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert_eq!(response.status(), StatusCode::OK);
    let body = common::read_body_json(response).await;
    assert!(body.is_array(), "Spaces should be an array");
}

#[tokio::test]
async fn test_get_default_space() {
    if common::skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = common::create_test_app().await;
    let unique = uuid::Uuid::new_v4();

    let auth = common::register_and_login(
        &app,
        &format!("spacedef_{}", unique),
        &format!("spacedef_{}@test.com", unique),
        "Password123!",
    )
    .await
    .expect("Failed to register/login");

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/spaces/default")
                .header("Authorization", common::auth_header(&auth.token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Request failed");

    // May return 200 with default space or 404 if none set
    assert!(
        response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND,
        "Expected OK or NOT_FOUND, got {}",
        response.status()
    );
}

#[tokio::test]
async fn test_spaces_unauthorized() {
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
                .uri("/api/v1/spaces")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
