use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
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

pub fn cache_key(method: &str, path: &str, query: Option<&str>) -> String {
    match query {
        Some(q) if !q.is_empty() => format!("{}:{}?{}", method, path, q),
        _ => format!("{}:{}", method, path),
    }
}

pub struct CacheHit {
    pub data: Vec<u8>,
    pub content_type: String,
}

impl ApiCache {
    pub async fn get_response(&self, key: &str) -> Option<CacheHit> {
        let (data, content_type) = self.get(key).await?;
        Some(CacheHit { data, content_type })
    }

    pub async fn set_response(
        &self,
        key: &str,
        data: Vec<u8>,
        content_type: &str,
        ttl: Option<Duration>,
    ) {
        self.set(key, data, content_type.to_string(), ttl).await;
    }

    pub async fn invalidate_documents(&self) {
        self.invalidate_prefix("GET:/api/v1/documents").await;
    }

    pub async fn invalidate_catalog(&self) {
        self.invalidate_prefix("GET:/api/v1/catalog").await;
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

    #[test]
    fn test_cache_key_with_query() {
        assert_eq!(
            cache_key("GET", "/api/v1/documents", Some("page=1&limit=10")),
            "GET:/api/v1/documents?page=1&limit=10"
        );
    }

    #[test]
    fn test_cache_key_without_query() {
        assert_eq!(
            cache_key("GET", "/api/v1/documents", None),
            "GET:/api/v1/documents"
        );
    }

    #[test]
    fn test_cache_key_empty_query() {
        assert_eq!(
            cache_key("GET", "/api/v1/documents", Some("")),
            "GET:/api/v1/documents"
        );
    }

    #[tokio::test]
    async fn test_get_response_cache_hit() {
        let cache = ApiCache::new(Duration::from_secs(60));
        cache
            .set(
                "test:key",
                b"hello".to_vec(),
                "text/plain".to_string(),
                None,
            )
            .await;

        let hit = cache.get_response("test:key").await.unwrap();
        assert_eq!(hit.data, b"hello");
        assert_eq!(hit.content_type, "text/plain");
    }

    #[tokio::test]
    async fn test_get_response_cache_miss() {
        let cache = ApiCache::new(Duration::from_secs(60));
        assert!(cache.get_response("nonexistent").await.is_none());
    }

    #[tokio::test]
    async fn test_set_response() {
        let cache = ApiCache::new(Duration::from_secs(60));
        cache
            .set_response("test:set", b"world".to_vec(), "application/json", None)
            .await;

        let hit = cache.get_response("test:set").await.unwrap();
        assert_eq!(hit.data, b"world");
        assert_eq!(hit.content_type, "application/json");
    }

    #[tokio::test]
    async fn test_invalidate_documents() {
        let cache = ApiCache::new(Duration::from_secs(60));
        cache
            .set(
                "GET:/api/v1/documents",
                b"[]".to_vec(),
                "application/json".to_string(),
                None,
            )
            .await;
        cache
            .set(
                "GET:/api/v1/documents?page=2",
                b"[]".to_vec(),
                "application/json".to_string(),
                None,
            )
            .await;
        cache
            .set(
                "GET:/api/v1/users",
                b"[]".to_vec(),
                "application/json".to_string(),
                None,
            )
            .await;

        cache.invalidate_documents().await;

        assert!(cache.get("GET:/api/v1/documents").await.is_none());
        assert!(cache.get("GET:/api/v1/documents?page=2").await.is_none());
        assert!(cache.get("GET:/api/v1/users").await.is_some());
    }

    #[tokio::test]
    async fn test_invalidate_catalog() {
        let cache = ApiCache::new(Duration::from_secs(60));
        cache
            .set(
                "GET:/api/v1/catalog/stats",
                b"{}".to_vec(),
                "application/json".to_string(),
                None,
            )
            .await;
        cache
            .set(
                "GET:/api/v1/documents",
                b"[]".to_vec(),
                "application/json".to_string(),
                None,
            )
            .await;

        cache.invalidate_catalog().await;

        assert!(cache.get("GET:/api/v1/catalog/stats").await.is_none());
        assert!(cache.get("GET:/api/v1/documents").await.is_some());
    }
}
