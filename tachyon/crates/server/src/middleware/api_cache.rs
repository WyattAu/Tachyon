use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct ApiCache {
    entries: Arc<RwLock<BTreeMap<String, CacheEntry>>>,
    default_ttl: Duration,
}

#[derive(Clone)]
struct CacheEntry {
    data: Vec<u8>,
    content_type: String,
    expires_at: Instant,
    /// Reserved for future use: cache hit statistics.
    #[allow(dead_code)]
    hit_count: u64,
}

impl ApiCache {
    pub fn new(default_ttl: Duration) -> Self {
        Self {
            entries: Arc::new(RwLock::new(BTreeMap::new())),
            default_ttl,
        }
    }

    pub async fn get(&self, key: &str) -> Option<(Vec<u8>, String)> {
        let entries = self.entries.read().await;
        entries.get(key).and_then(|entry| {
            if Instant::now() < entry.expires_at {
                Some((entry.data.clone(), entry.content_type.clone()))
            } else {
                None
            }
        })
    }

    pub async fn set(&self, key: &str, data: Vec<u8>, content_type: String, ttl: Option<Duration>) {
        let mut entries = self.entries.write().await;
        entries.insert(
            key.to_string(),
            CacheEntry {
                data,
                content_type,
                expires_at: Instant::now() + ttl.unwrap_or(self.default_ttl),
                hit_count: 0,
            },
        );
    }

    pub async fn invalidate(&self, key: &str) {
        let mut entries = self.entries.write().await;
        entries.remove(key);
    }

    pub async fn invalidate_prefix(&self, prefix: &str) {
        let mut entries = self.entries.write().await;
        entries.retain(|k, _| !k.starts_with(prefix));
    }

    pub async fn cleanup(&self) {
        let mut entries = self.entries.write().await;
        entries.retain(|_, v| Instant::now() < v.expires_at);
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
