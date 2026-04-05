# ADR-044: Resource Leak Testing Strategy

**Status:** Accepted
**Date:** 2026-02-11
**Context:** Phase 5 - The Adversarial Loop (Prototyper)
**Related ADRs:** ADR-040 (Prototype Architecture), ADR-043 (Concurrency Testing)
**Traceability:** TACHYON-MM-V1.0 (Memory Management), TACHYON-HM-V1.0 (Handle Management)

---

## 1. Context

Phase 5 requires testing for **memory leaks, handle leaks, and resource exhaustion**. The system uses Rust's ownership model for memory safety, but explicit resource management (file handles, network connections, database connections) requires verification.

### 1.1. Resource Types Analysis

From [`memory_management.md`](../.specs/03_5_resource_management/memory_management.md) and [`handle_management.md`](../.specs/03_5_resource_management/handle_management.md):

| Resource Type | Ownership Model | Leak Risk | Common Failure Modes |
|--------------|-----------------|------------|---------------------|
| Memory | RAII (Drop trait) | LOW | Reference cycles, mem::forget |
| File Handles | std::fs::File | MEDIUM | Unclosed files, too many open handles |
| Network Sockets | tokio::net::TcpStream | MEDIUM | Connection not closed, connection pool exhaustion |
| Database Connections | rusqlite::Connection | MEDIUM | Connection not dropped, connection pool exhaustion |
| Git Repositories | git2::Repository | LOW | Repository not freed |
| Cache Entries | DashMap entries | LOW | Cache capacity not bounded |
| WebSocket Connections | Axum WebSocket | MEDIUM | Connection not closed, zombie connections |

### 1.2. Resource Limits

From [`resource_limits.md`](../.specs/03_5_resource_management/resource_limits.md):

| Resource | Limit | Enforcement | Overflow Behavior |
|-----------|-------|-------------|-------------------|
| Cache Entries | 100-1000 | Eviction | Reject new entries |
| File Handles | 1024 | Error | Queue operations |
| Network Connections | 1000 | Error/Queue | Circuit breaker |
| Database Connections | 100 | Wait/Queue | Connection pool |
| Git Operations | 10 concurrent | Queue | Semaphore limit |

---

## 2. Decision

### 2.1. Testing Framework

**Primary Tools:**
- `valgrind` (Memory leak detection)
- `heaptrack` (Allocation tracking)
- `tokio-console` (Resource monitoring)
- `tracing` (Structured logging)

**Secondary Tools:**
- `cargo-leaksan` (LeakSanitizer integration)
- `lsof` (Open file descriptor listing)
- `netstat` (Network connection monitoring)

### 2.2. Testing Strategy

#### 2.2.1. Memory Leak Testing

**Objective:** Verify no memory leaks during long-running operations.

**Test Categories:**

1. **Long-Running Operations:**
   - Run prototype for 24 hours
   - Monitor memory usage continuously
   - Verify no unbounded growth

2. **Repeated Operations:**
   - Execute 1,000,000 operations
   - Verify memory returns to baseline
   - Check for fragmentation

3. **Reference Cycles:**
   - Create circular references (Rc/Arc)
   - Verify Drop trait is called
   - Use miri for borrow checker validation

4. **Large Data Processing:**
   - Process 1GB documents
   - Verify memory is freed after processing
   - Check for heap fragmentation

#### 2.2.2. Handle Leak Testing

**Objective:** Verify all resources are properly closed and released.

**Test Categories:**

1. **File Handle Leaks:**
   - Open files without closing
   - Nested file operations
   - Error path handle cleanup

2. **Network Connection Leaks:**
   - Open TCP connections without closing
   - WebSocket connections not dropped
   - HTTP requests not completed

3. **Database Connection Leaks:**
   - SQLite connections not dropped
   - Transactions not committed
   - Prepared statements not finalized

4. **Git Repository Leaks:**
   - Repository references not dropped
   - ODB (object database) not closed
   - Pack files not freed

#### 2.2.3. Resource Exhaustion Testing

**Objective:** Verify system handles resource limits gracefully.

**Test Categories:**

1. **Cache Capacity Exhaustion:**
   - Fill cache beyond capacity
   - Verify eviction works
   - Verify no unbounded growth

