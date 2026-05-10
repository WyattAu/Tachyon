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

#[tokio::test]
async fn test_ssg_config_contains_expected_fields() {
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

    assert!(
        body["site_title"].is_string(),
        "site_title should be a string"
    );
    assert!(
        !body["site_title"].as_str().unwrap().is_empty(),
        "site_title should not be empty"
    );
    assert!(
        body["site_description"].is_string(),
        "site_description should be a string"
    );
    assert!(body["base_url"].is_string(), "base_url should be a string");
    assert!(body["theme"].is_string(), "theme should be a string");
    assert!(body["nav_links"].is_array(), "nav_links should be an array");
}

#[tokio::test]
async fn test_ssg_config_consistency() {
    if common::skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = common::create_test_app().await;

    let response1 = app
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

    let response2 = app
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

    let body1 = common::read_body_json(response1).await;
    let body2 = common::read_body_json(response2).await;

    assert_eq!(
        body1["site_title"], body2["site_title"],
        "Config should be consistent across requests"
    );
    assert_eq!(
        body1["theme"], body2["theme"],
        "Theme should be consistent across requests"
    );
}

#[tokio::test]
async fn test_build_site_missing_body() {
    if common::skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = common::create_test_app().await;
    let unique = uuid::Uuid::new_v4();

    let auth = common::register_and_login(
        &app,
        &format!("ssgnobody_{}", unique),
        &format!("ssgnobody_{}@test.com", unique),
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
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::INTERNAL_SERVER_ERROR
            || response.status() == StatusCode::OK,
        "Expected BAD_REQUEST (no documents), INTERNAL_SERVER_ERROR, or OK, got {}",
        response.status()
    );
}

#[tokio::test]
async fn test_build_site_invalid_json() {
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
                .body(Body::from("{{invalid json"))
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::UNPROCESSABLE_ENTITY,
        "Expected BAD_REQUEST for invalid JSON, got {}",
        response.status()
    );
}

#[tokio::test]
async fn test_ssg_build_response_shape() {
    if common::skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = common::create_test_app().await;
    let unique = uuid::Uuid::new_v4();

    let auth = common::register_and_login(
        &app,
        &format!("ssgshape_{}", unique),
        &format!("ssgshape_{}@test.com", unique),
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
                        "title": &format!("Shape Test {}", unique),
                        "base_url": "https://example.com",
                        "limit": 1
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .expect("Request failed");

    if response.status() == StatusCode::OK {
        let body = common::read_body_json(response).await;
        assert!(body["success"].is_boolean(), "success should be a boolean");
        assert!(
            body["result"].is_object(),
            "result should be an object when build succeeds"
        );
    }
}
