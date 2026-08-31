# ADR-027: Resource Leak Detection

## Status

| Status | Accepted |
|---------|----------|
| Date | 2026-02-11 |
| Decision | Adopt multi-layered leak detection with compile-time and runtime strategies |
| Context | Zero tolerance for resource leaks |

---

## Context and Problem Statement

### Current Situation

Tachyon system manages numerous resource types including memory, file handles, sockets, database connections, and task handles. Resource leaks in a long-running knowledge management system can lead to:
- Memory exhaustion and OOM crashes
- File descriptor exhaustion
- Database connection pool depletion
- Performance degradation over time
- Security vulnerabilities (resource exhaustion attacks)

### Problem

Without comprehensive leak detection:
1. Leaks accumulate silently over time
2. Manual inspection is error-prone
3. Runtime detection has overhead
4. Different leak types require different strategies
5. Leaks may not manifest until critical failure

### Constraints

| Constraint | Value | Source |
|------------|--------|---------|
| Leak tolerance | 0 | memory_management.md:257-286 |
| Handle leak rate | 0 | handle_management.md:313-327 |
| Memory growth rate | <5%/hr | resource_limits.md:215-232 |
| Active handles | <1000 | resource_limits.md:112-119 |

---

## Decision Drivers

| Factor | Impact | Weight |
|---------|--------|--------|
| Resource Safety (PF-RQ-003) | CRITICAL | 35% |
| System Reliability | CRITICAL | 30% |
| Security (SC-RQ-006) | CRITICAL | 20% |
| Maintainability | MEDIUM | 15% |

---

## Considered Alternatives

### Alternative 1: Manual Inspection

**Description:** Developers manually review code for leaks.

**Pros:**
- Zero runtime overhead
- Human insight into code patterns
- Flexible analysis approach

**Cons:**
- High error rate (missed leaks)
- Not scalable to large codebases
- Requires continuous vigilance
- No automated alerts
- Reactive (find leaks after damage)

**Evaluation:** REJECTED - Insufficient for production system

### Alternative 2: Runtime Periodic Scanning

**Description:** Periodically scan for unreleased resources.

**Pros:**
- Detects active leaks
- Can provide metrics
- Independent of code path

**Cons:**
- High runtime overhead
- Scanning interval trade-off
- May miss short-lived leaks
- Complex to implement correctly

**Architecture:**
```rust
struct LeakScanner {
    interval: Duration,
    resources: Arc<RwLock<HashMap<ResourceId, Resource>>>,
}

impl LeakScanner {
    async fn scan(&self) -> Vec<Leak> {
        let resources = self.resources.read().await;
        resources
            .iter()
            .filter(|r| r.age() > Duration::from_secs(300))
            .map(|r| Leak { id: r.id, type: r.type })
            .collect()
    }
}
```

**Evaluation:** REJECTED - Too much overhead for production

### Alternative 3: Compile-Time Enforcement Only

**Description:** Rely on Rust compiler to prevent leaks.

**Pros:**
- Zero runtime overhead
- Guaranteed prevention
- Compiler enforces correctness

**Cons:**
- Cannot detect logic errors
- Cannot handle external library leaks
- No runtime visibility
- No alerting capability

**Evaluation:** REJECTED - Inadequate for complete protection

### Alternative 4: Multi-Layered Detection with Compile-Time and Runtime Strategies (SELECTED)

**Description:** Combine Rust compile-time guarantees with targeted runtime monitoring.

**Architecture:**

```
Layer 1: Compile-Time Prevention (Zero Overhead)
  - Drop trait enforcement
  - Borrow checker
  - Send/Sync traits
  - Clippy linting

Layer 2: Static Analysis (Low Overhead)
  - Cargo check
  - Clippy lints
  - Miri undefined behavior detection

Layer 3: Targeted Runtime Monitoring (Focused)
  - Handle counters
  - Memory metrics
  - Threshold-based alerting
  - Not continuous scanning

Layer 4: Testing Validation (No Production Overhead)
  - Unit tests
  - Integration tests
  - Stress tests
  - Loom model checking
```

**Pros:**
- Compile-time: Zero overhead, guaranteed prevention
- Static analysis: Low overhead, catches errors early
- Runtime: Targeted, low overhead, provides visibility
- Testing: No production overhead, catches edge cases
- Defense in depth approach

**Cons:**
- More complex implementation
- Multiple tooling required
- Requires integration discipline

**Evaluation:** ACCEPTED - Best balance of coverage and overhead

---

## Decision

**Adopt multi-layered leak detection with compile-time guarantees and targeted runtime monitoring.**

### Rationale

1. **Compile-Time Prevention:**
   - Rust Drop trait guarantees cleanup
   - Borrow checker prevents use-after-free
   - Send/Sync traits enforce thread safety
   - Zero runtime overhead

2. **Static Analysis:**
   - Clippy catches leak patterns
   - Miri detects undefined behavior
   - Catches errors before deployment

3. **Targeted Runtime Monitoring:**
   - Counters have minimal overhead
   - Threshold-based alerting (not continuous)
   - Metrics provide visibility

4. **Testing Validation:**
   - Unit tests verify Drop behavior
   - Integration tests catch edge cases
   - Stress tests reveal leaks under load

