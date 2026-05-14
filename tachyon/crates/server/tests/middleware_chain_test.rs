use axum::{
    body::Body,
    http::{HeaderMap, HeaderValue, Method, StatusCode},
    middleware::from_fn,
    routing::get,
    Router,
};
use tower::ServiceBuilder;
use tower::ServiceExt;

use tachyon_server::middleware::security_headers::{
    add_security_headers_with_config, SecurityHeadersConfig,
};
use tachyon_server::middleware::{
    add_security_headers_from_config, audit_middleware, cache_control_middleware,
    compression_layer, request_id_middleware, request_size_limit,
};

fn test_config() -> tachyon_server::config::ServerConfig {
    let mut config = tachyon_server::config::ServerConfig::default();
    config.jwt.secrets = vec!["a-sufficiently-long-secret-key-for-tests-32ch".to_string()];
    config.security.development = true;
    config
}

fn build_full_middleware_app(config: &tachyon_server::config::ServerConfig) -> Router {
    let security_config = std::sync::Arc::new(config.security.clone());

    Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/api/v1/documents", get(|| async { "protected" }))
        .route("/api/v1/auth/login", get(|| async { "public" }))
        .layer(
            ServiceBuilder::new()
                .layer(from_fn(request_id_middleware))
                .layer(from_fn(cache_control_middleware))
                .layer(from_fn(audit_middleware))
                .layer(from_fn(request_size_limit))
                .layer(compression_layer())
                .map_response(move |response: axum::response::Response| {
                    add_security_headers_from_config(response, &security_config)
                }),
        )
}

async fn send_request(
    app: Router,
    method: Method,
    path: &str,
    headers: HeaderMap,
) -> axum::response::Response {
    let req = axum::extract::Request::builder()
        .method(method)
        .uri(path)
        .header("host", "localhost")
        .header("connection", "close")
        .body(Body::empty())
        .unwrap();

    let mut req = req;
    for (name, value) in headers.iter() {
        req.headers_mut().insert(name, value.clone());
    }

    app.oneshot(req).await.unwrap()
}

// ============================================================================
// 1. Full middleware chain injects all expected headers
// ============================================================================

#[tokio::test]
async fn test_full_middleware_chain_injects_headers() {
    let config = test_config();
    let app = build_full_middleware_app(&config);

    let response = send_request(app, Method::GET, "/health", HeaderMap::new()).await;

    assert_eq!(response.status(), StatusCode::OK);
    assert!(response.headers().contains_key("x-frame-options"));
    assert!(response.headers().contains_key("x-content-type-options"));
    assert!(response.headers().contains_key("referrer-policy"));
}

#[tokio::test]
async fn test_full_chain_security_headers_on_api_route() {
    let config = test_config();
    let app = build_full_middleware_app(&config);

    let response = send_request(app, Method::GET, "/api/v1/documents", HeaderMap::new()).await;

    assert_eq!(response.status(), StatusCode::OK);

    let headers = response.headers();
    assert!(headers.contains_key("x-request-id"));
    assert!(headers.contains_key("x-frame-options"));
    assert!(headers.contains_key("x-content-type-options"));
    assert!(headers.contains_key("referrer-policy"));
    assert!(headers.contains_key("content-security-policy"));
    assert!(headers.contains_key("cache-control"));
    assert!(headers.contains_key("etag"));

    let xfo = headers.get("x-frame-options").unwrap().to_str().unwrap();
    assert_eq!(xfo, "DENY");

    let xcto = headers
        .get("x-content-type-options")
        .unwrap()
        .to_str()
        .unwrap();
    assert_eq!(xcto, "nosniff");

    let rp = headers.get("referrer-policy").unwrap().to_str().unwrap();
    assert_eq!(rp, "strict-origin-when-cross-origin");
}

#[tokio::test]
async fn test_full_chain_permissions_policy_present() {
    let config = test_config();
    let app = build_full_middleware_app(&config);

    let response = send_request(app, Method::GET, "/health", HeaderMap::new()).await;

    assert!(response.headers().contains_key("permissions-policy"));
}

// ============================================================================
// 2. Request ID middleware
// ============================================================================

#[tokio::test]
async fn test_request_id_generated_on_every_request() {
    let config = test_config();
    let app = build_full_middleware_app(&config);

    let r1 = send_request(app.clone(), Method::GET, "/health", HeaderMap::new()).await;
    let r2 = send_request(app, Method::GET, "/health", HeaderMap::new()).await;

    let id1 = r1.headers().get("x-request-id").unwrap().to_str().unwrap();
    let id2 = r2.headers().get("x-request-id").unwrap().to_str().unwrap();
    assert_ne!(id1, id2);
}

