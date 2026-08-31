# ADR-010: Synchronization Primitives Selection
# Status: Accepted
# Date: 2026-02-11
# Phase: 2.5 (Concurrency Analysis)

---

## Context

**Problem Statement:**
The Tachyon system requires appropriate synchronization primitives for different concurrency scenarios. Selecting the wrong primitive can lead to poor performance, deadlocks, or race conditions.

**Drivers:**
- synchronization_design.md: Component-specific synchronization requirements
- PF-RQ-003: High concurrency with bounded resources
- RE-RQ-005: LRU cache with high-frequency access
- CM-RQ-007: Concurrent Git operations

---

## Decision

**Selected Strategy:** Use tokio sync primitives with primitive selection based on access pattern

**Rationale:**
tokio provides battle-tested async-aware synchronization primitives that integrate with Rust's ownership system. Selecting the appropriate primitive per use case ensures optimal performance and correctness.

---

## Alternatives Considered

| Option | Description | Pros | Cons | Decision |
|---------|-------------|------|-----------|
| std::sync::Mutex | Simple, reliable | Read contention | REJECTED |
| std::sync::RwLock | Read-write separation | Write contention | PARTIAL |
| std::sync::Semaphore | Bounded concurrency | Requires manual management | SELECTED |
| DashMap | Lock-free map | High contention | SELECTED |
| broadcast::channel | Lock-free pub-sub | Channel overflow risk | SELECTED |

**Selection Rationale:**
tokio primitives are already a dependency. Using tokio's Mutex, RwLock, and Semaphore provides async-aware blocking that integrates with the tokio runtime. DashMap and broadcast::channel are selected for specific use cases where their benefits outweigh the overhead.

---

## Consequences

**Positive Consequences:**
- Async-aware blocking with tokio runtime
- High throughput with lock-free structures (DashMap, broadcast)
- Read-write separation with RwLock
- Bounded concurrency with Semaphore

**Negative Consequences:**
- Additional dependency on tokio (already included)
- Learning curve for tokio async patterns
- Potential for lock contention if not properly designed

**Mitigation Strategies:**
- Use Mutex for exclusive access with short critical sections
- Use RwLock for read-heavy patterns
- Use Semaphore for bounded concurrency
- Use DashMap for high-frequency map access
- Use broadcast channel for pub-sub patterns

---

## Implementation Details

### 1. Primitive Selection Guidelines

**Decision Matrix:**

| Access Pattern | Read Ratio | Write Frequency | Recommended Primitive |
|--------------|-----------|-----------------|---------------------|
| Exclusive access | N/A | High | tokio::sync::Mutex |
| Read-heavy (>10:1) | Low | tokio::sync::RwLock |
| Read-write balanced (5:1 - 10:1) | Medium | tokio::sync::RwLock |
| Write-heavy (1:>10) | N/A | tokio::sync::Mutex |
| Bounded resource | N/A | High | tokio::sync::Semaphore |
| High-frequency map | N/A | High | DashMap |
| Multi-consumer pub-sub | N/A | High | tokio::sync::broadcast |

### 2. tokio Primitive Usage

#### 2.1. Mutex

**Use Case:** Exclusive access to shared state

**Implementation:**
```rust
use tokio::sync::Mutex;

pub struct SharedState<T> {
    state: tokio::sync::Mutex<T>,
}

impl<T> SharedState<T> {
    pub async fn with_lock<F, R>(&self, f: F) -> R {
        let guard = self.state.lock().await;
        f(&mut guard)
    }
}
```

**Characteristics:**
- Fairness: FIFO queue-based
- Blocking: Yes
- Async-aware: Yes
- Cancellation support: Via token cancellation

#### 2.2. RwLock

**Use Case:** Read-write separation

**Implementation:**
```rust
use tokio::sync::RwLock;

pub struct ReadWriteState<T> {
    state: tokio::sync::RwLock<T>,
}

impl<T> ReadWriteState<T> {
    pub async fn with_read<F, R>(&self, f: F) -> R {
        let guard = self.state.read().await;
        f(&guard)
    }
    
    pub async fn with_write<F, R>(&self, f: F) -> R {
        let guard = self.state.write().await;
        f(&mut guard)
    }
}
```

