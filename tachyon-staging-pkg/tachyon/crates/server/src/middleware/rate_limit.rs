// Rate limiting middleware using token bucket algorithm
// Supports per-IP and per-user rate limiting with Redis backend for distributed systems

use axum::{
    Json,
    extract::{Request, State},
    http::{HeaderMap, StatusCode, Uri},
    middleware::Next,
    response::{IntoResponse, Response},
};
use dashmap::DashMap;
use deadpool_redis::{Pool as RedisPool, Runtime, redis::AsyncCommands};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tracing::warn;

static LAST_CLEANUP: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub enabled: bool,
    pub redis_url: Option<String>,
    pub default_requests_per_minute: u32,
    pub cleanup_interval_secs: u64,
    pub endpoint_limits: HashMap<String, RateLimit>,
}

impl From<&crate::config::RateLimitConfig> for RateLimitConfig {
    fn from(config: &crate::config::RateLimitConfig) -> Self {
        Self {
            enabled: config.enabled,
            redis_url: config.redis_url.clone(),
            default_requests_per_minute: config.default_requests_per_minute,
            cleanup_interval_secs: config.cleanup_interval_secs,
            endpoint_limits: config
                .endpoint_limits
                .iter()
                .map(|(k, v)| (k.clone(), RateLimit::new(v.max_requests, v.window_secs)))
                .collect(),
        }
    }
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
    if let Some(forwarded) = headers.get("x-forwarded-for")
        && let Ok(forwarded_str) = forwarded.to_str()
        && let Some(first_ip) = forwarded_str.split(',').next()
    {
        return first_ip.trim().to_string();
    }

    if let Some(real_ip) = headers.get("x-real-ip")
        && let Ok(ip_str) = real_ip.to_str()
    {
        return ip_str.to_string();
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

    fn cleanup(&self, window_secs: u64) {
        let cutoff = Instant::now() - Duration::from_secs(window_secs * 2);
        self.buckets
            .retain(|_, bucket| bucket.last_refill >= cutoff);
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
            .unwrap_or(std::time::Duration::ZERO)
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
            let now_secs = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or(std::time::Duration::ZERO)
                .as_secs();
            let reset = limit.window_secs - (now_secs - window_start);
            Ok(RateLimitInfo {
                limit: limit.max_requests,
                remaining: remaining as u32,
                reset,
                retry_after: None,
            })
        } else {
            let now_secs = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or(std::time::Duration::ZERO)
                .as_secs();
            let retry_after = limit.window_secs - (now_secs - window_start);
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

fn insert_rate_limit_headers(headers: &mut HeaderMap, info: &RateLimitInfo) {
    let now_unix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs();
    let reset_unix = now_unix + info.reset;

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
        axum::http::HeaderValue::from_str(&reset_unix.to_string())
            .unwrap_or_else(|_| axum::http::HeaderValue::from_static("0")),
    );

    if let Some(retry_after) = info.retry_after {
        headers.insert(
            "Retry-After",
            axum::http::HeaderValue::from_str(&retry_after.to_string())
                .unwrap_or_else(|_| axum::http::HeaderValue::from_static("0")),
        );
    }
}

pub async fn rate_limit_middleware(
    State(state): State<RateLimitState>,
    request: Request,
    next: Next,
) -> Response {
    if !state.config.enabled {
        return next.run(request).await;
    }

    if let RateLimitStore::InMemory(store) = &state.store {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs();
        let last = LAST_CLEANUP.load(Ordering::Relaxed);
        if now.saturating_sub(last) > 60 {
            LAST_CLEANUP.store(now, Ordering::Relaxed);
            store.cleanup(state.config.cleanup_interval_secs);
        }
    }

    let path = request.uri().path();
    let headers = request.headers();

    let user_id = request
        .extensions()
        .get::<crate::middleware::auth::AuthContext>()
        .map(|ctx| ctx.user_id.as_str());

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

                let body = Json(RateLimitErrorResponse {
                    error: "RATE_LIMIT_EXCEEDED".to_string(),
                    message: format!("Rate limit exceeded. Try again in {} seconds.", retry_after),
                    retry_after,
                });
                let mut response = (StatusCode::TOO_MANY_REQUESTS, body).into_response();
                insert_rate_limit_headers(response.headers_mut(), &info);
                return response;
            }

            let mut response = next.run(request).await;
            insert_rate_limit_headers(response.headers_mut(), &info);
            response
        }
        Err(_) => next.run(request).await,
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

// ============================================================================
// Brute-Force Protection: Login Attempt Tracker
// ============================================================================

/// Tracks failed login attempts per IP address with progressive backoff.
///
/// After exceeding the threshold, further login attempts from that IP are blocked
/// for an increasing duration. Successful logins reset the counter.
///
/// Thresholds:
/// - 5 failures: 1 minute lockout
/// - 10 failures: 5 minute lockout
/// - 20 failures: 15 minute lockout
/// - 50 failures: 1 hour lockout
/// - 100+ failures: 24 hour lockout
///
/// Counters decay after the lockout period expires.
#[derive(Debug, Clone)]
pub struct LoginAttemptTracker {
    /// Maps IP address → (failed_count, first_failure_timestamp)
    attempts: Arc<DashMap<String, (u32, Instant)>>,
}

/// Lockout tiers: (failure_threshold, lockout_duration_seconds)
const LOCKOUT_TIERS: &[(u32, u64)] = &[
    (5, 60),      // 5 failures → 1 min
    (10, 300),    // 10 failures → 5 min
    (20, 900),    // 20 failures → 15 min
    (50, 3600),   // 50 failures → 1 hour
    (100, 86400), // 100 failures → 24 hours
];

impl LoginAttemptTracker {
    pub fn new() -> Self {
        Self {
            attempts: Arc::new(DashMap::new()),
        }
    }

