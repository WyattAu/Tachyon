// Rate limiting middleware using token bucket algorithm
// Supports per-IP and per-user rate limiting with Redis backend for distributed systems

use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode, Uri},
    middleware::Next,
    response::Response,
    Json,
};
use deadpool_redis::{redis::AsyncCommands, Pool as RedisPool, Runtime};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::warn;

#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub enabled: bool,
    pub redis_url: Option<String>,
    pub default_requests_per_minute: u32,
    pub cleanup_interval_secs: u64,
    pub endpoint_limits: HashMap<String, RateLimit>,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        let mut endpoint_limits = HashMap::new();

        endpoint_limits.insert("/api/v1/auth/login".to_string(), RateLimit::new(5, 60));
        endpoint_limits.insert("/api/v1/auth/guest".to_string(), RateLimit::new(3, 60));
        endpoint_limits.insert("/api/v1/documents".to_string(), RateLimit::new(100, 60));

        Self {
            enabled: true,
            redis_url: None,
            default_requests_per_minute: 1000,
            cleanup_interval_secs: 60,
            endpoint_limits,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RateLimit {
    pub max_requests: u32,
    pub window_secs: u64,
}

impl RateLimit {
    pub fn new(max_requests: u32, window_secs: u64) -> Self {
        Self {
            max_requests,
            window_secs,
        }
    }

    pub fn requests_per_minute(rpm: u32) -> Self {
        Self {
            max_requests: rpm,
            window_secs: 60,
        }
    }
}

#[derive(Debug, Clone)]
struct TokenBucket {
    tokens: f64,
    max_tokens: f64,
    refill_rate: f64,
    last_refill: Instant,
}

impl TokenBucket {
    fn new(max_tokens: u32, window_secs: u64) -> Self {
        let refill_rate = max_tokens as f64 / window_secs as f64;
        Self {
            tokens: max_tokens as f64,
            max_tokens: max_tokens as f64,
            refill_rate,
            last_refill: Instant::now(),
        }
    }

    fn try_consume(&mut self, tokens: f64) -> bool {
        self.refill();
        if self.tokens >= tokens {
            self.tokens -= tokens;
            true
        } else {
            false
        }
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.max_tokens);
        self.last_refill = now;
    }

    fn time_to_next_token(&self) -> Duration {
        if self.tokens >= 1.0 {
            Duration::ZERO
        } else {
            let needed = 1.0 - self.tokens;
            Duration::from_secs_f64(needed / self.refill_rate)
        }
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum RateLimitKey {
    Ip(String),
    User(String),
    Endpoint(String),
    Combined { ip: String, endpoint: String },
}

impl RateLimitKey {
    pub fn from_request(headers: &HeaderMap, uri: &Uri, user_id: Option<&str>) -> Self {
        let ip = extract_client_ip(headers);
        let path = uri.path().to_string();

        if let Some(uid) = user_id {
            RateLimitKey::User(uid.to_string())
        } else {
            RateLimitKey::Combined { ip, endpoint: path }
        }
    }
}

fn extract_client_ip(headers: &HeaderMap) -> String {
    if let Some(forwarded) = headers.get("x-forwarded-for") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            if let Some(first_ip) = forwarded_str.split(',').next() {
                return first_ip.trim().to_string();
            }
        }
    }

    if let Some(real_ip) = headers.get("x-real-ip") {
        if let Ok(ip_str) = real_ip.to_str() {
            return ip_str.to_string();
        }
    }

    "unknown".to_string()
}

#[derive(Debug, Clone, Serialize)]
pub struct RateLimitInfo {
    pub limit: u32,
    pub remaining: u32,
    pub reset: u64,
    pub retry_after: Option<u64>,
}

#[derive(Debug)]
struct InMemoryStore {
    buckets: dashmap::DashMap<String, TokenBucket>,
}

impl InMemoryStore {
    fn new() -> Self {
        Self {
            buckets: dashmap::DashMap::new(),
        }
    }

    async fn check_rate_limit(&self, key: &str, limit: RateLimit) -> Result<RateLimitInfo, ()> {
        let mut bucket = self
            .buckets
            .entry(key.to_string())
            .or_insert_with(|| TokenBucket::new(limit.max_requests, limit.window_secs));

        if bucket.try_consume(1.0) {
            let remaining = bucket.tokens.floor() as u32;
            let reset = bucket.time_to_next_token().as_secs();

            Ok(RateLimitInfo {
                limit: limit.max_requests,
                remaining,
                reset,
                retry_after: None,
            })
        } else {
            let retry_after = bucket.time_to_next_token().as_secs();

            Ok(RateLimitInfo {
                limit: limit.max_requests,
                remaining: 0,
                reset: retry_after,
                retry_after: Some(retry_after),
            })
        }
    }
}

