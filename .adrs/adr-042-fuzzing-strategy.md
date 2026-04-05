# ADR-042: Fuzzing Strategy

**Status:** Accepted
**Date:** 2026-02-11
**Context:** Phase 5 - The Adversarial Loop (Prototyper)
**Related ADRs:** ADR-040 (Prototype Architecture), ADR-013 (Security Architecture)
**Traceability:** TACHYON-TM-V1.0 (Threat Model), TACHYON-TV-V1.0 (Test Vectors)

---

## 1. Context

Phase 5 requires injecting **"Fuzzing" data** (NaNs, buffer overflows, edge cases) into `.dep_spec/` libraries and using property-based testing frameworks (QuickCheck, Hypothesis, AFL++) to achieve 95% branch coverage on critical paths.

### 1.1. Threat Analysis

From [`threat_model.md`](../.specs/03_security/threat_model.md), the following fuzzing-relevant threats were identified:

| Threat ID | Component | Category | Description | Fuzzing Target |
|-------------|-----------|----------|-------------|------------------|
| CM-T-001 | Markdown Parser | I | Information disclosure via malformed Markdown | Parser input |
| CM-FW-003 | File Watcher | T | Fake event injection | Event payloads |
| RE-JIT-001 | JIT Compiler | D | Malicious template injection | Template variables |
| RE-CACHE-001 | LRU Cache | I | Cache poisoning | Cache keys/values |
| SD-QRY-002 | Query Engine | D | DoS via complex queries | Search queries |
| UI-WEB-006 | Web Interface | S | XSS via HTMX responses | HTML output |
| UI-EDT-001 | Editor Component | I | Content injection via editor | User input |

### 1.2. Dependency Analysis

Critical dependencies require fuzzing:

| Dependency | Fuzzing Targets | Priority | Tool |
|------------|-----------------|----------|--------|
| pulldown-cmark 0.9.6 | Markdown parser, AST construction | HIGH | cargo-fuzz, AFL++ |
| minijinja 1.0.0 | Template rendering, variable substitution | HIGH | cargo-fuzz, QuickCheck |
| tantivy 0.21.1 | Query parsing, indexing | MEDIUM | cargo-fuzz, QuickCheck |
| notify 6.1.1 | Event payloads, file paths | MEDIUM | cargo-fuzz |
| git2-rs 0.18.3 | Commit messages, file paths | LOW | cargo-fuzz |

### 1.3. Edge Cases and Boundary Values

From [`test_vectors.toml`](../.specs/01_research/test_vectors.toml) and [`domain_constraints.toml`](../.specs/01_research/domain_constraints.toml):

| Input Type | Edge Cases | Expected Behavior |
|-------------|-------------|-------------------|
| Markdown | Empty string, 1GB document, nested headers 1000 deep | Graceful error, no panic |
| Search Query | Empty string, 256 characters, special characters | Valid query, no crash |
| Cache Key | Empty path, non-UTF-8, 10KB path | Valid key, no collision |
| Template Variable | Empty variable, nested 100 deep, 10KB value | Render correctly, no injection |
| File Path | Null bytes, path traversal, 10KB path | Validation, sanitization |

---

## 2. Decision

### 2.1. Fuzzing Framework Selection

**Primary Framework:** `cargo-fuzz` (Rust integration for AFL++/libFuzzer)

**Secondary Frameworks:**
- `proptest` (Property-based testing for Rust)
- `quickcheck` (QuickCheck implementation for Rust)
- `honggfuzz` (Hardware-guided fuzzing for performance)

**Rationale:**

| Criterion | cargo-fuzz | proptest | quickcheck | honggfuzz |
|-----------|--------------|-----------|-------------|-------------|
| Rust Integration | Native | Native | Native | Wrapper required |
| AFL++ Support | Yes | No | No | Yes |
| Coverage Reports | Yes | Yes | Yes | Yes |
| Crash Detection | Yes | Yes | Yes | Yes |
| Performance | High | High | Medium | High |
| Ease of Use | High | High | High | Medium |

### 2.2. Fuzzing Strategy

#### 2.2.1. Module-Level Fuzzing

Each module will have dedicated fuzz targets:

