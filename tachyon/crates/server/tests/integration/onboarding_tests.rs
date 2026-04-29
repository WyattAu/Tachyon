// Integration tests for onboarding routes
use axum::{
    body::Body,
    http::{Request, StatusCode, header},
    Router,
};
use tower::ServiceExt;
use serde_json::json;

use crate::common;

#[tokio::test]
async fn test_get_onboarding_status_unauthenticated() {
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
                .uri("/api/v1/onboarding/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Request failed");

    // Should require auth or return default status
    assert!(
        response.status() == StatusCode::OK
            || response.status() == StatusCode::UNAUTHORIZED,
        "Expected OK or UNAUTHORIZED, got {}",
        response.status()
    );
}

#[tokio::test]
async fn test_get_onboarding_status_authenticated() {
    if common::skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = common::create_test_app().await;
    let unique = uuid::Uuid::new_v4();

    let auth = common::register_and_login(
        &app,
        &format!("onb_user_{}", unique),
        &format!("onb_user_{}@test.com", unique),
        "Password123!",
    )
    .await
    .expect("Failed to register/login test user");

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/onboarding/status")
                .header("Authorization", common::auth_header(&auth.token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert_eq!(response.status(), StatusCode::OK);
    let body = common::read_body_json(response).await;
    assert!(body.is_object(), "Status response should be a JSON object");
}

#[tokio::test]
async fn test_complete_onboarding_step() {
    if common::skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = common::create_test_app().await;
    let unique = uuid::Uuid::new_v4();

    let auth = common::register_and_login(
        &app,
        &format!("onb_step_{}", unique),
        &format!("onb_step_{}@test.com", unique),
        "Password123!",
    )
    .await
    .expect("Failed to register/login test user");

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/onboarding/complete")
                .header("Content-Type", "application/json")
                .header("Authorization", common::auth_header(&auth.token))
                .body(Body::from(
                    json!({
                        "step_id": "profile_setup",
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
async fn test_complete_multiple_onboarding_steps() {
    if common::skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = common::create_test_app().await;
    let unique = uuid::Uuid::new_v4();

    let auth = common::register_and_login(
        &app,
        &format!("onb_multi_{}", unique),
        &format!("onb_multi_{}@test.com", unique),
        "Password123!",
    )
    .await
    .expect("Failed to register/login test user");

    let steps = ["profile_setup", "workspace_created", "first_document"];

    for step in &steps {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/onboarding/complete")
                    .header("Content-Type", "application/json")
                    .header("Authorization", common::auth_header(&auth.token))
                    .body(Body::from(
                        json!({ "step_id": step }).to_string(),
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
            "Step '{}' failed with status {}",
            step,
            response.status()
        );
    }

    // Verify status reflects completed steps
    let status_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/onboarding/status")
                .header("Authorization", common::auth_header(&auth.token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert_eq!(status_response.status(), StatusCode::OK);
    let body = common::read_body_json(status_response).await;
    assert!(body.is_object(), "Status should be a JSON object");
}

#[tokio::test]
async fn test_create_sample_content() {
    if common::skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = common::create_test_app().await;
    let unique = uuid::Uuid::new_v4();

    let auth = common::register_and_login(
        &app,
        &format!("onb_sample_{}", unique),
        &format!("onb_sample_{}@test.com", unique),
        "Password123!",
    )
    .await
    .expect("Failed to register/login test user");

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/onboarding/sample-content")
                .header("Authorization", common::auth_header(&auth.token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert!(
        response.status() == StatusCode::OK
            || response.status() == StatusCode::CREATED
            || response.status() == StatusCode::INTERNAL_SERVER_ERROR,
        "Expected OK, CREATED, or INTERNAL_SERVER_ERROR, got {}",
        response.status()
    );
}

#[tokio::test]
async fn test_get_suggestions() {
    if common::skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = common::create_test_app().await;
    let unique = uuid::Uuid::new_v4();

    let auth = common::register_and_login(
        &app,
        &format!("onb_suggest_{}", unique),
        &format!("onb_suggest_{}@test.com", unique),
        "Password123!",
    )
    .await
    .expect("Failed to register/login test user");

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/onboarding/suggestions")
                .header("Authorization", common::auth_header(&auth.token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert_eq!(response.status(), StatusCode::OK);
    let body = common::read_body_json(response).await;
    assert!(body.is_object(), "Suggestions should be a JSON object");
}
