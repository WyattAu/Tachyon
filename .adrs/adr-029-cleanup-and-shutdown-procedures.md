# ADR-029: Cleanup and Shutdown Procedures

## Status

| Status | Accepted |
|---------|----------|
| Date | 2026-02-11 |
| Decision | Adopt graceful shutdown with phased cleanup and data persistence |
| Context | Clean shutdown and data integrity |

---

## Context and Problem Statement

### Current Situation

Tachyon system manages persistent state including Git repositories, search indices, database connections, WebSocket connections, and in-memory caches. Improper shutdown can lead to:
- Data loss (uncommitted changes)
- Corrupted indexes
- Broken connections
- Incomplete writes
- Resource leaks on restart

### Problem

Without structured shutdown procedures:
1. Abrupt termination causes data corruption
2. In-flight operations are interrupted
3. Resources not released properly
4. Temporary files not cleaned
5. State inconsistent across restarts
6. No recovery mechanism

### Constraints

| Constraint | Value | Source |
|------------|--------|---------|
| Shutdown timeout | 30s | resource_limits.md |
| Data persistence | Required | CM-RQ-005 |
| Connection graceful close | Required | IN-RQ-004 |
| Index commit | Required | SD-RQ-002 |
| Zero data loss tolerance | 0 | CM-RQ-006 |

---

## Decision Drivers

| Factor | Impact | Weight |
|---------|--------|--------|
| Data Integrity (CM-RQ-005) | CRITICAL | 40% |
| Resource Cleanup (PF-RQ-003) | CRITICAL | 30% |
| Graceful Degradation | HIGH | 15% |
| Recovery Capability | HIGH | 15% |

---

## Considered Alternatives

### Alternative 1: Immediate Termination (SIGKILL)

**Description:** Send SIGKILL to all processes immediately.

**Pros:**
- Fastest shutdown
- Simple implementation
- Guaranteed termination

**Cons:**
- Data loss (uncommitted changes)
- Corrupted state
- Resource leaks (sockets, handles)
- No graceful degradation
- Poor user experience

**Evaluation:** REJECTED - Unacceptable data loss risk

### Alternative 2: Best-Effort Cleanup

**Description:** Attempt cleanup but fail fast on errors.

**Pros:**
- Better than immediate kill
- Some data saved

**Cons:**
- Incomplete cleanup
- State inconsistency
- No rollback capability
- Unpredictable duration

**Evaluation:** REJECTED - Inadequate reliability

### Alternative 3: Sequential Cleanup (No Phasing)

**Description:** Clean up resources one by one in arbitrary order.

**Pros:**
- Simple implementation
- Predictable order

**Cons:**
- Wrong order can cause issues
- No parallel cleanup
- Slow shutdown
- Dependencies may time out
- No rollback on failure

**Evaluation:** REJECTED - Inefficient and error-prone

### Alternative 4: Phased Graceful Shutdown with Rollback (SELECTED)

**Description:** Implement multi-phase shutdown with dependency ordering, timeout enforcement, and rollback capability.

**Architecture:**

```
Phase 1: Stop Acceptance (Immediate)
  - Reject new requests
  - Return 503 (Service Unavailable)
  - Drain existing request queues
  - Duration: 0-5s

Phase 2: Graceful Drain (5-15s)
  - Complete in-flight operations
  - Flush pending writes
  - Close WebSocket connections gracefully
  - Notify connected clients

Phase 3: Resource Cleanup (15-30s)
  - Commit Git changes
  - Flush search index
  - Close database connections
  - Release file handles
  - Clear caches

Phase 4: Persistent State (30-60s)
  - Ensure data persistence
  - Verify integrity
  - Write shutdown markers
  - Flush buffers to disk

Phase 5: Forced Termination (60s+)
  - SIGKILL remaining processes
  - Last resort only
  - Log forced shutdown
```

**Implementation:**
```rust
struct ShutdownManager {
    timeout: Duration,
    phase: Arc<AtomicU8>,
    rollback_state: Arc<RwLock<RollbackState>>,
}

enum ShutdownPhase {
    Acceptance = 0,
    GracefulDrain = 1,
    ResourceCleanup = 2,
    PersistentState = 3,
    ForcedTermination = 4,
}

impl ShutdownManager {
    async fn shutdown(&self) -> Result<()> {
        // Phase 1: Stop Acceptance
        self.phase.store(ShutdownPhase::Acceptance, Ordering::SeqCst);
        self.stop_acceptance().await?;

        // Phase 2: Graceful Drain
        self.phase.store(ShutdownPhase::GracefulDrain, Ordering::SeqCst);
        self.graceful_drain().await?;

        // Phase 3: Resource Cleanup
        self.phase.store(ShutdownPhase::ResourceCleanup, Ordering::SeqCst);
        self.resource_cleanup().await?;

        // Phase 4: Persistent State
        self.phase.store(ShutdownPhase::PersistentState, Ordering::SeqCst);
        self.persistent_state().await?;

        Ok(())
    }

    async fn rollback(&self) -> Result<()> {
        let state = self.rollback_state.read().await;
        if state.needs_rollback() {
            self.rollback_git().await?;
            self.rollback_index().await?;
        }
        Ok(())
    }
}
```

**Pros:**
- Data integrity preserved
- Resources cleaned properly
- Graceful user experience
- Rollback on failure
- Time-bounded phases
- Recovery capability

**Cons:**
- More complex implementation
- Phase coordination required
- Timeout tuning needed
- Rollback state management

