# ADR-003: LRU Cache Configuration and Capacity Target

## Status

| Status | Accepted |
|---------|----------|
| Date | 2026-02-11 |
| Decision | Adopt LRU cache with role-based keys and configurable capacity (100-1000 entries, 10-500MB) |
| Context | Rendering Engine Cache Architecture |

---

## Context and Problem Statement

### Current Situation

Tachyon requires high-performance caching for JIT rendering to meet sub-15ms targets. The cache must:
- Support role-based redaction (different HTML for admin vs. guest users)
- Invalidate on file modifications
- Achieve >80% hit rate for typical workloads
- Maintain configurable capacity for different repository sizes

### Problem

Selecting optimal LRU cache configuration is critical for:
- Performance: Sub-1ms cache hit response (RE-RQ-005)
- Memory efficiency: 10-500MB configurable capacity (domain_constraints.toml:113-124)
- Hit rate: >80% for typical workloads (domain_constraints.toml:196-208)
- Multi-user support: Role-based cache keys (CM-RQ-010)

### Constraints

| Constraint | Value | Source |
|------------|--------|---------|
| Cache hit response | <1ms P99 | domain_constraints.toml:59-70 |
| Cache capacity | 10-500MB | domain_constraints.toml:113-124 |
| Cache entry count | 100-10000 entries | domain_constraints.toml:185-196 |
| Hit rate target | >80% | domain_constraints.toml:196-208 |
| Eviction time | <5ms | domain_constraints.toml:208-223 |

### Research Findings

Multi-lingual research (78 sources across 15 languages) confirms:
- LRU is universally accepted cache eviction policy for web applications
- Hit rate target >80% is achievable with proper capacity configuration
- Multi-level caching (L1 RAM, L2 optional disk) recommended

Source: integrated_findings.md:172-196 (LRU Caching Mechanisms)

---

## Decision Drivers

| Factor | Impact | Weight |
|---------|--------|--------|
| Performance (RE-RQ-005) | CRITICAL | 40% |
| Memory Efficiency (PF-RQ-004) | HIGH | 30% |
| Multi-User Support (CM-RQ-010) | HIGH | 20% |
| Scalability | MEDIUM | 10% |

---

## Considered Alternatives

### Alternative 1: Fixed-Size Cache (No Eviction)

**Description:** Pre-allocate fixed cache size, disable eviction when full.

**Pros:**
- Simple implementation
- No eviction overhead
- Predictable memory usage

**Cons:**
- Wasted memory for unused capacity
- Cannot adapt to changing workloads
- Poor hit rate for large repositories

**Evaluation:** REJECTED - Does not meet >80% hit rate target

### Alternative 2: LFU (Least-Frequently-Used)

**Description:** Evict entries with lowest access frequency.

**Pros:**
- Better for read-heavy workloads
- Higher hit rate for stable access patterns

**Cons:**
- Poor performance for temporal locality (recently accessed files)
- More complex implementation than LRU
- Less research validation

**Evaluation:** REJECTED - Unsuitable for document editing workloads

### Alternative 3: FIFO (First-In-First-Out)

**Description:** Evict oldest entries first.

**Pros:**
- Simple implementation
- Fair eviction policy

**Cons:**
- Does not reflect access patterns in editing workflows
- Poor hit rate for hot documents
- No research support for document systems

**Evaluation:** REJECTED - Inappropriate for Tachyon use case

### Alternative 4: ARC (Adaptive Replacement Cache)

**Description:** Dynamically adjust eviction based on access patterns.

**Pros:**
- Adapts to workload patterns
- Potentially higher hit rates than LRU

**Cons:**
- Complex implementation
- Additional computational overhead
- Unpredictable memory usage
- Not well-tested in Tachyon context

**Evaluation:** REJECTED - Too complex for initial implementation

### Alternative 5: LRU Cache with Role-Based Keys (SELECTED)

**Description:** Implement LRU cache with composite keys (file_path || commit_hash || user_role).

**Configuration:**
- Capacity: 100-1000 entries (default: 1000)
- Memory: 10-500MB configurable
- Key: SHA256(file_path || commit_hash || user_role)
- Eviction: LRU (Least-Recently-Used)
- Hit rate target: >80%

**Pros:**
- LRU is universally recommended (multi-lingual consensus)
- Role-based keys enable multi-user support
- Configurable capacity for different repository sizes
- Sub-1ms hit response achievable
- >80% hit rate with proper tuning

