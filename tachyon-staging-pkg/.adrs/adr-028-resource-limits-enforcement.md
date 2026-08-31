# ADR-028: Resource Limits Enforcement

## Status

| Status | Accepted |
|---------|----------|
| Date | 2026-02-11 |
| Decision | Adopt tiered limit enforcement with soft/hard limits and backpressure |
| Context | DoS prevention and resource stability |

---

## Context and Problem Statement

### Current Situation

Tachyon system must enforce resource limits to prevent exhaustion attacks, ensure fair resource allocation, and maintain system stability. Resources requiring limits include:
- Memory per component
- Network connections
- File handles
- Database connections
- Task concurrency
- Cache capacity

### Problem

Without structured limit enforcement:
1. No protection against resource exhaustion attacks
2. Unbounded memory growth causes OOM
3. No fairness in multi-tenant environments
4. System crashes under resource pressure
5. No visibility into resource utilization

### Constraints

| Constraint | Value | Source |
|------------|--------|---------|
| Desktop memory limit | 1.5GB | resource_limits.md:34-50 |
| Server memory limit | 6GB | resource_limits.md:51-66 |
| Max connections | 10000 | resource_limits.md:87-97 |
| Max concurrent tasks | 10000 | resource_limits.md:169-177 |
| Request rate limit | 1000/sec | resource_limits.md:183-201 |

---

## Decision Drivers

| Factor | Impact | Weight |
|---------|--------|--------|
| Security (SC-RQ-006) | CRITICAL | 35% |
| System Stability (PF-RQ-001) | CRITICAL | 30% |
| Fairness (AC-RQ-001) | HIGH | 20% |
| Performance (RE-RQ-001) | MEDIUM | 15% |

---

## Considered Alternatives

### Alternative 1: Unlimited Resources

**Description:** No limits on resource usage.

**Pros:**
- Maximum performance
- No rejection of requests
- Simplest implementation

**Cons:**
- DoS vulnerability
- Memory exhaustion
- System instability
- No fairness guarantee
- Crash under load

**Evaluation:** REJECTED - Security and stability risks

### Alternative 2: Hard Limits Only

**Description:** Reject requests exceeding limits with no soft warnings.

**Pros:**
- Simple implementation
- Enforced resource boundaries
- Predictable behavior

**Cons:**
- Poor user experience (hard rejections)
- No adaptation to load
- Wasted capacity during low usage
- No backpressure mechanism

**Architecture:**
```rust
struct HardLimitEnforcer {
    limit: usize,
}

impl HardLimitEnforcer {
    fn check(&self, current: usize) -> Result<()> {
        if current > self.limit {
            return Err(Error::LimitExceeded);
        }
        Ok(())
    }
}
```

**Evaluation:** REJECTED - Poor user experience

### Alternative 3: Per-User Quotas Only

**Description:** Enforce quotas per user without global limits.

**Pros:**
- Fairness between users
- Multi-tenant support
- Per-user accountability

**Cons:**
- No global resource protection
- Complex quota tracking
- Shared resource contention
- System-wide OOM possible

**Evaluation:** REJECTED - Inadequate system protection

### Alternative 4: Tiered Enforcement with Soft/Hard Limits and Backpressure (SELECTED)

**Description:** Implement three-tier limit enforcement with advisory soft limits, throttling, and hard rejection.

**Architecture:**

```
Tier 1: Advisory Limits (Warning Only)
  - 80% of limit
  - Log warning
  - Continue operation
  - No action required

Tier 2: Soft Limits (Apply Backpressure)
  - 90% of limit
  - Throttle operations
  - Return 503 (Service Unavailable)
  - Automatic retry recommended

Tier 3: Hard Limits (Reject Requests)
  - 100% of limit
  - Reject new requests
  - Return 429 (Too Many Requests)
  - Require manual intervention
```

**Implementation:**
```rust
struct TieredLimiter {
    advisory_threshold: usize,
    soft_threshold: usize,
    hard_limit: usize,
    current_usage: Arc<AtomicUsize>,
}

impl TieredLimiter {
    async fn acquire(&self, needed: usize) -> Result<Permit> {
        let current = self.current_usage.load(Ordering::SeqCst);
        let new_total = current.saturating_add(needed);

        // Tier 3: Hard Limit
        if new_total > self.hard_limit {
            return Err(Error::HardLimitExceeded);
        }

        // Tier 2: Soft Limit (Backpressure)
        if new_total > self.soft_threshold {
            tokio::time::sleep(Duration::from_millis(100)).await;
            return Err(Error::SoftLimitExceeded);
        }

        // Tier 1: Advisory Warning
        if new_total > self.advisory_threshold {
            warn!("Resource usage at {}%", new_total * 100 / self.hard_limit);
        }

        // Grant permit
        self.current_usage.fetch_add(needed, Ordering::SeqCst);
        Ok(Permit::new(self.current_usage.clone()))
    }
}
```

**Pros:**
- Graceful degradation (soft limits)
- System protection (hard limits)
- Fair resource allocation
- User experience maintained
- Backpressure prevents overload

