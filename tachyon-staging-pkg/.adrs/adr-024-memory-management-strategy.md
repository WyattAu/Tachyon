# ADR-024: Memory Management Strategy

## Status

| Status | Accepted |
|---------|----------|
| Date | 2026-02-11 |
| Decision | Adopt Rust RAII memory management with Arc-based shared ownership |
| Context | Memory leak prevention and optimal memory usage |

---

## Context and Problem Statement

### Current Situation

Tachyon system manages multiple memory-intensive components including LRU cache, search index, Git objects, WebSocket buffers, and rendering AST. Memory leaks and excessive memory consumption can lead to:
- System instability
- Performance degradation
- Out-of-memory (OOM) crashes
- Resource exhaustion attacks

### Problem

Without a structured memory management strategy:
1. Manual memory management is error-prone
2. Shared state across threads requires synchronization
3. Cache eviction policies need careful design
4. Memory growth must be bounded
5. Leaks in long-running processes accumulate

### Constraints

| Constraint | Value | Source |
|------------|--------|---------|
| Max rendering latency | <15ms | domain_constraints.toml:32-48 |
| Cache hit response | <1ms | domain_constraints.toml:59-70 |
| Desktop memory budget | 1.5GB | resource_limits.md:34-50 |
| Server memory budget | 6GB | resource_limits.md:51-66 |
| Search index memory | 512MB | resource_limits.md:148-158 |

---

## Decision Drivers

| Factor | Impact | Weight |
|---------|--------|--------|
| Memory Safety (PF-RQ-003) | CRITICAL | 40% |
| Performance (RE-RQ-001) | CRITICAL | 30% |
| Maintainability | HIGH | 15% |
| Rust Idioms | MEDIUM | 15% |

---

## Considered Alternatives

### Alternative 1: Manual Memory Management (C-style)

**Description:** Explicit malloc/free with manual tracking.

**Pros:**
- Maximum control over allocation
- No language overhead
- Precise memory layout

**Cons:**
- High leak risk (forgotten free)
- Use-after-free bugs
- Double-free vulnerabilities
- No thread safety guarantees
- Violates Rust best practices

**Evaluation:** REJECTED - Unacceptable security and reliability risks

### Alternative 2: Garbage Collection (Go/Java-style)

**Description:** Rely on GC for automatic cleanup.

**Pros:**
- Automatic memory reclamation
- No manual deallocation
- Simpler code

**Cons:**
- Non-deterministic pauses (STW)
- GC overhead
- Higher memory usage
- Latency spikes during collection
- Poor fit for real-time requirements

**Evaluation:** REJECTED - Incompatible with <15ms latency target

### Alternative 3: Reference Counting with GC (Swift-style)

**Description:** Hybrid of reference counting and GC.

**Pros:**
- Automatic cleanup for cycles
- Reference counting for immediate release

**Cons:**
- Complex implementation
- Cycle detection overhead
- Still has GC pauses
- Language not Rust

**Evaluation:** REJECTED - Not applicable to Rust ecosystem

### Alternative 4: Rust RAII with Arc-based Shared Ownership (SELECTED)

**Description:** Leverage Rust ownership, Drop trait, and Arc for thread-safe sharing.

**Architecture:**

```
Ownership Model:
  - Single owner per value
  - Move semantics for transfers
  - Borrow checker prevents data races

Drop Trait:
  - Automatic cleanup on scope exit
  - Compiler-enforced implementation
  - Panic-safe cleanup

Arc<T> for Sharing:
  - Atomic reference counting
  - Thread-safe (Send + Sync)
  - Weak references for cycle prevention

Memory Pools:
  - Arena allocation for short-lived data
  - Buffer pooling for I/O
  - String interning for deduplication
```

**Pros:**
- Zero memory leaks (Drop guarantee)
- No use-after-free (borrow checker)
- Thread-safe (Send/Sync traits)
- Zero GC overhead
- Deterministic cleanup
- Compiler-time safety checks

**Cons:**
- Learning curve for ownership model
- Careful Arc usage required for shared state
- Weak references needed for cycles

**Evaluation:** ACCEPTED - Best balance of safety, performance, and maintainability

---

## Decision

**Adopt Rust RAII memory management with Arc-based shared ownership.**

### Rationale

1. **Memory Safety Guarantees:**
   - Drop trait ensures cleanup
   - Borrow checker prevents data races
   - Compiler enforces memory safety
   - No manual memory management errors

