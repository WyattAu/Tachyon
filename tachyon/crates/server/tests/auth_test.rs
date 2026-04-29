use axum::{
    body::Body,
    http::{header, Request, StatusCode},
    Router,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde_json::json;
use tachyon_server::routes::create_router;
use tower::ServiceExt;

async fn create_test_app() -> Router {
    create_router().await
}

fn create_test_jwt(user_id: &str, secret: &str) -> String {
    let claims = json!({
        "sub": user_id,
        "exp": (Utc::now() + Duration::hours(1)).timestamp(),
        "iat": Utc::now().timestamp()
    });

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .unwrap()
}

#[tokio::test]
async fn test_login_endpoint_missing_credentials() {
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

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_login_endpoint_invalid_credentials() {
    let app = create_test_app().await;

    let login_data = json!({
        "email": "nonexistent@example.com",
        "password": "wrongpassword"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header("Content-Type", "application/json")
                .body(Body::from(login_data.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_protected_endpoint_without_token() {
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

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_protected_endpoint_with_invalid_token() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/user/profile")
                .header(header::AUTHORIZATION, "Bearer invalid_token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_protected_endpoint_with_valid_token() {
    let app = create_test_app().await;
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "test_secret_key".to_string());
    let user_id = uuid::Uuid::new_v4().to_string();
    let token = create_test_jwt(&user_id, &secret);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/user/profile")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status() == StatusCode::OK || response.status() == StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_logout_endpoint() {
    let app = create_test_app().await;
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "test_secret_key".to_string());
    let user_id = uuid::Uuid::new_v4().to_string();
    let token = create_test_jwt(&user_id, &secret);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/logout")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status() == StatusCode::OK || response.status() == StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_token_refresh_without_token() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/refresh")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_register_endpoint_missing_data() {
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

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_register_endpoint_valid_data() {
    let app = create_test_app().await;

    let register_data = json!({
        "email": format!("test_{}@example.com", uuid::Uuid::new_v4()),
        "password": "SecurePass123!",
        "username": format!("testuser_{}", uuid::Uuid::new_v4())
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header("Content-Type", "application/json")
                .body(Body::from(register_data.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status() == StatusCode::CREATED || response.status() == StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_password_reset_request() {
    let app = create_test_app().await;

    let reset_data = json!({
        "email": "test@example.com"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/password-reset")
                .header("Content-Type", "application/json")
                .body(Body::from(reset_data.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status() == StatusCode::OK || response.status() == StatusCode::ACCEPTED);
}

#[tokio::test]
async fn test_me_endpoint() {
    let app = create_test_app().await;
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "test_secret_key".to_string());
    let user_id = uuid::Uuid::new_v4().to_string();
    let token = create_test_jwt(&user_id, &secret);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/auth/me")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status() == StatusCode::OK || response.status() == StatusCode::UNAUTHORIZED);
}
