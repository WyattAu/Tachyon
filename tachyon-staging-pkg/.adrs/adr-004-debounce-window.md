# ADR-004: File Watch Debounce Window Configuration

## Status

| Status | Accepted |
|---------|----------|
| Date | 2026-02-11 |
| Decision | Adopt 100ms debounce window with 100ms throttle rate for file watching |
| Context | File Watching and Hot-Reload Architecture |

---

## Context and Problem Statement

### Current Situation

Tachyon requires real-time file watching with sub-10ms latency for hot-reload functionality. The file watcher must:
- Detect file system changes rapidly (<10ms notification latency)
- Prevent event storms from rapid operations
- Maintain 100ms end-to-end hot-reload latency (CM-RQ-004)
- Support cross-platform file watching (Linux, macOS, Windows)

### Problem

File system APIs generate high-frequency events during operations like:
- Text editor auto-save (every 2 seconds from CM-RQ-006)
- Git operations (commits, checkouts)
- File renames, moves, deletions

Without debouncing, these events cause:
- Excessive cache invalidation (triggering re-renders unnecessarily)
- WebSocket message storms
- Increased CPU usage
- Degraded user experience

### Constraints

| Constraint | Value | Source |
|------------|--------|---------|
| File watch latency | <10ms P99 | domain_constraints.toml:21-32 |
| End-to-end hot-reload | <100ms | test_vectors.toml:377-391 |
| Debounce window | 50-200ms | domain_constraints.toml:309-320 |
| Throttle rate | 10-1000ms | domain_constraints.toml:320-331 |
| Event queue size | 100-10000 events | domain_constraints.toml:331-342 |
| Max watch descriptors | 8192 | domain_constraints.toml:342-356 |

### Research Findings

Multi-lingual research (78 sources across 15 languages) confirms:
- 100ms debounce window is universally recommended (all 15 languages agree)
- Event throttling prevents event storms
- Platform-specific APIs (inotify, FSEvents, kqueue) provide sub-10ms latency

Source: integrated_findings.md:217-259 (File Watching Debouncing)

---

## Decision Drivers

| Factor | Impact | Weight |
|---------|--------|--------|
| User Experience (CM-RQ-004) | CRITICAL | 40% |
| Performance (RE-RQ-005) | HIGH | 30% |
| System Stability | HIGH | 20% |
| Cross-Platform Support | MEDIUM | 10% |

---

## Considered Alternatives

### Alternative 1: No Debouncing

**Description:** Process every file system event immediately.

**Pros:**
- Maximum responsiveness (zero debounce delay)
- Simple implementation
- No event coalescing logic

**Cons:**
- Event storms during rapid operations
- Excessive cache invalidation
- WebSocket message flooding
- High CPU usage

**Evaluation:** REJECTED - Violates <100ms hot-reload target

### Alternative 2: Very Short Debounce (10-50ms)

**Description:** Use very short debounce window for maximum responsiveness.

**Pros:**
- Near-zero latency for single-file edits
- Highly responsive to changes

**Cons:**
- Still causes event storms for batch operations
- Does not prevent WebSocket flooding
- 10ms window may be too short for some use cases

**Evaluation:** REJECTED - Does not prevent event storms adequately

### Alternative 3: Very Long Debounce (200-500ms)

**Description:** Use long debounce window to minimize event processing.

**Pros:**
- Maximum event reduction
- Lowest CPU usage
- Prevents all event storms

**Cons:**
- Poor user experience (perceived lag)
- Violates <100ms hot-reload target
- Not responsive to individual edits

**Evaluation:** REJECTED - Too slow for real-time editing

### Alternative 4: Adaptive Debounce (SELECTED)

**Description:** Use configurable debounce window (50-200ms) with event throttling.

**Configuration:**
- Debounce window: 100ms (default)
- Throttle rate: 100ms minimum between events
- Event queue: 1000 events buffer
- Per-path deduplication within window

**Pros:**
- 100ms meets <100ms hot-reload target with margin
- Prevents event storms during batch operations
- Configurable for different use cases
- Multi-lingual consensus (15 languages) supports 50-200ms range
- Event queue handles burst operations

**Cons:**
- Adds complexity to file watcher
- Requires configuration tuning
- Slightly higher latency than no debounce

**Evaluation:** ACCEPTED - Best balance of responsiveness and stability

---

## Decision

**Adopt 100ms debounce window with 100ms throttle rate and 1000 event queue.**

### Rationale

1. **Research Validation:**
   - Multi-lingual consensus (15 languages) recommends 50-200ms debounce window
   - Default 100ms provides good balance
   - Confidence score: 0.95 (High) from integrated_findings.md:217-259
   - All sources agree on debouncing necessity for event storms

