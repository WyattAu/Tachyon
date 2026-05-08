use axum::{
    Router,
    body::Body,
    extract::Request,
    http::{HeaderMap, HeaderValue, Method, StatusCode},
    middleware::{from_fn, from_fn_with_state},
    response::Response,
    routing::get,
};
use std::collections::HashMap;
use tower::ServiceBuilder;
use tower::ServiceExt;

use crate::middleware::rate_limit::{RateLimitConfig, RateLimitState};
use crate::middleware::security_headers::{
    SecurityHeadersConfig, add_security_headers_with_config,
};
use crate::middleware::{
    add_security_headers_from_config, audit_middleware, cache_control_middleware,
    compression_layer, rate_limit_middleware, request_id_middleware, request_size_limit,
};

fn test_config() -> crate::config::ServerConfig {
    let mut config = crate::config::ServerConfig::default();
    config.jwt.secret = "a-sufficiently-long-secret-key-for-tests-32ch".to_string();
    config.security.development = true;
    config
}

fn health_handler() -> Router {
    Router::new().route("/health", get(|| async { "ok" }))
}

fn api_handler() -> Router {
    async fn list_docs() -> axum::Json<serde_json::Value> {
        axum::Json(serde_json::json!({"docs": []}))
    }
    Router::new().route("/api/v1/documents", get(list_docs))
}

fn build_test_app(config: &crate::config::ServerConfig) -> Router {
    let security_config = std::sync::Arc::new(config.security.clone());

    Router::new()
        .merge(health_handler())
        .merge(api_handler())
        .layer(
            ServiceBuilder::new()
                .layer(from_fn(request_id_middleware))
                .layer(from_fn(cache_control_middleware))
                .layer(from_fn(audit_middleware))
                .layer(from_fn(request_size_limit))
                .layer(compression_layer())
                .map_response(move |response: Response| {
                    add_security_headers_from_config(response, &security_config)
                }),
        )
}

fn build_rate_limit_app(
    config: &crate::config::ServerConfig,
    rate_limit_config: RateLimitConfig,
) -> Router {
    let security_config = std::sync::Arc::new(config.security.clone());
    let rate_limit_state = RateLimitState::new(rate_limit_config);

    Router::new()
        .merge(health_handler())
        .merge(api_handler())
        .layer(
            ServiceBuilder::new()
                .layer(from_fn(request_id_middleware))
                .layer(from_fn_with_state(rate_limit_state, rate_limit_middleware))
                .layer(from_fn(cache_control_middleware))
                .layer(from_fn(audit_middleware))
                .layer(from_fn(request_size_limit))
                .layer(compression_layer())
                .map_response(move |response: Response| {
                    add_security_headers_from_config(response, &security_config)
                }),
        )
}

