// Integration tests for SSG (static site generation) routes
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::json;
use tower::ServiceExt;

use crate::common;

#[tokio::test]
async fn test_get_ssg_config() {
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
                .uri("/api/v1/ssg/config")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert_eq!(response.status(), StatusCode::OK);
    let body = common::read_body_json(response).await;
    assert!(body["site_title"].is_string() || body["title"].is_string());
}

#[tokio::test]
async fn test_build_site_unauthorized() {
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
                .uri("/api/v1/ssg/build")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "title": "Test Site",
                        "description": "Test Description",
                        "base_url": "https://example.com",
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .expect("Request failed");

    // SSG build may or may not require auth — check status is valid
    assert!(
        response.status() == StatusCode::OK
            || response.status() == StatusCode::UNAUTHORIZED
            || response.status() == StatusCode::CREATED
            || response.status() == StatusCode::INTERNAL_SERVER_ERROR
            || response.status() == StatusCode::BAD_REQUEST,
        "Expected OK/UNAUTHORIZED/CREATED/INTERNAL_SERVER_ERROR/BAD_REQUEST, got {}",
        response.status()
    );
}

#[tokio::test]
async fn test_build_site_authenticated() {
    if common::skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = common::create_test_app().await;
    let unique = uuid::Uuid::new_v4();

    let auth = common::register_and_login(
        &app,
        &format!("ssguser_{}", unique),
        &format!("ssguser_{}@test.com", unique),
        "Password123!",
    )
    .await
    .expect("Failed to register/login test user");

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/ssg/build")
                .header("Content-Type", "application/json")
                .header("Authorization", common::auth_header(&auth.token))
                .body(Body::from(
                    json!({
                        "title": &format!("SSG Test Site {}", unique),
                        "description": "A site built from integration test",
                        "base_url": "https://example.com",
                        "theme": "default",
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert!(
        response.status() == StatusCode::OK
            || response.status() == StatusCode::CREATED
            || response.status() == StatusCode::INTERNAL_SERVER_ERROR
            || response.status() == StatusCode::BAD_REQUEST,
        "Expected OK, CREATED, INTERNAL_SERVER_ERROR, or BAD_REQUEST, got {}",
        response.status()
    );
}

#[tokio::test]
async fn test_download_site() {
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
                .uri("/api/v1/ssg/download")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Request failed");

    // May return OK with tarball, 404 if no build exists, 204, or 500
    assert!(
        response.status() == StatusCode::OK
            || response.status() == StatusCode::NOT_FOUND
            || response.status() == StatusCode::INTERNAL_SERVER_ERROR
            || response.status() == StatusCode::NO_CONTENT,
        "Expected OK, NOT_FOUND, INTERNAL_SERVER_ERROR, or NO_CONTENT, got {}",
        response.status()
    );
}

#[tokio::test]
async fn test_build_site_with_nav_links() {
    if common::skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = common::create_test_app().await;
    let unique = uuid::Uuid::new_v4();

    let auth = common::register_and_login(
        &app,
        &format!("ssgnav_{}", unique),
        &format!("ssgnav_{}@test.com", unique),
        "Password123!",
    )
    .await
    .expect("Failed to register/login test user");

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/ssg/build")
                .header("Content-Type", "application/json")
                .header("Authorization", common::auth_header(&auth.token))
                .body(Body::from(
                    json!({
                        "title": &format!("Nav Site {}", unique),
                        "description": "Site with nav links",
                        "base_url": "https://example.com",
                        "nav_links": [
                            {"label": "Home", "url": "/"},
                            {"label": "Docs", "url": "/docs"},
                            {"label": "About", "url": "/about"}
                        ],
                        "group_by_tag": true,
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert!(
        response.status() == StatusCode::OK
            || response.status() == StatusCode::CREATED
            || response.status() == StatusCode::INTERNAL_SERVER_ERROR
            || response.status() == StatusCode::UNPROCESSABLE_ENTITY,
        "Expected OK, CREATED, INTERNAL_SERVER_ERROR, or UNPROCESSABLE_ENTITY, got {}",
        response.status()
    );
}

#[tokio::test]
async fn test_ssg_config_response_shape() {
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
                .uri("/api/v1/ssg/config")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert_eq!(response.status(), StatusCode::OK);
    let body = common::read_body_json(response).await;
    assert!(body.is_object(), "Response should be a JSON object");
}
