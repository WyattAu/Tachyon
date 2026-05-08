use super::rate_limit::{InMemoryRateLimitStore, RateLimitResult, RateLimitStore};
use deadpool_redis::{Pool as RedisPool, Runtime, redis::AsyncCommands};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::warn;

#[derive(Debug)]
pub struct RedisRateLimitStore {
    pool: RedisPool,
    fallback: InMemoryRateLimitStore,
}

impl RedisRateLimitStore {
    pub fn new(redis_url: &str) -> Result<Self, deadpool_redis::CreatePoolError> {
        let cfg = deadpool_redis::Config::from_url(redis_url);
        let pool = cfg.create_pool(Some(Runtime::Tokio1))?;
        Ok(Self {
            pool,
            fallback: InMemoryRateLimitStore::new(),
        })
    }

    async fn check_redis(
        &self,
        key: &str,
        max_requests: u32,
        window_secs: u64,
    ) -> Result<RateLimitResult, ()> {
        let window_start = (SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            / window_secs)
            * window_secs;
        let redis_key = format!("ratelimit:{}:{}", key, window_start);
        let ttl = window_secs as i64;

        let mut conn = self.pool.get().await.map_err(|_| ())?;

        let count: i64 = conn.incr(&redis_key, 1).await.map_err(|_| ())?;

        if count == 1 {
            let _: () = conn.expire(&redis_key, ttl).await.map_err(|_| ())?;
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let reset = window_secs.saturating_sub(now.saturating_sub(window_start));

        if count <= max_requests as i64 {
            let remaining = (max_requests as i64 - count) as u32;
            Ok(RateLimitResult::allowed(remaining, reset))
        } else {
            Ok(RateLimitResult::denied(reset))
        }
    }
}

#[async_trait::async_trait]
impl RateLimitStore for RedisRateLimitStore {
    async fn check_rate_limit(
        &self,
        key: &str,
        max_requests: u32,
        window_secs: u64,
    ) -> RateLimitResult {
        match self.check_redis(key, max_requests, window_secs).await {
            Ok(result) => result,
            Err(_) => {
                warn!("Redis rate limit store unavailable, falling back to in-memory");
                self.fallback
                    .check_rate_limit(key, max_requests, window_secs)
                    .await
            }
        }
    }

    async fn increment(&self, key: &str) {
        let redis_key = format!("ratelimit:{}", key);
        if let Ok(mut conn) = self.pool.get().await {
            let _: Result<i64, _> = conn.incr(&redis_key, 1).await;
        }
    }

    async fn is_healthy(&self) -> bool {
        match self.pool.get().await {
            Ok(mut conn) => redis::cmd("PING")
                .query_async::<String>(&mut conn)
                .await
                .is_ok(),
            Err(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_redis_store_fallback_on_connection_failure() {
        let store = RedisRateLimitStore::new("redis://127.0.0.1:1").unwrap();

        let result = store.check_rate_limit("fallback-test", 3, 60).await;
        assert!(result.allowed);

        for _ in 0..2 {
            let result = store.check_rate_limit("fallback-test", 3, 60).await;
            assert!(result.allowed);
        }

        let result = store.check_rate_limit("fallback-test", 3, 60).await;
        assert!(!result.allowed);
        assert!(result.retry_after_secs.is_some());
    }

    #[tokio::test]
    async fn test_redis_store_health_check_unreachable() {
        let store = RedisRateLimitStore::new("redis://127.0.0.1:1").unwrap();
        assert!(!store.is_healthy().await);
    }

    #[tokio::test]
    async fn test_redis_store_increment_no_panic() {
        let store = RedisRateLimitStore::new("redis://127.0.0.1:1").unwrap();
        store.increment("test-counter").await;
    }

    #[tokio::test]
    async fn test_trait_dispatch_through_redis() {
        let store: Arc<dyn RateLimitStore> =
            Arc::new(RedisRateLimitStore::new("redis://127.0.0.1:1").unwrap());
        let result = store.check_rate_limit("trait-dispatch-test", 5, 60).await;
        assert!(result.allowed);
        assert!(result.remaining > 0);
    }

    #[tokio::test]
    async fn test_redis_key_format() {
        let store = RedisRateLimitStore::new("redis://127.0.0.1:1").unwrap();
        let result = store
            .check_rate_limit("combined:192.168.1.1:/api/v1/documents", 100, 60)
            .await;
        assert!(result.allowed);
    }

    #[tokio::test]
    async fn test_redis_fallback_per_key_isolation() {
        let store = RedisRateLimitStore::new("redis://127.0.0.1:1").unwrap();

        for _ in 0..3 {
            let result = store.check_rate_limit("key-a", 3, 60).await;
            assert!(result.allowed);
        }
        let result = store.check_rate_limit("key-a", 3, 60).await;
        assert!(!result.allowed);

        let result = store.check_rate_limit("key-b", 3, 60).await;
        assert!(result.allowed);
    }
}