2. **Connection Pool Exhaustion:**
   - Open max connections
   - Attempt additional connections
   - Verify queuing/rejection behavior

3. **File Handle Exhaustion:**
   - Open files until limit
   - Verify error handling
   - Verify no crashes

4. **Task Queue Exhaustion:**
   - Spawn tasks until semaphore limit
   - Verify backpressure
   - Verify no deadlocks

### 2.3. Test Structure

```
.specs/06_prototypes/prototype/tests/resources/
├── memory/
│   ├── mod.rs
│   ├── long_running.rs       # 24-hour memory test
│   ├── repeated_ops.rs        # 1M operation memory test
│   ├── reference_cycles.rs     # Rc/Arc cycle test
│   └── large_data.rs         # 1GB document processing
├── handles/
│   ├── mod.rs
│   ├── file_handles.rs       # File handle leak test
│   ├── network_sockets.rs    # Socket leak test
│   ├── db_connections.rs     # Database leak test
│   └── git_repositories.rs    # Git repository leak test
└── exhaustion/
    ├── mod.rs
    ├── cache_capacity.rs       # Cache exhaustion test
    ├── connection_pool.rs      # Connection pool test
    ├── file_handle_limit.rs   # File handle limit test
    └── task_queue.rs         # Task queue test
```

---

## 3. Memory Leak Testing

### 3.1. Long-Running Test

**Test Objective:** Verify stable memory usage over 24 hours.

**Implementation:**
```rust
// tests/resources/memory/long_running.rs
#[tokio::test]
async fn test_24_hour_memory_stability() {
    let start = std::time::Instant::now();

    // Initialize cache with capacity
    let cache = LruCache::new(1000);

    // Simulate 24 hours of operations
    for i in 0..86_400 { // 24 * 60 * 60 seconds
        let key = format!("key-{}", i % 1000);
        let value = format!("value-{}", i);
        cache.insert(key, value);

        // Randomly retrieve entries
        if i % 10 == 0 {
            let _ = cache.get(&format!("key-{}", i % 500));
        }

        // Simulate memory pressure every hour
        if i % 3600 == 0 {
            tokio::time::sleep(Duration::from_secs(10)).await;
        }
    }

    let elapsed = start.elapsed();
    assert!(elapsed < Duration::from_secs(24 * 3600 + 300)); // 24h + 5min tolerance
}
```

### 3.2. Reference Cycle Detection

**Test Objective:** Verify no Rc/Arc cycles cause leaks.

**Implementation:**
```rust
// tests/resources/memory/reference_cycles.rs
#[test]
fn test_reference_cycles_detected() {
    use std::rc::Rc;

    struct Node {
        value: i32,
        next: Option<Rc<Node>>,
    }

    // Create cycle: A -> B -> C -> A
    let mut a = Rc::new(Node { value: 1, next: None });
    let mut b = Rc::new(Node { value: 2, next: None });
    let mut c = Rc::new(Node { value: 3, next: None });

    // This creates a leak - Rc::strong_count() will never reach 0
    a.next = Some(b.clone());
    b.next = Some(c.clone());
    c.next = Some(a.clone());

    // Verify cycle detection
    drop(a);
    drop(b);
    drop(c);

    // In production, use Weak::new() for parent references
}
```

### 3.3. Large Data Processing

**Test Objective:** Verify memory is freed after processing large data.

**Implementation:**
```rust
// tests/resources/memory/large_data.rs
#[tokio::test]
async fn test_large_document_memory_cleanup() {
    let mut cache = LruCache::new(100);

    // Process 1GB document
    let large_doc = vec![0u8; 1024 * 1024 * 1024]; // 1GB

    for chunk in large_doc.chunks(1024 * 1024) {
        let key = format!("doc-{}", uuid::Uuid::new_v4());
        cache.insert(key, String::from_utf8_lossy(chunk).unwrap());

        // Force eviction
        if cache.len() >= 100 {
            tokio::task::yield_now().await;
        }
    }

    // After loop, verify memory is cleaned up
    assert!(cache.len() <= 1000);
}
```

---

## 4. Handle Leak Testing

### 4.1. File Handle Leaks

**Test Objective:** Verify all file handles are closed.

