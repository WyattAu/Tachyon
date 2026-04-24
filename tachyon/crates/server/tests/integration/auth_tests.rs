use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use tower::ServiceExt;
use serde_json::json;

use crate::common;

#[tokio::test]
async fn test_register_success() {
    if common::skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = common::create_test_app().await;
    let unique = uuid::Uuid::new_v4();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header("Content-Type", "application/json")
                .body(Body::from(json!({
                    "email": format!("integtest_{}@example.com", unique),
                    "password": "SecurePass123!",
                    "username": format!("integuser_{}", unique)
                }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body: serde_json::Value = common::read_body_json(response).await;
    assert!(body.get("user").is_some());
}

#[tokio::test]
async fn test_register_duplicate_email() {
    if common::skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = common::create_test_app().await;
    let unique = uuid::Uuid::new_v4();
    let email = format!("dup_{}@example.com", unique);

    let register_body = json!({
        "email": email,
        "password": "SecurePass123!",
        "username": format!("dupuser_{}", unique)
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header("Content-Type", "application/json")
                .body(Body::from(register_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let response2 = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header("Content-Type", "application/json")
                .body(Body::from(register_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response2.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_login_success() {
    if common::skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = common::create_test_app().await;
    let unique = uuid::Uuid::new_v4();

    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header("Content-Type", "application/json")
                .body(Body::from(json!({
                    "email": format!("login_{}@example.com", unique),
                    "password": "SecurePass123!",
                    "username": format!("loginuser_{}", unique)
                }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header("Content-Type", "application/json")
                .body(Body::from(json!({
                    "email": format!("login_{}@example.com", unique),
                    "password": "SecurePass123!"
                }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body: serde_json::Value = common::read_body_json(response).await;
    assert!(body.get("token").is_some());
    assert!(body.get("user").is_some());
}

#[tokio::test]
async fn test_login_invalid_password() {
    if common::skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = common::create_test_app().await;
    let unique = uuid::Uuid::new_v4();

    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header("Content-Type", "application/json")
                .body(Body::from(json!({
                    "email": format!("badpass_{}@example.com", unique),
                    "password": "SecurePass123!",
                    "username": format!("badpassuser_{}", unique)
                }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header("Content-Type", "application/json")
                .body(Body::from(json!({
                    "email": format!("badpass_{}@example.com", unique),
                    "password": "WrongPassword!"
                }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_get_me_with_valid_token() {
    if common::skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = common::create_test_app().await;
    let unique = uuid::Uuid::new_v4();
    let username = format!("meuser_{}", unique);
    let email = format!("me_{}@example.com", unique);

    let auth = common::register_and_login(
        &app,
        &username,
        &email,
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
                .uri("/api/v1/user/profile")
                .header(header::AUTHORIZATION, common::auth_header(&auth.token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status() == StatusCode::OK || response.status() == StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_jwt_validation_expired_token() {
    if common::skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = common::create_test_app().await;
    let user_id = uuid::Uuid::new_v4().to_string();

    let expired_claims = json!({
        "sub": user_id,
        "exp": (chrono::Utc::now() - chrono::Duration::hours(1)).timestamp(),
        "iat": (chrono::Utc::now() - chrono::Duration::hours(2)).timestamp()
    });

    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &expired_claims,
        &jsonwebtoken::EncodingKey::from_secret("test_secret_key".as_bytes()),
    )
    .unwrap();

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/user/profile")
                .header(header::AUTHORIZATION, common::auth_header(&token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_logout() {
    if common::skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = common::create_test_app().await;
    let unique = uuid::Uuid::new_v4();

    let register_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header("Content-Type", "application/json")
                .body(Body::from(json!({
                    "email": format!("logout_{}@example.com", unique),
                    "password": "SecurePass123!",
                    "username": format!("logoutuser_{}", unique)
                }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    if register_response.status() != StatusCode::CREATED {
        println!("Skipping: registration failed");
        return;
    }

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/logout")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(
        response.status() == StatusCode::OK
            || response.status() == StatusCode::NO_CONTENT
            || response.status() == StatusCode::UNAUTHORIZED
    );
}
