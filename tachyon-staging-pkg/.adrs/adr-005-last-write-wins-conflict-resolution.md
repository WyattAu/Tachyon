# ADR-005: Last-Write-Wins Conflict Resolution Strategy

## Status

| Status | Accepted |
|---------|----------|
| Date | 2026-02-11 |
| Decision | Adopt Last-Write-Wins (LWW) conflict resolution with user notification |
| Context | Concurrent Editing and Multi-User Collaboration |

---

## Context and Problem Statement

### Current Situation

Tachyon supports concurrent editing with real-time synchronization via WebSocket. Multiple users may edit the same document simultaneously, causing conflicts when changes are merged. The system must:
- Handle concurrent edits deterministically
- Resolve conflicts without data loss
- Notify users of conflict resolution
- Support rollback to previous versions
- Maintain Git history integrity

### Problem

Git's default merge strategy creates merge commits for conflicts. For Tachyon's use case (real-time editing, multiple users), this approach:
- Requires manual conflict resolution (user intervention needed)
- Creates merge commits that complicate history
- May cause data loss if not resolved correctly
- Breaks real-time editing workflow
- Does not scale to multiple concurrent users

### Constraints

| Constraint | Value | Source |
|------------|--------|---------|
| Auto-save interval | 2000ms | requirements.md:148-162 |
| Multi-user support | 100 concurrent | domain_constraints.toml:74-84 |
| Conflict notification | Required | requirements.md:163-177 |
| Git integration | Direct libgit2 | requirements.md:103-117 |
| WebSocket latency | <5ms P95 | domain_constraints.toml:48-59 |

### Research Findings

Multi-lingual research (78 sources across 15 languages) confirms:
- LWW is the recommended deterministic conflict resolution for documentation systems
- Timestamp-based ordering ensures reproducibility
- User notification required for transparency
- German (DE), Japanese (JP), Portuguese (PT) sources confirm LWW for documentation

Source: integrated_findings.md:101-127 (LWW Conflict Resolution)

---

## Decision Drivers

| Factor | Impact | Weight |
|---------|--------|--------|
| Determinism | CRITICAL | 40% |
| User Experience | HIGH | 30% |
| Simplicity | HIGH | 20% |
| Git Compatibility | MEDIUM | 10% |

---

## Considered Alternatives

### Alternative 1: Three-Way Merge (Git Default)

**Description:** Use Git's default merge conflict resolution with manual intervention.

**Pros:**
- Preserves all changes from all contributors
- Git-native approach
- Full history preservation

**Cons:**
- Requires manual conflict resolution
- Creates merge commits that complicate history
- Breaks real-time editing workflow
- Not suitable for concurrent editing at scale

**Evaluation:** REJECTED - Too complex for Tachyon's real-time use case

### Alternative 2: First-Write-Wins (LWW) without Notification

**Description:** Automatically resolve conflicts by accepting the most recent change.

**Pros:**
- Automatic resolution (no user intervention)
- Simple implementation
- Preserves latest state
- No merge commits

**Cons:**
- Silent data loss (users unaware of conflicts)
- Poor user experience
- Violates CM-RQ-007 (notification requirement)

**Evaluation:** REJECTED - Violates notification requirement

### Alternative 3: Operational Transformation (OT)

**Description:** Use Operational Transformation for automatic conflict-free merging.

**Pros:**
- True concurrent editing without conflicts
- Automatic merge
- Full preservation of all changes

**Cons:**
- Complex implementation
- Requires OT protocol implementation
- Significant research and development effort
- Not compatible with standard Git

**Evaluation:** REJECTED - Too complex for Phase 2 scope

### Alternative 4: Three-Way Merge with Auto-Accept

**Description:** Use three-way merge but automatically accept current version.

**Pros:**
- Preserves all changes from all contributors
- Automatic resolution
- Git-compatible approach

**Cons:**
- Still creates merge commits
- Users may not be aware of conflict
- Adds complexity to merge logic

**Evaluation:** REJECTED - Does not provide adequate user notification

### Alternative 5: Last-Write-Wins with User Notification (SELECTED)

**Description:** Automatically resolve conflicts by accepting the most recent change and notify all affected users.

**Configuration:**
- Resolution strategy: Timestamp-based LWW
- Conflict notification: WebSocket broadcast to all connected users
- Rollback support: Users can view and restore previous versions
- Conflict logging: Audit trail for all resolved conflicts

**Pros:**
- Automatic resolution (no user intervention)
- Deterministic and reproducible
- Transparent to users (notification + logging)
- Simple implementation
- Compatible with Git history (no merge commits)
- Meets CM-RQ-007 (notification requirement)

**Cons:**
- Silent data loss (losing changes from older edits)
- Users must manually rollback if they disagree
- Requires conflict notification infrastructure
- Adds WebSocket complexity

**Evaluation:** ACCEPTED - Best balance for Tachyon's requirements

---

## Decision

**Adopt Last-Write-Wins (LWW) conflict resolution with user notification.**

