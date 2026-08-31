# ADR-025: Handle Lifecycle Management

## Status

| Status | Accepted |
|---------|----------|
| Date | 2026-02-11 |
| Decision | Adopt Rust Drop trait for automatic handle cleanup |
| Context | Handle leak prevention and resource cleanup |

---

## Context and Problem Statement

### Current Situation

Tachyon system manages numerous handle types including file handles, socket handles, database connections, Git repository handles, watch descriptors, and WebSocket connections. Each handle represents a system resource that must be properly acquired and released.

### Problem

Without structured handle lifecycle management:
1. Manual cleanup is error-prone (forgotten close)
2. Exception handling complicates resource release
3. Thread sharing requires careful synchronization
4. Resource leaks accumulate in long-running processes
5. Use-after-close bugs cause undefined behavior

### Constraints

| Constraint | Value | Source |
|------------|--------|---------|
| Max file handles | 1000 | resource_limits.md:112-119 |
| Max watch descriptors | 8192 | resource_limits.md:112-119 |
| Max WebSocket connections | 10000 | resource_limits.md:87-97 |
| Connection timeout | 5s | handle_management.md:147-155 |
| Handle leak rate | 0 | memory_management.md:257-286 |

---

## Decision Drivers

| Factor | Impact | Weight |
|---------|--------|--------|
| Resource Safety (PF-RQ-003) | CRITICAL | 40% |
| Exception Safety | CRITICAL | 30% |
| Thread Safety (ADR-007) | HIGH | 20% |
| Code Maintainability | MEDIUM | 10% |

---

## Considered Alternatives

### Alternative 1: Manual Cleanup (C-style)

**Description:** Explicit open/close with manual tracking.

**Pros:**
- Explicit control
- No language overhead
- Predictable cleanup timing

**Cons:**
- High leak risk (forgotten close)
- Exception safety requires try-finally
- Use-after-close bugs
- No compiler enforcement
- Error-prone in complex code

**Evaluation:** REJECTED - Unacceptable reliability risks

### Alternative 2: Scope Guards (Python-style with)

**Description:** Use pattern for automatic cleanup.

**Pros:**
- Automatic cleanup on scope exit
- Exception-safe
- Explicit lifetime

**Cons:**
- Runtime overhead
- Pattern-based (not enforced by compiler)
- Not idiomatic in Rust
- Can be forgotten

**Evaluation:** REJECTED - Not leveraging Rust compiler

### Alternative 3: Reference Counting (C++-style)

**Description:** Shared pointers with manual reference counting.

**Pros:**
- Automatic cleanup on last reference
- Shared ownership support

**Cons:**
- Manual increment/decrement
- Reference cycle leaks
- Thread safety requires atomics
- No compiler enforcement

**Evaluation:** REJECTED - Requires manual management

### Alternative 4: Rust Drop Trait (SELECTED)

**Description:** Leverage Rust ownership and Drop trait for automatic cleanup.

**Architecture:**

```
Drop Trait:
  - Compiler-enforced implementation
  - Automatic cleanup on scope exit
  - Panic-safe cleanup
  - No manual calls needed

Ownership Model:
  - Single owner per resource
  - Move semantics for transfer
  - Borrow checker prevents use-after-move

Arc for Sharing:
  - Atomic reference counting
  - Thread-safe (Send + Sync)
  - Last reference drops resource

Scope Guards:
  - RAII pattern for temporary handles
  - Automatic cleanup on exit
```

**Pros:**
- Zero handle leaks (Drop guarantee)
- Exception-safe (cleanup on panic)
- No use-after-close (borrow checker)
- Thread-safe with Arc
- Compiler-enforced correctness
- Zero runtime overhead

**Cons:**
- Requires Arc for sharing
- Careful lifetime management
- Learning curve for ownership

**Evaluation:** ACCEPTED - Best balance of safety and performance

---

## Decision

**Adopt Rust Drop trait for automatic handle cleanup.**

### Rationale