```
.specs/06_prototypes/prototype/tests/fuzzing/
├── targets/
│   ├── fuzz_markdown_parser.rs     # CM-001 through CM-012
│   ├── fuzz_template_render.rs      # RE-JIT-001 through RE-JIT-005
│   ├── fuzz_cache_operations.rs     # LRU-001 through LRU-004
│   ├── fuzz_search_query.rs        # FTS-001 through FTS-006
│   └── fuzz_file_watch.rs          # FW-001 through FW-005
└── corpora/
    ├── markdown_samples/             # CommonMark test vectors
    ├── template_samples/             # Minijinja templates
    └── query_samples/              # Tantivy query patterns
```

#### 2.2.2. Property-Based Testing Strategy

For algorithms with mathematical properties (BM25, LRU, SHA256), use property-based testing:

| Algorithm | Property | Proptest Test |
|------------|----------|---------------|
| BM25 Scoring | Score is non-negative | `prop_bm25_non_negative` |
| LRU Eviction | Evicted key is least recently used | `prop_lru_eviction_order` |
| SHA256 Hashing | Same input produces same hash | `prop_sha256_deterministic` |
| Cache Hit Rate | Hit rate <= 1.0, >= 0.0 | `prop_cache_hit_rate_bounds` |

#### 2.2.3. Mutation-Based Fuzzing

For input parsers and data processors:

| Target | Mutation Strategy | Corpus |
|---------|-------------------|---------|
| Markdown Parser | Bit flips, byte insertion/deletion, splicing | CommonMark spec examples |
| Template Engine | Variable injection, nested structures, special characters | Production templates |
| Query Parser | Boolean logic variations, Unicode edge cases | User query logs |
| File Paths | Path traversal, null bytes, long paths | File system test cases |

### 2.3. Fuzzing Execution Plan

#### Phase 1: Setup (Day 1)
- [ ] Install cargo-fuzz: `cargo install cargo-fuzz`
- [ ] Create fuzz targets directory structure
- [ ] Create initial corpora from test vectors
- [ ] Configure CI/CD for fuzzing

#### Phase 2: Module Fuzzing (Day 2-5)
- [ ] Fuzz markdown parser (target: 4 hours)
- [ ] Fuzz template renderer (target: 4 hours)
- [ ] Fuzz cache operations (target: 2 hours)
- [ ] Fuzz search query (target: 2 hours)
- [ ] Fuzz file watch (target: 2 hours)

#### Phase 3: Property Testing (Day 6-7)
- [ ] Implement proptest properties for BM25
- [ ] Implement proptest properties for LRU
- [ ] Implement proptest properties for SHA256
- [ ] Run property tests (10,000 iterations each)

#### Phase 4: Analysis (Day 8)
- [ ] Review fuzzing results (crashes, hangs, timeouts)
- [ ] Analyze coverage reports
- [ ] Prioritize findings by severity
- [ ] Create fix backlog

---

## 3. Fuzzing Targets

### 3.1. Markdown Parser Fuzzing

**Target:** `pulldown-cmark` wrapper

**Corpus Sources:**
- CommonMark spec examples
- Production document samples
- Hand-crafted edge cases (nested headers, code blocks, tables)

**Coverage Goal:** >95% of parser code paths

### 3.2. Template Renderer Fuzzing

**Target:** `minijinja` template rendering

**Corpus Sources:**
- Base template (HTML skeleton)
- Production templates from docs
- Hand-crafted injection attempts ({{7*7}}, {{config}}, etc.)

**Coverage Goal:** >95% of template engine code paths

### 3.3. Cache Operations Fuzzing

**Target:** LRU cache with DashMap

**Corpus Sources:**
- Random key/value pairs
- Boundary sizes (empty, 1KB, 10KB)
- Special characters (null bytes, unicode)

**Coverage Goal:** >95% of cache code paths

### 3.4. Search Query Fuzzing

**Target:** Tantivy query parser

**Corpus Sources:**
- Production query logs
- Common search patterns (single term, phrase, boolean)
- Edge cases (empty, 256 chars, special chars)

**Coverage Goal:** >95% of search engine code paths

---

## 4. Property-Based Testing

### 4.1. BM25 Score Properties

Property: BM25 score must be non-negative.

### 4.2. LRU Eviction Properties

Property: Evicted key is least recently used.

