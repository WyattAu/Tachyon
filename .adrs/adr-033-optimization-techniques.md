# ADR-033: Optimization Techniques

**Status:** Accepted
**Date:** 2026-02-11
**Context:** Phase 4 (Performance Engineering)
**Authors:** Performance Engineer Agent

---

## Context

The Tachyon system requires specific optimization techniques to meet performance requirements defined in [`performance_requirements.md`](.specs/04_performance/performance_requirements.md).

This ADR documents the approved optimization techniques, their implementation strategies, and associated trade-offs.

---

## Decision

**We adopt a multi-faceted optimization approach:**

1. **Cache Optimization:** Improve hit rates and reduce cache misses
2. **SIMD Acceleration:** Utilize CPU vector instructions for performance-critical paths
3. **Async Operations:** Batch operations to improve throughput and reduce blocking
4. **Memory Optimization:** Reduce allocations and enable pooling
5. **Lock Optimization:** Reduce contention through sharding and RCU patterns

### Optimization Technique Classification:

| Category | Techniques | Priority | Risk Level |
|-----------|-------------|----------|-------------|
| Cache | Hit rate improvement, prewarming | P1 | LOW |
| SIMD | Vector operations, CPU feature detection | P2 | MEDIUM |
| Async | Batch operations, spawn_blocking | P1 | LOW |
| Memory | String interning, arena allocation | P2 | LOW |
| Locks | Sharding, RCU, fine-grained locking | P1 | HIGH |

---

## Alternatives Considered

### Alternative 1: No Optimization (Premature Optimization)

**Description:** Optimize code before profiling and benchmarking.

**Pros:**
- Faster initial delivery
- No profiling overhead

**Cons:**
- Optimization may target non-critical paths
- May introduce bugs
- Wasted effort on ineffective changes
- Cannot measure actual improvement

**Rejection:** Data-driven optimization required.

---

### Alternative 2: Over-Optimization (Gold Plating)

**Description:** Optimize all code paths without regard for maintainability or effort.

**Pros:**
- Maximum performance achieved

**Cons:**
- Unmaintainable code
- Complex codebase
- High technical debt
- Difficult to debug
- Future changes more expensive

**Rejection:** Cost-benefit analysis required.

---

### Alternative 3: Optimization Without Trade-off Analysis

**Description:** Apply optimizations without documenting or analyzing trade-offs.

**Pros:**
- Faster development

**Cons:**
- Unexpected side effects
- Hidden costs
- Cannot make informed decisions
- Difficulty prioritizing future work

**Rejection:** Trade-off documentation required for ADR.

---

## Chosen Approach: Prioritized, Trade-off Documented Optimizations

**Rationale:**

1. **Data-Driven:** Optimizations based on profiling data and bottleneck analysis
2. **Prioritized:** Focus on P1 (critical path) optimizations first
3. **Trade-off Aware:** Document memory, complexity, and portability costs
4. **Maintainable:** Balance performance with code quality and maintainability
5. **Measurable:** Each optimization has success criteria and rollback plan

---

## Approved Optimization Techniques

### OPT-1: Cache Hit Rate Improvement

**Status:** P1 - Critical Path

**Description:** Improve LRU cache hit rate from 80% to 95% through smart caching strategies.

**Implementation:**
```rust
// Adaptive cache with prewarming and hit rate optimization
struct AdaptiveCache {
    map: DashMap<CacheKey, CacheEntry>,
    hit_count: AtomicU64,
    miss_count: AtomicU64,
    hit_rate_target: f64,
}

impl AdaptiveCache {
    fn calculate_hit_rate(&self) -> f64 {
        let total = self.hit_count.load(Ordering::Relaxed) + 
                     self.miss_count.load(Ordering::Relaxed);
        if total == 0 {
            return 0.0;
        }
        self.hit_count.load(Ordering::Relaxed) as f64 / total
    }

    fn adjust_capacity(&mut self) {
        let hit_rate = self.calculate_hit_rate();
        if hit_rate < self.hit_rate_target {
            // Increase cache size if hit rate is low
            self.capacity *= 2;
        }
    }

    async fn warm_cache(&self, frequent_paths: Vec<PathBuf>) {
        // Pre-render frequently accessed documents
        for path in frequent_paths {
            self.render_document(&path, "latest", "admin").await;
        }
    }
}
```