#[derive(Debug)]
struct RedisStore {
    pool: RedisPool,
}

impl RedisStore {
    fn new(redis_url: &str) -> Result<Self, deadpool_redis::CreatePoolError> {
        let cfg = deadpool_redis::Config::from_url(redis_url);
        let pool = cfg.create_pool(Some(Runtime::Tokio1))?;
        Ok(Self { pool })
    }

    async fn check_rate_limit(&self, key: &str, limit: RateLimit) -> Result<RateLimitInfo, ()> {
        let window_start = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            / limit.window_secs)
            * limit.window_secs;
        let redis_key = format!("rate_limit:{}:{}", key, window_start);
        let ttl = limit.window_secs as i64;

        let mut conn = self.pool.get().await.map_err(|_| ())?;

        let count: i64 = conn.incr(&redis_key, 1).await.map_err(|_| ())?;

        if count == 1 {
            let _: () = conn.expire(&redis_key, ttl).await.map_err(|_| ())?;
        }

        let remaining = if count > limit.max_requests as i64 {
            0
        } else {
            limit.max_requests as i64 - count
        };

        if count <= limit.max_requests as i64 {
            let reset = limit.window_secs
                - (std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    - window_start);
            Ok(RateLimitInfo {
                limit: limit.max_requests,
                remaining: remaining as u32,
                reset,
                retry_after: None,
            })
        } else {
            let retry_after = limit.window_secs
                - (std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    - window_start);
            Ok(RateLimitInfo {
                limit: limit.max_requests,
                remaining: 0,
                reset: retry_after,
                retry_after: Some(retry_after),
            })
        }
    }
}

#[derive(Clone)]
pub(crate) enum RateLimitStore {
    #[allow(private_interfaces)]
    InMemory(Arc<InMemoryStore>),
    #[allow(private_interfaces)]
    Redis(Arc<RedisStore>),
}

impl RateLimitStore {
    pub fn in_memory() -> Self {
        RateLimitStore::InMemory(Arc::new(InMemoryStore::new()))
    }

    pub fn redis(redis_url: &str) -> Result<Self, deadpool_redis::CreatePoolError> {
        Ok(RateLimitStore::Redis(Arc::new(RedisStore::new(redis_url)?)))
    }

    async fn check(&self, key: &str, limit: RateLimit) -> Result<RateLimitInfo, ()> {
        match self {
            RateLimitStore::InMemory(store) => store.check_rate_limit(key, limit).await,
            RateLimitStore::Redis(store) => store.check_rate_limit(key, limit).await,
        }
    }
}

#[derive(Clone)]
pub struct RateLimitState {
    config: RateLimitConfig,
    store: RateLimitStore,
}

impl RateLimitState {
    pub fn new(config: RateLimitConfig) -> Self {
        let store = if let Some(redis_url) = &config.redis_url {
            match RateLimitStore::redis(redis_url) {
                Ok(store) => store,
                Err(e) => {
                    tracing::warn!(
                        "Failed to create Redis rate limiter pool: {}, falling back to in-memory",
                        e
                    );
                    RateLimitStore::in_memory()
                }
            }
        } else {
            RateLimitStore::in_memory()
        };
        Self { config, store }
    }

    pub fn in_memory() -> Self {
        Self {
            config: RateLimitConfig::default(),
            store: RateLimitStore::in_memory(),
        }
    }

    fn get_limit_for_path(&self, path: &str) -> RateLimit {
        for (pattern, limit) in &self.config.endpoint_limits {
            if path.starts_with(pattern) || path == pattern {
                return *limit;
            }
        }

        RateLimit::requests_per_minute(self.config.default_requests_per_minute)
    }
}

#[derive(Debug, Serialize)]
pub struct RateLimitErrorResponse {
    pub error: String,
    pub message: String,
    pub retry_after: u64,
}