**Cons:**
- Cache invalidation complexity (3-key composite)
- Memory overhead for hash map + linked list
- Requires careful capacity planning

**Evaluation:** ACCEPTED - Best balance of performance, complexity, and multi-user support

---

## Decision

**Adopt LRU cache with role-based keys and configurable capacity (100-1000 entries, 10-500MB).**

### Rationale

1. **Research Validation:**
   - Multi-lingual consensus (15 languages) confirms LRU is optimal for web applications
   - Confidence score: 0.99 (Very High) from integrated_findings.md:172-196
   - 2015 ACM Computing Surveys paper validates LRU effectiveness

2. **Role-Based Cache Keys:**
   - Enables multi-user support (guest, user, admin see different HTML)
   - CM-RQ-010 (Content Redaction) requires per-role caching
   - Composite key ensures cache isolation between user roles
   - SHA256(file || commit || role) provides strong collision resistance

3. **Performance Targets:**
   - Sub-1ms hit response (domain_constraints.toml:59-70)
   - >80% hit rate (domain_constraints.toml:196-208)
   - Configurable capacity allows optimization per repository
   - 1000 entries default provides good hit rate for 1000-10000 document repositories

4. **Memory Efficiency:**
   - 10-500MB configurable (domain_constraints.toml:113-124)
   - 1000 entries at ~100KB HTML = 100MB typical usage
   - Scalable: 100-10000 entries maximum for enterprise deployments

5. **Implementation Simplicity:**
   - dashmap 5.5.3 provides concurrent HashMap
   - Rust ecosystem has mature LRU implementations
   - Well-understood algorithm with extensive research

6. **Cache Invalidation:**
   - File watcher integration (CM-RQ-004) triggers invalidation
   - Commit hash in key ensures version-aware caching
   - User role in key ensures redaction isolation

### Trade-offs

| Aspect | Benefit | Cost | Mitigation |
|---------|--------|---------|-------------|
| Multi-user support | Role-based caching | Cache invalidation complexity | Composite key with commit hash |
| Performance | >80% hit rate | Memory usage | Configurable capacity limits |
| Scalability | Up to 10000 entries | Cache warm-up time | Gradual population during indexing |

---

## Implementation Plan

### Phase 1: Core LRU Implementation

**Tasks:**
- Implement LRUCache struct with K, V generics
- Implement get() operation with move-to-front (O(1))
- Implement put() operation with capacity check and eviction (O(1))
- Implement remove() operation with key deletion
- Implement clear() operation for cache reset

**Traceability:** blue_paper.md:259-276 (RE-RQ-005)
- proof.lean:17-85 (LRU Cache Invariants)

**Dependencies:**
- dashmap 5.5.3 (from Cargo.toml:25)

**Data Structure:**
```rust
use dashmap::DashMap;

struct LRUCache<K, V> {
    capacity: usize,
    map: DashMap<K, (V, Node<K>)>,
    list: LinkedList<Node<K>>,
}

struct Node<K> {
    key: K,
    value: V,
    prev: *mut Node<K>,
    next: *mut Node<K>>,
}
```

### Phase 2: Cache Key Generation

**Tasks:**
- Implement SHA256 hash function for composite keys
- Implement key format: file_path || commit_hash || user_role
- Add key validation (file_path non-empty, commit_hash valid SHA-1)

**Traceability:** blue_paper.md:179-190 (LRU cache key generation)

**Key Format:**
```rust
fn generate_cache_key(file_path: &Path, commit_hash: &str, user_role: &str) -> String {
    let key_string = format!("{}||{}||{}", 
        file_path.display(), commit_hash, user_role);
    sha256(key_string.as_bytes())
}
```

### Phase 3: Cache Invalidation

**Tasks:**
- Integrate with file watcher (CM-RQ-004)
- Implement invalidation on file modification events
- Implement bulk invalidation for Git commit changes
- Implement cache warming on startup for hot documents

**Traceability:** blue_paper.md:277-295 (RE-RQ-006)
- adr-004: File watching integration

**Invalidation Logic:**
```rust
fn invalidate_on_file_change(file_path: &Path) {
    // Remove all cache entries for this file path
    let prefix = format!("{}||", file_path.display());
    cache.retain(|k| !k.starts_with(&prefix));
}
```

### Phase 4: Configuration Management

**Tasks:**
- Add TOML configuration file for cache settings
- Implement capacity configuration (entries and MB)
- Implement hit rate monitoring
- Add metrics export (hits, misses, evictions)

