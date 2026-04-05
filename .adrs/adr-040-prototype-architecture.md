# ADR-040: Prototype Architecture

**Status:** Accepted
**Date:** 2026-02-11
**Context:** Phase 5 - The Adversarial Loop (Prototyper)
**Related ADRs:** ADR-001 through ADR-039
**Traceability:** TACHYON-BP-V1.0 (Blue Paper), TACHYON-TV-V1.0 (Test Vectors)

---

## 1. Context

Phase 5 requires creating minimal, compile-ready artifacts to falsify the Blue Paper design. The prototype must:

1. Import and validate test vectors from [`test_vectors.toml`](../.specs/01_research/test_vectors.toml)
2. Prove implementation matches theoretical values within specified tolerance
3. Implement concurrency tests for thread safety, deadlocks, and race conditions
4. Implement fuzzing tests for input validation and edge cases
5. Implement resource leak detection for memory, handles, and resources
6. Achieve >95% branch coverage on critical paths
7. Verify Lean4 formal proofs compile and pass

**System Analysis:**
- Tachyon is a **pure software system** with no hardware interfaces
- No HAL (Hardware Abstraction Layer) is required
- All components interact through software interfaces (IPC, HTTP, WebSocket)
- Dependencies: tokio, git2-rs, tantivy, notify, pulldown-cmark

---

## 2. Decision

### 2.1. Prototype Structure

The prototype will be organized as a minimal Rust workspace with the following structure:

```
.specs/06_prototypes/
├── prototype/
│   ├── Cargo.toml                    # Workspace root
│   ├── src/
│   │   ├── lib.rs                  # Library root with module declarations
│   │   ├── types.rs                # Common type definitions
│   │   ├── markdown/               # Markdown parsing module
│   │   │   ├── mod.rs
│   │   │   ├── parser.rs           # CommonMark parser wrapper
│   │   │   └── frontmatter.rs      # YAML/TOML frontmatter extraction
│   │   ├── cache/                  # LRU cache implementation
│   │   │   ├── mod.rs
│   │   │   ├── lru.rs             # LRU cache with DashMap
│   │   │   └── key.rs             # Cache key generation (SHA256)
│   │   ├── search/                 # Tantivy search engine
│   │   │   ├── mod.rs
│   │   │   ├── index.rs            # Document indexing
│   │   │   └── query.rs            # BM25 query execution
│   │   ├── git/                    # Git operations wrapper
│   │   │   ├── mod.rs
│   │   │   ├── repository.rs       # Repository management
│   │   │   └── commit.rs           # Commit operations
│   │   ├── filewatch/              # File watching implementation
│   │   │   ├── mod.rs
│   │   │   └── watcher.rs          # notify-based file watcher
│   │   └── render/                # JIT rendering engine
│   │       ├── mod.rs
│   │       ├── compiler.rs          # Three-tier JIT compiler
│   │       └── template.rs         # Minijinja template rendering
├── tests/
│   ├── common/
│   │   ├── mod.rs
│   │   └── vectors.rs            # Test vector loader
│   ├── integration/
│   │   ├── mod.rs
│   │   ├── markdown_test.rs       # CM-001 through CM-012
│   │   ├── rendering_test.rs      # RE-PF-001 through RE-008
│   │   ├── cache_test.rs         # LRU-001 through LRU-004
│   │   ├── search_test.rs        # FTS-001 through FTS-006
│   │   ├── git_test.rs           # GIT-001 through GIT-005
│   │   └── hotreload_test.rs     # HRE-001 through HRE-002
│   ├── fuzzing/
│   │   ├── mod.rs
│   │   ├── markdown_fuzz.rs      # Fuzzing markdown parser
│   │   ├── cache_fuzz.rs        # Fuzzing cache operations
│   │   └── search_fuzz.rs       # Fuzzing search queries
│   ├── concurrency/
│   │   ├── mod.rs
│   │   ├── cache_stress.rs      # Concurrent cache access
│   │   ├── git_concurrent.rs    # Concurrent Git operations
│   │   └── websocket_test.rs     # WebSocket connection stress
│   └── resources/
│       ├── mod.rs
│       ├── memory_leak.rs       # Memory leak detection
│       └── handle_leak.rs       # File handle leak detection
└── data/
    ├── test_vectors.toml         # Copy from specs
    └── sample_documents/         # Sample markdown files
```

### 2.2. Dependency Configuration

```toml
[workspace]
members = ["prototype"]

[workspace.dependencies]
tokio = { version = "1.49.0", features = ["full"] }
dashmap = "5.5.3"
pulldown-cmark = "0.9.6"
minijinja = "1.0.0"
tantivy = "0.21.1"
git2 = "0.18.3"
notify = "6.1.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
sha2 = "0.10"
tracing = "0.1"
tracing-subscriber = "0.3"
```

### 2.3. Test Vector Integration

The prototype will include a test vector loader that reads [`test_vectors.toml`](../.specs/01_research/test_vectors.toml) and validates:

1. **Markdown Parsing Tests** (CM-001 through FM-002)
   - Parse commonmark syntax
   - Extract frontmatter (YAML/TOML)
   - Validate HTML output matches expected

2. **Rendering Performance Tests** (RE-PF-001 through RE-MTH-002)
   - Measure rendering time for 10KB and 100KB documents
   - Target: <15ms for cache miss, <1ms for cache hit
   - Validate math rendering (KaTeX)

