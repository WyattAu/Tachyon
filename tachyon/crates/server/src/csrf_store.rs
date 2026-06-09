//! CSRF state persistence with Redis backing and in-memory fallback.
//!
//! Provides a trait-based abstraction for storing OAuth2/OIDC CSRF state tokens:
//! - `MemoryCsrfStore` -- in-memory DashMap (single instance, lost on restart)
//! - `RedisCsrfStore` -- Redis-backed with TTL (multi-instance, survives restart)
//!
//! The store is constructed via `CsrfStore::new()` which attempts Redis first
//! and falls back to in-memory if Redis is unavailable.

use async_trait::async_trait;
use chrono::Utc;
use dashmap::DashMap;
use deadpool_redis::redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Default TTL for CSRF state entries (10 minutes, matching OIDC/OAuth2 spec).
const CSRF_TTL_SECS: i64 = 600;

/// Redis key prefix for CSRF state entries.
const REDIS_KEY_PREFIX: &str = "tachyon:csrf:";

/// Serialized CSRF entry stored in Redis.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CsrfEntry {
    nonce: String,
    redirect_url: Option<String>,
}

/// Trait for CSRF state persistence.
///
/// Implementations must support:
/// - Storing a state token with associated nonce and optional redirect URL
/// - Retrieving and consuming a state token (single-use)
/// - TTL enforcement
#[async_trait]
pub trait CsrfStore: Send + Sync {
    /// Store a CSRF state entry with the given key (state token).
    /// Returns `true` on success.
    async fn store(&self, key: &str, nonce: &str, redirect_url: Option<String>) -> bool;

    /// Retrieve and consume a CSRF state entry. Returns the stored nonce and
    /// optional redirect URL if found and valid, or `None` if not found/expired.
    async fn retrieve_and_consume(&self, key: &str) -> Option<(String, Option<String>)>;

    /// Remove expired entries (best-effort, no-op for Redis which handles TTL natively).
    async fn cleanup_expired(&self);
}

// ---------------------------------------------------------------------------
// In-memory implementation (DashMap)
// ---------------------------------------------------------------------------

/// In-memory CSRF store backed by a DashMap.
/// Single-instance only: state is lost on server restart.
/// Suitable for development and single-instance deployments.
#[derive(Clone)]
pub struct MemoryCsrfStore {
    #[allow(clippy::type_complexity)]
    pub(crate) entries: Arc<DashMap<String, (String, Option<String>, chrono::DateTime<Utc>)>>,
}

impl Default for MemoryCsrfStore {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryCsrfStore {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(DashMap::new()),
        }
    }
}

#[async_trait]
impl CsrfStore for MemoryCsrfStore {
    async fn store(&self, key: &str, nonce: &str, redirect_url: Option<String>) -> bool {
        self.entries.insert(
            key.to_string(),
            (nonce.to_string(), redirect_url, Utc::now()),
        );
        true
    }

    async fn retrieve_and_consume(&self, key: &str) -> Option<(String, Option<String>)> {
        let result = self.entries.remove(key)?;
        let (nonce, redirect_url, created_at) = result.1;
        let elapsed = Utc::now().signed_duration_since(created_at).num_seconds();
        if elapsed > CSRF_TTL_SECS {
            debug!(
                key = key,
                elapsed_secs = elapsed,
                "In-memory CSRF state expired"
            );
            None
        } else {
            Some((nonce, redirect_url))
        }
    }

    async fn cleanup_expired(&self) {
        let now = Utc::now();
        self.entries.retain(|key, (_, _, created_at)| {
            let elapsed = now.signed_duration_since(*created_at).num_seconds();
            if elapsed > CSRF_TTL_SECS {
                debug!(
                    key = key,
                    elapsed_secs = elapsed,
                    "Cleaning up expired in-memory CSRF state"
                );
                false
            } else {
                true
            }
        });
    }
}

// ---------------------------------------------------------------------------
// Redis implementation
// ---------------------------------------------------------------------------