#[tokio::test]
async fn test_request_id_preserved_from_client() {
    let config = test_config();
    let app = build_full_middleware_app(&config);

    let mut headers = HeaderMap::new();
    headers.insert(
        "x-request-id",
        HeaderValue::from_static("client-provided-id-999"),
    );

    let response = send_request(app, Method::GET, "/health", headers).await;

    let rid = response
        .headers()
        .get("x-request-id")
        .unwrap()
        .to_str()
        .unwrap();
    assert_eq!(rid, "client-provided-id-999");
}

// ============================================================================
// 3. Cache control middleware
// ============================================================================

#[tokio::test]
async fn test_cache_control_set_on_api_response() {
    let config = test_config();
    let app = build_full_middleware_app(&config);

    let response = send_request(app, Method::GET, "/api/v1/documents", HeaderMap::new()).await;

    let cache = response
        .headers()
        .get("cache-control")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(cache.contains("max-age"));
}

#[tokio::test]
async fn test_cache_control_no_store_on_non_get() {
    let config = test_config();
    let app = build_full_middleware_app(&config);

    let response = send_request(app, Method::OPTIONS, "/api/v1/documents", HeaderMap::new()).await;

    let cache = response
        .headers()
        .get("cache-control")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(cache.contains("no-store"));
}

// ============================================================================
// 4. Request size limit
// ============================================================================

#[tokio::test]
async fn test_request_size_limit_valid_request_passes() {
    let config = test_config();
    let app = build_full_middleware_app(&config);

    let req = axum::extract::Request::builder()
        .method(Method::POST)
        .uri("/api/v1/documents")
        .header("host", "localhost")
        .header("content-length", "100")
        .body(Body::from(vec![0u8; 100]))
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_ne!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
}

#[tokio::test]
async fn test_request_size_limit_oversized_rejected() {
    let config = test_config();
    let app = build_full_middleware_app(&config);

    let oversized = 11 * 1024 * 1024;
    let req = axum::extract::Request::builder()
        .method(Method::POST)
        .uri("/api/v1/documents")
        .header("host", "localhost")
        .header("content-length", oversized.to_string().as_str())
        .body(Body::from(vec![0u8; 1024]))
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
}

// ============================================================================
// 5. SecurityHeadersConfig standalone
// ============================================================================

#[test]
fn test_security_headers_config_full_production_headers() {
    let config = SecurityHeadersConfig::default();
    let response = axum::response::Response::new(Body::empty());
    let response = add_security_headers_with_config(response, &config);

    let headers = response.headers();
    assert!(headers.contains_key("content-security-policy"));
    assert!(headers.contains_key("x-frame-options"));
    assert!(headers.contains_key("x-content-type-options"));
    assert!(headers.contains_key("strict-transport-security"));
    assert!(headers.contains_key("referrer-policy"));
    assert!(headers.contains_key("permissions-policy"));
    assert!(headers.contains_key("cross-origin-embedder-policy"));
    assert!(headers.contains_key("cross-origin-opener-policy"));
    assert!(headers.contains_key("cross-origin-resource-policy"));
}

#[test]
fn test_security_headers_config_no_hsts_when_disabled() {
    let config = SecurityHeadersConfig {
        strict_transport_security: None,
        ..Default::default()
    };

    let response = axum::response::Response::new(Body::empty());
    let response = add_security_headers_with_config(response, &config);

    assert!(!response.headers().contains_key("strict-transport-security"));
}

// ============================================================================
// 6. Middleware ordering — request-id available to downstream middleware
// ============================================================================

#[tokio::test]
async fn test_middleware_ordering_request_id_available_to_all_layers() {
    let config = test_config();
    let app = build_full_middleware_app(&config);

    let mut headers = HeaderMap::new();
    headers.insert(
        "x-request-id",
        HeaderValue::from_static("ordering-test-xyz"),
    );

    let response = send_request(app, Method::GET, "/api/v1/documents", headers).await;

    assert_eq!(response.status(), StatusCode::OK);
    let rid = response
        .headers()
        .get("x-request-id")
        .unwrap()
        .to_str()
        .unwrap();
    assert_eq!(rid, "ordering-test-xyz");
    assert!(response.headers().contains_key("x-frame-options"));
    assert!(response.headers().contains_key("cache-control"));
}

// ============================================================================
// 7. Public vs protected route access (without auth middleware layer)
// ============================================================================

#[tokio::test]
async fn test_public_route_accessible_without_auth() {
    let config = test_config();
    let app = build_full_middleware_app(&config);

    let response = send_request(app, Method::GET, "/api/v1/auth/login", HeaderMap::new()).await;
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_health_accessible_without_auth() {
    let config = test_config();
    let app = build_full_middleware_app(&config);

    let response = send_request(app, Method::GET, "/health", HeaderMap::new()).await;
    assert_eq!(response.status(), StatusCode::OK);
}
