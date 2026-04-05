// Authorization Cache Module
// High-performance authorization caching with TTL support

use crate::error::{RbacError, RbacResult};
use crate::types::{AccessDecision, Effect, Resource, Subject};
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

// ============================================================================
// Cache Entry
// ============================================================================

/// Cache entry with expiration support
#[derive(Debug, Clone)]
struct CacheEntry {
    /// Cached decision
    decision: AccessDecision,
    /// Cache TTL in seconds
    ttl: u64,
    /// Cached at timestamp
    cached_at: chrono::DateTime<chrono::Utc>,
}

impl CacheEntry {
    /// Create a new cache entry
    ///
    /// # Arguments
    /// * `decision` - Access decision to cache
    /// * `ttl` - Time to live in seconds
    ///
    /// # Returns
    /// New CacheEntry instance
    pub fn new(decision: AccessDecision, ttl: u64) -> Self {
        Self {
            decision,
            ttl,
            cached_at: chrono::Utc::now(),
        }
    }

    /// Check if the entry is expired
    ///
    /// # Returns
    /// True if entry is expired
    pub fn is_expired(&self) -> bool {
        let elapsed = chrono::Utc::now()
            .signed_duration_since(self.cached_at)
            .num_seconds()
            .abs() as u64;

        elapsed >= self.ttl
    }
}

// ============================================================================
// Authorization Cache
// ============================================================================

/// High-performance authorization cache with TTL support
#[derive(Debug)]
pub struct AuthorizationCache {
    /// Cache storage
    cache: DashMap<String, CacheEntry>,
    /// Maximum cache size
    max_size: usize,
    /// Current cache size
    current_size: Arc<RwLock<usize>>,
    /// Cache statistics
    stats: Arc<RwLock<CacheStats>>,
}

/// Cache statistics
#[derive(Debug, Clone, Default)]
struct CacheStats {
    /// Total cache hits
    pub hits: usize,
    /// Total cache misses
    pub misses: usize,
    /// Total evictions
    pub evictions: usize,
    /// Total invalidations
    pub invalidations: usize,
}

