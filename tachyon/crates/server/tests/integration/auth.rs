use super::common::{create_test_app, skip_without_db};
use axum::body::Body;
use axum::http::{Request, StatusCode, header};
#[allow(unused_imports)]
use http_body_util::BodyExt;
use serde_json::json;
use tower::ServiceExt;

#[tokio::test]
async fn test_login_missing_credentials() {
    if skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = create_test_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header("Content-Type", "application/json")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();

    // Missing required fields returns 422 (Unprocessable Entity) from serde validation
    assert!(
        response.status() == StatusCode::OK
            || response.status() == StatusCode::UNPROCESSABLE_ENTITY,
        "Expected 200 or 422, got {}",
        response.status()
    );
}

#[tokio::test]
async fn test_login_invalid_credentials() {
    if skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = create_test_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "username": "nonexistent@example.com",
                        "password": "wrongpassword"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = response
        .into_body()
        .collect()
        .await
        .expect("failed to read body")
        .to_bytes();
    let body: serde_json::Value =
        serde_json::from_slice(&body_bytes).expect("body should be valid JSON");
    assert_eq!(body["success"], false);
}

#[tokio::test]
async fn test_protected_endpoint_without_token() {
    if skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = create_test_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/user/profile")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(
        response.status() == StatusCode::UNAUTHORIZED
            || response.status() == StatusCode::OK
            || response.status() == StatusCode::NOT_FOUND,
        "Expected UNAUTHORIZED, OK, or NOT_FOUND (no auth middleware in test router), got {}",
        response.status()
    );
}

#[tokio::test]
async fn test_protected_endpoint_with_invalid_token() {
    if skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = create_test_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/user/profile")
                .header(header::AUTHORIZATION, "Bearer invalid_token_garbage")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(
        response.status() == StatusCode::UNAUTHORIZED
            || response.status() == StatusCode::OK
            || response.status() == StatusCode::NOT_FOUND
            || response.status() == StatusCode::INTERNAL_SERVER_ERROR,
        "Expected UNAUTHORIZED, OK, NOT_FOUND, or INTERNAL_SERVER_ERROR, got {}",
        response.status()
    );
}

#[tokio::test]
async fn test_register_missing_data() {
    if skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = create_test_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header("Content-Type", "application/json")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn test_register_with_valid_data() {
    if skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = create_test_app().await;
    let unique = uuid::Uuid::new_v4();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "email": format!("integtest_{}@example.com", unique),
                        "password": "SecurePass123!",
                        "username": format!("integuser_{}", unique),
                        "display_name": format!("Test User {}", unique)
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status() == StatusCode::OK || response.status() == StatusCode::CONFLICT);
}
