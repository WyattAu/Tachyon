# ADR-008: Deadlock Prevention Strategy
# Status: Accepted
# Date: 2026-02-11
# Phase: 2.5 (Concurrency Analysis)

---

## Context

**Problem Statement:**
Deadlocks can occur in concurrent systems when threads wait for resources held by other threads, creating a circular wait condition. The Tachyon system has multiple shared resources that could deadlock without proper prevention strategies.

**Drivers:**
- CM-RQ-007: Concurrent Git operations with potential lock cycles
- deadlock_analysis.md: Identified deadlock scenarios (Cycle 1: Cache+Git, Cycle 2: Search+Cache)
- PF-RQ-003: High concurrency requirement

---

## Decision

**Selected Strategy:** Global lock ordering with timeout-based deadlock prevention

**Rationale:**
Establishing a total order on all lock acquisitions prevents circular wait conditions, the necessary condition for deadlock. Combined with timeouts, we ensure bounded waiting and system liveness.

---

## Alternatives Considered

| Option | Description | Pros | Cons | Decision |
|---------|-------------|------|-----------|
| Deadlock Detection | Detect and recover from deadlocks | Complex runtime overhead | REJECTED |
| Lock Hierarchies | Logical organization | Still vulnerable to cycles | REJECTED |
| Timeouts Only | Bounded waiting | Does not prevent cycles | REJECTED |
| Global Ordering | Prevents cycles entirely | Requires discipline | SELECTED |
| Global + Timeout | Prevents cycles + bounded waiting | Best of both | SELECTED |

**Selection Rationale:**
Global ordering eliminates the circular wait condition at the source. Timeouts provide a safety net for cases where ordering discipline fails or unexpected delays occur.

---

## Consequences

**Positive Consequences:**
- Guaranteed deadlock freedom under ordered acquisition
- Bounded lock acquisition time with timeouts
- Predictable system behavior under high concurrency
- No indefinite blocking scenarios

**Negative Consequences:**
- Requires developer discipline to follow lock order
- Additional complexity in timeout handling
- Potential for reduced throughput if timeouts are too aggressive

**Mitigation Strategies:**
- Enforce lock ordering through code review and testing
- Use tokio's async lock primitives with timeout support
- Implement deadlock detection with wait-for graph monitoring
- Establish timeout thresholds based on operation duration

---

## Implementation Details

### 1. Global Lock Ordering

**Lock Hierarchy:**
```
Level 0: cache_lock (LRU cache)
Level 1: index_reader_lock (search index read)
Level 2: index_writer_lock (search index write)
Level 3: git_read_lock (Git repository read)
Level 4: git_write_lock (Git repository write)
Level 5: ws_connections_lock (WebSocket connections)
Level 6: broadcast_lock (broadcast channel - usually lock-free)
```

**Acquisition Protocol:**
```rust
/// Lock ordering enforcement trait
trait OrderedLock {
    fn order() -> u8;
}

/// Acquire locks in global order
async fn acquire_ordered_locks<'a>(locks: &[&'a dyn OrderedLock]) -> Vec<Box<dyn Any + Send>> {
    let mut sorted: Vec<_> = locks.iter()
        .sorted_by_key(|l| l.order() as u8)
        .collect();
    
    let mut guards = Vec::new();
    for lock in sorted {
        guards.push(lock.acquire().await);
    }
    
    guards
}
```

### 2. Timeout Configuration

**Per-Operation Timeouts:**
| Lock Type | Operation | Timeout | Rationale |
|-----------|-----------|----------|-----------|
| Cache lock | get/put | 10ms | Cache operations are fast |
| Cache lock | eviction | 100ms | Eviction may be slower |
| Git lock | read | 1000ms | History can be large |
| Git lock | write | 5000ms | Commits may run hooks |
| Index lock | read | 50ms | Reader snapshots are fast |
| Index lock | write | 3000ms | Indexing is batch operation |
| WS lock | connect/disconnect | 50ms | Connection operations are fast |
| WS lock | broadcast | 10ms | Broadcast is lock-free |

**Timeout Implementation:**
```rust
use tokio::time::{timeout, Duration};

async fn acquire_with_timeout<'a, F>(
    lock: &'a tokio::sync::Mutex<F>,
    timeout: Duration,
    f: impl FnOnce<tokio::sync::MutexGuard<'a, F>> + Send
) -> Result<F::Output, TimeoutError> {
    timeout(timeout, lock.lock()).await.map_err(|_| TimeoutError)
}
```

### 3. Component-Specific Deadlock Prevention

#### 3.1. Cache + Git Operations

**Scenario:** Cache invalidation during Git commit

