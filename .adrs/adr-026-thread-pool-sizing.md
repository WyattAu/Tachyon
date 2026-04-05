# ADR-026: Thread Pool Sizing

## Status

| Status | Accepted |
|---------|----------|
| Date | 2026-02-11 |
| Decision | Adopt adaptive thread pool sizing with tokio multi-threaded scheduler |
| Context | Optimal concurrency and resource utilization |

---

## Context and Problem Statement

### Current Situation

Tachyon system uses tokio async runtime for concurrent task execution across rendering, search indexing, Git operations, WebSocket handling, and file watching. Thread pool sizing directly impacts:
- System responsiveness
- Resource utilization (CPU, memory)
- Latency and throughput
- Context switching overhead

### Problem

Without proper thread pool sizing:
1. Undersized pools cause queue buildup and latency
2. Oversized pools waste CPU on context switching
3. Fixed sizes cannot adapt to workload changes
4. Blocking tasks may starve async tasks
5. CPU-bound and I/O-bound tasks have different optimal sizes

### Constraints

| Constraint | Value | Source |
|------------|--------|---------|
| Max rendering tasks | 100 | resource_limits.md:67-78 |
| Max concurrent searches | 50 | resource_limits.md:67-78 |
| Max async tasks | 10000 | resource_limits.md:169-177 |
| Max blocking tasks | 10 | resource_limits.md:169-177 |
| CPU usage alert | 80% | resource_limits.md:215-232 |

---

## Decision Drivers

| Factor | Impact | Weight |
|---------|--------|--------|
| Performance (RE-RQ-001) | CRITICAL | 35% |
| Resource Efficiency (PF-RQ-001) | CRITICAL | 25% |
| Responsiveness | HIGH | 20% |
| Adaptability | MEDIUM | 20% |

---

## Considered Alternatives

### Alternative 1: Single-Threaded Runtime

**Description:** Use tokio current-thread scheduler (1 thread).

**Pros:**
- Minimal context switching
- Simple configuration
- Low memory overhead

**Cons:**
- Cannot utilize multiple cores
- Blocking tasks block all operations
- Poor scalability
- Cannot meet <15ms latency under load

**Evaluation:** REJECTED - Inadequate for multi-core systems

### Alternative 2: Fixed Multi-Threaded Pool

**Description:** Configure fixed thread count at startup.

**Pros:**
- Predictable behavior
- Simple to reason about
- Consistent performance

**Cons:**
- Cannot adapt to workload
- Always uses max cores
- Wastes resources on light load
- May oversubscribe CPU

**Configuration Example:**
```rust
tokio::Runtime::new().unwrap()
// Uses num_cpus::get() threads
// Fixed for lifetime
```

**Evaluation:** REJECTED - Inflexible for variable workloads

### Alternative 3: Separate Pools per Task Type

**Description:** Dedicated pools for CPU-bound, I/O-bound, blocking tasks.

**Pros:**
- Optimal sizing per task type
- Prevents task starvation
- Clear resource boundaries

**Cons:**
- Increased complexity
- Potential resource waste
- Higher memory overhead
- Difficult load balancing

**Architecture:**
```
CPU Pool: 4 threads
I/O Pool: 8 threads
Blocking Pool: 2 threads
```

**Evaluation:** REJECTED - Unnecessary complexity for tokio scheduler

### Alternative 4: Adaptive Multi-Threaded Runtime with Task Priorities (SELECTED)

**Description:** Use tokio multi-threaded scheduler with adaptive task scheduling and priority-based execution.

**Architecture:**

```
tokio Runtime Configuration:
  - Multi-threaded scheduler
  - Work-stealing algorithm
  - Adaptive worker threads
  - Per-task priority queues

Task Priorities:
  - Critical: Rendering (sub-15ms SLA)
  - High: WebSocket messages
  - Normal: File I/O
  - Low: Background indexing
```

**Sizing Strategy:**
```rust
struct ThreadPoolConfig {
    cpu_workers: usize,      // num_cpus::get()
    io_workers: usize,       // cpu_workers * 2
    blocking_workers: usize,  // max(2, cpu_workers / 2)
    max_blocking_threads: usize,
}

impl ThreadPoolConfig {
    fn new() -> Self {
        let cpu = num_cpus::get();
        ThreadPoolConfig {
            cpu_workers: cpu,
            io_workers: cpu * 2,
            blocking_workers: (cpu / 2).max(2),
            max_blocking_threads: cpu * 4,
        }
    }

    fn tokio_config(&self) -> tokio::runtime::Builder {
        let mut builder = tokio::runtime::Builder::new_multi_thread();
        builder.worker_threads(self.cpu_workers);
        builder.max_blocking_threads(self.max_blocking_threads);
        builder
    }
}
```

