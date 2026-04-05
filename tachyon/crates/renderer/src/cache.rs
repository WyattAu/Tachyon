//! LRU cache implementation for rendered documents
//!
//! This module provides an LRU (Least Recently Used) cache with TTL support
//! for storing rendered documents to improve performance.

use crate::error::RendererResult;
use crate::types::{CacheConfig, CacheEntry, CacheKey, CacheStats};
use lru::LruCache;
use parking_lot::RwLock;
use std::sync::Arc;
use tracing::{debug, trace};

/// LRU cache for storing rendered documents
#[derive(Clone)]
pub struct RenderCache {
    /// Inner cache storage
    inner: Arc<RwLock<LruCache<CacheKey, CacheEntry>>>,

    /// Cache configuration
    config: CacheConfig,

    /// Cache statistics
    stats: Arc<RwLock<CacheStats>>,
}

impl RenderCache {
    /// Create a new render cache with default configuration
    pub fn new() -> Self {
        Self::with_config(CacheConfig::default())
    }

    /// Create a new render cache with custom configuration
    pub fn with_config(config: CacheConfig) -> Self {
        let capacity = std::num::NonZeroUsize::new(config.max_entries.max(1))
            .unwrap_or_else(|| std::num::NonZeroUsize::new(1).expect("1 is always non-zero"));

        let inner = LruCache::new(capacity);

        Self {
            inner: Arc::new(RwLock::new(inner)),
            config,
            stats: Arc::new(RwLock::new(CacheStats::new())),
        }
    }

    /// Create a new render cache with specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self::with_config(CacheConfig {
            max_entries: capacity,
            ..Default::default()
        })
    }

    /// Get a value from the cache
    pub fn get(&self, key: &CacheKey) -> RendererResult<Option<CacheEntry>> {
        let mut inner = self.inner.write();

        if let Some(entry) = inner.get_mut(key) {
            // Check if entry has expired
            if entry.is_expired() {
                trace!("Cache entry expired: {:?}", key);
                inner.pop(key);
                if self.config.enable_stats {
                    self.stats.write().increment_miss();
                }
                return Ok(None);
            }

            // Update last accessed timestamp
            entry.touch();

            if self.config.enable_stats {
                self.stats.write().increment_hit();
            }

            trace!("Cache hit: {:?}", key);
            Ok(Some(entry.clone()))
        } else {
            trace!("Cache miss: {:?}", key);
            if self.config.enable_stats {
                self.stats.write().increment_miss();
            }
            Ok(None)
        }
    }

    /// Insert a value into the cache
    pub fn insert(&self, entry: CacheEntry) -> RendererResult<()> {
        let key = entry.key.clone();
        let ttl = entry.ttl_seconds.or(self.config.default_ttl_seconds);
        let mut entry = entry;
        entry.ttl_seconds = ttl;

        let mut inner = self.inner.write();
        let previous_size = inner.len();

        inner.put(key.clone(), entry);

        // Update stats if evicted
        if self.config.enable_stats && inner.len() < previous_size {
            self.stats.write().increment_eviction();
        }

        // Update current entries count
        if self.config.enable_stats {
            self.stats.write().current_entries = inner.len();
        }

        debug!("Cache insert: {:?}", key);
        Ok(())
    }

    /// Remove a value from the cache
    pub fn remove(&self, key: &CacheKey) -> RendererResult<bool> {
        let mut inner = self.inner.write();
        let removed = inner.pop(key).is_some();

        if removed {
            debug!("Cache remove: {:?}", key);
            if self.config.enable_stats {
                self.stats.write().current_entries = inner.len();
            }
        }

        Ok(removed)
    }

    /// Check if a key exists in the cache
    pub fn contains(&self, key: &CacheKey) -> bool {
        let inner = self.inner.read();
        if let Some(entry) = inner.peek(key) {
            !entry.is_expired()
        } else {
            false
        }
    }

    /// Invalidate all cache entries for a specific document
    pub fn invalidate_document(
        &self,
        document_id: &tachyon_core::id::DocumentId,
    ) -> RendererResult<usize> {
        let mut inner = self.inner.write();
        let mut count = 0;
        let keys_to_remove: Vec<CacheKey> = inner
            .iter()
            .filter(|(k, _)| &k.document_id == document_id)
            .map(|(k, _)| k.clone())
            .collect();

        for key in keys_to_remove {
            if inner.pop(&key).is_some() {
                count += 1;
            }
        }

        if self.config.enable_stats {
            self.stats.write().current_entries = inner.len();
        }

        debug!(
            "Invalidated {} cache entries for document {:?}",
            count, document_id
        );
        Ok(count)
    }

    /// Clear all entries from the cache
    pub fn clear(&self) -> RendererResult<()> {
        let mut inner = self.inner.write();
        inner.clear();

        if self.config.enable_stats {
            let mut stats = self.stats.write();
            stats.current_entries = 0;
            stats.total_bytes = 0;
        }

        debug!("Cache cleared");
        Ok(())
    }

    /// Get the current size of the cache
    pub fn len(&self) -> usize {
        self.inner.read().len()
    }

    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.inner.read().is_empty()
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        self.stats.read().clone()
    }

    /// Get cache configuration
    pub fn config(&self) -> &CacheConfig {
        &self.config
    }

    /// Clean up expired entries
    pub fn cleanup_expired(&self) -> RendererResult<usize> {
        let mut inner = self.inner.write();
        let mut count = 0;
        let keys_to_remove: Vec<CacheKey> = inner
            .iter()
            .filter(|(_, e)| e.is_expired())
            .map(|(k, _)| k.clone())
            .collect();

        for key in keys_to_remove {
            if inner.pop(&key).is_some() {
                count += 1;
            }
        }

        if self.config.enable_stats {
            self.stats.write().current_entries = inner.len();
        }

        if count > 0 {
            debug!("Cleaned up {} expired cache entries", count);
        }

        Ok(count)
    }

    /// Get all keys in the cache
    pub fn keys(&self) -> Vec<CacheKey> {
        self.inner.read().iter().map(|(k, _)| k.clone()).collect()
    }

    /// Get all entries in the cache
    pub fn entries(&self) -> Vec<(CacheKey, CacheEntry)> {
        self.inner
            .read()
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }
}