impl AuthorizationCache {
    /// Create a new authorization cache
    ///
    /// # Arguments
    /// * `max_size` - Maximum cache size
    ///
    /// # Returns
    /// New AuthorizationCache instance
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: DashMap::new(),
            max_size,
            current_size: Arc::new(RwLock::new(0)),
            stats: Arc::new(RwLock::new(CacheStats::default())),
        }
    }

    /// Get a cached decision
    ///
    /// # Arguments
    /// * `key` - Cache key
    ///
    /// # Returns
    /// Option containing the cached decision
    pub fn get(&self, key: &str) -> Option<AccessDecision> {
        if let Some(entry) = self.cache.get(key) {
            if entry.is_expired() {
                self.record_miss();
                self.invalidate(key);
                return None;
            }

            self.record_hit();
            Some(entry.decision.clone())
        } else {
            self.record_miss();
            None
        }
    }

    /// Insert a decision into the cache
    ///
    /// # Arguments
    /// * `key` - Cache key
    /// * `decision` - Access decision to cache
    pub fn insert(&self, key: String, decision: AccessDecision) {
        let ttl = decision.cache_ttl.unwrap_or(300); // Default 5 minutes
        let key_str = key.clone();
        let entry = CacheEntry::new(decision, ttl);

        // Check cache size and evict if necessary
        self.ensure_capacity();

        self.cache.insert(key, entry);

        // Update size
        let mut size_guard = self.current_size.blocking_write();
        *size_guard += 1;

        tracing::debug!("Cache insert: key={}, ttl={}s", key_str, ttl);
    }

    /// Check if a key exists in cache
    ///
    /// # Arguments
    /// * `key` - Cache key
    ///
    /// # Returns
    /// True if key exists
    pub fn contains(&self, key: &str) -> bool {
        self.cache.contains_key(key)
    }

    /// Invalidate a cache entry
    ///
    /// # Arguments
    /// * `key` - Cache key
    pub fn invalidate(&self, key: &str) {
        if self.cache.remove(key).is_some() {
            let mut size_guard = self.current_size.blocking_write();
            if *size_guard > 0 {
                *size_guard -= 1;
            }

            self.record_invalidation();
            tracing::debug!("Cache invalidate: key={}", key);
        }
    }

    /// Invalidate cache entries for a subject
    ///
    /// # Arguments
    /// * `subject` - Subject
    pub fn invalidate_subject(&self, subject: &Subject) {
        let prefix = format!("{}:", subject);
        let mut evicted = 0;

        self.cache.retain(|k, _| {
            let keep = !k.starts_with(&prefix);
            if !keep {
                evicted += 1;
            }
            keep
        });

        // Update size
        let mut size_guard = self.current_size.blocking_write();
        *size_guard -= evicted;

        // Update stats
        let mut stats_guard = self.stats.blocking_write();
        stats_guard.invalidations += evicted;

        tracing::debug!(
            "Cache invalidate subject: subject={}, evicted={}",
            subject,
            evicted
        );
    }

    /// Invalidate cache entries for a resource
    ///
    /// # Arguments
    /// * `resource` - Resource
    pub fn invalidate_resource(&self, resource: &Resource) {
        let prefix = format!("{}:{}:", resource.resource_type, resource.resource_id);
        let mut evicted = 0;

        self.cache.retain(|k, _| {
            let keep = !k.contains(&prefix);
            if !keep {
                evicted += 1;
            }
            keep
        });

        // Update size
        let mut size_guard = self.current_size.blocking_write();
        *size_guard -= evicted;

        // Update stats
        let mut stats_guard = self.stats.blocking_write();
        stats_guard.invalidations += evicted;

        tracing::debug!(
            "Cache invalidate resource: resource={}, evicted={}",
            resource,
            evicted
        );
    }

    /// Clear all cache entries
    pub fn clear(&self) {
        let size = self.cache.len();
        self.cache.clear();

        // Update size
        let mut size_guard = self.current_size.blocking_write();
        *size_guard = 0;

        // Update stats
        let mut stats_guard = self.stats.blocking_write();
        stats_guard.evictions += size;

        tracing::debug!("Cache clear: evicted={}", size);
    }

    /// Get current cache size
    ///
    /// # Returns
    /// Current number of entries in cache
    pub fn size(&self) -> usize {
        *self.current_size.blocking_read()
    }

    /// Get cache statistics
    ///
    /// # Returns
    /// Cache statistics
    pub fn get_stats(&self) -> CacheStats {
        self.stats.blocking_read().clone()
    }

    /// Reset cache statistics
    pub fn reset_stats(&self) {
        let mut stats_guard = self.stats.blocking_write();
        *stats_guard = CacheStats::default();
    }

    /// Ensure cache capacity by evicting entries when at capacity
    fn ensure_capacity(&self) {
        let current_size = *self.current_size.blocking_read();

        if current_size >= self.max_size {
            // First, try to evict expired entries
            let mut evicted = 0;
            let now = chrono::Utc::now();
            let target_evictions = (self.max_size / 2).max(1); // Evict at least 1, up to half

            self.cache.retain(|_k, v| {
                // Evict if expired OR if we haven't evicted enough yet
                let elapsed = now.signed_duration_since(v.cached_at).num_seconds().abs() as u64;
                let is_expired = elapsed >= v.ttl;

                // Keep if: not expired AND (we've evicted enough OR entry is new)
                let should_evict = is_expired || evicted < target_evictions;
                
                if should_evict {
                    evicted += 1;
                    false // remove this entry
                } else {
                    true // keep this entry
                }
            });

            // Update size
            let mut size_guard = self.current_size.blocking_write();
            *size_guard = self.cache.len();

            // Update stats
            let mut stats_guard = self.stats.blocking_write();
            stats_guard.evictions += evicted;

            if evicted > 0 {
                tracing::debug!("Cache evict: evicted={} entries", evicted);
            }
        }
    }

    /// Record a cache hit
    fn record_hit(&self) {
        let mut stats_guard = self.stats.blocking_write();
        stats_guard.hits += 1;
    }

    /// Record a cache miss
    fn record_miss(&self) {
        let mut stats_guard = self.stats.blocking_write();
        stats_guard.misses += 1;
    }

    /// Record a cache invalidation
    fn record_invalidation(&self) {
        let mut stats_guard = self.stats.blocking_write();
        stats_guard.invalidations += 1;
    }

    /// Calculate cache hit rate
    ///
    /// # Returns
    /// Hit rate as a percentage (0.0 to 1.0)
    pub fn hit_rate(&self) -> f64 {
        let stats = self.get_stats();
        let total = stats.hits + stats.misses;

        if total == 0 {
            0.0
        } else {
            stats.hits as f64 / total as f64
        }
    }
}

