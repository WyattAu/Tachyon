use axum::body::Body;
use axum::extract::Request;
use axum::http::{header, StatusCode};
use axum::middleware::Next;
use axum::routing::get;
use axum::Router;
use http_body_util::BodyExt;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde_json::json;
use tachyon_core::UserRole;
use tachyon_server::config::ServerConfig;
use tachyon_server::middleware::auth::{auth_middleware, AuthContext, AuthState};
use tower::ServiceExt;

const TEST_JWT_SECRET: &str = "integration-test-secret-key-at-least-32chars!";
const TEST_JWT_ISSUER: &str = "tachyon-test";
const TEST_JWT_AUDIENCE: &str = "tachyon-test";

fn test_config() -> ServerConfig {
    let mut config = ServerConfig::default();
    config.jwt.secret = TEST_JWT_SECRET.to_string();
    config.jwt.issuer = TEST_JWT_ISSUER.to_string();
    config.jwt.audience = TEST_JWT_AUDIENCE.to_string();
    config.jwt.expiration_secs = 3600;
    config.rate_limit.enabled = false;
    config.api_keys.enabled = false;
    config
}

fn make_auth_state(pool: tachyon_database::DatabasePool) -> AuthState {
    AuthState::new(test_config(), pool)
}

async fn ok_handler() -> StatusCode {
    StatusCode::OK
}

async fn admin_permission_check(request: Request, next: Next) -> Result<axum::response::Response, StatusCode> {
    let auth_context = request
        .extensions()
        .get::<AuthContext>()
        .cloned()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if auth_context.role == UserRole::Admin {
        Ok(next.run(request).await)
    } else {
        Err(StatusCode::FORBIDDEN)
    }
}

fn build_test_router(pool: tachyon_database::DatabasePool) -> Router {
    let auth_state = make_auth_state(pool);

    let public_routes = Router::new()
        .route("/api/v1/auth/login", get(ok_handler))
        .route("/api/v1/auth/register", get(ok_handler))
        .route("/api/v1/auth/guest", get(ok_handler))
        .route("/api/v1/auth/refresh", get(ok_handler))
        .route("/api/v1/auth/mfa/authenticate", get(ok_handler))
        .route("/api/v1/auth/password-reset/request", get(ok_handler))
        .route("/api/v1/auth/email-verification/request", get(ok_handler))
        .route("/api/v1/billing/webhook", get(ok_handler))
        .route("/api/health", get(ok_handler))
        .route("/health", get(ok_handler))
        .route("/metrics", get(ok_handler))
        .route("/", get(ok_handler))
        .route("/api/docs", get(ok_handler))
        .route("/api/static/test.css", get(ok_handler));

    let protected_routes = Router::new()
        .route("/api/v1/user/profile", get(ok_handler))
        .route("/api/v1/documents", get(ok_handler));

    let admin_routes = Router::new()
        .route("/api/v1/admin/settings", get(ok_handler))
        .layer(axum::middleware::from_fn(
            admin_permission_check,
        ));

    let all_routes = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .merge(admin_routes);

    all_routes.layer(axum::middleware::from_fn_with_state(
        auth_state,
        auth_middleware,
    ))
}

fn generate_jwt(user_id: &str, role: &str) -> String {
    generate_jwt_with_extra(user_id, role, json!({}))
}

fn generate_jwt_with_extra(user_id: &str, role: &str, extra: serde_json::Value) -> String {
    let now = jsonwebtoken::get_current_timestamp();
    let mut claims = json!({
        "sub": user_id,
        "iss": TEST_JWT_ISSUER,
        "aud": TEST_JWT_AUDIENCE,
        "exp": (now + 3600),
        "iat": now,
        "role": role,
        "permissions": [],
        "team_id": null,
    });
    if let Some(obj) = claims.as_object_mut() {
        if let Some(extra_obj) = extra.as_object() {
            for (k, v) in extra_obj {
                obj.insert(k.clone(), v.clone());
            }
        }
    }

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(TEST_JWT_SECRET.as_ref()),
    )
    .unwrap()
}

fn generate_expired_jwt(user_id: &str) -> String {
    let now = jsonwebtoken::get_current_timestamp();
    let claims = json!({
        "sub": user_id,
        "iss": TEST_JWT_ISSUER,
        "aud": TEST_JWT_AUDIENCE,
        "exp": (now - 3600),
        "iat": (now - 7200),
        "role": "reader",
        "permissions": [],
        "team_id": null,
    });

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(TEST_JWT_SECRET.as_ref()),
    )
    .unwrap()
}

fn generate_jwt_wrong_secret(user_id: &str) -> String {
    let now = jsonwebtoken::get_current_timestamp();
    let claims = json!({
        "sub": user_id,
        "iss": TEST_JWT_ISSUER,
        "aud": TEST_JWT_AUDIENCE,
        "exp": (now + 3600),
        "iat": now,
        "role": "reader",
        "permissions": [],
        "team_id": null,
    });

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret("definitely-the-wrong-secret-key-123456".as_ref()),
    )
    .unwrap()
}

async fn setup_pool() -> tachyon_database::DatabasePool {
    let db_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://tachyon_test:tachyon_test@127.0.0.1:5433/tachyon_test".to_string());
    tachyon_database::DatabasePool::new(&db_url)
        .await
        .expect("Failed to connect to test database")
}

fn skip_without_db() -> bool {
    std::env::var("TEST_DATABASE_URL").is_err()
}

async fn read_body_json(
    response: axum::response::Response<Body>,
) -> serde_json::Value {
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("failed to read body")
        .to_bytes();
    serde_json::from_slice(&bytes).expect("body should be valid JSON")
}

// ── Test 1: No auth header on protected route → 401 ──────────────────