pub async fn rate_limit_middleware(
    State(state): State<RateLimitState>,
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<RateLimitErrorResponse>)> {
    if !state.config.enabled {
        return Ok(next.run(request).await);
    }

    let path = request.uri().path();
    let headers = request.headers();

    let user_id = None;

    let limit = state.get_limit_for_path(path);
    let key = RateLimitKey::from_request(headers, request.uri(), user_id);
    let key_str = match key {
        RateLimitKey::Ip(ip) => format!("ip:{}", ip),
        RateLimitKey::User(uid) => format!("user:{}", uid),
        RateLimitKey::Endpoint(ep) => format!("endpoint:{}", ep),
        RateLimitKey::Combined { ip, endpoint } => format!("combined:{}:{}", ip, endpoint),
    };

    match state.store.check(&key_str, limit).await {
        Ok(info) => {
            if let Some(retry_after) = info.retry_after {
                warn!(
                    path = %path,
                    key = %key_str,
                    retry_after = retry_after,
                    "Rate limit exceeded"
                );

                return Err((
                    StatusCode::TOO_MANY_REQUESTS,
                    Json(RateLimitErrorResponse {
                        error: "RATE_LIMIT_EXCEEDED".to_string(),
                        message: format!(
                            "Rate limit exceeded. Try again in {} seconds.",
                            retry_after
                        ),
                        retry_after,
                    }),
                ));
            }

            let response = next.run(request).await;

            let mut response = response;
            let headers = response.headers_mut();

            headers.insert(
                "X-RateLimit-Limit",
                axum::http::HeaderValue::from_str(&info.limit.to_string())
                    .unwrap_or_else(|_| axum::http::HeaderValue::from_static("0")),
            );
            headers.insert(
                "X-RateLimit-Remaining",
                axum::http::HeaderValue::from_str(&info.remaining.to_string())
                    .unwrap_or_else(|_| axum::http::HeaderValue::from_static("0")),
            );
            headers.insert(
                "X-RateLimit-Reset",
                axum::http::HeaderValue::from_str(&info.reset.to_string())
                    .unwrap_or_else(|_| axum::http::HeaderValue::from_static("0")),
            );

            Ok(response)
        }
        Err(_) => Ok(next.run(request).await),
    }
}

pub fn configure_endpoint_rate_limit(
    path: &str,
    max_requests: u32,
    window_secs: u64,
) -> (String, RateLimit) {
    (path.to_string(), RateLimit::new(max_requests, window_secs))
}

pub struct RateLimitRule {
    pub path_pattern: String,
    pub max_requests: usize,
    pub window_secs: u64,
}

pub fn default_rules() -> Vec<RateLimitRule> {
    vec![
        RateLimitRule {
            path_pattern: "/api/auth/login".to_string(),
            max_requests: 5,
            window_secs: 60,
        },
        RateLimitRule {
            path_pattern: "/api/auth/register".to_string(),
            max_requests: 3,
            window_secs: 60,
        },
        RateLimitRule {
            path_pattern: "/api/auth/password-reset".to_string(),
            max_requests: 3,
            window_secs: 3600,
        },
        RateLimitRule {
            path_pattern: "/api/*".to_string(),
            max_requests: 100,
            window_secs: 60,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_bucket_creation() {
        let bucket = TokenBucket::new(10, 60);
        assert_eq!(bucket.tokens, 10.0);
        assert_eq!(bucket.max_tokens, 10.0);
    }

    #[test]
    fn test_token_bucket_consume() {
        let mut bucket = TokenBucket::new(5, 60);

        assert!(bucket.try_consume(1.0));
        assert!((bucket.tokens - 4.0).abs() < 0.01);

        assert!(bucket.try_consume(3.0));
        assert!((bucket.tokens - 1.0).abs() < 0.01);

        assert!(bucket.try_consume(1.0));
        assert!(bucket.tokens < 0.01);

        assert!(!bucket.try_consume(1.0));
    }

    #[test]
    fn test_rate_limit_key_extraction() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", "192.168.1.1, 10.0.0.1".parse().unwrap());

        let uri: Uri = "/api/v1/documents".parse().unwrap();
        let key = RateLimitKey::from_request(&headers, &uri, None);

        match key {
            RateLimitKey::Combined { ip, endpoint } => {
                assert_eq!(ip, "192.168.1.1");
                assert_eq!(endpoint, "/api/v1/documents");
            }
            _ => panic!("Expected Combined key"),
        }
    }

    #[tokio::test]
    async fn test_in_memory_store() {
        let store = InMemoryStore::new();
        let limit = RateLimit::new(5, 60);

        for _ in 0..5 {
            let result = store.check_rate_limit("test-key", limit).await;
            assert!(result.is_ok());
            let info = result.unwrap();
            assert!(info.retry_after.is_none());
        }

        let result = store.check_rate_limit("test-key", limit).await;
        assert!(result.is_ok());
        let info = result.unwrap();
        assert!(info.retry_after.is_some());
    }
}