**Success Criteria:**
- Cache hit rate > 95%
- Cache miss latency < 20ms
- No memory budget exceeded

**Trade-offs:**
- Increased memory for larger cache
- Cache warming adds startup latency
- Adaptive logic adds complexity

**Traceability:** [`performance_requirements.md`](.specs/04_performance/performance_requirements.md) PR-LAT-001, [`optimization_roadmap.md`](.specs/04_performance/optimization_roadmap.md) OPT-REND-001

---

### OPT-2: SIMD-Accelerated Markdown Parsing

**Status:** P2 - Performance Improvement

**Description:** Enable SIMD features in pulldown-cmark for vector-accelerated Markdown parsing.

**Implementation:**
```rust
// Cargo.toml
[dependencies]
pulldown-cmark = { version = "0.9.6", features = ["simd"] }

// CPU feature detection
#[cfg(target_arch = "x86_64")]
fn check_simd_support() -> bool {
    is_x86_feature_detected!("avx2") ||
    is_x86_feature_detected!("avx512f") ||
    is_x86_feature_detected!("sse4.2")
}

// SIMD-optimized string comparison
#[target_feature(enable = "avx2")]
unsafe fn fast_compare_avx2(a: &[u8], b: &[u8]) -> bool {
    let len = a.len().min(b.len());
    let chunks = (len + 31) / 32;
    
    for i in 0..chunks {
        let a_chunk = _mm256_loadu_si256(a.get_unchecked(i * 32));
        let b_chunk = _mm256_loadu_si256(b.get_unchecked(i * 32));
        let cmp = _mm256_cmpeq_epi8(a_chunk, b_chunk);
        
        if _mm256_movemask_epi8(cmp) != 0xFF {
            return false;
        }
    }
    
    true
}
```

**Success Criteria:**
- Markdown parsing latency -30%
- No regression in rendering quality
- SIMD code path verified on supported CPUs

**Trade-offs:**
- Reduced portability (requires CPU feature detection)
- Increased binary size
- Complex fallback code for non-SIMD CPUs
- Unsafe code blocks

**Traceability:** [`performance_requirements.md`](.specs/04_performance/performance_requirements.md) PR-LAT-002, [`optimization_roadmap.md`](.specs/04_performance/optimization_roadmap.md) OPT-REND-002

---

### OPT-3: Template Precompilation

**Status:** P2 - Performance Improvement

**Description:** Precompile Minijinja templates to avoid repeated compilation overhead.

**Implementation:**
```rust
struct TemplateCache {
    compiled: DashMap<String, minijinja::CompiledTemplate<'static>>,
}

impl TemplateCache {
    async fn get_or_compile(
        &self,
        template_name: &str
    ) -> Result<&CompiledTemplate<'static>> {
        if let Some(compiled) = self.compiled.get(template_name) {
            return Ok(compiled);
        }

        let source = load_template_source(template_name).await?;
        let compiled = minijinja::compile(&source)?;
        self.compiled.insert(template_name.to_string(), compiled);
        Ok(self.compiled.get(template_name).unwrap())
    }

    fn invalidate(&self, template_name: &str) {
        self.compiled.remove(template_name);
    }
}
```

**Success Criteria:**
- Template rendering latency -60%
- Cache hit rate improved (templates cached)
- No stale template rendering

**Trade-offs:**
- Increased memory for compiled templates
- Cache invalidation complexity
- Template changes require cache clear
- Memory leak risk if not properly managed

**Traceability:** [`performance_requirements.md`](.specs/04_performance/performance_requirements.md) PR-LAT-003, [`optimization_roadmap.md`](.specs/04_performance/optimization_roadmap.md) OPT-REND-003

---

### OPT-4: Async Batch Indexing

**Status:** P1 - Critical Path

**Description:** Batch document indexing operations to improve throughput and reduce per-document overhead.

**Implementation:**
```rust
struct BatchIndexer {
    buffer: Vec<Document>,
    batch_size: usize,
    flush_interval: Duration,
}

impl BatchIndexer {
    async fn add_document(&mut self, doc: Document) {
        self.buffer.push(doc);

        if self.buffer.len() >= self.batch_size {
            self.flush().await;
        }
    }

    async fn flush(&mut self) -> Result<()> {
        let batch = std::mem::take(&mut self.buffer);
        let writer = self.index_writer.lock().await;
        
        for doc in batch {
            writer.add_document(doc).await?;
        }
        
        writer.commit().await?;
    }
}
```