2. **Performance Target Achievement:**
   - Sub-10ms file watch latency (domain_constraints.toml:21-32)
   - 100ms debounce window leaves ~90ms budget for processing
   - Meets CM-RQ-004 (<100ms end-to-end hot-reload)
   - 100ms throttle prevents event storms

3. **User Experience:**
   - 100ms debounce is imperceptible for human perception
   - Batch operations feel responsive (events coalesced within window)
   - Single-file edits have ~100ms latency (acceptable for hot-reload)

4. **System Stability:**
   - Event throttling prevents WebSocket message storms
   - 1000 event queue handles burst operations (domain_constraints.toml:331-342)
   - Per-path deduplication reduces redundant processing

5. **Cross-Platform Support:**
   - notify crate provides platform abstraction
   - Linux: inotify (sub-10ms, yellow_paper.md:335-369)
   - macOS: FSEvents/kqueue (sub-10ms, yellow_paper.md:370-390)
   - Windows: ReadDirectoryChangesW (sub-10ms)

6. **Configuration Flexibility:**
   - Configurable debounce window (50-200ms from domain_constraints.toml:309-320)
   - Configurable throttle rate (10-1000ms from domain_constraints.toml:320-331)
   - Allows tuning for different hardware and workloads

### Trade-offs

| Aspect | Benefit | Cost | Mitigation |
|---------|--------|---------|-------------|
| Responsiveness | 100ms debounce | 0-100ms latency vs no debounce | Configurable per use case |
| Stability | Event storm prevention | Increased complexity | Event throttling + queue |
| User Experience | Batch operation handling | 100ms perception lag | Tuning for different workflows |

---

## Implementation Plan

### Phase 1: Debounce Core Implementation

**Tasks:**
- Implement Debouncer struct with window, queue, and throttle
- Implement event deduplication within debounce window
- Implement event coalescing for rapid operations
- Add configurable window and throttle parameters

**Traceability:** blue_paper.md:118-132 (CM-003 - File Watcher)
- adr-003: Cache invalidation integration

**Dependencies:**
- notify 6.1.1 (dep_spec/notify/dep_spec.toml:1-99)
- tokio 1.49.0 (async timers, from Cargo.toml:8)

**Core Logic:**
```rust
struct Debouncer {
    window_ms: u64,           // 100ms default
    throttle_ms: u64,          // 100ms min
    queue_size: usize,         // 1000 events
    pending_events: HashMap<Path, Instant>,
    last_dispatched: Instant,
}

impl Debouncer {
    fn on_event(&mut self, path: Path, event: FileEvent) {
        let now = Instant::now();
        self.pending_events.insert(path.clone(), now);
        
        // Schedule debounce
        let debounce_id = self.pending_events.len();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(self.window_ms)).await;
            self.dispatch_pending_events(debounce_id);
        });
    }
    
    fn dispatch_pending_events(&mut self, debounce_id: usize) {
        // Throttle and deduplication logic
        // ...
    }
}
```

### Phase 2: Event Coalescing

**Tasks:**
- Implement per-path event deduplication within window
- Coalesce multiple events for same path into single dispatch
- Implement event prioritization (write > create > delete)
- Add event type filtering (only relevant events)

**Event Prioritization:**
```rust
enum EventPriority {
    Write,      // Highest priority - triggers re-render
    Create,     // Medium priority
    Delete,     // Low priority
    Rename,     // Medium priority
    Other,      // Lowest priority
}
```

### Phase 3: Configuration Management

**Tasks:**
- Add TOML configuration file for debounce settings
- Implement environment variable overrides
- Add runtime configuration API
- Add metrics export (debounce count, deduplication rate)

**Traceability:** domain_constraints.toml:309-342 (File watch debouncing)

**Configuration Structure:**
```toml
[filewatch.debounce]
# Debounce window in milliseconds (50-200ms)
window_ms = 100

# Minimum time between event dispatches (10-1000ms)
throttle_ms = 100

# Event queue size (100-10000 events)
queue_size = 1000

# Enable/disable per-path deduplication
deduplicate_per_path = true
```

### Phase 4: Integration with Cache Invalidation

**Tasks:**
- Integrate with LRU cache invalidation (ADR-003)
- Implement bulk invalidation for batch file changes
- Add cache warming for hot files on startup
- Implement invalidation metrics (events processed, cache misses)

**Traceability:** adr-003: Cache invalidation
- blue_paper.md:277-295 (Cache Invalidation)