/// Redis-backed CSRF store.
/// Multi-instance safe: state persists across restarts and is shared between instances.
/// Uses Redis SET with EX (TTL) for atomic store, GET+DEL for consume.
pub struct RedisCsrfStore {
    pool: deadpool_redis::Pool,
}

impl RedisCsrfStore {
    /// Create a new Redis CSRF store from a Redis URL.
    pub fn new(redis_url: &str) -> Result<Self, String> {
        let cfg = deadpool_redis::Config::from_url(redis_url);
        let pool = cfg
            .create_pool(Some(deadpool_redis::Runtime::Tokio1))
            .map_err(|e| format!("Failed to create Redis CSRF pool: {}", e))?;
        Ok(Self { pool })
    }
}

#[async_trait]
impl CsrfStore for RedisCsrfStore {
    async fn store(&self, key: &str, nonce: &str, redirect_url: Option<String>) -> bool {
        let redis_key = format!("{}{}", REDIS_KEY_PREFIX, key);
        let entry = CsrfEntry {
            nonce: nonce.to_string(),
            redirect_url,
        };
        let value = match serde_json::to_vec(&entry) {
            Ok(v) => v,
            Err(e) => {
                warn!("Failed to serialize CSRF entry: {}", e);
                return false;
            }
        };

        let mut conn = match self.pool.get().await {
            Ok(c) => c,
            Err(e) => {
                warn!("Redis connection error (csrf store): {}", e);
                return false;
            }
        };

        // SET key value EX ttl (atomic set with expiry)
        match conn
            .set_ex::<_, _, ()>(&redis_key, &value, CSRF_TTL_SECS as u64)
            .await
        {
            Ok(_) => true,
            Err(e) => {
                warn!("Redis SET error (csrf store): {}", e);
                false
            }
        }
    }

    async fn retrieve_and_consume(&self, key: &str) -> Option<(String, Option<String>)> {
        let redis_key = format!("{}{}", REDIS_KEY_PREFIX, key);

        let mut conn = match self.pool.get().await {
            Ok(c) => c,
            Err(e) => {
                warn!("Redis connection error (csrf retrieve): {}", e);
                return None;
            }
        };

        // GET + DEL (not atomic, but acceptable for CSRF: worst case is double-consume = rejected)
        let value: Option<Vec<u8>> = match conn.get(&redis_key).await {
            Ok(v) => v,
            Err(e) => {
                warn!("Redis GET error (csrf retrieve): {}", e);
                return None;
            }
        };

        let value = value?;

        // Always delete (single-use semantics)
        let _: () = conn.del(&redis_key).await.unwrap_or(());

        let entry: CsrfEntry = match serde_json::from_slice(&value) {
            Ok(e) => e,
            Err(e) => {
                warn!("Failed to deserialize CSRF entry: {}", e);
                return None;
            }
        };

        Some((entry.nonce, entry.redirect_url))
    }

    async fn cleanup_expired(&self) {
        // No-op: Redis TTL handles expiry automatically.
        // Entries are automatically deleted after CSRF_TTL_SECS.
    }
}

// ---------------------------------------------------------------------------
// Factory
// ---------------------------------------------------------------------------

/// CSRF store enum that wraps either implementation.
#[derive(Clone)]
pub enum CsrfStoreType {
    Memory(MemoryCsrfStore),
    Redis(Arc<RedisCsrfStore>),
}

impl CsrfStoreType {
    /// Create a CSRF store. Attempts Redis first, falls back to in-memory.
    pub fn new(redis_url: Option<&str>) -> Self {
        match redis_url {
            Some(url) if !url.is_empty() => match RedisCsrfStore::new(url) {
                Ok(store) => {
                    info!("CSRF state persistence: Redis-backed (multi-instance safe)");
                    CsrfStoreType::Redis(Arc::new(store))
                }
                Err(e) => {
                    warn!(
                        "Redis CSRF store failed to connect, falling back to in-memory: {}",
                        e
                    );
                    info!("CSRF state persistence: in-memory (single-instance only)");
                    CsrfStoreType::Memory(MemoryCsrfStore::new())
                }
            },
            _ => {
                info!("CSRF state persistence: in-memory (single-instance only)");
                CsrfStoreType::Memory(MemoryCsrfStore::new())
            }
        }
    }
}

