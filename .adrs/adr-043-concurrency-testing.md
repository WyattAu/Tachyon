# ADR-043: Concurrency Testing Strategy

**Status:** Accepted
**Date:** 2026-02-11
**Context:** Phase 5 - The Adversarial Loop (Prototyper)
**Related ADRs:** ADR-040 (Prototype Architecture), ADR-040 (Fuzzing Strategy)
**Traceability:** TACHYON-TSA-V1.0 (Thread Safety Analysis), TACHYON-DA-V1.0 (Deadlock Analysis)

---

## 1. Context

Phase 5 requires testing for **race conditions, deadlocks, and thread safety** on critical paths. The system uses tokio async runtime with shared state managed via synchronization primitives (DashMap, RwLock, Mutex, channels).

### 1.1. Concurrent Components Analysis

From [`thread_safety_analysis.md`](../.specs/02_5_concurrency/thread_safety_analysis.md):

| Component | Shared Resource | Sync Primitive | Access Pattern | Hazard Level |
|-----------|----------------|---------------|----------------|---------------|
| LRU Cache | DashMap<String, String> | Internal sharded mutex | Read-heavy, occasional writes | LOW |
| Git Repository | Repository (Arc) | RwLock<Repository> | Write-locked operations | MEDIUM |
| Search Index | IndexWriter, IndexReader | Mutex<IndexWriter>, Mutex<IndexReader> | Batch writes, concurrent reads | MEDIUM |
| File Watch Events | broadcast::Sender, mpsc::Sender | Producer-consumer | LOW |
| WebSocket Connections | HashMap<SessionId, WebSocket> | RwLock | High-concurrency broadcasts | LOW |
| Async Tasks | tokio runtime | Work-stealing scheduler | Task submission | LOW |

### 1.2. Deadlock Scenarios

From [`deadlock_analysis.md`](../.specs/02_5_concurrency/deadlock_analysis.md):

| Cycle ID | Components Involved | Severity | Probability | Mitigation |
|-----------|---------------------|----------|-------------|
| Cycle 1 | Cache + Git Operations | MEDIUM | LOW | Global lock ordering |
| Cycle 2 | Search + Cache Operations | LOW | VERY LOW | Separate stats from critical path |

---

## 2. Decision

### 2.1. Concurrency Testing Framework

**Primary Framework:** `loom` (Rust concurrency model checker)

**Secondary Frameworks:**
- `tokio::test` (Async test utilities)
- `rayon` (Parallel testing)
- `std::thread` (Native thread spawning)

**Rationale:**

| Criterion | loom | tokio::test | rayon | std::thread |
|-----------|------|-------------|---------|------------|
| Model Checking | Yes | No | No | No |
| Max Threads | 4 | Unlimited | Unlimited | Unlimited |
| Deterministic | Yes | No | No | Yes |
| Rust Native | Yes | Yes | Yes | Yes |
| Integration | Cargo | Native | External | Native |

### 2.2. Testing Strategy

#### 2.2.1. Unit-Level Concurrency Tests

Each module will have dedicated concurrency tests:

```
.specs/06_prototypes/prototype/tests/concurrency/
├── cache/
│   ├── mod.rs
│   ├── concurrent_get_put.rs    # Concurrent cache access
│   ├── stress_test.rs           # High-throughput cache stress
│   └── eviction_order.rs       # LRU eviction under concurrency
├── git/
│   ├── mod.rs
│   ├── concurrent_commits.rs    # Simultaneous Git commits
│   └── read_during_write.rs     # Read while write
├── search/
│   ├── mod.rs
│   ├── concurrent_index.rs       # Simultaneous indexing
│   └── search_during_index.rs  # Search while indexing
├── websocket/
│   ├── mod.rs
│   ├── connection_stress.rs       # High-concurrency connections
│   └── broadcast_stress.rs      # Broadcast stress test
└── lock_ordering/
    ├── mod.rs
    └── global_order.rs          # Global lock ordering verification
```

#### 2.2.2. Loom Model Checking

Loom will explore all possible interleavings for critical sections:

```rust
// tests/concurrency/loom_tests.rs
#[cfg(loom)]
mod loom_tests {
    use loom::sync::Arc;
    use loom::thread;
    use tachyon_prototype::cache::LruCache;

    #[test]
    fn loom_concurrent_cache_access() {
        loom::model(|| {
            let cache = Arc::new(LruCache::new(100));
            let mut handles = Vec::new();

            // Spawn multiple threads
            for i in 0..4 {
                let cache = Arc::clone(&cache);
                handles.push(thread::spawn(move || {
                    for j in 0..100 {
                        let key = format!("key-{}-{}", i, j);
                        let value = format!("value-{}", j);
                        cache.insert(key, value);
                        let _ = cache.get(&key);
                    }
                }));
            }

            // Wait for all threads
            for handle in handles {
                handle.join().unwrap();
            }
        });
    }
}
```