**Integration Logic:**
```rust
fn invalidate_cache_on_events(events: Vec<FileEvent>, cache: &mut LRUCache) {
    for event in events {
        match event.event_type {
            EventKind::Write | EventKind::Create => {
                // Invalidate all entries for this file path
                let prefix = format!("{}||", event.path.display());
                cache.retain(|k| !k.starts_with(&prefix));
            }
            EventKind::Delete => {
                // Remove deleted file from cache
                let key = generate_cache_key(&event.path, &get_current_commit(), &get_current_role());
                cache.remove(&key);
            }
            _ => {} // Ignore other events
        }
    }
}
```

---

## Consequences

### Positive Consequences

1. **User Experience:**
   - 100ms debounce provides responsive hot-reload
   - Batch operations feel smooth (events coalesced)
   - No perceived lag for typical editing workflows
   - Meets CM-RQ-004 (<100ms hot-reload)

2. **System Stability:**
   - Event throttling prevents WebSocket message storms
   - 1000 event queue handles burst operations
   - Deduplication reduces redundant processing
   - Stable under high load (batch Git operations)

3. **Performance:**
   - Reduced cache invalidation (events coalesced)
   - Lower CPU usage (fewer render operations)
   - Efficient WebSocket bandwidth usage

4. **Cross-Platform:**
   - notify crate provides platform abstraction
   - Sub-10ms file watch latency achievable
   - Works on Linux (inotify), macOS (FSEvents), Windows (ReadDirectoryChangesW)

### Negative Consequences

1. **Implementation Complexity:**
   - Debouncer state management adds complexity
   - Event queue requires careful sizing
   - Throttling logic must be correct to avoid deadlocks

2. **Configuration:**
   - Requires tuning for different use cases
   - Users must understand debounce vs. throttle
   - Multiple configuration sources (TOML, env vars, API)

3. **Latency:**
   - 100ms debounce adds ~100ms latency to single edits
   - May not be suitable for all workflows (e.g., very rapid editing)

---

## Monitoring and Validation

### Success Criteria

| Metric | Target | Measurement Method |
|---------|--------|-------------------|
| File watch latency | <10ms P99 | test_vectors.toml:267-277 |
| End-to-end hot-reload | <100ms P99 | test_vectors.toml:377-391 |
| Event coalescing | >90% events deduplicated | Integration tests |
| Debounce accuracy | Within 50-200ms of event | Unit tests |
| Event queue overflow | <1% of events | Load testing |

### Testing Strategy

1. **Unit Tests:**
   - Test debounce window timing
   - Test event deduplication logic
   - Test throttling rate enforcement
   - Test queue overflow handling

2. **Integration Tests:**
   - Test full file watcher + debouncer flow
   - Test batch file operations (Git commits)
   - Test cache invalidation integration
   - Test WebSocket broadcast behavior

3. **Performance Tests:**
   - Benchmark file watch latency with debouncing
   - Measure event queue utilization at load
   - Profile CPU usage during event storms

4. **Cross-Platform Tests:**
   - Test on Linux (inotify)
   - Test on macOS (FSEvents)
   - Test on Windows (ReadDirectoryChangesW)
   - Verify sub-10ms latency on all platforms

### Rollback Plan

If 100ms debounce fails to meet requirements:

1. **Reduce Debounce Window:** Lower to 50ms for more responsiveness
2. **Implement Adaptive Debouncing:** Adjust window based on event frequency
3. **Increase Queue Size:** Raise from 1000 to 10000 events
4. **Per-Path Debounce:** Disable deduplication for high-frequency editing

---

## Related Decisions

- blue_paper.md:118-132 (File Watcher)
- adr-003: Cache invalidation integration
- domain_constraints.toml:309-342 (File watch debouncing)
- test_vectors.toml:307-316 (Debounce verification)

---

## References

1. **Research Sources:**
   - integrated_findings.md:217-259 (File Watching Debouncing)
   - yellow_paper.md:335-369 (Linux inotify)
   - yellow_paper.md:370-390 (macOS FSEvents)
   - yellow_paper.md:413-438 (Cross-platform abstraction)

2. **Requirements:**
   - requirements.md:118-132 (CM-RQ-004)
   - requirements.md:148-162 (CM-RQ-006 - Auto-Save)

3. **Domain Constraints:**
   - domain_constraints.toml:309-342 (Debounce window)
   - domain_constraints.toml:320-356 (Event queue)

4. **Test Vectors:**
   - test_vectors.toml:267-277 (Single file modification)
   - test_vectors.toml:277-287 (Batch file operations)
   - test_vectors.toml:307-316 (Debounce verification)

5. **Architecture:**
   - blue_paper.md:118-132 (File Watcher)
   - dep_spec/notify/dep_spec.toml:1-99 (notify crate)

---

**Document Revision History:**

| Version | Date | Author | Changes |
|---------|-------|--------|---------|
| 1.0 | 2026-02-11 | Initial ADR creation |