**Traceability:** domain_constraints.toml:185-223 (LRU cache configuration)

**Configuration Structure:**
```toml
[cache.lru]
# Maximum number of cache entries
capacity_entries = 1000

# Maximum cache size in MB
capacity_mb = 100

# Target hit rate (0.0-1.0)
target_hit_rate = 0.80

# Eviction time threshold (ms)
eviction_timeout_ms = 5
```

---

## Consequences

### Positive Consequences

1. **Performance:**
   - Sub-1ms cache hit response for hot documents
   - >80% hit rate achievable with 1000 entries
   - Reduces rendering latency from <15ms to <4ms average

2. **Multi-User Support:**
   - Role-based cache keys enable per-role HTML caching
   - Guest, user, admin users see different redacted content
   - Supports concurrent users with proper cache isolation

3. **Scalability:**
   - Configurable capacity (100-10000 entries) for different repository sizes
   - 10-500MB memory range for resource-constrained environments
   - Supports personal (1000 docs) to enterprise (100K docs) deployments

4. **Maintainability:**
   - Well-understood LRU algorithm with extensive research
   - Mature Rust ecosystem (dashmap, concurrent HashMap)
   - Simple implementation with clear invariants

### Negative Consequences

1. **Cache Invalidation Complexity:**
   - Composite keys require prefix-based invalidation
   - Commit hash changes invalidate multiple entries per file
   - Increased invalidation overhead

2. **Memory Usage:**
   - 1000 entries at ~100KB = 100MB typical
   - Maximum 10000 entries may require 1GB memory
   - Requires capacity planning for large repositories

3. **Implementation Complexity:**
   - Linked list management for LRU ordering
   - DashMap integration for concurrent access
   - Cache key generation overhead

---

## Monitoring and Validation

### Success Criteria

| Metric | Target | Measurement Method |
|---------|--------|-------------------|
| Cache hit latency | <1ms P99 | test_vectors.toml:190-203 |
| Cache hit rate | >80% | test_vectors.toml:190-203 |
| Eviction time | <5ms | test_vectors.toml:190-203 |
| Memory usage | 10-500MB | Memory profiling |
| Multi-user isolation | No cross-role cache pollution | Integration tests |

### Testing Strategy

1. **Unit Tests:**
   - Test LRU eviction order
   - Test cache capacity limits
   - Test key collision handling
   - Test concurrent access patterns

2. **Integration Tests:**
   - Test cache invalidation on file changes
   - Test role-based cache key isolation
   - Test cache warming on startup

3. **Performance Tests:**
   - Benchmark hit rate for typical workloads
   - Measure eviction latency
   - Profile memory usage at capacity limits

4. **Multi-User Tests:**
   - Test guest/user/admin cache isolation
   - Test concurrent access from different roles
   - Verify redaction isolation

### Rollback Plan

If LRU cache fails to meet performance targets:

1. **Increase Default Capacity:** Raise from 1000 to 2000 entries
2. **Implement L2 Cache:** Add optional disk-based cache layer
3. **Optimize Key Generation:** Use faster hash function (blake3)
4. **Implement Prefetching:** Predict and cache likely-to-access documents

---

## Related Decisions

- blue_paper.md:259-276 (LRU Cache Implementation)
- proof.lean:17-85 (LRU Cache Invariants)
- adr-004: File watching integration (cache invalidation)
- domain_constraints.toml:185-223 (LRU cache configuration)

---

## References

1. **Research Sources:**
   - integrated_findings.md:172-196 (LRU Caching Mechanisms)
   - yellow_paper.md:278-311 (LRU and Its Variants)

2. **Requirements:**
   - requirements.md:315-329 (RE-RQ-005)
   - requirements.md:330-344 (RE-RQ-006)

3. **Domain Constraints:**
   - domain_constraints.toml:185-223 (LRU cache configuration)
   - domain_constraints.toml:208-223 (LRU cache eviction)

4. **Test Vectors:**
   - test_vectors.toml:157-168 (LRU sequential access)
   - test_vectors.toml:179-190 (LRU cache key generation)

5. **Architecture:**
   - blue_paper.md:259-276 (LRU Cache Implementation)
   - blue_paper.md:277-295 (Cache Invalidation)

---

**Document Revision History:**

| Version | Date | Author | Changes |
|---------|-------|--------|---------|
| 1.0 | 2026-02-11 | Initial ADR creation |