#### 2.2.3. Stress Testing

High-concurrency stress tests to identify performance bottlenecks:

| Test Type | Configuration | Duration | Metrics |
|-----------|--------------|----------|---------|
| Cache Stress | 100 concurrent readers, 10 writers | 60 seconds | Throughput, latency, contention |
| Git Commit | 10 concurrent commits | 30 seconds | Commit time, lock wait time |
| Search Index | 5 concurrent indexers, 20 readers | 120 seconds | Index time, query latency |
| WebSocket | 1000 concurrent connections | 60 seconds | Message rate, broadcast latency |

### 2.3. Execution Plan

#### Phase 1: Setup (Day 1)
- [ ] Install loom: `cargo install loom`
- [ ] Create concurrency test structure
- [ ] Configure Cargo.toml for loom

#### Phase 2: Unit Tests (Day 2-4)
- [ ] Implement concurrent cache tests
- [ ] Implement concurrent Git tests
- [ ] Implement concurrent search tests
- [ ] Implement WebSocket stress tests

#### Phase 3: Loom Tests (Day 5-6)
- [ ] Implement loom cache model
- [ ] Implement loom Git model
- [ ] Implement loom lock ordering model
- [ ] Run loom tests with all thread interleavings

#### Phase 4: Stress Tests (Day 7-8)
- [ ] Implement cache stress test
- [ ] Implement Git commit stress
- [ ] Implement search indexing stress
- [ ] Run stress tests with metrics

---

## 3. Critical Path Testing

### 3.1. LRU Cache Concurrency

**Test Objective:** Verify DashMap thread safety under high concurrency.

**Test Cases:**
1. **Concurrent Get/Put:**
   - 100 threads, 10,000 operations each
   - Verify no data corruption
   - Verify no panics occur

2. **Cache Hit Under Load:**
   - 50 concurrent readers, 10 concurrent writers
   - Measure hit rate degradation
   - Target: <5% degradation

3. **Eviction During Access:**
   - Force cache capacity limit
   - Trigger eviction during concurrent access
   - Verify correct key is evicted

### 3.2. Git Repository Concurrency

**Test Objective:** Verify RwLock prevents concurrent commit conflicts.

**Test Cases:**
1. **Concurrent Commits:**
   - 10 threads attempting to commit simultaneously
   - Verify only one commit succeeds
   - Verify others wait or fail gracefully

2. **Read During Write:**
   - Start write operation
   - Attempt read during write
   - Verify read waits for write completion

3. **Commit History Read:**
   - Read history while commits are in progress
   - Verify consistent history view

### 3.3. Search Index Concurrency

**Test Objective:** Verify index reader-writer separation.

**Test Cases:**
1. **Concurrent Indexing:**
   - 5 threads indexing different documents
   - Verify no index corruption
   - Verify all documents are searchable

2. **Search During Index:**
   - Start indexing operation
   - Execute search queries
   - Verify searches complete or wait

3. **Reader Clones:**
   - 20 concurrent search operations
   - Verify IndexReader clones work correctly

### 3.4. WebSocket Concurrency

**Test Objective:** Verify broadcast channel thread safety.

**Test Cases:**
1. **Connection Map:**
   - 1000 concurrent connections
   - Add/remove connections simultaneously
   - Verify no race conditions

2. **Broadcast Stress:**
   - 1000 connections, 10,000 messages/second
   - Measure broadcast latency
   - Verify all connections receive messages

---

## 4. Deadlock Prevention Testing

### 4.1. Lock Ordering Verification

**Test Objective:** Verify global lock ordering prevents circular waits.

**Lock Order (from deadlock_analysis.md):**
```
1. cache_lock          (LRU cache)
2. stats_lock           (Atomic counters, usually no lock)
3. index_reader_lock    (Search index read)
4. index_writer_lock   (Search index write)
5. git_read_lock       (Git repository read)
6. git_write_lock      (Git repository write)
7. ws_connections_lock (WebSocket connections)
```

**Test:**
```rust
#[tokio::test]
async fn test_lock_ordering_compliance() {
    // Verify locks are acquired in global order
    let cache = LruCache::new(100);
    let git_repo = GitRepository::new("./test_repo").await;
    let search_index = SearchIndex::new().await;

    // Acquire locks in order: cache -> git_write -> index_writer
    let _cache_guard = cache.lock.write().await;
    let _git_guard = git_repo.lock.write().await;
    let _index_guard = search_index.lock.write().await;

    // All locks held - should not deadlock
    // Release in reverse order
}
```

