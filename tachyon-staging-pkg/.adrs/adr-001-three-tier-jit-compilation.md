# ADR-001: Three-Tier JIT Compilation for Document Rendering

## Status

| Status | Accepted |
|---------|----------|
| Date | 2026-02-11 |
| Decision | Adopt three-tier Just-In-Time (JIT) compilation architecture |
| Context | Rendering Engine Architecture |

---

## Context and Problem Statement

### Current Situation

Tachyon requires sub-15ms rendering latency for real-time editing experience (RE-RQ-001). The system processes Markdown documents, applies role-based content redaction, renders templates, and serves HTML to clients.

### Problem

Direct Markdown-to-HTML conversion on every request fails to meet performance targets for:
- Large documents (>50KB)
- Repeated requests for the same document
- Complex templates with variable substitution
- Role-based redaction operations

### Constraints

| Constraint | Value | Source |
|------------|--------|---------|
| Max rendering latency | <15ms | domain_constraints.toml:32-48 |
| Cache hit response | <1ms | domain_constraints.toml:59-70 |
| Cache capacity | 100-1000 entries | domain_constraints.toml:185-196 |
| Target hit rate | >80% | domain_constraints.toml:196-208 |

### Research Findings

Multi-lingual research (78 sources across 15 languages) confirms:
- V8, SpiderMonkey, JavaScriptCore all use multi-tier compilation
- Three tiers provide optimal balance between startup time and performance
- Baseline compiler enables rapid first-render
- Optimizing compiler improves hot-path performance

Source: yellow_paper.md:23-50 (V8), yellow_paper.md:51-76 (SpiderMonkey), yellow_paper.md:77-103 (JavaScriptCore)

---

## Decision Drivers

| Factor | Impact | Weight |
|---------|--------|--------|
| Performance (RE-RQ-001) | CRITICAL | 40% |
| Cache Efficiency (RE-RQ-005) | HIGH | 30% |
| Code Maintainability | MEDIUM | 15% |
| Accessibility (WCAG-2.1) | MEDIUM | 15% |

---

## Considered Alternatives

### Alternative 1: Single-Pass Rendering

**Description:** Parse Markdown and render HTML in single operation on every request.

**Pros:**
- Simple implementation
- No caching complexity
- Consistent behavior

**Cons:**
- Always <15ms for small documents only
- No optimization for hot documents
- High CPU usage for repeated requests

**Evaluation:** REJECTED - Does not meet <15ms target for typical workloads

### Alternative 2: Pre-compilation (SSG)

**Description:** Pre-compile all documents to static HTML on startup.

**Pros:**
- Zero rendering latency at runtime
- Simple deployment

**Cons:**
- Requires rebuild on document changes
- No real-time editing support
- Increased startup time
- No role-based redaction

**Evaluation:** REJECTED - Incompatible with local-first architecture

### Alternative 3: Two-Tier Caching

**Description:** Use parsed AST cache + rendered HTML cache.

**Pros:**
- Faster than single-pass for repeated requests
- Reduces parsing overhead

**Cons:**
- Increased memory usage (dual cache)
- Cache invalidation complexity
- Still requires template rendering on each request

**Evaluation:** REJECTED - Increased memory footprint and complexity

### Alternative 4: Three-Tier JIT Compilation (SELECTED)

**Description:** Implement three-tier compilation with baseline, optimized, and cached tiers.

**Architecture:**

```
Tier 1 (Baseline Compiler):
  - Direct Markdown-to-HTML parsing
  - Frontmatter extraction
  - Role-based AST redaction
  - Target latency: <15ms

Tier 2 (Template Compiler):
  - Template variable substitution
  - ARIA attribute processing
  - Math rendering (KaTeX)
  - Syntax highlighting (tree-sitter)
  - Target latency: <10ms

Tier 3 (Cached Renderer):
  - LRU cache lookup
  - Direct HTML return
  - Target latency: <1ms
```