**Success Criteria:**
- Indexing throughput 5x improvement
- Per-document latency within target
- Memory usage within budget

**Trade-offs:**
- Delayed index visibility (batch window)
- Increased memory for batch buffer
- Complex error recovery (partial batch failure)

**Traceability:** [`performance_requirements.md`](.specs/04_performance/performance_requirements.md) PR-LAT-005, PR-THR-003, [`optimization_roadmap.md`](.specs/04_performance/optimization_roadmap.md) OPT-SEARCH-001

---

### OPT-5: Query Result Caching

**Status:** P1 - Critical Path

**Description:** Cache search query results to avoid repeated BM25 calculations.

**Implementation:**
```rust
struct QueryCache {
    results: LruCache<QueryKey, Vec<SearchResult>>,
    ttl: Duration,
}

struct QueryKey {
    query: String,
    limit: usize,
    filters: SearchFilters,
}

impl QueryCache {
    async fn get_or_execute(
        &self,
        query: &str,
        limit: usize,
        filters: &SearchFilters
    ) -> Result<Vec<SearchResult>> {
        let key = QueryKey {
            query: query.to_string(),
            limit,
            filters: filters.clone(),
        };

        if let Some(cached) = self.results.get(&key) {
            return Ok(cached);
        }

        let results = self.execute_search(query, limit, filters).await?;
        self.results.put(key, results.clone());
        Ok(results)
    }
}
```

**Success Criteria:**
- Search latency -70% for repeated queries
- Cache hit rate > 80%
- No stale results served

**Trade-offs:**
- Stale results if documents change
- Increased memory for query cache
- Cache invalidation complexity

**Traceability:** [`performance_requirements.md`](.specs/04_performance/performance_requirements.md) PR-LAT-004, [`optimization_roadmap.md`](.specs/04_performance/optimization_roadmap.md) OPT-SEARCH-002

---

### OPT-6: String Interning

**Status:** P2 - Memory Optimization

**Description:** Deduplicate string allocations through interning pool.

**Implementation:**
```rust
struct StringInterner {
    pool: HashSet<Arc<str>>,
}

impl StringInterner {
    fn intern(&mut self, s: &str) -> Arc<str> {
        if let Some(existing) = self.pool.get(s) {
            return existing.clone();
        }

        let arc_str: Arc::from(s);
        self.pool.insert(arc_str.clone());
        arc_str
    }

    fn intern_bytes(&mut self, bytes: &[u8]) -> Arc<[u8]> {
        if let Some(existing) = self.pool.get_str_from_bytes(bytes) {
            return existing.clone();
        }

        let arc_bytes: Arc::from(bytes);
        self.pool.insert_bytes(arc_bytes.clone());
        arc_bytes
    }
}
```

**Success Criteria:**
- Memory usage -30%
- No functional regression
- Interning overhead acceptable

**Trade-offs:**
- Runtime overhead for interning lookups
- Memory for interning pool
- Complex cache invalidation

**Traceability:** [`memory_management.md`](.specs/03_5_resource_management/memory_management.md), [`performance_requirements.md`](.specs/04_performance/performance_requirements.md) PR-MEM-001, PR-MEM-004, [`optimization_roadmap.md`](.specs/04_performance/optimization_roadmap.md) OPT-MEM-001

---

### OPT-7: Lock-Free Broadcasting

**Status:** P1 - Critical Path

**Description:** Replace RwLock with lock-free data structures for WebSocket broadcasting.

**Implementation:**
```rust
use crossbeam::channel::{unbounded, Receiver, Sender};
use atomic_refcell::AtomicRefCell;

struct LockFreeBroadcaster {
    connections: AtomicRefCell<Vec<Arc<WebSocket>>>,
    message_tx: Sender<ServerMessage>,
}

impl LockFreeBroadcaster {
    fn new() -> Self {
        let (tx, _) = unbounded();
        LockFreeBroadcaster {
            connections: AtomicRefCell::new(Vec::new()),
            message_tx: tx,
        }
    }

    async fn broadcast(&self, message: ServerMessage) {
        self.message_tx.send(message).unwrap();

        // Clone connections without lock
        let connections = self.connections.borrow();
        for connection in connections.iter() {
            let conn = Arc::clone(connection);
            let msg = message.clone();
            
            // Spawn without blocking
            tokio::spawn(async move {
                let _ = conn.send(msg).await;
            });
        }
    }
}
```