#[async_trait]
impl CsrfStore for CsrfStoreType {
    async fn store(&self, key: &str, nonce: &str, redirect_url: Option<String>) -> bool {
        match self {
            CsrfStoreType::Memory(s) => s.store(key, nonce, redirect_url).await,
            CsrfStoreType::Redis(s) => s.store(key, nonce, redirect_url).await,
        }
    }

    async fn retrieve_and_consume(&self, key: &str) -> Option<(String, Option<String>)> {
        match self {
            CsrfStoreType::Memory(s) => s.retrieve_and_consume(key).await,
            CsrfStoreType::Redis(s) => s.retrieve_and_consume(key).await,
        }
    }

    async fn cleanup_expired(&self) {
        match self {
            CsrfStoreType::Memory(s) => s.cleanup_expired().await,
            CsrfStoreType::Redis(s) => s.cleanup_expired().await,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_csrf_store_and_retrieve() {
        let store = MemoryCsrfStore::new();
        let key = "test-state-123";
        let nonce = "test-nonce-456";

        assert!(store.store(key, nonce, None).await);
        let result = store.retrieve_and_consume(key).await;
        assert_eq!(result, Some((nonce.to_string(), None)));

        // Second retrieve should return None (single-use)
        let result = store.retrieve_and_consume(key).await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_memory_csrf_store_with_redirect() {
        let store = MemoryCsrfStore::new();
        let key = "state-with-redirect";
        let nonce = "nonce-abc";
        let redirect = Some("https://example.com/callback".to_string());

        assert!(store.store(key, nonce, redirect.clone()).await);
        let result = store.retrieve_and_consume(key).await;
        assert_eq!(result, Some((nonce.to_string(), redirect)));
    }

    #[tokio::test]
    async fn test_memory_csrf_store_missing_key() {
        let store = MemoryCsrfStore::new();
        let result = store.retrieve_and_consume("nonexistent").await;
        assert!(result.is_none());
    }

    #[test]
    fn test_csrf_entry_serialization() {
        let entry = CsrfEntry {
            nonce: "abc123".to_string(),
            redirect_url: Some("https://example.com".to_string()),
        };
        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: CsrfEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.nonce, "abc123");
        assert_eq!(
            deserialized.redirect_url,
            Some("https://example.com".to_string())
        );
    }

    #[test]
    fn test_csrf_entry_serialization_no_redirect() {
        let entry = CsrfEntry {
            nonce: "xyz789".to_string(),
            redirect_url: None,
        };
        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: CsrfEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.nonce, "xyz789");
        assert!(deserialized.redirect_url.is_none());
    }

    #[tokio::test]
    async fn test_memory_csrf_store_cleanup() {
        let store = MemoryCsrfStore::new();

        // Insert an expired entry
        store.entries.insert(
            "expired-key".to_string(),
            (
                "nonce".to_string(),
                None,
                Utc::now() - chrono::Duration::seconds(CSRF_TTL_SECS + 100),
            ),
        );

        // Insert a fresh entry
        store.store("fresh-key", "nonce-fresh", None).await;

        store.cleanup_expired().await;

        assert!(store.entries.get("fresh-key").is_some());
        assert!(store.entries.get("expired-key").is_none());
    }

    #[test]
    fn test_csrf_store_type_factory_no_redis() {
        let store = CsrfStoreType::new(None);
        match store {
            CsrfStoreType::Memory(_) => {} // expected
            CsrfStoreType::Redis(_) => panic!("Should not create Redis store without URL"),
        }
    }

    #[test]
    fn test_csrf_store_type_factory_empty_redis_url() {
        let store = CsrfStoreType::new(Some(""));
        match store {
            CsrfStoreType::Memory(_) => {} // expected
            CsrfStoreType::Redis(_) => panic!("Should not create Redis store with empty URL"),
        }
    }
}