    /// Record a failed login attempt from the given IP.
    /// Returns the lockout duration in seconds if the IP is now locked out, or `None` if allowed.
    pub fn record_failure(&self, ip: &str) -> Option<u64> {
        let mut entry = self
            .attempts
            .entry(ip.to_string())
            .or_insert_with(|| (0, Instant::now()));

        entry.0 += 1;
        // Reset timestamp on each failure within the same window
        entry.1 = Instant::now();

        self.calculate_lockout(entry.0)
    }

    /// Record a successful login from the given IP. Resets the failure counter.
    pub fn record_success(&self, ip: &str) {
        self.attempts.remove(ip);
    }

    /// Check if the given IP is currently locked out.
    /// Returns the remaining lockout time in seconds if locked, or `None` if allowed.
    pub fn check_lockout(&self, ip: &str) -> Option<u64> {
        let entry = self.attempts.get(ip)?;
        let (count, first_failure) = entry.value();

        let lockout_secs = self.calculate_lockout(*count)?;
        let elapsed = first_failure.elapsed().as_secs();

        if elapsed >= lockout_secs {
            // Lockout expired — clean up
            drop(entry);
            self.attempts.remove(ip);
            None
        } else {
            Some(lockout_secs - elapsed)
        }
    }

    /// Calculate the lockout duration based on failure count.
    fn calculate_lockout(&self, count: u32) -> Option<u64> {
        let mut lockout_secs = 0u64;
        for &(threshold, duration) in LOCKOUT_TIERS {
            if count >= threshold {
                lockout_secs = duration;
            }
        }
        if lockout_secs > 0 {
            Some(lockout_secs)
        } else {
            None
        }
    }

    /// Clean up expired entries. Call periodically (e.g., every 60s).
    pub fn cleanup(&self) {
        let max_lockout = LOCKOUT_TIERS.last().map(|(_, d)| *d).unwrap_or(86400);
        self.attempts
            .retain(|_: &String, (_, first_failure): &mut (u32, Instant)| {
                first_failure.elapsed().as_secs() < max_lockout
            });
    }

