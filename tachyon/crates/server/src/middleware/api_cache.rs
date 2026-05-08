use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct ApiCache {
    entries: Arc<RwLock<BTreeMap<String, CacheEntry>>>,
    default_ttl: Duration,
    analytics: CacheAnalytics,
}

#[derive(Clone)]
struct CacheEntry {
    data: Vec<u8>,
    content_type: String,
    expires_at: Instant,
}

#[derive(Debug, Clone, Default)]
pub struct CacheAnalytics {
    pub hits: Arc<AtomicU64>,
    pub misses: Arc<AtomicU64>,
    pub evictions: Arc<AtomicU64>,
}

impl CacheAnalytics {
    pub fn new() -> Self {
        Self {
            hits: Arc::new(AtomicU64::new(0)),
            misses: Arc::new(AtomicU64::new(0)),
            evictions: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn record_hit(&self) {
        self.hits.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_miss(&self) {
        self.misses.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_eviction(&self) {
        self.evictions.fetch_add(1, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> (u64, u64, u64) {
        (
            self.hits.load(Ordering::Relaxed),
            self.misses.load(Ordering::Relaxed),
            self.evictions.load(Ordering::Relaxed),
        )
    }
}

impl ApiCache {
    pub fn new(default_ttl: Duration) -> Self {
        Self {
            entries: Arc::new(RwLock::new(BTreeMap::new())),
            default_ttl,
            analytics: CacheAnalytics::new(),
        }
    }

    pub fn get_analytics(&self) -> CacheAnalytics {
        self.analytics.clone()
    }

    pub async fn get(&self, key: &str) -> Option<(Vec<u8>, String)> {
        let entries = self.entries.read().await;
        match entries.get(key) {
            Some(entry) if Instant::now() < entry.expires_at => {
                self.analytics.record_hit();
                Some((entry.data.clone(), entry.content_type.clone()))
            }
            Some(_) => {
                self.analytics.record_miss();
                None
            }
            None => {
                self.analytics.record_miss();
                None
            }
        }
    }

    pub async fn set(&self, key: &str, data: Vec<u8>, content_type: String, ttl: Option<Duration>) {
        let mut entries = self.entries.write().await;
        entries.insert(
            key.to_string(),
            CacheEntry {
                data,
                content_type,
                expires_at: Instant::now() + ttl.unwrap_or(self.default_ttl),
            },
        );
    }

    pub async fn invalidate(&self, key: &str) {
        let mut entries = self.entries.write().await;
        if entries.remove(key).is_some() {
            self.analytics.record_eviction();
        }
    }

    pub async fn invalidate_prefix(&self, prefix: &str) {
        let mut entries = self.entries.write().await;
        let before = entries.len();
        entries.retain(|k, _| !k.starts_with(prefix));
        let removed = before.saturating_sub(entries.len());
        for _ in 0..removed {
            self.analytics.record_eviction();
        }
    }

    pub async fn cleanup(&self) {
        let mut entries = self.entries.write().await;
        let before = entries.len();
        entries.retain(|_, v| Instant::now() < v.expires_at);
        let removed = before.saturating_sub(entries.len());
        for _ in 0..removed {
            self.analytics.record_eviction();
        }
    }

    pub async fn len(&self) -> usize {
        self.entries.read().await.len()
    }

    pub async fn is_empty(&self) -> bool {
        self.entries.read().await.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_set_get() {
        let cache = ApiCache::new(Duration::from_secs(60));
        cache
            .set("key1", b"hello".to_vec(), "text/plain".to_string(), None)
            .await;
        let result = cache.get("key1").await;
        assert!(result.is_some());
        let (data, ct) = result.unwrap();
        assert_eq!(data, b"hello");
        assert_eq!(ct, "text/plain");
    }

    #[tokio::test]
    async fn test_cache_miss() {
        let cache = ApiCache::new(Duration::from_secs(60));
        assert!(cache.get("nonexistent").await.is_none());
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let cache = ApiCache::new(Duration::from_millis(10));
        cache
            .set("key1", b"data".to_vec(), "text/plain".to_string(), None)
            .await;
        tokio::time::sleep(Duration::from_millis(20)).await;
        assert!(cache.get("key1").await.is_none());
    }

    #[tokio::test]
    async fn test_cache_invalidate() {
        let cache = ApiCache::new(Duration::from_secs(60));
        cache
            .set("key1", b"data".to_vec(), "text/plain".to_string(), None)
            .await;
        cache.invalidate("key1").await;
        assert!(cache.get("key1").await.is_none());
    }

    #[tokio::test]
    async fn test_cache_invalidate_prefix() {
        let cache = ApiCache::new(Duration::from_secs(60));
        cache
            .set("api:docs:1", b"a".to_vec(), "text/plain".to_string(), None)
            .await;
        cache
            .set("api:docs:2", b"b".to_vec(), "text/plain".to_string(), None)
            .await;
        cache
            .set("api:users:1", b"c".to_vec(), "text/plain".to_string(), None)
            .await;
        cache.invalidate_prefix("api:docs:").await;
        assert!(cache.get("api:docs:1").await.is_none());
        assert!(cache.get("api:docs:2").await.is_none());
        assert!(cache.get("api:users:1").await.is_some());
    }

    #[tokio::test]
    async fn test_cache_cleanup() {
        let cache = ApiCache::new(Duration::from_millis(10));
        cache
            .set(
                "expired",
                b"x".to_vec(),
                "text/plain".to_string(),
                Some(Duration::from_millis(5)),
            )
            .await;
        cache
            .set(
                "valid",
                b"y".to_vec(),
                "text/plain".to_string(),
                Some(Duration::from_secs(60)),
            )
            .await;
        tokio::time::sleep(Duration::from_millis(15)).await;
        cache.cleanup().await;
        assert!(cache.get("expired").await.is_none());
        assert!(cache.get("valid").await.is_some());
    }
}