#[tokio::test]
async fn test_no_auth_header_protected_route_returns_401() {
    if skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let pool = setup_pool().await;
    let app = build_test_router(pool);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/user/profile")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body = read_body_json(response).await;
    assert_eq!(body["error"], "Missing authorization header");
}

// ── Test 2: Valid JWT on protected route → 200 ───────────────────────

#[tokio::test]
async fn test_valid_jwt_protected_route_returns_200() {
    if skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let pool = setup_pool().await;
    let app = build_test_router(pool);
    let token = generate_jwt("user-123", "admin");

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/user/profile")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

// ── Test 3: Expired JWT → 401 ────────────────────────────────────────

#[tokio::test]
async fn test_expired_jwt_returns_401() {
    if skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let pool = setup_pool().await;
    let app = build_test_router(pool);
    let token = generate_expired_jwt("user-123");

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/user/profile")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body = read_body_json(response).await;
    assert_eq!(body["error"], "Token expired");
}

// ── Test 4: JWT with wrong secret → 401 ──────────────────────────────

#[tokio::test]
async fn test_wrong_secret_jwt_returns_401() {
    if skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let pool = setup_pool().await;
    let app = build_test_router(pool);
    let token = generate_jwt_wrong_secret("user-123");

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/user/profile")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body = read_body_json(response).await;
    assert_eq!(body["error"], "Invalid signature");
}

// ── Test 5: Public paths bypass auth ──────────────────────────────────

#[tokio::test]
async fn test_public_path_login_no_auth() {
    if skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let pool = setup_pool().await;
    let app = build_test_router(pool);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/auth/login")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_public_path_register_no_auth() {
    if skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let pool = setup_pool().await;
    let app = build_test_router(pool);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/auth/register")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_public_path_guest_no_auth() {
    if skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let pool = setup_pool().await;
    let app = build_test_router(pool);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/auth/guest")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_public_path_refresh_no_auth() {
    if skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let pool = setup_pool().await;
    let app = build_test_router(pool);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/auth/refresh")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_public_path_billing_webhook_no_auth() {
    if skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let pool = setup_pool().await;
    let app = build_test_router(pool);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/billing/webhook")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_public_path_health_no_auth() {
    if skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let pool = setup_pool().await;
    let app = build_test_router(pool);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_public_path_root_no_auth() {
    if skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let pool = setup_pool().await;
    let app = build_test_router(pool);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_public_path_password_reset_prefix_no_auth() {
    if skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let pool = setup_pool().await;
    let app = build_test_router(pool);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/auth/password-reset/request")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_public_path_email_verification_prefix_no_auth() {
    if skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let pool = setup_pool().await;
    let app = build_test_router(pool);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/auth/email-verification/request")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_public_path_mfa_authenticate_no_auth() {
    if skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let pool = setup_pool().await;
    let app = build_test_router(pool);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/auth/mfa/authenticate")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

// ── Test 6: Admin-only route with non-admin role → 403 ──────────────

#[tokio::test]
async fn test_admin_route_with_reader_role_returns_403() {
    if skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let pool = setup_pool().await;
    let app = build_test_router(pool);
    let token = generate_jwt("user-reader", "reader");

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/admin/settings")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_admin_route_with_writer_role_returns_403() {
    if skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let pool = setup_pool().await;
    let app = build_test_router(pool);
    let token = generate_jwt("user-writer", "writer");

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/admin/settings")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_admin_route_with_admin_role_returns_200() {
    if skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let pool = setup_pool().await;
    let app = build_test_router(pool);
    let token = generate_jwt("user-admin", "admin");

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/admin/settings")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

// ── Test 7: Empty Bearer token → 401 ─────────────────────────────────

#[tokio::test]
async fn test_empty_bearer_token_returns_401() {
    if skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let pool = setup_pool().await;
    let app = build_test_router(pool);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/user/profile")
                .header(header::AUTHORIZATION, "Bearer ")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// ── Test 8: Malformed Authorization header ───────────────────────────

#[tokio::test]
async fn test_malformed_auth_header_returns_401() {
    if skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let pool = setup_pool().await;
    let app = build_test_router(pool);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/user/profile")
                .header(header::AUTHORIZATION, "NotBearer sometoken")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// ── Test 9: OPTIONS preflight is always allowed ──────────────────────

#[tokio::test]
async fn test_options_preflight_bypasses_auth() {
    if skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let pool = setup_pool().await;
    let app = build_test_router(pool);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("OPTIONS")
                .uri("/api/v1/user/profile")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // OPTIONS bypasses auth middleware. The route is GET-only, so
    // axum returns 405. Either way, auth did not block it.
    assert!(
        response.status() != StatusCode::UNAUTHORIZED,
        "OPTIONS should bypass auth middleware, got {}",
        response.status()
    );
}

// ── Test 10: JWT with custom permissions is accepted ─────────────────

#[tokio::test]
async fn test_jwt_with_custom_permissions_returns_200() {
    if skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let pool = setup_pool().await;
    let app = build_test_router(pool);

    let token = generate_jwt_with_extra(
        "user-with-perms",
        "editor",
        json!({
            "permissions": ["read", "write"],
            "team_id": "team-abc"
        }),
    );

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/user/profile")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

// ── Test 11: Admin-only route without auth header → 401 ──────────────

#[tokio::test]
async fn test_admin_route_no_auth_returns_401() {
    if skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let pool = setup_pool().await;
    let app = build_test_router(pool);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/admin/settings")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // auth_middleware runs first and returns 401
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// ── Test 12: Editor role on admin-only route → 403 ───────────────────

#[tokio::test]
async fn test_admin_route_with_editor_role_returns_403() {
    if skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let pool = setup_pool().await;
    let app = build_test_router(pool);
    let token = generate_jwt("user-editor", "editor");

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/admin/settings")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