    /// Get the current failure count for an IP (for monitoring/debugging).
    pub fn failure_count(&self, ip: &str) -> u32 {
        self.attempts
            .get(ip)
            .map(|e: dashmap::mapref::one::Ref<String, (u32, Instant)>| e.value().0)
            .unwrap_or(0)
    }
}

impl Default for LoginAttemptTracker {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Per-User Rate Limiter: Higher limits for authenticated users
// ============================================================================

/// Per-user rate limiter that provides higher limits for authenticated users
/// compared to anonymous (per-IP) requests. Works alongside the existing
/// per-IP limiter.
///
/// Configurable via environment variables:
/// - `TACHYON_RATE_LIMIT_AUTHENTICATED_RPM`: Requests per minute for authenticated users (default: 3000)
/// - `TACHYON_RATE_LIMIT_ANONYMOUS_RPM`: Requests per minute for anonymous users (default: 1000)
/// - `TACHYON_RATE_LIMIT_USER_WINDOW_SECS`: Window in seconds (default: 60)
#[derive(Debug, Clone)]
pub struct UserRateLimiter {
    buckets: Arc<DashMap<String, TokenBucket>>,
    authenticated_rpm: u32,
    anonymous_rpm: u32,
    window_secs: u64,
    endpoint_overrides: Arc<HashMap<String, RateLimit>>,
}

impl Default for UserRateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

impl UserRateLimiter {
    pub fn new() -> Self {
        let authenticated_rpm: u32 = std::env::var("TACHYON_RATE_LIMIT_AUTHENTICATED_RPM")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(3000);

        let anonymous_rpm: u32 = std::env::var("TACHYON_RATE_LIMIT_ANONYMOUS_RPM")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(1000);

        let window_secs: u64 = std::env::var("TACHYON_RATE_LIMIT_USER_WINDOW_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(60);

        let endpoint_overrides = Self::load_endpoint_overrides();

        Self {
            buckets: Arc::new(DashMap::new()),
            authenticated_rpm,
            anonymous_rpm,
            window_secs,
            endpoint_overrides: Arc::new(endpoint_overrides),
        }
    }

    fn load_endpoint_overrides() -> HashMap<String, RateLimit> {
        let mut overrides = HashMap::new();

        overrides.insert("/api/v1/auth/login".to_string(), RateLimit::new(5, 60));
        overrides.insert("/api/v1/auth/register".to_string(), RateLimit::new(3, 60));
        overrides.insert(
            "/api/v1/auth/password-reset".to_string(),
            RateLimit::new(3, 60),
        );
        overrides.insert("/health".to_string(), RateLimit::new(1000, 60));
        overrides.insert("/ready".to_string(), RateLimit::new(1000, 60));

        if let Ok(json_str) = std::env::var("TACHYON_RATE_LIMIT_ENDPOINTS")
            && let Ok(parsed) = serde_json::from_str::<EndpointRateLimitsJson>(&json_str)
        {
            for (path, cfg) in parsed {
                overrides.insert(path, RateLimit::new(cfg.max, cfg.window));
            }
        }

        overrides
    }

    pub fn with_endpoint_overrides(overrides: HashMap<String, RateLimit>) -> Self {
        let authenticated_rpm: u32 = std::env::var("TACHYON_RATE_LIMIT_AUTHENTICATED_RPM")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(3000);

        let anonymous_rpm: u32 = std::env::var("TACHYON_RATE_LIMIT_ANONYMOUS_RPM")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(1000);

        let window_secs: u64 = std::env::var("TACHYON_RATE_LIMIT_USER_WINDOW_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(60);

        Self {
            buckets: Arc::new(DashMap::new()),
            authenticated_rpm,
            anonymous_rpm,
            window_secs,
            endpoint_overrides: Arc::new(overrides),
        }
    }

    fn get_endpoint_limit(&self, path: &str) -> Option<RateLimit> {
        for (pattern, limit) in self.endpoint_overrides.iter() {
            if path.starts_with(pattern) || path == pattern {
                return Some(*limit);
            }
        }
        None
    }

    /// Check rate limit for a user. Pass `Some(user_id)` for authenticated users
    /// and `None` for anonymous requests.
    pub fn check(&self, user_id: Option<&str>) -> UserRateLimitResult {
        let key = match user_id {
            Some(uid) => format!("user:{}", uid),
            None => "anonymous".to_string(),
        };

        let rpm = if user_id.is_some() {
            self.authenticated_rpm
        } else {
            self.anonymous_rpm
        };

        let mut bucket = self
            .buckets
            .entry(key)
            .or_insert_with(|| TokenBucket::new(rpm, self.window_secs));

        if bucket.try_consume(1.0) {
            UserRateLimitResult::Allowed {
                remaining: bucket.tokens.floor() as u32,
                limit: rpm,
                authenticated: user_id.is_some(),
            }
        } else {
            let retry_after = bucket.time_to_next_token().as_secs();
            UserRateLimitResult::Throttled {
                retry_after,
                limit: rpm,
                authenticated: user_id.is_some(),
            }
        }
    }

    /// Check rate limit for a user against a specific endpoint path.
    /// If an endpoint-specific override exists, it is used; otherwise falls back
    /// to the global user/anonymous limit.
    pub fn check_for_endpoint(&self, user_id: Option<&str>, path: &str) -> UserRateLimitResult {
        if let Some(endpoint_limit) = self.get_endpoint_limit(path) {
            let key = match user_id {
                Some(uid) => format!("user:{}:{}", uid, path),
                None => format!("anonymous:{}", path),
            };

            let mut bucket = self.buckets.entry(key).or_insert_with(|| {
                TokenBucket::new(endpoint_limit.max_requests, endpoint_limit.window_secs)
            });

            if bucket.try_consume(1.0) {
                UserRateLimitResult::Allowed {
                    remaining: bucket.tokens.floor() as u32,
                    limit: endpoint_limit.max_requests,
                    authenticated: user_id.is_some(),
                }
            } else {
                let retry_after = bucket.time_to_next_token().as_secs();
                UserRateLimitResult::Throttled {
                    retry_after,
                    limit: endpoint_limit.max_requests,
                    authenticated: user_id.is_some(),
                }
            }
        } else {
            self.check(user_id)
        }
    }

    /// Clean up stale buckets. Call periodically (e.g., every 60s).
    pub fn cleanup(&self) {
        let cutoff =
            std::time::Instant::now() - std::time::Duration::from_secs(self.window_secs * 2);
        self.buckets
            .retain(|_, bucket| bucket.last_refill >= cutoff);
    }

    /// Get the configured RPM for authenticated users.
    pub fn authenticated_rpm(&self) -> u32 {
        self.authenticated_rpm
    }

    /// Get the configured RPM for anonymous users.
    pub fn anonymous_rpm(&self) -> u32 {
        self.anonymous_rpm
    }

    /// Get a reference to the endpoint overrides map.
    pub fn endpoint_overrides(&self) -> &HashMap<String, RateLimit> {
        &self.endpoint_overrides
    }
}

#[derive(Debug, Clone, Deserialize)]
struct EndpointRateConfig {
    max: u32,
    window: u64,
}

type EndpointRateLimitsJson = HashMap<String, EndpointRateConfig>;

#[derive(Debug, Clone)]
pub enum UserRateLimitResult {
    Allowed {
        remaining: u32,
        limit: u32,
        authenticated: bool,
    },
    Throttled {
        retry_after: u64,
        limit: u32,
        authenticated: bool,
    },
}

impl UserRateLimitResult {
    pub fn is_allowed(&self) -> bool {
        matches!(self, UserRateLimitResult::Allowed { .. })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointRateLimits {
    pub limits: HashMap<String, RateLimit>,
}

impl EndpointRateLimits {
    pub fn new() -> Self {
        Self {
            limits: HashMap::new(),
        }
    }

    pub fn with_defaults() -> Self {
        let mut limits = HashMap::new();
        limits.insert("/api/v1/auth/login".to_string(), RateLimit::new(5, 60));
        limits.insert("/api/v1/auth/register".to_string(), RateLimit::new(3, 60));
        limits.insert(
            "/api/v1/auth/password-reset".to_string(),
            RateLimit::new(3, 60),
        );
        limits.insert("/health".to_string(), RateLimit::new(1000, 60));
        limits.insert("/ready".to_string(), RateLimit::new(1000, 60));
        Self { limits }
    }

    pub fn insert(&mut self, path: impl Into<String>, limit: RateLimit) {
        self.limits.insert(path.into(), limit);
    }

    pub fn get_limit(&self, path: &str) -> Option<&RateLimit> {
        for (pattern, limit) in &self.limits {
            if path.starts_with(pattern) || path == pattern {
                return Some(limit);
            }
        }
        None
    }

    pub fn from_env() -> Self {
        let mut limits = Self::with_defaults();
        if let Ok(json_str) = std::env::var("TACHYON_RATE_LIMIT_ENDPOINTS")
            && let Ok(parsed) = serde_json::from_str::<EndpointRateLimitsJson>(&json_str)
        {
            for (path, cfg) in parsed {
                limits
                    .limits
                    .insert(path, RateLimit::new(cfg.max, cfg.window));
            }
        }
        limits
    }
}

impl Default for EndpointRateLimits {
    fn default() -> Self {
        Self::new()
    }
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

    // ── Login Attempt Tracker Tests ──────────────────────────────────────

    #[test]
    fn test_login_tracker_no_lockout_under_threshold() {
        let tracker = LoginAttemptTracker::new();

        // 4 failures should not trigger lockout
        for _ in 0..4 {
            assert!(
                tracker.record_failure("1.2.3.4").is_none(),
                "Under 5 failures should not lock out"
            );
        }
        assert_eq!(tracker.failure_count("1.2.3.4"), 4);
    }

    #[test]
    fn test_login_tracker_first_lockout_tier() {
        let tracker = LoginAttemptTracker::new();

        // First 4 failures should not lock out
        for _ in 0..4 {
            assert!(
                tracker.record_failure("1.2.3.4").is_none(),
                "Under 5 failures should not lock out"
            );
        }

        // 5th failure triggers 60s lockout
        let lockout = tracker.record_failure("1.2.3.4");
        assert_eq!(lockout, Some(60), "5 failures should trigger 60s lockout");
    }

    #[test]
    fn test_login_tracker_progressive_lockout() {
        let tracker = LoginAttemptTracker::new();

        // 10 failures → 300s
        for _ in 0..10 {
            tracker.record_failure("1.2.3.4");
        }
        let lockout = tracker.check_lockout("1.2.3.4");
        assert_eq!(
            lockout,
            Some(300),
            "10 failures should trigger 5min lockout"
        );

        // 20 failures → 900s
        for _ in 0..10 {
            tracker.record_failure("1.2.3.4");
        }
        let lockout = tracker.check_lockout("1.2.3.4");
        assert_eq!(
            lockout,
            Some(900),
            "20 failures should trigger 15min lockout"
        );
    }

    #[test]
    fn test_login_tracker_success_resets_counter() {
        let tracker = LoginAttemptTracker::new();

        for _ in 0..5 {
            tracker.record_failure("1.2.3.4");
        }
        assert!(tracker.check_lockout("1.2.3.4").is_some());

        tracker.record_success("1.2.3.4");
        assert!(
            tracker.check_lockout("1.2.3.4").is_none(),
            "Success should reset lockout"
        );
        assert_eq!(tracker.failure_count("1.2.3.4"), 0);
    }

    #[test]
    fn test_login_tracker_ip_isolation() {
        let tracker = LoginAttemptTracker::new();

        for _ in 0..10 {
            tracker.record_failure("1.2.3.4");
        }

        // Different IP should not be affected
        assert!(
            tracker.check_lockout("5.6.7.8").is_none(),
            "Different IP should not be locked out"
        );
    }

    #[test]
    fn test_login_tracker_unknown_ip_no_lockout() {
        let tracker = LoginAttemptTracker::new();
        assert!(
            tracker.check_lockout("never-seen-ip").is_none(),
            "Unknown IP should not be locked out"
        );
    }

    #[test]
    fn test_login_tracker_cleanup() {
        let tracker = LoginAttemptTracker::new();

        tracker.record_failure("1.2.3.4");
        assert_eq!(tracker.attempts.len(), 1);

        // Cleanup should not remove entries that haven't expired
        tracker.cleanup();
        assert_eq!(tracker.attempts.len(), 1);
    }

    // ── User Rate Limiter Tests ──────────────────────────────────────────

    #[test]
    fn test_user_rate_limiter_allows_authenticated() {
        let limiter = UserRateLimiter::new();
        let result = limiter.check(Some("user-123"));
        assert!(result.is_allowed());
        if let UserRateLimitResult::Allowed { authenticated, .. } = result {
            assert!(authenticated);
        } else {
            panic!("Expected Allowed");
        }
    }

    #[test]
    fn test_user_rate_limiter_allows_anonymous() {
        let limiter = UserRateLimiter::new();
        let result = limiter.check(None);
        assert!(result.is_allowed());
        if let UserRateLimitResult::Allowed { authenticated, .. } = result {
            assert!(!authenticated);
        } else {
            panic!("Expected Allowed");
        }
    }

    #[test]
    fn test_user_rate_limiter_authenticated_higher_limit() {
        let limiter = UserRateLimiter::new();
        assert!(
            limiter.authenticated_rpm() > limiter.anonymous_rpm(),
            "Authenticated users should have higher RPM than anonymous"
        );
    }

    #[test]
    fn test_user_rate_limiter_user_isolation() {
        let limiter = UserRateLimiter::new();

        // Exhaust anonymous limit
        for _ in 0..limiter.anonymous_rpm() {
            let result = limiter.check(None);
            assert!(result.is_allowed());
        }

        // Authenticated user should still be allowed
        let result = limiter.check(Some("user-456"));
        assert!(result.is_allowed());
    }

    #[test]
    fn test_user_rate_limiter_cleanup() {
        let limiter = UserRateLimiter::new();

        limiter.check(Some("user-cleanup-test"));
        assert_eq!(limiter.buckets.len(), 1);

        limiter.cleanup();
        assert_eq!(
            limiter.buckets.len(),
            1,
            "Fresh entry should not be cleaned up"
        );
    }

    #[test]
    fn test_endpoint_rate_limit_applies_tighter_limit() {
        let overrides = HashMap::from([("/api/v1/auth/login".to_string(), RateLimit::new(5, 60))]);
        let limiter = UserRateLimiter::with_endpoint_overrides(overrides);

        for _ in 0..5 {
            let result = limiter.check_for_endpoint(None, "/api/v1/auth/login");
            assert!(result.is_allowed());
        }

        let result = limiter.check_for_endpoint(None, "/api/v1/auth/login");
        assert!(
            !result.is_allowed(),
            "6th request to auth/login should be throttled"
        );
    }

    #[test]
    fn test_endpoint_fallback_to_global_limit() {
        let overrides = HashMap::from([("/api/v1/auth/login".to_string(), RateLimit::new(5, 60))]);
        let limiter = UserRateLimiter::with_endpoint_overrides(overrides);

        let result = limiter.check_for_endpoint(Some("user-abc"), "/api/v1/documents");
        assert!(result.is_allowed());
        if let UserRateLimitResult::Allowed { limit, .. } = result {
            assert_eq!(
                limit,
                limiter.authenticated_rpm(),
                "Should use global limit for unmatched path"
            );
        }
    }

    #[test]
    fn test_endpoint_rate_limit_per_path_isolation() {
        let overrides = HashMap::from([
            ("/api/v1/auth/login".to_string(), RateLimit::new(2, 60)),
            ("/api/v1/auth/register".to_string(), RateLimit::new(2, 60)),
        ]);
        let limiter = UserRateLimiter::with_endpoint_overrides(overrides);

        limiter.check_for_endpoint(None, "/api/v1/auth/login");
        limiter.check_for_endpoint(None, "/api/v1/auth/login");

        let login_result = limiter.check_for_endpoint(None, "/api/v1/auth/login");
        assert!(!login_result.is_allowed(), "Login should be throttled");

        let register_result = limiter.check_for_endpoint(None, "/api/v1/auth/register");
        assert!(
            register_result.is_allowed(),
            "Register should still be allowed (separate bucket)"
        );
    }

    #[test]
    fn test_endpoint_rate_limits_default_overrides() {
        let limiter = UserRateLimiter::new();
        assert!(
            limiter
                .endpoint_overrides()
                .contains_key("/api/v1/auth/login")
        );
        assert!(limiter.endpoint_overrides().contains_key("/health"));
        assert!(limiter.endpoint_overrides().contains_key("/ready"));
    }

    #[test]
    fn test_endpoint_rate_limits_health_relaxed() {
        let overrides = HashMap::from([("/health".to_string(), RateLimit::new(1000, 60))]);
        let limiter = UserRateLimiter::with_endpoint_overrides(overrides);

        for _ in 0..10 {
            let result = limiter.check_for_endpoint(None, "/health");
            assert!(
                result.is_allowed(),
                "Health endpoint should allow many requests"
            );
        }
    }
}