### Rationale

1. **Research Validation:**
   - Multi-lingual consensus confirms LWW for documentation systems
   - German (DE), Japanese (JP), Portuguese (PT) sources all agree
   - Confidence score: 0.95 (High) from integrated_findings.md:101-127
   - Original Git paper (2005) establishes LWW as default

2. **Requirement Compliance:**
   - CM-RQ-007 requires conflict notification
   - LWW with notification satisfies this requirement
   - Deterministic resolution supports multi-user editing
   - Real-time editing workflow preserved

3. **Git Compatibility:**
   - LWW is Git's native conflict resolution model
   - Direct libgit2 integration (ADR-006) supports timestamp-based ordering
   - No merge commits required (cleaner history)
   - Compatible with Git's object model (Commit, Tree, Blob)

4. **User Experience:**
   - Automatic resolution reduces friction
   - Conflict notification provides transparency
   - Rollback support allows recovery from mistaken LWW
   - Meets ISO-25010 (Appropriateness) requirements

5. **Implementation Simplicity:**
   - Straightforward algorithm (sort by timestamp)
   - No complex merge logic required
   - Well-understood pattern with extensive research

6. **Scalability:**
   - LWW scales to 100 concurrent users (domain_constraints.toml:74-84)
   - O(n log n) complexity for conflict resolution
   - Minimal memory overhead

### Trade-offs

| Aspect | Benefit | Cost | Mitigation |
|---------|--------|---------|-------------|
| Automatic resolution | No user intervention | Silent data loss | Conflict notification + rollback |
| Determinism | Reproducible behavior | May lose changes | Timestamp ordering + logging |
| Simplicity | Easy implementation | Limited conflict resolution | User education on LWW |
| Git compatibility | No merge commits | Requires notification | WebSocket broadcast |

---

## Implementation Plan

### Phase 1: Conflict Detection

**Tasks:**
- Implement concurrent edit tracking per document
- Implement conflict detection (multiple edits to same file)
- Implement timestamp ordering for all edits
- Add conflict metadata (editors, timestamps)

**Traceability:** blue_paper.md:163-177 (CM-RQ-007)
- proof.lean:95-115 (LWW Determinism)

**Conflict Detection Logic:**
```rust
struct DocumentState {
    edits: Vec<Edit>,
    last_commit: String,
    last_timestamp: u64,
}

struct Edit {
    user_id: String,
    timestamp: u64,
    content: String,
}

fn detect_conflict(document_state: &DocumentState) -> Option<Conflict> {
    if document_state.edits.len() > 1 {
        // Check if multiple edits exist
        let timestamps: Vec<u64> = document_state.edits.iter()
            .map(|e| e.timestamp).collect();
        
        // Check for concurrent edits within time window
        let recent_edits: Vec<&Edit> = timestamps.iter()
            .filter(|&t| *t > current_time - 5000) // 5 second window
            .collect();
        
        if recent_edits.len() > 1 {
            Some(Conflict {
                document_path: document_state.path.clone(),
                conflicting_edits: recent_edits,
                resolved_edit: None, // To be determined by LWW
            })
        } else {
            None
        }
    } else {
        None
    }
}
```

### Phase 2: LWW Resolution

**Tasks:**
- Implement timestamp-based resolution (select most recent)
- Implement conflict notification via WebSocket
- Implement rollback support for previous versions
- Add conflict audit logging
- Implement conflict recovery API

**Traceability:** blue_paper.md:277-295 (LWW Conflict Resolution)

**Resolution Logic:**
```rust
fn resolve_lww_conflict(conflict: Conflict) -> Resolution {
    // Sort edits by timestamp (descending)
    let sorted_edits: Vec<&Edit> = conflict.conflicting_edits
        .iter()
        .sorted_by(|a, b| b.timestamp.cmp(&a.timestamp))
        .collect();
    
    // Select most recent edit (LWW)
    let winning_edit = sorted_edits.first().unwrap();
    
    Resolution {
        document_path: conflict.document_path,
        winning_edit: winning_edit.clone(),
        losing_edits: sorted_edits[1..].to_vec(),
        resolved_at: current_timestamp(),
    }
}
```

### Phase 3: Conflict Notification

**Tasks:**
- Implement WebSocket broadcast of conflict events
- Implement conflict event types (conflict_detected, conflict_resolved)
- Add user acknowledgment mechanism
- Implement conflict history API
- Add UI notifications for desktop clients

**WebSocket Event Types:**
```rust
enum ConflictEventType {
    ConflictDetected {
        document_path: String,
        conflicting_users: Vec<String>,
        timestamp: u64,
    },
    ConflictResolved {
        document_path: String,
        winning_user: String,
        losing_users: Vec<String>,
        timestamp: u64,
        rolled_back_available: bool,
    },
}
```

### Phase 4: Rollback Support

**Tasks:**
- Implement version history tracking per document
- Implement rollback API to restore previous versions
- Implement conflict recovery (apply losing changes)
- Add UI for viewing and comparing versions
- Integrate with Git for version restoration