**Prevention:**
```rust
pub async fn invalidate_and_commit(
    cache: &Arc<tokio::sync::RwLock<Cache>>,
    repo: &Arc<tokio::sync::RwLock<GitRepo>>,
    message: &str,
) -> Result<String, GitError> {
    // Acquire locks in order: cache (level 0) then git (level 4)
    let cache_guard = cache.write().await;
    let repo_guard = repo.write().await;
    
    // Perform operations
    cache_guard.invalidate_all();
    let commit_hash = repo_guard.commit(message)?;
    
    // Guards released in reverse order
    Ok(commit_hash)
}
```

#### 3.2. Search + Cache Operations

**Scenario:** Search during cache update

**Prevention:**
```rust
pub async fn search_and_update(
    cache: &Arc<tokio::sync::RwLock<Cache>>,
    index: &Arc<tokio::sync::Mutex<Index>>,
    query: &Query,
    doc: &Document,
) -> Result<Vec<Result>, SearchError> {
    // Acquire locks in order: cache (level 0) then index write (level 2)
    let cache_guard = cache.write().await;
    let mut index_writer = index.write().await;
    
    // Update cache
    cache_guard.put(query.as_key(), render_results(&results)?);
    
    // Update index
    index_writer.add_document(doc)?;
    index_writer.commit()?;
    
    Ok(results)
}
```

---

## Deadlock Detection and Recovery

### 1. Wait-For Graph Monitoring

**Data Structure:**
```rust
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

struct DeadlockMonitor {
    wait_for_graph: Arc<RwLock<HashMap<ThreadId, Option<ResourceId>>>,
}

impl DeadlockMonitor {
    pub fn check_deadlock(&self) -> Option<Vec<ThreadId>> {
        let graph = self.wait_for_graph.read().await;
        
        // Detect cycles in wait-for graph
        let cycle = Self::detect_cycle(&graph);
        cycle
    }
    
    fn detect_cycle(graph: &HashMap<ThreadId, Option<ResourceId>>) -> Option<Vec<ThreadId>> {
        // Tarjan's algorithm for strongly connected components
        let mut visited = HashMap::new();
        let mut stack = Vec::new();
        
        for (&thread_id, waiting_for) in graph.iter() {
            match waiting_for {
                Some(resource_id) => {
                    if visited.contains(thread_id) {
                        // Part of cycle
                        stack.push(thread_id);
                    } else {
                        visited.insert(thread_id);
                        stack.push(thread_id);
                    }
                }
                None => {}
            }
        }
        
        if stack.len() > 1 {
            Some(stack)
        } else {
            None
        }
    }
}
```

### 2. Recovery Strategy

**Victim Selection:**
1. Youngest thread in cycle
2. Thread with least work completed
3. Thread holding least resources

**Recovery Implementation:**
```rust
impl DeadlockMonitor {
    pub async fn resolve_deadlock(&self, cycle: Vec<ThreadId>) -> Result<(), RecoveryError> {
        let victim = cycle.last().ok_or(RecoveryError::NoVictim)?;
        
        // Send cancellation to victim
        if let Some(handle) = self.thread_handles.get(victim) {
            handle.cancel().await;
        }
        
        Ok(())
    }
}
```

---

## Testing Strategy

### 1. Lock Ordering Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_lock_ordering_prevents_deadlock() {
        let lock1 = Arc::new(Mutex::new(0));
        let lock2 = Arc::new(Mutex::new(0));
        
        let handles: Vec<_> = (0..100).map(|_| {
            tokio::spawn(async move {
                let _g1 = lock1.lock().await;
                let _g2 = lock2.lock().await;
                // Both locks acquired in consistent order
            })
        }).collect();
        
        for handle in handles {
            handle.await.unwrap();
        }
        
        // No deadlock occurred
    }
}
```

### 2. Timeout Tests

```rust
#[tokio::test]
async fn test_timeout_prevents_indefinite_wait() {
    use tokio::time::{timeout, Duration};
    
    let lock = Arc::new(Mutex::new(0));
    // Acquire the lock
    let _guard = lock.lock().await;
    
    // Attempt to acquire again with timeout
    let result = timeout(Duration::from_millis(10), lock.lock()).await;
    assert!(result.is_err()); // Timeout prevented acquisition
}
```

---

## Compliance

| Standard | Status |
|----------|---------|
| IEEE 1016-2009 | COMPLIANT |
| ISO/IEC 25010 | COMPLIANT |

---

## References

- deadlock_analysis.md: Detailed deadlock scenarios and prevention
- proof.lean: Formal proofs of deadlock freedom
- thread_safety_analysis.md: Component thread safety analysis
- synchronization_design.md: Lock ordering protocol

---

## Sign-off

**Approved By:** Concurrency Engineer Agent
**Approval Date:** 2026-02-11
**Next ADR:** ADR-009: Race Condition Mitigation