**Success Criteria:**
- Broadcast latency -60%
- Lock contention eliminated
- No deadlocks

**Trade-offs:**
- Increased code complexity
- Memory overhead for atomic operations
- Potential message duplication

**Traceability:** [`performance_requirements.md`](.specs/04_performance/performance_requirements.md) PR-LAT-009, [`thread_safety_analysis.md`](.specs/02_5_concurrency/thread_safety_analysis.md), [`deadlock_analysis.md`](.specs/02_5_concurrency/deadlock_analysis.md), [`optimization_roadmap.md`](.specs/04_performance/optimization_roadmap.md) OPT-WS-001

---

## Implementation Roadmap

### Phase 1: Critical Path Optimizations (v1.0)

| Optimization | Priority | Effort | Target Release |
|--------------|----------|----------|---------------|
| OPT-1 | P1 | 2 weeks | v1.0 |
| OPT-4 | P1 | 1 week | v1.0 |
| OPT-6 | P1 | 2 weeks | v1.0 |

**Acceptance:** All P1 optimizations must be complete before v1.0 release.

### Phase 2: Performance Improvements (v1.1)

| Optimization | Priority | Effort | Target Release |
|--------------|----------|----------|---------------|
| OPT-2 | P2 | 3 weeks | v1.1 |
| OPT-3 | P2 | 1 week | v1.1 |
| OPT-7 | P2 | 1 week | v1.1 |

**Acceptance:** All P2 optimizations must be complete before v1.1 release.

### Phase 3: Advanced Optimizations (v2.0)

| Optimization | Priority | Effort | Target Release |
|--------------|----------|----------|---------------|
| OPT-5 | P1 | 1 week | v2.0 |

**Acceptance:** All remaining optimizations complete before v2.0.

---

## Rollback Strategy

### Rollback Criteria

| Optimization | Rollback Trigger | Rollback Action |
|--------------|------------------|----------------|
| OPT-1 | Cache hit rate < 90% | Revert to adaptive logic |
| OPT-2 | Regression > 5% | Disable SIMD feature |
| OPT-3 | Memory leak detected | Revert to cached templates |
| OPT-4 | Index quality degraded | Revert to synchronous indexing |
| OPT-5 | Stale results detected | Revert to query cache |
| OPT-6 | Deadlock detected | Revert to RwLock broadcasting |
| OPT-7 | Memory increase > 20% | Revert to interning pool |

### Rollback Procedure

1. Detect regression through benchmarks
2. Create rollback PR with revert commit
3. Document failure analysis
4. Update optimization roadmap with alternative approach
5. Schedule fix for next release

---

## Consequences

### Positive Consequences

1. **Data-Driven:** All optimizations based on profiling data and bottleneck analysis
2. **Prioritized:** P1 critical path optimizations prioritized
3. **Trade-off Aware:** Documented costs and risks for each optimization
4. **Measurable:** Success criteria and rollback plans defined
5. **Maintainable:** Balance between performance and code quality

### Negative Consequences

1. **Implementation Complexity:** Multiple optimizations require significant development effort
2. **Testing Overhead:** Each optimization requires benchmark validation
3. **Rollback Risk:** Failed optimizations may require rollback and re-implementation
4. **Maintenance Burden:** Optimized code requires ongoing maintenance

---

## Related Documents

- [`performance_requirements.md`](.specs/04_performance/performance_requirements.md) - Performance targets
- [`benchmark_suite.md`](.specs/04_performance/benchmark_suite.md) - Benchmark design
- [`optimization_roadmap.md`](.specs/04_performance/optimization_roadmap.md) - Optimization plan
- [`profiling_strategy.md`](.adrs/adr-031-profiling-strategy.md) - Profiling methodology
- [`memory_management.md`](.specs/03_5_resource_management/memory_management.md) - Memory optimization
- [`thread_safety_analysis.md`](.specs/02_5_concurrency/thread_safety_analysis.md) - Concurrency analysis

---

## References

- Optimization Patterns: "Optimizing Software in C++" by Agner Fog
- System Performance: "Computer Systems: A Programmer's Perspective" by Randal E. Bryant
- Rust Performance: "The Rust Performance Book" by Nicholas Matsakis and Niklaus Matsakis