### 4.2. Timeout Testing

**Test Objective:** Verify timeout mechanisms prevent indefinite blocking.

**Timeout Configuration:**
| Lock Type | Timeout | Test Case |
|-----------|----------|-----------|
| Cache read | 10ms | Hold lock, attempt concurrent read |
| Cache write | 100ms | Hold lock, attempt concurrent write |
| Git read | 1000ms | Hold lock, attempt concurrent read |
| Git write | 5000ms | Hold lock, attempt concurrent write |
| Index write | 3000ms | Hold lock, attempt concurrent write |

---

## 5. Metrics and Observability

### 5.1. Concurrency Metrics

| Metric | Measurement | Tool |
|---------|-------------|-------|
| Lock contention time | Histogram | tracing |
| Lock wait time | Histogram | tracing |
| Thread count | Counter | tracing |
| Deadlock detection | Boolean | custom monitor |
| Race condition count | Counter | custom monitor |

### 5.2. Instrumentation

```rust
use tracing::{info, warn, instrument};

#[instrument(skip(self))]
pub async fn cache_get(&self, key: &str) -> Option<String> {
    let start = Instant::now();
    let result = self.map.get(key);
    let elapsed = start.elapsed();

    if elapsed > Duration::from_millis(10) {
        warn!(key = %key, latency_ms = %elapsed.as_millis(), "Cache get slow");
    }

    result
}
```

---

## 6. Consequences

### 6.1. Positive Consequences

1. **Thread Safety Verification:**
   - Loom explores all thread interleavings
   - High concurrency stress tests reveal race conditions
   - Confident thread-safe implementation

2. **Deadlock Prevention:**
   - Global lock ordering verified
   - Timeout mechanisms tested
   - No circular wait conditions

3. **Performance Understanding:**
   - Lock contention measured
   - Bottlenecks identified
   - Optimization targets defined

### 6.2. Negative Consequences

1. **Loom Execution Time:**
   - Loom tests can take significant time
   - CI/CD pipeline timeout increases
   - (Mitigation: Limit loom preemption bound)

2. **Test Complexity:**
   - Concurrent tests are difficult to debug
   - Race conditions may be non-deterministic
   - (Mitigation: Use loom for determinism, add logging)

3. **Platform-Specific Behavior:**
   - tokio behavior may differ across platforms
   - Thread scheduling varies by OS
   - (Mitigation: Test on Linux, macOS, Windows)

### 6.3. Mitigation Strategies

1. **Loom Preemption Bound:**
   - Set maximum preemptions per operation
   - Limit test complexity
   - Use `max-threads = 4` in loom configuration

2. **Race Condition Logging:**
   - Add structured logging to critical sections
   - Use `instrument` attribute for automatic metrics
   - Review logs after test runs

3. **Timeout Fallback:**
   - Implement graceful degradation on timeout
   - Return error rather than block indefinitely
   - Document timeout behavior in API docs

---

## 7. Compliance

### 7.1. Standards Compliance

| Standard | Requirement | Status |
|-----------|--------------|---------|
| IEEE 1016-2009 | Software Design Description | COMPLIANT |
| ISO/IEC 25010 | Time behavior under load | COMPLIANT |
| OWASP ASVS V2 | Concurrent access controls | COMPLIANT |

### 7.2. Requirement Traceability

| Requirement | Concurrency Test | Coverage |
|-------------|-----------------|-----------|
| PF-RQ-003 | tokio runtime tests | 100% |
| CM-RQ-007 | Concurrent commit tests | 100% |
| RE-RQ-005 | Cache concurrency tests | 100% |
| SD-RQ-002 | Search concurrency tests | 100% |
| IN-RQ-004 | WebSocket concurrency tests | 100% |

---

## 8. Approval

**Status:** ACCEPTED
**Approved By:** Breaker (Prototyper) Agent
**Date:** 2026-02-11
**Rationale:** Concurrency testing strategy using loom and stress tests provides comprehensive race condition and deadlock detection for critical paths.

---

## 9. References

- [Thread Safety Analysis](../.specs/02_5_concurrency/thread_safety_analysis.md)
- [Deadlock Analysis](../.specs/02_5_concurrency/deadlock_analysis.md)
- [ADR-040: Prototype Architecture](./adr-040-prototype-architecture.md)
- [ADR-042: Fuzzing Strategy](./adr-042-fuzzing-strategy.md)
- [Memory Management](../.specs/03_5_resource_management/memory_management.md)
- [Handle Management](../.specs/03_5_resource_management/handle_management.md)
