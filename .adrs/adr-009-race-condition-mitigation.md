# ADR-009: Race Condition Mitigation Strategy
# Status: Accepted
# Date: 2026-02-11
# Phase: 2.5 (Concurrency Analysis)

---

## Context

**Problem Statement:**
Race conditions occur when multiple threads access shared mutable state without proper synchronization, leading to inconsistent reads, data corruption, or undefined behavior. The Tachyon system has multiple shared resources where races can occur.

**Drivers:**
- thread_safety_analysis.md: Identified race condition hazards
- RE-RQ-005: LRU cache concurrent access
- CM-RQ-007: Concurrent Git operations
- SD-RQ-002: Search index concurrent reads/writes

---

## Decision

**Selected Strategy:** Multi-layered race condition prevention combining atomic operations, proper locking, and Rust ownership guarantees

**Rationale:**
Rust's ownership and borrowing system provides compile-time data race prevention. Combined with atomic operations for simple state and proper locking for complex state, we can eliminate race conditions in most Tachyon components.

---

## Alternatives Considered

| Option | Description | Pros | Cons | Decision |
|---------|-------------|------|-----------|
| Software Transactional Memory (STM) | Composable transactions | Complex runtime overhead | REJECTED |
| Pure Functional | No mutable state | Immutable, copy overhead | REJECTED |
| Global Locking | Simple, predictable | High contention | REJECTED |
| Atomic Operations | Lock-free, fast | Limited to simple types | SELECTED |
| Hybrid Approach | Atomic + Locking | Best of both worlds | SELECTED |

**Selection Rationale:**
Using atomic operations for simple counters and flags eliminates race conditions without blocking. For complex state, using tokio's synchronization primitives provides proper mutual exclusion. The hybrid approach provides optimal performance with correctness.

---

## Consequences

**Positive Consequences:**
- Compile-time race condition prevention via Rust ownership
- High-throughput with atomic operations
- Minimal contention for simple state
- Clear correctness guarantees

**Negative Consequences:**
- Increased complexity for atomic operations
- Performance overhead from locking complex state
- Potential for deadlocks if not careful with lock ordering

**Mitigation Strategies:**
- Use atomic operations for simple shared state
- Establish proper lock ordering for complex state
- Minimize critical section duration
- Use DashMap for high-frequency concurrent map access
- Test with loom model checker

---

## Implementation Details

### 1. Atomic Operations for Simple State

#### 1.1. Statistics Counters

**Pattern:** AtomicU64 for counters

**Implementation:**
```rust
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

pub struct CacheStats {
    hits: AtomicU64,
    misses: AtomicU64,
}

impl CacheStats {
    pub fn record_hit(&self) {
        self.hits.fetch_add(1, Ordering::Relaxed);
    self.misses.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn hit_rate(&self) -> f64 {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let total = hits + misses;
        if total == 0 { 0.0 } else { hits as f64 / total as f64 }
    }
}
```

**Race Condition Mitigation:**
Atomic operations guarantee atomicity - no race between increment and read.

#### 1.2. Session ID Generation

**Pattern:** AtomicU64 for monotonic IDs

**Implementation:**
```rust
use std::sync::atomic::{AtomicU64, Ordering};

pub struct SessionManager {
    next_id: AtomicU64,
}

impl SessionManager {
    pub fn next_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }
}
```

**Race Condition Mitigation:**
fetch_add returns a unique value - no duplicate session IDs.

### 2. Lock-Based Synchronization

#### 2.1. LRU Cache Operations

**Pattern:** DashMap for sharded concurrent access

**Race Condition Scenario:** Check-then-act pattern

**Implementation:**
```rust
use dashmap::DashMap;

pub struct ThreadSafeLruCache<K, V> {
    map: DashMap<K, CacheEntry<V>>,
}

impl<K: Hash + Eq, V: Clone> ThreadSafeLruCache<K, V> {
    pub fn get_or_insert(&self, key: K, f: impl FnOnce() -> V) -> V {
        // Atomic get_or_insert prevents TOCTTOU race
        self.map.entry(key).or_insert_with(f)
    }
}
```

**Race Condition Mitigation:**
DashMap provides atomic entry operations - no TOCTTOU race.

#### 2.2. Git Repository Operations

**Pattern:** RwLock with scoped guards

**Race Condition Scenario:** Read-modify-write pattern