**Evaluation:** ACCEPTED - Best balance of reliability and user experience

---

## Decision

**Adopt phased graceful shutdown with rollback capability.**

### Rationale

1. **Data Integrity:**
   - Phased shutdown ensures completion
   - Rollback on cleanup failure
   - Persistent state verification
   - Zero data loss tolerance

2. **Resource Cleanup:**
   - Proper handle release (Drop trait)
   - Connection closure
   - Cache clearing
   - Temporary file deletion

3. **User Experience:**
   - Graceful degradation
   - Client notification
   - Predictable shutdown duration
   - Service unavailable response

4. **Recovery:**
   - Rollback state maintained
   - Markers for recovery
   - Idempotent shutdown
   - Can resume from checkpoint

5. **System Reliability:**
   - Time-bounded phases
   - Timeout prevents hangs
   - Forced termination as fallback
   - Monitoring for stuck phases

---

## Implementation Plan

### Phase 1: Stop Acceptance

**Tasks:**
- Reject new requests
- Drain request queues
- Set maintenance mode

**Traceability:** resource_limits.md:233-267

### Phase 2: Graceful Drain

**Tasks:**
- Complete in-flight operations
- Flush pending writes
- Close WebSocket connections
- Notify clients

**Traceability:** IN-RQ-004

### Phase 3: Resource Cleanup

**Tasks:**
- Commit Git changes
- Flush search index
- Close database connections
- Clear caches

**Traceability:** CM-RQ-005, CM-RQ-006, SD-RQ-002

### Phase 4: Persistent State

**Tasks:**
- Verify data integrity
- Write shutdown markers
- Flush buffers

**Traceability:** CM-RQ-005

### Phase 5: Rollback Mechanism

**Tasks:**
- Implement rollback state
- Create recovery markers
- Test rollback procedures

**Traceability:** PF-RQ-003

---

## Consequences

### Positive Consequences

1. **Data Integrity:**
   - Uncommitted changes preserved
   - Indexes properly closed
   - No corrupted state

2. **Resource Cleanup:**
   - All handles released (Drop)
   - No resource leaks
   - Temporary files deleted

3. **User Experience:**
   - Graceful degradation
   - Clear shutdown notification
   - Predictable behavior

4. **System Reliability:**
   - Time-bounded shutdown
   - Forced termination fallback
   - Monitoring for stuck phases

### Negative Consequences

1. **Complexity:**
   - Phased shutdown more complex
   - Rollback state management
   - Phase coordination required

2. **Duration:**
   - Shutdown takes 30-60 seconds
   - User waits for completion

3. **Configuration:**
   - Timeouts require tuning
   - Phase ordering critical
   - Rollback triggers defined

---

## Monitoring and Validation

### Success Criteria

| Metric | Target | Measurement Method |
|---------|--------|-------------------|
| Data loss events | 0 | Error logs |
| Resource leaks | 0 | Memory monitoring |
| Shutdown success rate | >99% | Shutdown logs |
| Rollback success rate | >95% | Recovery tests |
| Client notification | 100% | WebSocket logs |

### Testing Strategy

1. **Unit Tests:**
   - Test each phase independently
   - Verify rollback mechanisms
   - Test timeout enforcement

2. **Integration Tests:**
   - Test full shutdown sequence
   - Test with in-flight operations
   - Test rollback scenarios

3. **Chaos Tests:**
   - Test shutdown under load
   - Test with failing components
   - Test forced termination

**Test Example:**
```rust
#[tokio::test]
async fn test_graceful_shutdown() {
    let manager = ShutdownManager::new(Duration::from_secs(30));
    let (server_tx, mut server_rx) = mpsc::channel(100);

    // Simulate in-flight operations
    server_tx.send(Task::new(1)).await.unwrap();
    server_tx.send(Task::new(2)).await.unwrap();

    // Start shutdown
    let handle = tokio::spawn(async move {
        manager.shutdown().await.unwrap();
    });

    // Verify operations completed
    let result = tokio::time::timeout(Duration::from_secs(35), handle).await;
    assert!(result.is_ok());

    // Verify no data loss
    let task1 = server_rx.recv().await.unwrap();
    let task2 = server_rx.recv().await.unwrap();
    assert!(task1.completed);
    assert!(task2.completed);
}
```

---

## Related Decisions

- [ADR-024](adr-024-memory-management-strategy.md) - RAII cleanup
- [ADR-025](adr-025-handle-lifecycle-management.md) - Handle release
- [ADR-027](adr-027-resource-leak-detection.md) - Leak prevention

---

## References

1. **Research Sources:**
   - resource_limits.md:34-66 (Shutdown Timeout)
   - memory_management.md:75-85 (Drop Trait)

2. **Requirements:**
   - requirements.md:CM-RQ-005 (Content Versioning)
   - requirements.md:CM-RQ-006 (Auto-Save)
   - requirements.md:IN-RQ-004 (WebSocket API)
   - PF-RQ-003 (Resource Safety)

3. **Architecture:**
   - blue_paper.md:88-102 (CM-002 Git Operations)
   - blue_paper.md:202-208 (IF-003 WebSocket Server)

4. **Dependencies:**
   - dep_spec/tokio/dep_spec.toml (tokio shutdown)
   - dep_spec/git2-rs/dep_spec.toml (Git cleanup)

---

**Document Revision History:**

| Version | Date | Author | Changes |
|---------|-------|--------|---------|
| 1.0 | 2026-02-11 | Initial ADR creation |
