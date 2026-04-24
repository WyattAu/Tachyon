use axum::{
    body::Body,
    http::{Request, StatusCode, header},
    Router,
};
use http_body_util::BodyExt;
use tower::ServiceExt;
use serde_json::json;

pub fn skip_without_db() -> bool {
    std::env::var("TEST_DATABASE_URL").is_err()
}

pub async fn create_test_app() -> Router {
    tachyon_server::routes::create_router().await
}

pub fn auth_header(token: &str) -> header::HeaderValue {
    header::HeaderValue::from_str(&format!("Bearer {}", token)).expect("invalid token")
}

pub fn create_test_jwt(user_id: &str, secret: &str) -> String {
    use jsonwebtoken::{encode, EncodingKey, Header};
    let claims = json!({
        "sub": user_id,
        "iss": "tachyon",
        "aud": "tachyon",
        "exp": (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp(),
        "iat": chrono::Utc::now().timestamp(),
        "role": "user"
    });

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .unwrap()
}

pub struct TestAuth {
    pub user_id: String,
    pub token: String,
}

/// Read the full body bytes from an axum response
pub async fn read_body_bytes(response: axum::response::Response<Body>) -> bytes::Bytes {
    let collected = response.into_body().collect().await.expect("failed to read body");
    collected.to_bytes()
}

/// Read the full body as a String from an axum response
pub async fn read_body_string(response: axum::response::Response<Body>) -> String {
    String::from_utf8(read_body_bytes(response).await.to_vec()).expect("body should be valid UTF-8")
}

/// Read the full body and parse as JSON Value
pub async fn read_body_json(response: axum::response::Response<Body>) -> serde_json::Value {
    let bytes = read_body_bytes(response).await;
    serde_json::from_slice(&bytes).expect("body should be valid JSON")
}

pub async fn register_and_login(app: &Router, username: &str, email: &str, password: &str) -> Option<TestAuth> {
    let register_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header("Content-Type", "application/json")
                .body(Body::from(json!({
                    "email": email,
                    "password": password,
                    "username": username,
                }).to_string()))
                .unwrap(),
        )
        .await
        .ok()?;

    if register_response.status() != StatusCode::CREATED {
        return None;
    }

    let login_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header("Content-Type", "application/json")
                .body(Body::from(json!({
                    "email": email,
                    "password": password,
                }).to_string()))
                .unwrap(),
        )
        .await
        .ok()?;

    if login_response.status() != StatusCode::OK {
        return None;
    }

    let body: serde_json::Value = read_body_json(login_response).await;
    let token = body.get("token").and_then(|t| t.as_str()).map(|s| s.to_string())?;

    let user_id = body.get("user")
        .and_then(|u| u.get("id"))
        .and_then(|i| i.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    Some(TestAuth { user_id, token })
}

pub async fn create_test_user(
    app: &Router,
    username: &str,
    email: &str,
    password: &str,
) -> Option<TestAuth> {
    register_and_login(app, username, email, password).await
}