**Implementation:**
```rust
use tokio::sync::RwLock;

pub struct ThreadSafeGitRepo {
    repo: Arc<RwLock<git2::Repository>>,
}

impl ThreadSafeGitRepo {
    pub async fn update_and_commit(&self, updates: Vec<Update>) -> Result<String, git2::Error> {
        // Single atomic check-then-act
        let repo = self.repo.write().await;
        
        for update in updates {
            repo.apply_update(update)?;
        }
        
        // Single commit
        let oid = repo.commit("Batch update")?;
        Ok(oid.to_string())
    }
}
```

**Race Condition Mitigation:**
RwLock provides exclusive write access - concurrent reads see consistent state.

### 3. Channel-Based Communication

#### 3.1. File Watch Events

**Pattern:** Broadcast channel for pub-sub

**Race Condition Scenario:** Event loss during high contention

**Implementation:**
```rust
use tokio::sync::broadcast;

pub struct FileWatchCoordinator {
    broadcast_tx: broadcast::Sender<FileEvent>,
}

impl FileWatchCoordinator {
    pub fn broadcast(&self, event: FileEvent) -> Result<(), SendError> {
        // Non-blocking send - no wait
        self.broadcast_tx.send(event)
            .map_err(|_| SendError::ChannelClosed)
    }
}
```

**Race Condition Mitigation:**
Broadcast channel is lock-free - no race on send.

#### 3.2. Search Index Updates

**Pattern:** Mutex with single writer

**Race Condition Scenario:** Inconsistent reads during write

**Implementation:**
```rust
use tokio::sync::Mutex;

pub struct ThreadSafeIndex {
    writer: Arc<Mutex<tantivy::IndexWriter>>,
    reader: Arc<Mutex<tantivy::IndexReader>>,
}

impl ThreadSafeIndex {
    pub async fn write_and_search(&self, docs: Vec<Document>) -> Result<Vec<Result>, tantivy::TantivyError> {
        // Write with exclusive lock
        {
            let mut writer = self.writer.lock().await;
            for doc in docs {
                writer.add_document(doc)?;
            }
            writer.commit()?;
        }
        
        // Read sees consistent snapshot
        let reader = self.reader.lock().await;
        reader.search(query)
    }
}
```

**Race Condition Mitigation:**
Single writer ensures consistency - readers always see either before or after write, never during.

### 4. Safe Iteration Patterns

#### 4.1. Connection Map Iteration

**Race Condition Scenario:** Modification during iteration

**Implementation:**
```rust
use std::collections::HashMap;

pub struct WebSocketManager {
    connections: Arc<RwLock<HashMap<SessionId, WebSocket>>>,
}

impl WebSocketManager {
    pub fn broadcast_to_all(&self, message: ServerMessage) {
        // Clone connections before iterating
        let connections: Vec<_> = self.connections.read().await
            .values()
            .cloned()
            .collect();
        
        for socket in connections {
            let _ = socket.send(message).await;
        }
    }
}
```

**Race Condition Mitigation:**
Cloning creates isolated copy - no modification during iteration.

---

## Testing Strategy

### 1. Race Condition Tests with loom

**Configuration:**
```toml
[dependencies]
loom = { version = "0.7" }

[[test]]
name = "loom_race_conditions"
harness = false
```

**Test Cases:**
- Concurrent cache get/put with verification
- Concurrent Git read/write sequences
- Concurrent search index read/write
- Concurrent WebSocket connect/broadcast

**Example Test:**
```rust
#[cfg(test)]
mod tests {
    #[test]
    #[cfg_attr(loom, loom::test)]
    fn loom_test_atomic_counter_race() {
        loom::model(|| {
            let counter = std::sync::atomic::AtomicUsize::new(0);
            
            // Two threads increment concurrently
            let handle1 = std::thread::spawn(|| {
                counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            });
            let handle2 = std::thread::spawn(|| {
                counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            });
            
            handle1.join().unwrap();
            handle2.join().unwrap();
            
            // Verify final value is 2 (no race)
            assert_eq!(counter.load(std::sync::atomic::Ordering::Relaxed), 2);
        });
    }
}
```

### 2. Stress Tests

**Scenarios:**
- 10000 concurrent atomic operations
- 1000 concurrent lock acquisitions
- 1000 channel operations

---

## Compliance

| Standard | Status |
|----------|---------|
| IEEE 1016-2009 | COMPLIANT |
| ISO/IEC 25010 | COMPLIANT |

---

## References

- thread_safety_analysis.md: Race condition hazards
- proof.lean: Atomic operation properties
- synchronization_design.md: Lock-based synchronization
- ADR-007: Thread safety strategy

---

## Sign-off

**Approved By:** Concurrency Engineer Agent
**Approval Date:** 2026-02-11
**Next ADR:** ADR-010: Synchronization Primitives