2. **Performance Targets:**
   - Zero GC overhead maintains <15ms latency
   - Arc shared ownership enables cache hits <1ms
   - Arena allocation reduces allocation overhead

3. **Thread Safety:**
   - Arc provides atomic reference counting
   - Send/Sync traits enforce thread safety
   - DashMap for concurrent cache access

4. **Memory Boundaries:**
   - Configurable limits per component
   - LRU eviction maintains bounded memory
   - Metrics for proactive monitoring

5. **Rust Ecosystem Alignment:**
   - tokio async runtime (1.49.0) uses Arc
   - DashMap (5.5.3) uses Arc internally
   - Tantivy (0.21.1) uses Arc for index sharing

---

## Implementation Plan

### Phase 1: Drop Trait Implementation

**Tasks:**
- Implement Drop for all custom types
- Ensure Drop is panic-safe
- Test Drop on all code paths

**Traceability:** memory_management.md:75-85

### Phase 2: Arc-based Sharing

**Tasks:**
- Use Arc for cross-thread shared data
- Implement Weak references for cycles
- Optimize Arc cloning with Cow pattern

**Traceability:** memory_management.md:52-61

### Phase 3: Memory Pools

**Tasks:**
- Implement arena allocation for AST
- Implement buffer pooling for I/O
- Implement string interning for cache keys

**Traceability:** memory_management.md:221-247

### Phase 4: Limit Enforcement

**Tasks:**
- Enforce component memory budgets
- Implement LRU eviction
- Add memory monitoring

**Traceability:** resource_limits.md:34-158

---

## Consequences

### Positive Consequences

1. **Memory Safety:**
   - Zero memory leaks guaranteed
   - No use-after-free possible
   - Compile-time safety checks

2. **Performance:**
   - Zero GC overhead
   - Deterministic cleanup
   - Sub-1ms cache hits achievable

3. **Thread Safety:**
   - Arc ensures safe sharing
   - DashMap for concurrent access
   - Lock-free atomic operations

4. **Maintainability:**
   - Clear ownership model
   - Compiler enforces correctness
   - No manual memory management

### Negative Consequences

1. **Complexity:**
   - Ownership model has learning curve
   - Arc cloning adds cognitive overhead

2. **Reference Cycles:**
   - Arc cycles prevent deallocation
   - Weak references required

3. **Memory Overhead:**
   - Arc adds pointer + counter overhead
   - Weak references add complexity

---

## Monitoring and Validation

### Success Criteria

| Metric | Target | Measurement Method |
|---------|--------|-------------------|
| Memory leak rate | 0 | heaptrack, valgrind |
| RSS stability | <5% growth/hr | system monitoring |
| Cache memory | <90% of budget | internal metrics |
| Index memory | <90% of budget | Tantivy API |
| Task memory | <1GB | tokio-console |

### Testing Strategy

1. **Unit Tests:**
   - Test Drop on all paths
   - Verify Arc reference counting
   - Test pool deallocation

2. **Integration Tests:**
   - Long-running memory tests
   - Stress tests for limits
   - Leak detection runs

3. **Static Analysis:**
   - Clippy lint for memory issues
   - Miri for undefined behavior
   - Loom for concurrency

---

## Related Decisions

- [ADR-025](adr-025-handle-lifecycle-management.md) - Handle RAII enforcement
- [ADR-026](adr-026-thread-pool-sizing.md) - Task memory sizing
- [ADR-027](adr-027-resource-leak-detection.md) - Leak detection mechanisms

---

## References

1. **Research Sources:**
   - memory_management.md:13-27 (Rust Memory Model)
   - memory_management.md:75-85 (RAII Pattern)

2. **Requirements:**
   - requirements.md:255-269 (RE-RQ-001)
   - requirements.md:315-329 (RE-RQ-005)
   - PF-RQ-003 (Memory safety)

3. **Architecture:**
   - blue_paper.md:134-143 (RE-002 LRU Cache)
   - thread_safety_analysis.md:269-290 (Thread Safety)

4. **Dependencies:**
   - dep_spec/tokio/dep_spec.toml:18-31 (tokio memory model)
   - dep_spec/tantivy/dep_spec.toml (Tantivy Arc usage)

---

**Document Revision History:**

| Version | Date | Author | Changes |
|---------|-------|--------|---------|
| 1.0 | 2026-02-11 | Initial ADR creation |