**Characteristics:**
- Fairness: FIFO queue-based
- Read concurrency: Unlimited
- Write concurrency: Exclusive
- Blocking: Yes
- Async-aware: Yes

#### 2.3. Semaphore

**Use Case:** Bounded concurrency

**Implementation:**
```rust
use tokio::sync::Semaphore;

pub struct ConcurrencyLimiter {
    semaphore: tokio::sync::Semaphore,
}

impl ConcurrencyLimiter {
    pub async fn acquire(&self) -> SemaphorePermit {
        self.semaphore.acquire().await.unwrap_or_else(|_| {
            SemaphorePermit::new(self.semaphore.clone())
        })
    }
    
    pub fn try_acquire(&self) -> Option<SemaphorePermit> {
        self.semaphore.try_acquire().ok()
    }
}
```

**Characteristics:**
- Fairness: FIFO queue-based
- Blocking: Yes
- Async-aware: Yes
- Cancellation support: Via permit drop

### 3. Lock-Free Structures

#### 3.1. DashMap for LRU Cache

**Use Case:** High-frequency concurrent hash map

**Implementation:**
```rust
use dashmap::DashMap;

pub struct ThreadSafeLruCache<K, V> {
    map: DashMap<K, CacheEntry<V>>,
}

impl<K: Hash + Eq, V: Clone> ThreadSafeLruCache<K, V> {
    pub fn get(&self, key: &K) -> Option<V> {
        self.map.get(key).map(|entry| entry.value.clone())
    }
    
    pub fn put(&self, key: K, value: V) {
        self.map.insert(key, CacheEntry::new(value));
    }
}
```

**Characteristics:**
- Sharding: Internal (configurable)
- Lock-free: Yes
- Performance: Excellent for reads
- Consistency: Eventual per shard

#### 3.2. Broadcast Channel for File Watch

**Use Case:** Multi-consumer pub-sub pattern

**Implementation:**
```rust
use tokio::sync::broadcast;

pub struct FileWatchCoordinator {
    broadcast_tx: broadcast::Sender<FileEvent>,
}

impl FileWatchCoordinator {
    pub fn broadcast(&self, event: FileEvent) -> Result<(), SendError> {
        self.broadcast_tx.send(event)
            .map_err(|_| SendError::ChannelClosed)
    }
}
```

**Characteristics:**
- Multi-producer, multi-consumer
- Lock-free: Yes
- Backpressure: Channel capacity limit
- Async-aware: Yes

---

## Configuration

**Primitive Configuration:**
```toml
[concurrency]
# tokio primitives
mutex_timeout_ms = 100
rwlock_timeout_ms = 500
semaphore_permits = 10

# DashMap
shard_count = 64

# Broadcast channel
channel_capacity = 1000
```

---

## Testing Strategy

### 1. Primitive Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;
    
    #[tokio::test]
    async fn test_mutex_exclusive_access() {
        let state = Arc::new(tokio::sync::Mutex::new(0));
        
        let handles: Vec<_> = (0..10).map(|_| {
            let state = Arc::clone(&state);
            tokio::spawn(async move {
                let _guard = state.lock().await;
                assert_eq!(*guard, 42);
            })
        }).collect();
        
        for handle in handles {
            handle.await.unwrap();
        }
    }
}
```

### 2. Performance Benchmarks

**Metrics to Collect:**
- Lock acquisition time (P50, P95, P99)
- Lock hold time
- Contention percentage

---

## Compliance

| Standard | Status |
|----------|---------|
| IEEE 1016-2009 | COMPLIANT |
| ISO/IEC 25010 | COMPLIANT |

---

## References

- synchronization_design.md: Detailed synchronization design
- thread_safety_analysis.md: Thread safety requirements
- deadlock_analysis.md: Deadlock prevention needs
- race_condition_mitigation.md: Race condition prevention
- dep_spec/tokio/dep_spec.toml: tokio documentation

---

## Sign-off

**Approved By:** Concurrency Engineer Agent
**Approval Date:** 2026-02-11
**Next ADR:** ADR-011: Lock-Free Data Structures