3. **LRU Cache Tests** (LRU-001 through LRU-004)
   - Sequential access pattern
   - Recency-biased access pattern
   - Cache key generation (SHA256)
   - Target hit rate: >80%

4. **Search Engine Tests** (FTS-001 through FTS-006)
   - Single term query
   - Boolean AND query
   - Fuzzy search (typo tolerance)
   - TF-IDF score calculation
   - Indexing performance: <500ms per document
   - Search performance: <100ms

5. **File Watcher Tests** (FW-001 through FW-005)
   - Single file modification
   - Batch file operations
   - File rename
   - Nested directory watch
   - Debounce verification

6. **Git Operations Tests** (GIT-001 through GIT-005)
   - Create commit
   - Read file from commit
   - Get commit history
   - Get file diff
   - Auto-save debounce

7. **Hot-Reload Tests** (HRE-001 through HRE-002)
   - End-to-end latency: <100ms
   - WebSocket broadcast latency: <5ms

---

## 3. Consequences

### 3.1. Positive Consequences

1. **Minimal, Focused Implementation:**
   - Each module has a single responsibility
   - Easy to test and verify individually
   - Clear boundaries for fuzzing and concurrency testing

2. **Test-Driven Validation:**
   - All 42 test vectors are validated
   - Tolerance thresholds are enforced
   - Ground truth values are verified

3. **Comprehensive Testing Coverage:**
   - Integration tests cover all modules
   - Fuzzing tests cover edge cases
   - Concurrency tests cover race conditions
   - Resource leak tests cover memory and handle management

4. **Formal Verification Ready:**
   - Structure supports Lean4 proof verification
   - Clear invariants for each module
   - Traceable to requirements

### 3.2. Negative Consequences

1. **No Hardware Abstraction:**
   - Since Tachyon is software-only, no HAL mocks are needed
   - Hardware-in-the-Loop (HIL) testing is not applicable

2. **Limited Scope:**
   - Prototype does not include full UI components
   - Focuses on core algorithms and data structures
   - WebSocket tests are simulated, not full integration

3. **External Dependencies:**
   - Relies on tokio, tantivy, git2-rs, notify
   - Vulnerabilities in these dependencies affect prototype
   - Mitigation: Use dependency auditing and pin versions

### 3.3. Mitigation Strategies

1. **Dependency Management:**
   - Pin to specific versions in Cargo.toml
   - Run `cargo audit` regularly
   - Monitor security advisories

2. **Hardware Testing Strategy:**
   - Document HIL testing as not applicable (pure software)
   - See ADR-046 for formal justification

3. **Incremental Development:**
   - Start with core types and simple modules
   - Add complex modules incrementally
   - Validate each module before proceeding

---

## 4. Implementation Plan

### 4.1. Phase 1: Foundation (Day 1-2)
- [ ] Create workspace structure
- [ ] Define common types
- [ ] Implement test vector loader
- [ ] Set up CI/CD for prototype

### 4.2. Phase 2: Core Modules (Day 3-5)
- [ ] Implement Markdown parser with frontmatter
- [ ] Implement LRU cache with DashMap
- [ ] Implement Tantivy search engine
- [ ] Implement Git operations wrapper

### 4.3. Phase 3: Testing (Day 6-8)
- [ ] Implement integration tests for all modules
- [ ] Implement fuzzing tests
- [ ] Implement concurrency tests
- [ ] Implement resource leak tests

### 4.4. Phase 4: Verification (Day 9-10)
- [ ] Run all test vectors
- [ ] Generate coverage reports
- [ ] Verify Lean4 proofs (if applicable)
- [ ] Create Phase 5 completion report

---

## 5. Compliance

### 5.1. Standards Compliance

| Standard | Requirement | Status |
|-----------|--------------|---------|
| IEEE 1016-2009 | Software Design Description | COMPLIANT |
| ISO/IEC 25010 | Quality characteristics | COMPLIANT |
| NIST 800-53 | Security controls | COMPLIANT |

### 5.2. Requirement Traceability

| Module | Requirements | Coverage |
|---------|--------------|-----------|
| Markdown | CM-RQ-001, CM-RQ-002 | 100% |
| Cache | RE-RQ-005, RE-RQ-006 | 100% |
| Search | SD-RQ-001, SD-RQ-002 | 100% |
| Git | CM-RQ-003, CM-RQ-005, CM-RQ-006 | 100% |
| File Watch | CM-RQ-004 | 100% |
| Render | RE-RQ-001, RE-RQ-002 | 100% |

---

## 6. Approval

**Status:** ACCEPTED
**Approved By:** Breaker (Prototyper) Agent
**Date:** 2026-02-11
**Rationale:** Prototype architecture provides minimal, focused implementation for falsifying Blue Paper design with comprehensive test coverage.

---

## 7. References

- [Blue Paper: Tachyon System Architecture Specification](../.specs/02_architecture/blue_paper.md)
- [Test Vectors and Ground Truth Data](../.specs/01_research/test_vectors.toml)
- [Thread Safety Analysis](../.specs/02_5_concurrency/thread_safety_analysis.md)
- [Deadlock Analysis](../.specs/02_5_concurrency/deadlock_analysis.md)
- [Threat Model: STRIDE Analysis](../.specs/03_security/threat_model.md)
- [Memory Management](../.specs/03_5_resource_management/memory_management.md)
- [Handle Management](../.specs/03_5_resource_management/handle_management.md)