**Cons:**
- More complex than simple limits
- Threshold tuning required
- Backpressure implementation needed

**Evaluation:** ACCEPTED - Best balance of protection and experience

---

## Decision

**Adopt tiered limit enforcement with soft/hard limits and backpressure.**

### Rationale

1. **Security Protection:**
   - Hard limits prevent exhaustion attacks
   - DoS protection for all resources
   - System stability guaranteed

2. **Fairness:**
   - Per-user quotas ensure fairness
   - Backpressure allows fair access
   - No resource monopolization

3. **User Experience:**
   - Advisory warnings provide early notice
   - Soft limits allow continued operation
   - Graceful degradation under load

4. **System Stability:**
   - Bounded resource usage
   - Prevents OOM crashes
   - Predictable behavior

5. **Adaptability:**
   - Backpressure responds to load
   - Soft limits adjust to conditions
   - Hard limits enforce absolute bounds

---

## Implementation Plan

### Phase 1: Limit Configuration

**Tasks:**
- Define resource limits per deployment mode
- Configure tier thresholds
- Set up per-user quotas

**Traceability:** resource_limits.md:34-177

### Phase 2: Soft Limit Enforcement

**Tasks:**
- Implement advisory logging
- Implement backpressure mechanism
- Configure throttling

**Traceability:** resource_limits.md:233-267

### Phase 3: Hard Limit Enforcement

**Tasks:**
- Implement hard limit checking
- Configure rejection responses
- Set up rate limiting

**Traceability:** resource_limits.md:183-201

### Phase 4: Monitoring Integration

**Tasks:**
- Track resource utilization
- Generate limit alerts
- Export metrics

**Traceability:** resource_limits.md:215-232

---

## Consequences

### Positive Consequences

1. **Security:**
   - DoS protection enforced
   - Resource exhaustion prevented
   - Attack mitigation in place

2. **System Stability:**
   - Bounded resource usage
   - No OOM crashes
   - Predictable behavior

3. **User Experience:**
   - Graceful degradation
   - Advisory warnings
   - Fair resource allocation

4. **Observability:**
   - Clear limit status
   - Resource metrics available
   - Alert on approach

### Negative Consequences

1. **Complexity:**
   - Tiered enforcement more complex
   - Multiple limit types to manage
   - Backpressure requires tuning

2. **Configuration:**
   - Thresholds require calibration
   - Per-user quotas need setup
   - Limits vary by deployment mode

---

## Monitoring and Validation

### Success Criteria

| Metric | Target | Measurement Method |
|---------|--------|-------------------|
| DoS protection | 100% | Penetration tests |
| Resource limit enforcement | 100% | Load tests |
| Backpressure effectiveness | >95% | Latency metrics |
| Fairness index | >0.9 | User feedback |
| Alert accuracy | >95% | Alert verification |

### Testing Strategy

1. **Unit Tests:**
   - Test limit enforcement
   - Verify backpressure
   - Test rejection responses

2. **Integration Tests:**
   - Test limit transitions
   - Verify per-user quotas
   - Test multi-tenant scenarios

3. **Load Tests:**
   - Test under limit
   - Test at limit
   - Test over limit
   - Measure backpressure effectiveness

**Test Example:**
```rust
#[tokio::test]
async fn test_limit_enforcement() {
    let limiter = TieredLimiter::new(100, 90, 80);

    // Test under limit
    for _ in 0..50 {
        assert!(limiter.acquire(1).await.is_ok());
    }

    // Test advisory warning
    for _ in 50..70 {
        let result = limiter.acquire(1).await;
        assert!(result.is_ok());
    }

    // Test soft limit
    for _ in 70..80 {
        let result = limiter.acquire(1).await;
        // Should trigger backpressure
    }

    // Test hard limit
    for _ in 80..110 {
        let result = limiter.acquire(1).await;
        assert!(matches!(result, Err(Error::HardLimitExceeded)));
    }
}
```

---

## Related Decisions

- [ADR-024](adr-024-memory-management-strategy.md) - Memory limits
- [ADR-026](adr-026-thread-pool-sizing.md) - Task concurrency limits
- [ADR-027](adr-027-resource-leak-detection.md) - Leak detection

---

## References

1. **Research Sources:**
   - resource_limits.md:34-50 (Memory Limits)
   - resource_limits.md:87-97 (Network Limits)
   - resource_limits.md:169-177 (Task Limits)

2. **Requirements:**
   - requirements.md:PF-RQ-001 (Performance Requirements)
   - requirements.md:AC-RQ-001 (Access Control)
   - requirements.md:SC-RQ-006 (DoS Prevention)
   - OWASP ASVS V5 (Resource Management)

3. **Architecture:**
   - threat_model.md:CM-FW-001 (File Watch DoS)
   - threat_model.md:IF-WS-003 (WebSocket Flood)
   - resource_limits.md:233-267 (Backpressure)

4. **Dependencies:**
   - dep_spec/tokio/dep_spec.toml (tokio primitives)

---

**Document Revision History:**

| Version | Date | Author | Changes |
|---------|-------|--------|---------|
| 1.0 | 2026-02-11 | Initial ADR creation |
