use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use tower::ServiceExt;
use tachyon_server::routes::create_router;
use serde_json::json;

fn skip_without_db() -> bool {
    std::env::var("TEST_DATABASE_URL").is_err()
}

async fn create_test_app() -> axum::Router {
    create_router().await
}

#[allow(dead_code)]
fn create_test_jwt(user_id: &str, secret: &str) -> String {
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

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
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
                        "email": "nonexistent@example.com",
                        "password": "wrongpassword"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
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

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
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

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
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

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
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
                        "username": format!("integuser_{}", unique)
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(
        response.status() == StatusCode::CREATED || response.status() == StatusCode::CONFLICT
    );
}