**Traceability:** blue_paper.md:133-147 (CM-RQ-005 - Content Versioning)

**Rollback Logic:**
```rust
fn rollback_to_version(document_path: &str, target_commit: &str) -> Result<()> {
    // Use git2-rs to restore document to specific commit
    let repo = Repository::open(document_path)?;
    let obj = repo.find_commit(target_commit)?;
    repo.checkout(&obj)?;
    
    // Update Tachyon's internal state
    update_document_state(document_path, target_commit);
    
    Ok(())
}
```

### Phase 5: Git Integration

**Tasks:**
- Integrate with git2-rs (ADR-006) for commit operations
- Implement commit history retrieval
- Implement timestamp-based commit ordering
- Add conflict metadata to Git commits (conflict markers)
- Implement atomic commit operations

**Traceability:** adr-006: Direct libgit2 integration
- blue_paper.md:103-117 (CM-RQ-003 - Git Integration)

---

## Consequences

### Positive Consequences

1. **Determinism:**
   - Conflict resolution is reproducible (same inputs always produce same output)
   - Timestamp ordering ensures predictable behavior
   - Meets ISO-25010 (Appropriateness) requirements

2. **User Experience:**
   - Automatic resolution reduces friction
   - Conflict notification provides transparency
   - Rollback support allows recovery from errors
   - Meets CM-RQ-007 (notification requirement)

3. **System Stability:**
   - No merge commits (cleaner Git history)
   - LWW is Git's native model (max compatibility)
   - Simple algorithm reduces bugs

4. **Scalability:**
   - Scales to 100 concurrent users (domain_constraints.toml:74-84)
   - O(n log n) complexity for conflict resolution
   - Minimal memory overhead

### Negative Consequences

1. **Silent Data Loss:**
   - Older edits are silently discarded
   - Users may not be aware of data loss
   - Requires user vigilance to notice conflicts

2. **User Dependency on Rollback:**
   - Users must proactively check for conflicts
   - Rollback requires manual user action
   - Not automatic for all conflict types

3. **WebSocket Complexity:**
   - Conflict notifications add to WebSocket traffic
   - Requires reliable broadcast mechanism
   - Conflict history storage overhead

---

## Monitoring and Validation

### Success Criteria

| Metric | Target | Measurement Method |
|---------|--------|-------------------|
| Conflict detection time | <100ms | Integration tests |
| LWW resolution time | <50ms | Performance tests |
| Notification delivery | <10ms P95 to all users | WebSocket monitoring |
| Rollback success rate | >95% | Integration tests |
| Conflict audit completeness | 100% | Audit log verification |
| Concurrent user support | 100 users | Load testing |

### Testing Strategy

1. **Unit Tests:**
   - Test conflict detection logic
   - Test LWW resolution with known conflicts
   - Test timestamp ordering
   - Test notification event generation

2. **Integration Tests:**
   - Test full conflict flow (detect -> resolve -> notify)
   - Test rollback functionality
   - Test WebSocket broadcast to multiple clients
   - Test Git integration (commit history)

3. **Performance Tests:**
   - Benchmark conflict detection overhead
   - Measure LWW resolution time
   - Profile WebSocket broadcast latency
   - Test with 100 concurrent users

4. **Multi-User Tests:**
   - Test simultaneous edits from multiple users
   - Test conflict isolation between users
   - Test rollback after conflict resolution
   - Test notification delivery to all clients

### Rollback Plan

If LWW with notification fails to meet requirements:

1. **Implement Manual Conflict Resolution:** Require user to choose between versions
2. **Add Conflict Warning UI:** Prominent notification before applying LWW
3. **Implement Version Comparison:** Show diff between conflicting versions
4. **Add Conflict Escalation:** Require explicit user confirmation for data loss

---

## Related Decisions

- blue_paper.md:163-177 (CM-RQ-007 - Conflict Resolution)
- blue_paper.md:277-295 (LWW Conflict Resolution)
- adr-006: Direct libgit2 integration (Git operations)
- proof.lean:95-115 (LWW Determinism)

---

## References

1. **Research Sources:**
   - integrated_findings.md:101-127 (LWW Conflict Resolution)
   - yellow_paper.md:173-194 (Git Operations - LWW)

2. **Requirements:**
   - requirements.md:163-177 (CM-RQ-007)
   - requirements.md:133-147 (CM-RQ-005 - Content Versioning)

3. **Domain Constraints:**
   - domain_constraints.toml:74-84 (Concurrent users)

4. **Test Vectors:**
   - test_vectors.toml:321-331 (Git auto-save debounce)

5. **Architecture:**
   - blue_paper.md:103-117 (Git Integration)
   - blue_paper.md:163-177 (Conflict Resolution)

---

**Document Revision History:**

| Version | Date | Author | Changes |
|---------|-------|--------|---------|
| 1.0 | 2026-02-11 | Initial ADR creation |