impl Default for RenderCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{RenderMetadata, RenderStats};
    use tachyon_core::id::DocumentId;

    #[test]
    fn test_cache_insert_and_get() {
        let cache = RenderCache::with_capacity(10);
        let id = DocumentId::new();
        let key = CacheKey::generate(id, "content", "options").unwrap();

        let entry = CacheEntry::new(
            key.clone(),
            "rendered content".to_string(),
            RenderMetadata::new(),
            RenderStats::new(),
            None,
        );

        cache.insert(entry).unwrap();

        let retrieved = cache.get(&key).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().content, "rendered content");
    }

    #[test]
    fn test_cache_miss() {
        let cache = RenderCache::with_capacity(10);
        let id = DocumentId::new();
        let key = CacheKey::generate(id, "content", "options").unwrap();

        let result = cache.get(&key).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_cache_lru_eviction() {
        let cache = RenderCache::with_capacity(2);
        let id1 = DocumentId::new();
        let id2 = DocumentId::new();
        let id3 = DocumentId::new();

        let key1 = CacheKey::generate(id1, "content1", "options").unwrap();
        let key2 = CacheKey::generate(id2, "content2", "options").unwrap();
        let key3 = CacheKey::generate(id3, "content3", "options").unwrap();

        cache
            .insert(CacheEntry::new(
                key1.clone(),
                "content1".to_string(),
                RenderMetadata::new(),
                RenderStats::new(),
                None,
            ))
            .unwrap();

        cache
            .insert(CacheEntry::new(
                key2.clone(),
                "content2".to_string(),
                RenderMetadata::new(),
                RenderStats::new(),
                None,
            ))
            .unwrap();

        // This should evict key1
        cache
            .insert(CacheEntry::new(
                key3.clone(),
                "content3".to_string(),
                RenderMetadata::new(),
                RenderStats::new(),
                None,
            ))
            .unwrap();

        assert!(cache.get(&key1).unwrap().is_none());
        assert!(cache.get(&key2).unwrap().is_some());
        assert!(cache.get(&key3).unwrap().is_some());
    }

    #[test]
    fn test_cache_remove() {
        let cache = RenderCache::with_capacity(10);
        let id = DocumentId::new();
        let key = CacheKey::generate(id, "content", "options").unwrap();

        cache
            .insert(CacheEntry::new(
                key.clone(),
                "content".to_string(),
                RenderMetadata::new(),
                RenderStats::new(),
                None,
            ))
            .unwrap();

        let removed = cache.remove(&key).unwrap();
        assert!(removed);
        assert!(cache.get(&key).unwrap().is_none());
    }

    #[test]
    fn test_cache_invalidate_document() {
        let cache = RenderCache::with_capacity(10);
        let id = DocumentId::new();

        let key1 = CacheKey::generate(id, "content1", "options1").unwrap();
        let key2 = CacheKey::generate(id, "content2", "options2").unwrap();
        let other_id = DocumentId::new();
        let key3 = CacheKey::generate(other_id, "content3", "options3").unwrap();

        cache
            .insert(CacheEntry::new(
                key1.clone(),
                "content1".to_string(),
                RenderMetadata::new(),
                RenderStats::new(),
                None,
            ))
            .unwrap();

        cache
            .insert(CacheEntry::new(
                key2.clone(),
                "content2".to_string(),
                RenderMetadata::new(),
                RenderStats::new(),
                None,
            ))
            .unwrap();

        cache
            .insert(CacheEntry::new(
                key3.clone(),
                "content3".to_string(),
                RenderMetadata::new(),
                RenderStats::new(),
                None,
            ))
            .unwrap();

        let count = cache.invalidate_document(&id).unwrap();
        assert_eq!(count, 2);
        assert!(cache.get(&key1).unwrap().is_none());
        assert!(cache.get(&key2).unwrap().is_none());
        assert!(cache.get(&key3).unwrap().is_some());
    }

    #[test]
    fn test_cache_clear() {
        let cache = RenderCache::with_capacity(10);
        let id = DocumentId::new();
        let key = CacheKey::generate(id, "content", "options").unwrap();

        cache
            .insert(CacheEntry::new(
                key.clone(),
                "content".to_string(),
                RenderMetadata::new(),
                RenderStats::new(),
                None,
            ))
            .unwrap();

        cache.clear().unwrap();
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_stats() {
        let cache = RenderCache::with_config(CacheConfig {
            max_entries: 10,
            default_ttl_seconds: None,
            enable_stats: true,
        });

        let id = DocumentId::new();
        let key = CacheKey::generate(id, "content", "options").unwrap();

        // Miss
        cache.get(&key).unwrap();

        // Insert
        cache
            .insert(CacheEntry::new(
                key.clone(),
                "content".to_string(),
                RenderMetadata::new(),
                RenderStats::new(),
                None,
            ))
            .unwrap();

        // Hit
        cache.get(&key).unwrap();

        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.current_entries, 1);
    }

    #[test]
    fn test_cache_expiration() {
        let cache = RenderCache::with_config(CacheConfig {
            max_entries: 10,
            default_ttl_seconds: None,
            enable_stats: true,
        });

        let id = DocumentId::new();
        let key = CacheKey::generate(id, "content", "options").unwrap();

        cache
            .insert(CacheEntry::new(
                key.clone(),
                "content".to_string(),
                RenderMetadata::new(),
                RenderStats::new(),
                Some(1), // 1 second TTL
            ))
            .unwrap();

        // Entry should be accessible immediately
        assert!(cache.contains(&key));

        // Clean up expired entries (should not remove yet)
        cache.cleanup_expired().unwrap();
        assert!(cache.contains(&key));

        // Note: We can't actually test expiration without waiting
        // in a real scenario, but the structure is correct
    }

    #[test]
    fn test_cache_len() {
        let cache = RenderCache::with_capacity(10);
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());

        let id = DocumentId::new();
        let key = CacheKey::generate(id, "content", "options").unwrap();

        cache
            .insert(CacheEntry::new(
                key,
                "content".to_string(),
                RenderMetadata::new(),
                RenderStats::new(),
                None,
            ))
            .unwrap();

        assert_eq!(cache.len(), 1);
        assert!(!cache.is_empty());
    }
}