**Pros:**
- Baseline tier: Rapid first-render, simple implementation
- Optimized tier: Reuses parsed AST for template rendering
- Cached tier: Sub-1ms response for cache hits
- 80%+ hit rate achievable with role-based keys
- Resolves CONFLICT-001 (WCAG vs. Performance) via ARIA budgeting

**Cons:**
- Cache invalidation complexity (multi-tier keys)
- Memory usage for three caches
- Implementation complexity

**Evaluation:** ACCEPTED - Best balance of performance, complexity, and maintainability

---

## Decision

**Adopt three-tier JIT compilation architecture for document rendering.**

### Rationale

1. **Performance Target Achievement:**
   - Baseline tier: <15ms for cache misses (domain_constraints.toml:32-48)
   - Cached tier: <1ms for cache hits (domain_constraints.toml:59-70)
   - 80% hit rate ensures average <4ms for typical workloads

2. **Research Validation:**
   - Multi-lingual consensus on multi-tier compilation (confidence: 0.98)
   - V8, SpiderMonkey, JavaScriptCore all use this pattern
   - Proven performance benefits over single-pass approach

3. **Accessibility Compliance:**
   - Dedicated ARIA optimization tier resolves WCAG-2.1 vs. Performance conflict
   - ARIA budgeting: 15ms total - 12ms rendering - 3ms ARIA processing
   - Maintains WCAG 2.1 AA compliance while meeting performance targets

4. **Cache Efficiency:**
   - Role-based cache keys enable multi-user support
   - LRU eviction ensures optimal memory usage
   - Configurable capacity (100-1000 entries) balances hit rate and memory

5. **Architectural Fit:**
   - Aligns with local-first architecture (no pre-compilation)
   - Supports real-time editing and hot-reload
   - Compatible with Tauri desktop and Axum web interfaces

### Trade-offs

| Aspect | Benefit | Cost | Mitigation |
|---------|--------|---------|-------------|
| Complexity | Optimal performance | Implementation effort | Well-understood pattern from research |
| Memory | 80%+ hit rate | 3-tier cache overhead | Configurable capacity limits |
| Latency | <1ms for hot documents | Cache invalidation | File watcher integration (CM-RQ-004) |

---

## Implementation Plan

### Phase 1: Baseline Compiler (Tier 1)

**Tasks:**
- Integrate pulldown-cmark 0.9.6 with SIMD acceleration
- Implement frontmatter extraction (YAML/TOML)
- Implement role-based AST redaction for `::: internal` blocks
- Generate HTML from AST

**Traceability:** blue_paper.md:73-87 (CM-001), blue_paper.md:88-102 (CM-RQ-002)

**Dependencies:**
- pulldown-cmark 0.9.6 (dep_spec/pulldown-cmark/dep_spec.toml:1-63)
- serde_yaml 0.9 (for frontmatter)

### Phase 2: Template Compiler (Tier 2)

**Tasks:**
- Integrate minijinja templating engine
- Implement template variable substitution
- Implement KaTeX server-side math rendering
- Implement tree-sitter syntax highlighting
- Implement ARIA attribute processing (3ms budget)

**Traceability:** blue_paper.md:215-241 (RE-RQ-001), blue_paper.md:242-258 (RE-RQ-002)

**Dependencies:**
- minijinja (currently commented in Cargo.toml, to be added)
- katex (math rendering library)
- tree-sitter (syntax highlighting)

**Note:** minijinja feature commented in Cargo.toml:24 - to be enabled for Phase 3

### Phase 3: Cached Renderer (Tier 3)

**Tasks:**
- Implement LRU cache with dashmap 5.5.3
- Implement cache key generation: SHA256(file_path || commit_hash || user_role)
- Implement cache invalidation on file modification
- Implement cache eviction on capacity limit

**Traceability:** blue_paper.md:259-276 (RE-RQ-005), blue_paper.md:277-295 (RE-RQ-006)

**Dependencies:**
- dashmap 5.5.3 (from Cargo.toml:25)

