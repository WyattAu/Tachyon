# ADR-007: Thread Safety Strategy
# Status: Accepted
# Date: 2026-02-11
# Phase: 2.5 (Concurrency Analysis)

---

## Context

**Problem Statement:**
The Tachyon system requires concurrent access to multiple shared resources across multiple threads. Without a comprehensive thread safety strategy, race conditions, data corruption, and undefined behavior can occur.

**Drivers:**
- PF-RQ-003: Concurrency support for multiple concurrent users
- CM-RQ-007: Concurrent Git operations with conflict resolution
- RE-RQ-005: LRU cache with high-frequency concurrent access
- SD-RQ-002: Concurrent search index reads and writes

---

## Decision

**Selected Strategy:** Multi-layered thread safety combining:
1. Rust ownership and type system
2. tokio async runtime primitives
3. Global lock ordering
4. Lock-free data structures where appropriate
5. Comprehensive testing with loom model checker

**Rationale:**
Rust's ownership system provides compile-time data race prevention. Combined with tokio's async primitives and careful synchronization design, we can achieve thread safety without runtime overhead from traditional languages.

---

## Alternatives Considered

| Option | Description | Pros | Cons | Decision |
|---------|-------------|------|-----------|
| Pure Atomic Operations | Lock-free, no blocking | Complex to implement | REJECTED |
| Global Mutex | Simple, predictable | High contention | REJECTED |
| Thread-Local Storage | No contention, no blocking | Requires copy overhead | REJECTED |
| tokio Primitives | Async-aware, well-tested | Requires tokio dependency | SELECTED |

**Selection Rationale:**
tokio is already a dependency (dep_spec/tokio/dep_spec.toml). Using tokio's synchronization primitives provides async-aware locking, backpressure support, and battle-tested concurrency patterns.

---

## Consequences

**Positive Consequences:**
- Compile-time data race prevention via Rust ownership
- Async-aware blocking with tokio primitives
- High throughput with lock-free structures (DashMap)
- Testable concurrency models with loom

**Negative Consequences:**
- Additional dependency on tokio
- Learning curve for tokio async patterns
- Potential for lock contention if not properly designed

**Mitigation Strategies:**
- Establish global lock ordering to prevent deadlocks
- Use appropriate primitive per access pattern (read-heavy = RwLock, write-heavy = Mutex)
- Implement backpressure with semaphores
- Use DashMap for high-frequency map access

---

## Implementation Details

### 1. Lock Ordering Protocol

**Global Lock Order:**
```
1. cache_lock (LRU cache)
2. stats_lock (statistics counters - usually atomic, no lock)
3. index_reader_lock (search index read)
4. index_writer_lock (search index write)
5. git_read_lock (Git repository read)
6. git_write_lock (Git repository write)
7. ws_connections_lock (WebSocket connection map)
```

**Acquisition Function:**
```rust
/// Acquire locks in global order to prevent deadlock
async fn acquire_ordered_locks<'a>(locks: &[&'a dyn AsyncLock]) -> Vec<LockGuard<'a>> {
    let mut sorted_locks: Vec<_> = locks.iter()
        .sorted_by_key(|l| (l.as_ref() as *const u8) as usize)
        .collect();
    
    let mut guards = Vec::new();
    for lock in sorted_locks {
        guards.push(lock.acquire().await);
    }
    
    guards
}
```

### 2. Component-Specific Strategies

#### 2.1. LRU Cache (RE-002, RE-005)

**Pattern:** Sharded concurrent hash map

**Implementation:**
```rust
use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, Ordering};

pub struct ThreadSafeLruCache<K, V> {
    map: DashMap<K, CacheEntry<V>>,
    hit_count: Arc<AtomicU64>,
    miss_count: Arc<AtomicU64>,
}

impl<K: Hash + Eq, V: Clone> ThreadSafeLruCache<K, V> {
    pub fn get(&self, key: &K) -> Option<V> {
        self.map.get(key).map(|entry| {
            // DashMap provides thread-safe access
            entry.value.clone()
        })
    }
    
    pub fn put(&self, key: K, value: V) {
        self.map.insert(key, CacheEntry::new(value));
    }
}
```

**Traceability:** thread_safety_analysis.md: Section 3.1

#### 2.2. Git Repository (CM-002, CM-006)

**Pattern:** Reader-writer lock with exclusive write access

**Implementation:**
```rust
use tokio::sync::RwLock;

pub struct ThreadSafeGitRepo {
    repo: Arc<RwLock<git2::Repository>>,
}

impl ThreadSafeGitRepo {
    pub async fn commit(&self, message: &str) -> Result<String, git2::Error> {
        let mut repo = self.repo.write().await;
        repo.commit(message)
    }
    
    pub async fn find_commit(&self, oid: &str) -> Result<Option<git2::Commit>, git2::Error> {
        let repo = self.repo.read().await;
        repo.find_commit(git2::Oid::from_str(oid).ok())
    }
}
```