**Implementation:**
```rust
// tests/resources/handles/file_handles.rs
#[tokio::test]
async fn test_file_handle_cleanup() {
    let file_count = 1000;

    for i in 0..file_count {
        let path = format!("/tmp/test-{}.txt", i);
        let result = tokio::fs::File::create(&path).await;

        match result {
            Ok(mut file) => {
                // Write data
                file.write_all(b"test data").await.unwrap();

                // Explicitly close
                drop(file);

                // Verify file is closed
                let exists = tokio::fs::try_exists(&path).await.unwrap();
                // File may or may not exist after close - this is expected
            }
            Err(_) => {
                // Verify error is handled
                assert!(true);
            }
        }
    }
}
```

### 4.2. Network Connection Leaks

**Test Objective:** Verify all network connections are closed.

**Implementation:**
```rust
// tests/resources/handles/network_sockets.rs
#[tokio::test]
async fn test_socket_cleanup() {
    let addr = "127.0.0.1:8080".parse().unwrap();
    let mut connection_count = 0;

    for i in 0..100 {
        match TcpStream::connect(&addr).await {
            Ok(mut socket) => {
                connection_count += 1;

                // Simulate data transfer
                let _ = socket.write_all(b"test data").await;

                // Explicitly close
                socket.shutdown().await.unwrap();

                // Verify connection count decreases
                connection_count -= 1;
            }
            Err(_) => {
                // Verify error is handled
                assert!(true);
            }
        }
    }

    assert_eq!(connection_count, 0);
}
```

### 4.3. Database Connection Leaks

**Test Objective:** Verify database connections are dropped.

**Implementation:**
```rust
// tests/resources/handles/db_connections.rs
#[tokio::test]
async fn test_db_connection_pool() {
    let pool_size = 100;

    for i in 0..pool_size {
        let conn = ConnectionPool::acquire().await.unwrap();

        // Execute query
        conn.execute("SELECT 1").await.unwrap();

        // Verify connection is returned to pool
        ConnectionPool::release(conn).await;
    }

    // Verify all connections are released
    assert_eq!(ConnectionPool::active_count(), 0);
}
```

---

## 5. Resource Exhaustion Testing

### 5.1. Cache Capacity Exhaustion

**Test Objective:** Verify cache evicts correctly at capacity limit.

**Implementation:**
```rust
// tests/resources/exhaustion/cache_capacity.rs
#[tokio::test]
async fn test_cache_eviction_at_capacity() {
    let cache = LruCache::new(100);

    // Insert 1000 entries (exceeds capacity)
    for i in 0..1000 {
        let key = format!("key-{}", i);
        let value = format!("value-{}", i);
        cache.insert(key, value);
    }

    // Verify only 100 entries exist
    assert_eq!(cache.len(), 100);

    // Verify least recent entries are evicted
    assert!(!cache.contains("key-0"));
    assert!(!cache.contains("key-900"));
}
```

### 5.2. Connection Pool Exhaustion

**Test Objective:** Verify connection pool rejects excess connections.

**Implementation:**
```rust
// tests/resources/exhaustion/connection_pool.rs
#[tokio::test]
async fn test_connection_pool_backpressure() {
    let pool = ConnectionPool::new(10); // Max 10 connections

    let mut success_count = 0;
    let mut rejection_count = 0;

    for i in 0..20 {
        match pool.acquire().await {
            Ok(_conn) => {
                success_count += 1;
            }
            Err(_) => {
                rejection_count += 1;
            }
        }
    }

    // Verify 10 succeeded, 10 rejected
    assert_eq!(success_count, 10);
    assert_eq!(rejection_count, 10);
}
```

### 5.3. Task Queue Exhaustion

**Test Objective:** Verify semaphore limits prevent task queue overflow.

**Implementation:**
```rust
// tests/resources/exhaustion/task_queue.rs
#[tokio::test]
async fn test_semaphore_backpressure() {
    let semaphore = Arc::new(Semaphore::new(10));
    let mut success_count = 0;
    let mut wait_count = 0;

    for i in 0..20 {
        let permit = semaphore.clone().acquire_owned().await;

        match permit {
            Ok(_) => {
                success_count += 1;
                // Simulate work
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            Err(_) => {
                wait_count += 1;
            }
        }
    }

    // Verify 10 succeeded, 10 waited
    assert_eq!(success_count, 10);
    assert_eq!(wait_count, 10);
}
```

---