**Pros:**
- Adapts to workload automatically
- Optimal core utilization
- Work-stealing prevents idle cores
- Priority-based scheduling
- Meets <15ms latency target
- Blocking tasks don't starve async

**Cons:**
- Slightly more complex
- Requires priority tuning
- Some context switching overhead

**Evaluation:** ACCEPTED - Best balance of performance and adaptability

---

## Decision

**Adopt adaptive multi-threaded runtime with task priorities.**

### Rationale

1. **Performance Targets:**
   - Multi-threaded scheduling meets <15ms latency
   - Work-stealing utilizes all cores
   - Critical tasks get priority

2. **Resource Efficiency:**
   - Adaptive sizing prevents waste
   - Blocking threads isolated from async
   - No over-subscription of CPU

3. **Responsiveness:**
   - Priority queues ensure responsiveness
   - Critical tasks preempt lower priority
   - WebSocket messages handled promptly

4. **tokio Ecosystem:**
   - Native tokio support
   - Work-stealing scheduler
   - Well-tested in production

5. **Workload Adaptation:**
   - Automatically adjusts to load
   - No manual tuning required
   - Handles burst traffic

---

## Implementation Plan

### Phase 1: Runtime Configuration

**Tasks:**
- Configure multi-threaded tokio runtime
- Set worker thread count to CPU count
- Configure blocking thread limit

**Traceability:** resource_limits.md:169-177

### Phase 2: Task Prioritization

**Tasks:**
- Implement critical task queue
- Implement high priority queue
- Implement normal priority queue
- Implement low priority queue

**Traceability:** resource_limits.md:67-78

### Phase 3: Blocking Thread Isolation

**Tasks:**
- Configure max blocking threads
- Use spawn_blocking for CPU tasks
- Separate from async worker pool

**Traceability:** resource_limits.md:169-177

### Phase 4: Metrics and Adaptation

**Tasks:**
- Monitor thread utilization
- Track queue depths
- Adapt worker count based on load

**Traceability:** resource_limits.md:215-232

---

## Consequences

### Positive Consequences

1. **Performance:**
   - Sub-15ms latency maintained
   - All CPU cores utilized
   - Work-stealing prevents idle cores

2. **Resource Efficiency:**
   - No over-provisioning
   - Adaptive sizing prevents waste
   - Blocking tasks isolated

3. **Responsiveness:**
   - Critical tasks prioritized
   - WebSocket handling responsive
   - User-facing operations prioritized

### Negative Consequences

1. **Complexity:**
   - Priority tuning required
   - More complex than single-threaded

2. **Context Switching:**
   - Some overhead from multiple threads
   - Cache locality impact

---

## Monitoring and Validation

### Success Criteria

| Metric | Target | Measurement Method |
|---------|--------|-------------------|
| Rendering latency P95 | <15ms | tokio-console |
| Search latency P95 | <100ms | tokio-console |
| CPU utilization | 60-80% | system monitoring |
| Queue depth | <100 | tokio metrics |
| Thread idle rate | <10% | tokio metrics |

### Testing Strategy

1. **Unit Tests:**
   - Test task priority execution
   - Verify blocking thread isolation
   - Test queue depth limits

2. **Load Tests:**
   - Test under varying loads
   - Measure latency at each load
   - Verify core utilization

3. **Benchmarking:**
   - Compare single vs multi-threaded
   - Measure context switching overhead
   - Profile CPU utilization

---

## Related Decisions

- [ADR-007](adr-007-thread-safety-strategy.md) - Thread safety primitives
- [ADR-008](adr-008-deadlock-prevention.md) - Lock ordering
- [ADR-024](adr-024-memory-management-strategy.md) - Task memory management

---

## References

1. **Research Sources:**
   - resource_limits.md:67-78 (Task Concurrency Limits)
   - resource_limits.md:169-177 (Thread Pool Configuration)

2. **Requirements:**
   - requirements.md:RE-RQ-001 (JIT Rendering)
   - requirements.md:SD-RQ-001 (Search Engine)
   - requirements.md:IN-RQ-004 (WebSocket API)
   - PF-RQ-001 (Performance Requirements)

3. **Architecture:**
   - thread_safety_analysis.md:291-305 (Async Task Scheduler)
   - blue_paper.md:193-197 (IF-001 Async Runtime)

4. **Dependencies:**
   - dep_spec/tokio/dep_spec.toml:18-31 (tokio scheduler)

---

**Document Revision History:**

| Version | Date | Author | Changes |
|---------|-------|--------|---------|
| 1.0 | 2026-02-11 | Initial ADR creation |