5. **Defense in Depth:**
   - Multiple layers provide redundancy
   - Each layer catches different leak types
   - Failure in one layer compensated by others

---

## Implementation Plan

### Phase 1: Compile-Time Prevention

**Tasks:**
- Implement Drop for all custom types
- Enable Clippy leak lints
- Configure Miri for CI

**Traceability:** memory_management.md:75-85

### Phase 2: Static Analysis Integration

**Tasks:**
- Configure cargo check pipeline
- Add Clippy lint rules
- Enable Miri in CI/CD

**Clippy Configuration:**
```toml
[clippy]
deny = [
    "mem_forget",
    "mem_replace_option_with_none",
    "filetype_is_file",
    "temporary_cstring_as_ptr",
]

warn = [
    "mem_discriminant_non_enum",
]
```

**Traceability:** memory_management.md:393-425

### Phase 3: Runtime Metrics

**Tasks:**
- Implement handle counters
- Implement memory tracking
- Configure threshold alerts

**Counter Implementation:**
```rust
struct ResourceCounters {
    file_handles: Arc<AtomicUsize>,
    sockets: Arc<AtomicUsize>,
    db_connections: Arc<AtomicUsize>,
    tasks: Arc<AtomicUsize>,
}

impl ResourceCounters {
    fn track_acquire<T>(&self) {
        match std::any::TypeId::of::<T>() {
            id if id == std::any::TypeId::of::<File>() => {
                self.file_handles.fetch_add(1, Ordering::SeqCst);
            }
            id if id == std::any::TypeId::of::<Socket>() => {
                self.sockets.fetch_add(1, Ordering::SeqCst);
            }
            // ... other types
        }
    }

    fn track_release<T>(&self) {
        match std::any::TypeId::of::<T>() {
            id if id == std::any::TypeId::of::<File>() => {
                self.file_handles.fetch_sub(1, Ordering::SeqCst);
            }
            // ... other types
        }
    }
}
```

**Traceability:** memory_management.md:313-327

### Phase 4: Alert Integration

**Tasks:**
- Configure threshold alerts
- Integrate with monitoring system
- Define alert escalation

**Traceability:** resource_limits.md:215-232

---

## Consequences

### Positive Consequences

1. **Leak Prevention:**
   - Zero tolerance for resource leaks
   - Multiple detection layers
   - Early detection in development

2. **Performance:**
   - Minimal runtime overhead
   - No continuous scanning
   - Compile-time guarantees

3. **Observability:**
   - Metrics provide visibility
   - Alerts for proactive response
   - Historical leak tracking

### Negative Consequences

1. **Complexity:**
   - Multiple layers to maintain
   - Tooling integration required
   - Alert tuning needed

2. **False Positives:**
   - Threshold-based may alert on transient spikes
   - Requires careful calibration

---

## Monitoring and Validation

### Success Criteria

| Metric | Target | Measurement Method |
|---------|--------|-------------------|
| Compile-time leaks | 0 | Clippy, Miri |
| Runtime leaks | 0 | handle_management.md:313-327 |
| Memory leak rate | 0 | memory_management.md:257-286 |
| Alert accuracy | >95% | Alert verification |
| Test coverage | >90% | Code coverage |

### Testing Strategy

1. **Unit Tests:**
   - Test Drop implementation
   - Verify counter updates
   - Test threshold alerting

2. **Integration Tests:**
   - Long-running leak tests
   - Resource stress tests
   - Alert verification tests

3. **Static Analysis:**
   - Clippy in CI pipeline
   - Miri for undefined behavior
   - Loom for concurrency

**Test Example:**
```rust
#[tokio::test]
async fn test_no_handle_leaks() {
    let counters = ResourceCounters::new();

    for i in 0..1000 {
        let handle = FileHandle::open(&format!("file-{}.md", i)).unwrap();
        counters.track_acquire::<File>();
        // Use handle...
        drop(handle);
        counters.track_release::<File>();
    }

    assert_eq!(counters.file_handles.load(Ordering::SeqCst), 0);
}
```

---

## Related Decisions

- [ADR-024](adr-024-memory-management-strategy.md) - RAII memory management
- [ADR-025](adr-025-handle-lifecycle-management.md) - Handle Drop enforcement
- [ADR-028](adr-028-resource-limits-enforcement.md) - Limit enforcement

---

## References

1. **Research Sources:**
   - memory_management.md:257-286 (Memory Leak Prevention)
   - handle_management.md:313-327 (Handle Leak Detection)

2. **Requirements:**
   - requirements.md:PF-RQ-003 (Resource Safety)
   - requirements.md:SC-RQ-006 (DoS Prevention)
   - OWASP ASVS V5 (Resource Management)

3. **Architecture:**
   - thread_safety_analysis.md:307-318 (Thread Safety Guarantees)
   - deadlock_analysis.md:346-384 (Deadlock Prevention)

4. **Dependencies:**
   - dep_spec/tokio/dep_spec.toml (tokio memory safety)

---

**Document Revision History:**

| Version | Date | Author | Changes |
|---------|-------|--------|---------|
| 1.0 | 2026-02-11 | Initial ADR creation |