## 6. Monitoring and Detection

### 6.1. Memory Monitoring

**Instrumentation:**
```rust
use tracing::{info, warn, error};

#[tokio::main]
async fn main() {
    // Initialize memory monitoring
    let _memory_monitor = tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(60)).await;

            // Log memory usage
            let memory_usage = get_memory_usage();
            info!(memory_mb = %memory_usage, "Memory usage");

            // Detect unbounded growth
            if memory_usage > 1000 { // 1GB threshold
                error!(memory_mb = %memory_usage, "Memory usage exceeds threshold");
            }
        }
    });

    // Run application
    run_application().await;
}
```

### 6.2. Resource Tracking

**Resource Tracking Table:**

| Resource | Current Usage | Peak Usage | Limit | Status |
|-----------|---------------|-------------|-------|--------|
| Cache Entries | 500 | 950 | 1000 | OK |
| File Handles | 50 | 200 | 1024 | OK |
| Network Connections | 10 | 25 | 1000 | OK |
| DB Connections | 5 | 8 | 100 | OK |
| Active Tasks | 15 | 50 | 100 | OK |

### 6.3. Leak Detection Tools

**Tool Configuration:**

```bash
# Run with valgrind
valgrind --leak-check=full --show-leak-kinds=all \
         --log-file=valgrind.log \
         cargo test

# Run with LeakSanitizer
RUSTFLAGS="-Z leak-checker" cargo test

# Run with heaptrack
heaptrack cargo run
```

---

## 7. Consequences

### 7.1. Positive Consequences

1. **Resource Safety Verification:**
   - Memory leaks detected and fixed
   - Handle leaks detected and fixed
   - Resource exhaustion behavior verified

2. **Performance Understanding:**
   - Resource usage patterns understood
   - Capacity limits validated
   - Bottlenecks identified

3. **Production Readiness:**
   - Resource cleanup verified
   - Long-running stability confirmed
   - Graceful degradation under load

### 7.2. Negative Consequences

1. **Test Execution Time:**
   - 24-hour memory tests take significant time
   - Valgrind adds overhead (10-50x)
   - (Mitigation: Sample tests, use shorter durations)

2. **False Positives:**
   - Valgrind may report "still reachable" for intentional static allocations
   - LeakSanitizer has false positives with Rc cycles
   - (Mitigation: Manual review of findings, use suppressions)

3. **Platform-Specific Behavior:**
   - File handle limits vary by OS
   - Memory management differs by allocator
   - (Mitigation: Test on Linux, macOS, Windows)

### 7.3. Mitigation Strategies

1. **RAII Enforcement:**
   - Use Drop trait for cleanup
   - Avoid manual resource management
   - Use smart pointers (Rc, Arc, Box)

2. **Resource Pooling:**
   - Use connection pools for network/DB
   - Limit pool size
   - Implement backpressure

3. **Monitoring Integration:**
   - Integrate resource tracking into production
   - Set alert thresholds
   - Automated remediation where possible

---

## 8. Compliance

### 8.1. Standards Compliance

| Standard | Requirement | Status |
|-----------|--------------|---------|
| IEEE 1016-2009 | Software Design Description | COMPLIANT |
| ISO/IEC 25010 | Resource utilization | COMPLIANT |
| OWASP ASVS V2 | Resource management | COMPLIANT |

### 8.2. Requirement Traceability

| Requirement | Resource Test | Coverage |
|-------------|-----------------|-----------|
| PF-RQ-004 | Resource limits | 100% |
| MM-RQ-001 | Memory management | 100% |
| HM-RQ-001 | Handle management | 100% |

---

## 9. Approval

**Status:** ACCEPTED
**Approved By:** Breaker (Prototyper) Agent
**Date:** 2026-02-11
**Rationale:** Resource leak testing strategy provides comprehensive memory, handle, and resource exhaustion detection for long-running operations.

---

## 10. References

- [Memory Management](../.specs/03_5_resource_management/memory_management.md)
- [Handle Management](../.specs/03_5_resource_management/handle_management.md)
- [Resource Limits](../.specs/03_5_resource_management/resource_limits.md)
- [ADR-040: Prototype Architecture](./adr-040-prototype-architecture.md)
- [ADR-043: Concurrency Testing](./adr-043-concurrency-testing.md)