impl Default for AuthorizationCache {
    fn default() -> Self {
        Self::new(10000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Action;

    #[test]
    fn test_cache_entry_creation() {
        let decision = AccessDecision::new(Effect::Allow, "Test decision");
        let entry = CacheEntry::new(decision.clone(), 60);

        assert_eq!(entry.decision, decision);
        assert_eq!(entry.ttl, 60);
        assert!(!entry.is_expired());
    }

    #[test]
    fn test_cache_entry_expiration() {
        let decision = AccessDecision::new(Effect::Allow, "Test decision");
        let entry = CacheEntry::new(decision, 0); // 0 seconds = expired

        assert!(entry.is_expired());
    }

    #[test]
    fn test_authorization_cache() {
        let cache = AuthorizationCache::new(10);

        let key = "test_key".to_string();
        let decision = AccessDecision::new(Effect::Allow, "Test decision");

        // Insert
        cache.insert(key.clone(), decision.clone());
        assert_eq!(cache.size(), 1);

        // Get
        let retrieved = cache.get(&key);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().effect, Effect::Allow);

        // Check stats
        let stats = cache.get_stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 0);
        assert_eq!(cache.hit_rate(), 1.0);
    }

    #[test]
    fn test_cache_miss() {
        let cache = AuthorizationCache::new(10);

        let key = "test_key".to_string();

        // Get non-existent key
        let retrieved = cache.get(&key);
        assert!(retrieved.is_none());

        // Check stats
        let stats = cache.get_stats();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 1);
        assert_eq!(cache.hit_rate(), 0.0);
    }

    #[test]
    fn test_cache_invalidation() {
        let cache = AuthorizationCache::new(10);

        let key = "test_key".to_string();
        let decision = AccessDecision::new(Effect::Allow, "Test decision");

        // Insert
        cache.insert(key.clone(), decision.clone());
        assert_eq!(cache.size(), 1);

        // Invalidate
        cache.invalidate(&key);
        assert_eq!(cache.size(), 0);

        // Check stats
        let stats = cache.get_stats();
        assert_eq!(stats.invalidations, 1);
    }

    #[test]
    fn test_cache_clear() {
        let cache = AuthorizationCache::new(10);

        // Insert multiple entries
        for i in 0..5 {
            let decision = AccessDecision::new(Effect::Allow, &format!("Decision {}", i));
            cache.insert(format!("key{}", i), decision);
        }

        assert_eq!(cache.size(), 5);

        // Clear
        cache.clear();
        assert_eq!(cache.size(), 0);

        // Check stats
        let stats = cache.get_stats();
        assert_eq!(stats.evictions, 5);
    }

    #[test]
    fn test_subject_invalidation() {
        let cache = AuthorizationCache::new(10);

        let subject = Subject::new("user", "user123");
        let resource = Resource::new("document", "doc123");
        let action = Action::new("read");

        // Insert entries for same subject
        for i in 0..3 {
            let decision = AccessDecision::new(Effect::Allow, &format!("Decision {}", i));
            let key = format!("{}:{}:{}", subject, resource.resource_id, i);
            cache.insert(key, decision);
        }

        assert_eq!(cache.size(), 3);

        // Invalidate subject
        cache.invalidate_subject(&subject);
        assert_eq!(cache.size(), 0);
    }

    #[test]
    fn test_resource_invalidation() {
        let cache = AuthorizationCache::new(10);

        let subject1 = Subject::new("user", "user1");
        let subject2 = Subject::new("user", "user2");
        let resource = Resource::new("document", "doc123");
        let action = Action::new("read");

        // Insert entries for same resource
        // Key format: "subject:resource_type:resource_id:action"
        for subject in &[&subject1, &subject2] {
            let decision = AccessDecision::new(Effect::Allow, "Test decision");
            let key = format!("{}:{}:{}:{}", subject, resource.resource_type, resource.resource_id, action.action_name);
            cache.insert(key, decision);
        }

        assert_eq!(cache.size(), 2);

        // Invalidate resource
        cache.invalidate_resource(&resource);
        assert_eq!(cache.size(), 0);
    }

    #[test]
    fn test_cache_capacity() {
        let cache = AuthorizationCache::new(3);

        // Insert 3 entries
        for i in 0..3 {
            let decision = AccessDecision::new(Effect::Allow, "Test decision");
            cache.insert(format!("key{}", i), decision);
        }

        assert_eq!(cache.size(), 3);

        // Insert 4th entry (should evict)
        let decision = AccessDecision::new(Effect::Allow, "Test decision");
        cache.insert("key3".to_string(), decision);

        // Size should be at most max_size
        assert!(cache.size() <= 3);

        // Check stats
        let stats = cache.get_stats();
        assert!(stats.evictions >= 1);
    }
}