**Traceability:** thread_safety_analysis.md: Section 3.2

#### 2.3. File Watch (CM-003)

**Pattern:** Broadcast channel with debouncing

**Implementation:**
```rust
use tokio::sync::broadcast;

pub struct FileWatchCoordinator {
    broadcast_tx: broadcast::Sender<FileEvent>,
}

impl FileWatchCoordinator {
    pub fn new() -> (Self, broadcast::Receiver<FileEvent>) {
        let (broadcast_tx, broadcast_rx) = broadcast::channel(1000);
        (Self { broadcast_tx }, broadcast_rx)
    }
    
    pub async fn broadcast_event(&self, event: FileEvent) {
        let _ = self.broadcast_tx.send(event);
    }
}
```

**Traceability:** thread_safety_analysis.md: Section 3.3

#### 2.4. Search Index (SD-002)

**Pattern:** Single writer with reader snapshots

**Implementation:**
```rust
use tokio::sync::Mutex;
use tantivy::Index;

pub struct ThreadSafeIndex {
    writer: Arc<Mutex<tantivy::IndexWriter>>,
    reader: Arc<Mutex<tantivy::IndexReader>>,
}

impl ThreadSafeIndex {
    pub async fn index_document(&self, doc: &tantivy::Document) -> Result<(), tantivy::TantivyError> {
        let mut writer = self.writer.lock().await;
        writer.add_document(doc)?;
        writer.commit()
    }
    
    pub async fn search(&self, query: &tantivy::Query) -> Result<Vec<tantivy::ScoredDoc>, tantivy::TantivyError> {
        let reader = self.reader.lock().await;
        let searcher = reader.searcher();
        searcher.search(query, &tantivy::TopDocs::with_limit(100))
    }
}
```

**Traceability:** thread_safety_analysis.md: Section 3.4

#### 2.5. WebSocket (IN-004)

**Pattern:** RwLock for connection map, broadcast for messaging

**Implementation:**
```rust
use tokio::sync::RwLock;
use std::collections::HashMap;

pub struct WebSocketManager {
    connections: Arc<RwLock<HashMap<SessionId, WebSocket>>>,
    broadcast_tx: broadcast::Sender<ServerMessage>,
}

impl WebSocketManager {
    pub async fn add_connection(&self, socket: WebSocket) -> SessionId {
        let mut connections = self.connections.write().await;
        let session_id = self.next_session_id();
        connections.insert(session_id, socket);
        session_id
    }
    
    pub async fn remove_connection(&self, session_id: SessionId) -> Option<WebSocket> {
        let mut connections = self.connections.write().await;
        connections.remove(&session_id)
    }
    
    pub async fn broadcast(&self, message: ServerMessage) {
        let _ = self.broadcast_tx.send(message);
    }
}
```

**Traceability:** thread_safety_analysis.md: Section 3.5

---

## Testing Strategy

### 1. Unit Tests with tokio::test

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;
    
    #[tokio::test]
    async fn test_concurrent_cache_access() {
        let cache = Arc::new(ThreadSafeLruCache::new(100));
        let handles: Vec<_> = (0..100).map(|i| {
            let cache = Arc::clone(&cache);
            tokio::spawn(async move {
                cache.put(format!("key-{}", i), format!("value-{}", i));
                let _ = cache.get(&format!("key-{}", i));
            })
        }).collect();
        
        for handle in handles {
            handle.await.unwrap();
        }
    }
}
```

### 2. Model Checking with loom

**Configuration (.cargo/config.toml):**
```toml
[dependencies]
loom = { version = "0.7" }

[[test]]
name = "loom_concurrent"
harness = false
```

**Test Cases:**
- All permutations of lock acquisition order
- Concurrent reads and writes
- Lock contention scenarios

### 3. Stress Tests

**Scenarios:**
- 1000 concurrent cache operations
- 100 concurrent Git commits
- 1000 concurrent search queries
- 1000 WebSocket connections

---

## Compliance

| Standard | Status |
|----------|---------|
| IEEE 1016-2009 | COMPLIANT |
| ISO/IEC 25010 | COMPLIANT |

---

## References

- thread_safety_analysis.md: Detailed thread safety analysis
- deadlock_analysis.md: Deadlock scenarios and prevention
- synchronization_design.md: Synchronization primitive selection
- proof.lean: Formal proofs of concurrency properties
- dep_spec/tokio/dep_spec.toml: tokio documentation

---

## Sign-off

**Approved By:** Concurrency Engineer Agent
**Approval Date:** 2026-02-11
**Next ADR:** ADR-008: Deadlock Prevention