1. **Resource Safety:**
   - Drop trait guarantees cleanup
   - Compiler enforces implementation
   - No manual close calls needed

2. **Exception Safety:**
   - Drop runs on panic unwind
   - Resources always released
   - No try-finally required

3. **Thread Safety:**
   - Arc provides thread-safe sharing
   - Send/Sync traits enforced
   - Atomic reference counting

4. **Zero Overhead:**
   - Compile-time enforcement
   - No runtime cost for Drop
   - Deterministic cleanup

5. **Rust Idioms:**
   - Aligns with Rust ownership model
   - Uses standard library types
   - Leverages type system

---

## Implementation Plan

### Phase 1: Drop Implementation

**Tasks:**
- Implement Drop for all handle types
- Ensure Drop is panic-safe
- Test Drop on all code paths

**Traceability:** handle_management.md:75-85

### Phase 2: Arc-based Sharing

**Tasks:**
- Use Arc for cross-thread handles
- Implement handle pooling
- Optimize Arc cloning

**Traceability:** handle_management.md:238-268

### Phase 3: Handle Validation

**Tasks:**
- Validate handles on acquisition
- Check handle validity
- Error recovery strategies

**Traceability:** handle_management.md:200-238

### Phase 4: Leak Detection

**Tasks:**
- Implement handle metrics
- Track active handles
- Alert on leaks

**Traceability:** handle_management.md:269-307

---

## Consequences

### Positive Consequences

1. **Resource Safety:**
   - Zero handle leaks guaranteed
   - Exception-safe cleanup
   - No use-after-close possible

2. **Thread Safety:**
   - Arc ensures safe sharing
   - Lock-free atomic operations
   - Clear ownership semantics

3. **Maintainability:**
   - Clear lifetime rules
   - Compiler enforces correctness
   - No manual cleanup code

### Negative Consequences

1. **Complexity:**
   - Ownership model has learning curve
   - Arc cloning requires care

2. **Reference Cycles:**
   - Arc cycles prevent deallocation
   - Weak references needed

---

## Monitoring and Validation

### Success Criteria

| Metric | Target | Measurement Method |
|---------|--------|-------------------|
| Handle leak rate | 0 | handle_management.md:313-327 |
| Active handles stable | <5% drift | internal metrics |
| Cleanup on panic | 100% | exception tests |
| Thread-safe access | 100% | loom tests |

### Testing Strategy

1. **Unit Tests:**
   - Test Drop on all paths
   - Verify cleanup on panic
   - Test Arc sharing

2. **Integration Tests:**
   - Long-running handle tests
   - Concurrent access tests
   - Leak detection runs

3. **Static Analysis:**
   - Clippy lint for handle issues
   - Loom for concurrency safety
   - Miri for undefined behavior

---

## Related Decisions

- [ADR-024](adr-024-memory-management-strategy.md) - RAII memory management
- [ADR-026](adr-026-thread-pool-sizing.md) - Task handle sizing
- [ADR-007](adr-007-thread-safety-strategy.md) - Thread safety
- [ADR-008](adr-008-deadlock-prevention.md) - Lock ordering

---

## References

1. **Research Sources:**
   - handle_management.md:13-32 (Handle Types)
   - handle_management.md:75-127 (RAII Pattern)

2. **Requirements:**
   - requirements.md:CM-RQ-004 (File Watching)
   - requirements.md:IN-RQ-004 (WebSocket API)
   - PF-RQ-003 (Resource Safety)

3. **Architecture:**
   - blue_paper.md:182-195 (Git Operations)
   - blue_paper.md:202-208 (File Watcher)

4. **Dependencies:**
   - dep_spec/git2-rs/dep_spec.toml (git2-rs handles)
   - dep_spec/tokio/dep_spec.toml (tokio handles)

---

**Document Revision History:**

| Version | Date | Author | Changes |
|---------|-------|--------|---------|
| 1.0 | 2026-02-11 | Initial ADR creation |