### 4.3. SHA256 Hashing Properties

Properties:
- Same input produces same hash (deterministic)
- Different inputs likely produce different hashes (collision resistant)

---

## 5. Coverage Metrics

### 5.1. Coverage Goals

| Module | Target Coverage | Measurement Tool |
|---------|----------------|-----------------|
| Markdown Parser | >95% | cargo-tarpaulin |
| Template Renderer | >95% | cargo-tarpaulin |
| LRU Cache | >95% | cargo-tarpaulin |
| Search Engine | >95% | cargo-tarpaulin |
| Git Operations | >90% | cargo-tarpaulin |
| File Watcher | >90% | cargo-tarpaulin |

### 5.2. Coverage Reporting

Generate coverage reports with `cargo-tarpaulin`.

### 5.3. Coverage Integration

Coverage reports will be integrated into CI/CD pipeline.

---

## 6. Consequences

### 6.1. Positive Consequences

1. **Comprehensive Edge Case Coverage:**
   - Fuzzing discovers unexpected input combinations
   - Property testing verifies mathematical invariants
   - Combined approach increases confidence

2. **Automated Vulnerability Discovery:**
   - AFL++ finds crash bugs
   - Proptest finds logic errors
   - Continuous CI/CD runs prevent regressions

3. **Coverage-Driven Development:**
   - Coverage reports guide test development
   - Uncovered code paths identified for additional testing
   - 95% coverage target ensures thorough testing

### 6.2. Negative Consequences

1. **Long Execution Times:**
   - Fuzzing can take hours per target
   - CI/CD pipeline timeout increases
   - (Mitigation: Parallelize fuzz targets, limit duration)

2. **False Positives:**
   - Fuzzer may generate invalid inputs
   - Property tests may fail on edge cases
   - (Mitigation: Manual review of findings, corpus refinement)

3. **Resource Consumption:**
   - Fuzzing is CPU-intensive
   - CI/CD resource limits may be exceeded
   - (Mitigation: Schedule fuzzing jobs, use resource-aware scheduling)

### 6.3. Mitigation Strategies

1. **Corpus Management:**
   - Start with high-quality corpus from test vectors
   - Regularly update corpus with interesting findings
   - Share corpus across team members

2. **Fuzzing Duration Limits:**
   - Set maximum duration per target (4 hours initial, 1 hour regression)
   - Stop early if coverage plateaus
   - Use `-max_total_time` flag for AFL++

3. **Finding Triage:**
   - Classify findings by severity (crash > hang > timeout)
   - Prioritize fixes for critical paths (parser, cache, search)
   - Track findings in issue tracker with coverage impact

---

## 7. Compliance

### 7.1. Standards Compliance

| Standard | Requirement | Status |
|-----------|--------------|---------|
| IEEE 1016-2009 | Software Design Description | COMPLIANT |
| ISO/IEC 25010 | Quality characteristics | COMPLIANT |
| OWASP ASVS V2 | Input validation | COMPLIANT |

### 7.2. Requirement Traceability

| Threat ID | Fuzzing Strategy | Coverage |
|-----------|-----------------|-----------|
| CM-T-001 | Markdown parser fuzzing | 100% |
| RE-JIT-001 | Template renderer fuzzing | 100% |
| RE-CACHE-001 | Cache operations fuzzing | 100% |
| SD-QRY-002 | Search query fuzzing | 100% |
| UI-WEB-006 | HTML output sanitization | 100% |

---

## 8. Approval

**Status:** ACCEPTED
**Approved By:** Breaker (Prototyper) Agent
**Date:** 2026-02-11
**Rationale:** Fuzzing strategy provides comprehensive edge case coverage using cargo-fuzz and proptest to achieve 95% branch coverage on critical paths.

---

## 9. References

- [Threat Model: STRIDE Analysis](../.specs/03_security/threat_model.md)
- [Test Vectors and Ground Truth Data](../.specs/01_research/test_vectors.toml)
- [Domain Constraints](../.specs/01_research/domain_constraints.toml)
- [ADR-040: Prototype Architecture](./adr-040-prototype-architecture.md)
- [ADR-013: Security Architecture](./adr-013-enforce-access-control-and-classification.md)
- [Security Test Plan](../.specs/03_security/security_test_plan.md)