---

## Consequences

### Positive Consequences

1. **Performance:**
   - Average rendering latency <4ms for typical workloads
   - Sub-1ms response for cache hits
   - Meets RE-RQ-001 (<15ms JIT rendering)

2. **Scalability:**
   - Configurable cache capacity supports 100-100,000 document repositories
   - 80%+ hit rate reduces database load

3. **Accessibility:**
   - Resolves CONFLICT-001 in compliance_matrix.md
   - WCAG 2.1 AA compliance maintained

4. **Maintainability:**
   - Clear separation of concerns across three tiers
   - Well-understood pattern with extensive research support

### Negative Consequences

1. **Implementation Complexity:**
   - Three-tier architecture requires careful cache invalidation
   - Multi-key cache management (file_path, commit_hash, user_role)

2. **Memory Usage:**
   - 3-tier cache structure increases memory footprint
   - Requires capacity monitoring and configuration

3. **Development Overhead:**
   - ARIA optimization tier adds complexity
   - Requires performance testing across all tiers

---

## Monitoring and Validation

### Success Criteria

| Metric | Target | Measurement Method |
|---------|--------|-------------------|
| Baseline tier latency | <15ms P95 | test_vectors.toml:113-130 |
| Template tier latency | <10ms P95 | test_vectors.toml:113-130 |
| Cache hit latency | <1ms P99 | test_vectors.toml:190-203 |
| Overall hit rate | >80% | test_vectors.toml:190-203 |
| End-to-end latency | <100ms P99 | test_vectors.toml:377-391 |

### Testing Strategy

1. **Unit Tests:**
   - Test each tier independently
   - Mock cache for isolated testing
   - Verify ARIA attribute generation

2. **Integration Tests:**
   - Test hot-reload flow across all tiers
   - Verify cache invalidation on file modification
   - Verify role-based cache keys

3. **Performance Tests:**
   - Benchmark rendering latency for 10KB, 50KB, 100KB documents
   - Measure cache hit rates for typical workloads
   - Profile memory usage at capacity limits

4. **Accessibility Tests:**
   - WCAG 2.1 AA compliance testing
   - Screen reader testing (NVDA, VoiceOver)
   - Keyboard navigation testing

### Rollback Plan

If three-tier JIT compilation fails to meet performance targets:

1. **Fallback to Two-Tier Caching:** Combine baseline and cached tiers only
2. **Reduce ARIA Processing:** Move to post-render phase
3. **Increase Cache Capacity:** Compensate for lower hit rate
4. **SIMD Optimization:** Ensure pulldown-cmark SIMD features enabled

---

## Related Decisions

- [ADR-003](adr-003-lru-cache-target.md) - LRU cache configuration
- [ADR-004](adr-004-debounce-window.md) - File watching integration
- compliance_matrix.md:CONFLICT-001 - WCAG vs. Performance resolution

---

## References

1. **Research Sources:**
   - yellow_paper.md:23-76 (V8 multi-tier compilation)
   - yellow_paper.md:51-76 (SpiderMonkey multi-tier compilation)
   - yellow_paper.md:77-103 (JavaScriptCore multi-tier compilation)

2. **Requirements:**
   - requirements.md:255-269 (RE-RQ-001)
   - requirements.md:270-284 (RE-RQ-002)
   - requirements.md:315-329 (RE-RQ-005)

3. **Domain Constraints:**
   - domain_constraints.toml:32-48 (JIT rendering latency)
   - domain_constraints.toml:59-70 (Cache hit response)

4. **Test Vectors:**
   - test_vectors.toml:113-130 (Rendering performance)

5. **Architecture:**
   - blue_paper.md:215-241 (Three-tier JIT compilation)
   - blue_paper.md:259-276 (LRU Cache)

---

**Document Revision History:**

| Version | Date | Author | Changes |
|---------|-------|--------|---------|
| 1.0 | 2026-02-11 | Initial ADR creation |