async fn send_request(app: Router, method: Method, path: &str, headers: HeaderMap) -> Response {
    let req = Request::builder()
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

async fn send_request_with_body(
    app: Router,
    method: Method,
    path: &str,
    headers: HeaderMap,
    body: impl Into<Body>,
) -> Response {
    let req = Request::builder()
        .method(method)
        .uri(path)
        .header("host", "localhost")
        .header("connection", "close")
        .body(body.into())
        .unwrap();

    let mut req = req;
    for (name, value) in headers.iter() {
        req.headers_mut().insert(name, value.clone());
    }

    app.oneshot(req).await.unwrap()
}

// ============================================================================
// 1. Request ID Propagation
// ============================================================================

#[tokio::test]
async fn request_id_set_on_response() {
    let config = test_config();
    let app = build_test_app(&config);

    let response = send_request(app, Method::GET, "/health", HeaderMap::new()).await;

    assert_eq!(response.status(), StatusCode::OK);
    let request_id = response.headers().get("x-request-id");
    assert!(
        request_id.is_some(),
        "X-Request-Id header should be present"
    );
    let id = request_id.unwrap().to_str().unwrap();
    assert!(!id.is_empty(), "X-Request-Id should not be empty");
}

#[tokio::test]
async fn request_id_preserved_if_already_present() {
    let config = test_config();
    let app = build_test_app(&config);

    let mut headers = HeaderMap::new();
    headers.insert(
        "x-request-id",
        HeaderValue::from_static("my-custom-request-id-123"),
    );

    let response = send_request(app, Method::GET, "/health", headers).await;

    assert_eq!(response.status(), StatusCode::OK);
    let request_id = response
        .headers()
        .get("x-request-id")
        .unwrap()
        .to_str()
        .unwrap();
    assert_eq!(request_id, "my-custom-request-id-123");
}

#[tokio::test]
async fn request_id_unique_per_request() {
    let config = test_config();
    let app = build_test_app(&config);

    let r1 = send_request(app.clone(), Method::GET, "/health", HeaderMap::new()).await;
    let r2 = send_request(app, Method::GET, "/health", HeaderMap::new()).await;

    let id1 = r1
        .headers()
        .get("x-request-id")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let id2 = r2
        .headers()
        .get("x-request-id")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    assert_ne!(id1, id2, "Each request should get a unique X-Request-Id");
}

// ============================================================================
// 2. Security Headers
// ============================================================================

#[tokio::test]
async fn security_headers_present() {
    let config = test_config();
    let app = build_test_app(&config);

    let response = send_request(app, Method::GET, "/health", HeaderMap::new()).await;

    assert_eq!(response.status(), StatusCode::OK);

    let headers = response.headers();

    assert!(
        headers.contains_key("x-frame-options"),
        "X-Frame-Options header should be present"
    );
    let xfo = headers.get("x-frame-options").unwrap().to_str().unwrap();
    assert_eq!(xfo, "DENY");

    assert!(
        headers.contains_key("x-content-type-options"),
        "X-Content-Type-Options header should be present"
    );
    let xcto = headers
        .get("x-content-type-options")
        .unwrap()
        .to_str()
        .unwrap();
    assert_eq!(xcto, "nosniff");

    assert!(
        headers.contains_key("referrer-policy"),
        "Referrer-Policy header should be present"
    );
    let rp = headers.get("referrer-policy").unwrap().to_str().unwrap();
    assert_eq!(rp, "strict-origin-when-cross-origin");
}

#[tokio::test]
async fn security_headers_csp_present() {
    let config = test_config();
    let app = build_test_app(&config);

    let response = send_request(app, Method::GET, "/health", HeaderMap::new()).await;

    assert!(
        response.headers().contains_key("content-security-policy"),
        "Content-Security-Policy header should be present"
    );
    let csp = response
        .headers()
        .get("content-security-policy")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(
        csp.contains("default-src"),
        "CSP should contain default-src directive"
    );
}

#[tokio::test]
async fn security_headers_permissions_policy_present() {
    let config = test_config();
    let app = build_test_app(&config);

    let response = send_request(app, Method::GET, "/health", HeaderMap::new()).await;

    assert!(
        response.headers().contains_key("permissions-policy"),
        "Permissions-Policy header should be present"
    );
}

#[tokio::test]
async fn security_headers_no_hsts_in_dev() {
    let config = test_config();
    let app = build_test_app(&config);

    let response = send_request(app, Method::GET, "/health", HeaderMap::new()).await;

    assert!(
        !response.headers().contains_key("strict-transport-security"),
        "HSTS should not be present in development mode"
    );
}

// ============================================================================
// 3. CORS
// ============================================================================

#[tokio::test]
async fn cors_preflight_allowed() {
    let config = test_config();
    let security_config = std::sync::Arc::new(config.security.clone());
    let cors_layer = crate::build_cors_layer(&config);

    let app = Router::new()
        .route(
            "/api/v1/documents",
            get(|| async { "ok" }).options(|| async { "ok" }),
        )
        .layer(
            ServiceBuilder::new()
                .layer(from_fn(request_id_middleware))
                .layer(from_fn(cache_control_middleware))
                .layer(from_fn(request_size_limit))
                .layer(compression_layer())
                .layer(cors_layer)
                .map_response(move |response: Response| {
                    add_security_headers_from_config(response, &security_config)
                }),
        );

    let mut headers = HeaderMap::new();
    headers.insert("origin", HeaderValue::from_static("http://example.com"));
    headers.insert(
        "access-control-request-method",
        HeaderValue::from_static("GET"),
    );
    headers.insert(
        "access-control-request-headers",
        HeaderValue::from_static("content-type,authorization"),
    );

    let response = send_request(app, Method::OPTIONS, "/api/v1/documents", headers).await;

    assert_eq!(response.status(), StatusCode::OK);

    let resp_headers = response.headers();
    assert!(
        resp_headers.contains_key("access-control-allow-origin"),
        "CORS preflight should include Access-Control-Allow-Origin"
    );
    assert!(
        resp_headers.contains_key("access-control-allow-methods"),
        "CORS preflight should include Access-Control-Allow-Methods"
    );
    assert!(
        resp_headers.contains_key("access-control-allow-headers"),
        "CORS preflight should include Access-Control-Allow-Headers"
    );
}

#[tokio::test]
async fn cors_non_preflight_has_headers() {
    let config = test_config();
    let security_config = std::sync::Arc::new(config.security.clone());
    let cors_layer = crate::build_cors_layer(&config);

    let app = Router::new()
        .route("/api/v1/documents", get(|| async { "ok" }))
        .layer(
            ServiceBuilder::new()
                .layer(from_fn(request_id_middleware))
                .layer(from_fn(cache_control_middleware))
                .layer(from_fn(request_size_limit))
                .layer(compression_layer())
                .layer(cors_layer)
                .map_response(move |response: Response| {
                    add_security_headers_from_config(response, &security_config)
                }),
        );

    let mut headers = HeaderMap::new();
    headers.insert("origin", HeaderValue::from_static("http://example.com"));

    let response = send_request(app, Method::GET, "/api/v1/documents", headers).await;

    assert!(
        response
            .headers()
            .contains_key("access-control-allow-origin"),
        "Non-preflight request should include Access-Control-Allow-Origin"
    );
}

// ============================================================================
// 4. Rate Limiting
// ============================================================================

#[tokio::test]
async fn rate_limit_headers_present() {
    let config = test_config();
    let rate_limit_config = RateLimitConfig {
        enabled: true,
        redis_url: None,
        default_requests_per_minute: 100,
        cleanup_interval_secs: 60,
        endpoint_limits: HashMap::new(),
    };
    let app = build_rate_limit_app(&config, rate_limit_config);

    let response = send_request(app, Method::GET, "/api/v1/documents", HeaderMap::new()).await;

    assert_eq!(response.status(), StatusCode::OK);

    let headers = response.headers();
    assert!(
        headers.contains_key("x-ratelimit-limit"),
        "X-RateLimit-Limit header should be present"
    );
    assert!(
        headers.contains_key("x-ratelimit-remaining"),
        "X-RateLimit-Remaining header should be present"
    );
    assert!(
        headers.contains_key("x-ratelimit-reset"),
        "X-RateLimit-Reset header should be present"
    );
}

#[tokio::test]
async fn rate_limit_exceeded_returns_429() {
    let config = test_config();
    let rate_limit_config = RateLimitConfig {
        enabled: true,
        redis_url: None,
        default_requests_per_minute: 3,
        cleanup_interval_secs: 60,
        endpoint_limits: HashMap::new(),
    };
    let app = build_rate_limit_app(&config, rate_limit_config);

    for i in 0..3 {
        let response = send_request(
            app.clone(),
            Method::GET,
            "/api/v1/documents",
            HeaderMap::new(),
        )
        .await;
        assert_eq!(
            response.status(),
            StatusCode::OK,
            "Request {} should succeed",
            i + 1
        );
    }

    let response = send_request(app, Method::GET, "/api/v1/documents", HeaderMap::new()).await;
    assert_eq!(
        response.status(),
        StatusCode::TOO_MANY_REQUESTS,
        "Request exceeding rate limit should return 429"
    );
}

#[tokio::test]
async fn rate_limit_decrementing_remaining() {
    let config = test_config();
    let rate_limit_config = RateLimitConfig {
        enabled: true,
        redis_url: None,
        default_requests_per_minute: 10,
        cleanup_interval_secs: 60,
        endpoint_limits: HashMap::new(),
    };
    let app = build_rate_limit_app(&config, rate_limit_config);

    let r1 = send_request(
        app.clone(),
        Method::GET,
        "/api/v1/documents",
        HeaderMap::new(),
    )
    .await;
    let remaining1: u32 = r1
        .headers()
        .get("x-ratelimit-remaining")
        .unwrap()
        .to_str()
        .unwrap()
        .parse()
        .unwrap();

    let r2 = send_request(app, Method::GET, "/api/v1/documents", HeaderMap::new()).await;
    let remaining2: u32 = r2
        .headers()
        .get("x-ratelimit-remaining")
        .unwrap()
        .to_str()
        .unwrap()
        .parse()
        .unwrap();

    assert!(
        remaining1 > remaining2,
        "Remaining count should decrease: {} > {}",
        remaining1,
        remaining2
    );
}

#[tokio::test]
async fn rate_limit_applied_globally() {
    let config = test_config();
    let rate_limit_config = RateLimitConfig {
        enabled: true,
        redis_url: None,
        default_requests_per_minute: 2,
        cleanup_interval_secs: 60,
        endpoint_limits: HashMap::new(),
    };
    let app = build_rate_limit_app(&config, rate_limit_config);

    let r1 = send_request(app.clone(), Method::GET, "/health", HeaderMap::new()).await;
    assert_eq!(r1.status(), StatusCode::OK);

    let r2 = send_request(app.clone(), Method::GET, "/health", HeaderMap::new()).await;
    assert_eq!(r2.status(), StatusCode::OK);

    let r3 = send_request(app, Method::GET, "/health", HeaderMap::new()).await;
    assert_eq!(
        r3.status(),
        StatusCode::TOO_MANY_REQUESTS,
        "Third request should be rate limited"
    );
}

// ============================================================================
// 5. Cache Control
// ============================================================================

#[tokio::test]
async fn cache_control_api_response() {
    let config = test_config();
    let app = build_test_app(&config);

    let response = send_request(app, Method::GET, "/api/v1/documents", HeaderMap::new()).await;

    assert_eq!(response.status(), StatusCode::OK);
    let cache_header = response
        .headers()
        .get("cache-control")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(
        cache_header.contains("max-age=10"),
        "API response should have short max-age: got {}",
        cache_header
    );
    assert!(
        cache_header.contains("stale-while-revalidate"),
        "API response should include stale-while-revalidate"
    );
}

#[tokio::test]
async fn cache_control_non_get_no_cache() {
    let config = test_config();
    let app = build_test_app(&config);

    let response = send_request(app, Method::OPTIONS, "/api/v1/documents", HeaderMap::new()).await;

    let cache_header = response
        .headers()
        .get("cache-control")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(
        cache_header.contains("no-store"),
        "Non-GET response should have no-store: got {}",
        cache_header
    );
}

#[tokio::test]
async fn cache_control_health_endpoint() {
    let config = test_config();
    let app = build_test_app(&config);

    let response = send_request(app, Method::GET, "/health", HeaderMap::new()).await;

    let cache_header = response
        .headers()
        .get("cache-control")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(
        cache_header.contains("max-age"),
        "Health endpoint should be cacheable: got {}",
        cache_header
    );
}

#[tokio::test]
async fn cache_control_etag_present() {
    let config = test_config();
    let app = build_test_app(&config);

    let response = send_request(app, Method::GET, "/api/v1/documents", HeaderMap::new()).await;

    assert!(
        response.headers().contains_key("etag"),
        "GET response should include an ETag header"
    );
    let etag = response.headers().get("etag").unwrap().to_str().unwrap();
    assert!(etag.starts_with("W/\""), "ETag should be weak (W/...)");
}

// ============================================================================
// 6. Request Size Limit
// ============================================================================

#[tokio::test]
async fn request_size_limit_valid() {
    let config = test_config();
    let app = build_test_app(&config);

    let mut headers = HeaderMap::new();
    headers.insert("content-length", HeaderValue::from_static("100"));

    let response = send_request_with_body(
        app,
        Method::POST,
        "/api/v1/documents",
        headers,
        Body::from(vec![0u8; 100]),
    )
    .await;

    assert_ne!(
        response.status(),
        StatusCode::PAYLOAD_TOO_LARGE,
        "Request within size limit should not be rejected"
    );
}

#[tokio::test]
async fn request_size_limit_exceeded() {
    let config = test_config();
    let app = build_test_app(&config);

    let oversized = 11 * 1024 * 1024;
    let mut headers = HeaderMap::new();
    headers.insert(
        "content-length",
        HeaderValue::from_str(&oversized.to_string()).unwrap(),
    );

    let response = send_request_with_body(
        app,
        Method::POST,
        "/api/v1/documents",
        headers,
        Body::from(vec![0u8; 1024]),
    )
    .await;

    assert_eq!(
        response.status(),
        StatusCode::PAYLOAD_TOO_LARGE,
        "Oversized request should be rejected with 413"
    );
}

// ============================================================================
// 7. Full Pipeline
// ============================================================================

#[tokio::test]
async fn full_pipeline_all_headers_present() {
    let config = test_config();
    let app = build_test_app(&config);

    let response = send_request(app, Method::GET, "/api/v1/documents", HeaderMap::new()).await;

    assert_eq!(response.status(), StatusCode::OK);

    let headers = response.headers();

    assert!(
        headers.contains_key("x-request-id"),
        "Full pipeline: X-Request-Id missing"
    );
    assert!(
        headers.contains_key("x-frame-options"),
        "Full pipeline: X-Frame-Options missing"
    );
    assert!(
        headers.contains_key("x-content-type-options"),
        "Full pipeline: X-Content-Type-Options missing"
    );
    assert!(
        headers.contains_key("referrer-policy"),
        "Full pipeline: Referrer-Policy missing"
    );
    assert!(
        headers.contains_key("content-security-policy"),
        "Full pipeline: Content-Security-Policy missing"
    );
    assert!(
        headers.contains_key("cache-control"),
        "Full pipeline: Cache-Control missing"
    );
    assert!(headers.contains_key("etag"), "Full pipeline: ETag missing");
}

#[tokio::test]
async fn full_pipeline_order_request_id_before_security() {
    let config = test_config();
    let app = build_test_app(&config);

    let mut headers = HeaderMap::new();
    headers.insert("x-request-id", HeaderValue::from_static("order-test-123"));

    let response = send_request(app, Method::GET, "/api/v1/documents", headers).await;

    let rid = response
        .headers()
        .get("x-request-id")
        .unwrap()
        .to_str()
        .unwrap();
    assert_eq!(rid, "order-test-123");

    assert!(response.headers().contains_key("x-frame-options"));
    assert!(response.headers().contains_key("cache-control"));
}

// ============================================================================
// 8. Security Headers With Config
// ============================================================================

#[test]
fn security_headers_config_full() {
    let config = SecurityHeadersConfig::default();
    let response = Response::new(axum::body::Body::empty());
    let response = add_security_headers_with_config(response, &config);

    let headers = response.headers();
    assert!(headers.contains_key("content-security-policy"));
    assert!(headers.contains_key("x-frame-options"));
    assert!(headers.contains_key("x-content-type-options"));
    assert!(headers.contains_key("x-xss-protection"));
    assert!(headers.contains_key("strict-transport-security"));
    assert!(headers.contains_key("referrer-policy"));
    assert!(headers.contains_key("permissions-policy"));
    assert!(headers.contains_key("cross-origin-embedder-policy"));
    assert!(headers.contains_key("cross-origin-opener-policy"));
    assert!(headers.contains_key("cross-origin-resource-policy"));
}

#[test]
fn security_headers_config_no_sts() {
    let config = SecurityHeadersConfig {
        strict_transport_security: None,
        ..Default::default()
    };

    let response = Response::new(axum::body::Body::empty());
    let response = add_security_headers_with_config(response, &config);

    assert!(
        !response.headers().contains_key("strict-transport-security"),
        "STS should not be present when disabled in config"
    );
}

#[test]
fn security_headers_config_no_xss() {
    let config = SecurityHeadersConfig {
        x_xss_protection: false,
        ..Default::default()
    };

    let response = Response::new(axum::body::Body::empty());
    let response = add_security_headers_with_config(response, &config);

    assert!(
        !response.headers().contains_key("x-xss-protection"),
        "X-XSS-Protection should not be present when disabled"
    );
}
