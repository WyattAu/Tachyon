# Tachyon Roadmap

**Date:** 2026-05-10
**Status:** Post-audit, production-viable
**Version:** 10.0.0 (per tachyon/CHANGELOG.md)

---

## Current State Summary

| Metric | Value |
|--------|-------|
| Workspace crates | 18 |
| Total tests (passing) | 1,296 (lib + integration + doc) |
| Clippy warnings | 0 (with `-D warnings`) |
| Formatting | Clean (rustfmt) |
| Dead code | None detected |
| Production stubs | None in critical paths |
| Pre-commit hook | Installed (fmt + clippy + tests + secrets + artifacts) |

### Issues Resolved This Session

1. **Stack overflow in utoipa OpenAPI generation** -- `TreeNode` self-referential struct caused infinite recursion in utoipa 5.x schema derivation. Fixed with manual JSON Schema `$ref` pattern and `OnceLock` caching.
2. **12 documentation factual errors** -- Wrong framework (Yew -> Leptos), wrong env vars, wrong ports, wrong API syntax, nonexistent crate references, inaccurate CRDT attribution, self-contradictory status claims.
3. **No pre-commit hook** -- Created comprehensive gate enforcing formatting, linting, tests, secret detection, and artifact exclusion.

### Remaining Known Issues

| Issue | Severity | Location |
|-------|----------|----------|
| Version inconsistency across docs (1.1.0 / 4.1.0 / 5.0.0 / 10.0.0) | Medium | VERSION.md, CHANGELOG.md |
| Unsubstantiated 100% compliance claims (9 standards) | Medium | VERSION.md:360-370 |
| XSS warnings (markdown content sanitization incomplete) | Medium | VERSION.md:419-423 |
| CORS not configured for production origins | Low | VERSION.md:422 |
| No rate limiting middleware | Low | VERSION.md:423 |
| Tauri NVIDIA+WebKitGTK EGL display issue | Low | VERSION.md:487 |
| Hardcoded developer paths in VERSION.md | Low | VERSION.md:132-133 |
| Demo credentials in VERSION.md | Low | VERSION.md:148-149 |
| Test count in root docs outdated (276 vs actual 1,296) | Low | README.md, VERSION.md |

---

## Phase 1: Hardening (0-2 weeks)

### 1.1 Security Remediation

- [ ] Implement HTML sanitization in markdown renderer (ammonia is already a dep, verify coverage)
- [ ] Sanitize document titles on output
- [ ] Configure CORS with specific allowed origins (env-based)
- [ ] Add rate limiting middleware (token bucket per IP, configurable limits)
- [ ] Add request body size limits validation with proper error responses
- [ ] Remove demo credentials from documentation, reference `.env.example`

### 1.2 Version Unification

- [ ] Designate `tachyon/CHANGELOG.md` as single authoritative version source
- [ ] Update root `VERSION.md` to reference inner version, not maintain separate tracking
- [ ] Remove or archive root `CHANGELOG.md` (severely outdated at v1.1.0)
- [ ] Add version to `Cargo.toml` workspace package and automate CHANGELOG updates

### 1.3 Compliance Claims Substantiation

- [ ] Remove or downgrade "100% compliance" claims for standards without evidence
- [ ] Add self-assessment methodology and evidence links where claims are retained
- [ ] Generate compliance evidence artifacts for `.specs/09_compliance/evidence/`

### 1.4 Documentation Cleanup

- [ ] Remove hardcoded developer paths from VERSION.md
- [ ] Update test counts across all docs to match actual `cargo test` output
- [ ] Trim VERSION.md to under 100 lines (move benchmarks/security/CI to dedicated files)
- [ ] Verify all code examples in documentation compile and run

---

## Phase 2: Testing & Coverage (2-4 weeks)

### 2.1 Coverage Thresholds

- [ ] Set up `cargo-tarpaulin` or `cargo-llvm-cov` in CI
- [ ] Establish baseline coverage per crate
- [ ] Target: >80% branch coverage overall, >95% on critical paths
- [ ] Add coverage gate to pre-commit hook (warning only, not blocking)

### 2.2 Integration Test Expansion

- [ ] The 212 integration tests in `tachyon/crates/server/tests/` require PostgreSQL. Verify CI Docker service is configured correctly.
- [ ] Add integration tests for billing endpoints (currently untested in integration suite)
- [ ] Add integration tests for OAuth2 flows
- [ ] Add integration tests for plugin runtime WASM invocation
- [ ] Add integration tests for SSG build + download

### 2.3 Property-Based Testing

- [ ] Expand `proptest` usage beyond current coverage
- [ ] Add property tests for document state machine transitions
- [ ] Add property tests for search index consistency
- [ ] Add property tests for CRDT convergence under concurrent operations

### 2.4 Fuzzing

- [ ] Verify `cargo-fuzz` harnesses in `tachyon/crates/testing/src/fuzz/` are operational
- [ ] Add fuzz targets for markdown parser (pulldown-cmark edge cases)
- [ ] Add fuzz targets for search query parser
- [ ] Add fuzz targets for YAML frontmatter parser

---

## Phase 3: Performance (4-6 weeks)

### 3.1 Benchmarking Infrastructure

- [ ] Verify Criterion benchmarks compile and run (`tachyon/crates/benchmarks/`)
- [ ] Add CI benchmark regression detection (compare against main branch baseline)
- [ ] Establish performance budgets per endpoint

### 3.2 Database Performance

- [ ] Add database connection pool monitoring (deadpool metrics)
- [ ] Implement query optimization for high-frequency paths (document list, search)
- [ ] Add database index analysis and optimization
- [ ] Benchmark with realistic data volumes (10K+ documents)

### 3.3 Renderer Performance

- [ ] Benchmark markdown rendering with large documents (100K+ characters)
- [ ] Profile and optimize tree-sitter syntax highlighting for large code blocks
- [ ] Implement streaming rendering for very large documents
- [ ] Cache rendered output with content-hash invalidation

### 3.4 Search Performance

- [ ] Benchmark Tantivy index with large corpora
- [ ] Optimize BM25 ranking parameters against real query patterns
- [ ] Implement query caching for repeated searches
- [ ] Add autocomplete/suggestion latency benchmarks

---

## Phase 4: Architecture Evolution (6-12 weeks)

### 4.1 Real-Time Collaboration

- [ ] Verify CRDT sync protocol handles edge cases (network partition, reconnection)
- [ ] Implement operational transformation fallback for conflict scenarios
- [ ] Add presence indicator debouncing and cleanup
- [ ] Implement cursor position sync with jitter compensation

### 4.2 Plugin System Hardening

- [ ] Add WASM sandbox resource limits (memory, CPU, filesystem)
- [ ] Implement plugin permission model (which APIs a plugin can access)
- [ ] Add plugin marketplace client (registry-client feature exists but untested)
- [ ] Create plugin development SDK and documentation

### 4.3 API Versioning

- [ ] Implement API version negotiation middleware
- [ ] Add deprecation headers for old endpoints
- [ ] Create API migration guide
- [ ] Consider GraphQL as primary API (schema already exists)

### 4.4 Observability

- [ ] Implement structured logging with correlation IDs across all services
- [ ] Add distributed tracing (OpenTelemetry)
- [ ] Create operational dashboards (Prometheus + Grafana configs)
- [ ] Add health check endpoints with dependency status (DB, Redis, search index)

---

## Phase 5: Ecosystem & Distribution (12-16 weeks)

### 5.1 Desktop Application

- [ ] Resolve NVIDIA+WebKitGTK EGL display issue
- [ ] Implement auto-update mechanism (Tauri plugin)
- [ ] Add native file system watcher integration
- [ ] Create installer packages (deb, rpm, MSI, DMG)

### 5.2 Mobile Considerations

- [ ] Evaluate Leptos mobile target feasibility
- [ ] Design responsive UI breakpoints
- [ ] Implement offline-first sync queue with conflict resolution

### 5.3 Multi-Tenancy

- [ ] Implement tenant isolation at database level (row-level security or schema separation)
- [ ] Add tenant-scoped API routes
- [ ] Implement resource quotas per tenant
- [ ] Add tenant provisioning/deprovisioning lifecycle

### 5.4 Import/Export Expansion

- [ ] Add Confluence import
- [ ] Add Notion import
- [ ] Add Markdown with wikilinks roundtrip fidelity
- [ ] Implement incremental export (delta since last export)

---

## Phase 6: Formal Verification & Correctness (Ongoing)

### 6.1 Type Safety

- [ ] Add `#![deny(clippy::unwrap_used)]` progressively across crates
- [ ] Eliminate `expect()` calls on non-infallible operations
- [ ] Add `serde(deny_unknown_fields)` to all request/response types
- [ ] Implement exhaustive error type matching

### 6.2 Invariants

- [ ] Formalize document state machine invariants as compile-time checks
- [ ] Add runtime invariant assertions in debug builds
- [ ] Implement property-based invariant testing

### 6.3 Concurrency

- [ ] Add `loom` tests for concurrent data structures (DashMap usage patterns)
- [ ] Verify deadlock-freedom of lock acquisition orderings
- [ ] Add `tokio::sync` stress tests

---

## Decision Framework

### Priority Matrix

| Criterion | Weight | Description |
|-----------|--------|-------------|
| Security impact | 30% | CVE potential, data exposure, auth bypass |
| User impact | 25% | Feature completeness, UX blocking issues |
| Correctness | 20% | Data integrity, state machine violations |
| Performance | 15% | Latency, throughput, resource usage |
| Developer experience | 10% | Build times, test reliability, documentation |

### Phase Gates

Each phase requires:
1. All tests passing (unit + integration)
2. Clippy clean with `-D warnings`
3. No new security warnings
4. Documentation updated to reflect changes
5. Performance regression check (if applicable)

---

## Success Metrics (6-month target)

| Metric | Current | Target |
|--------|---------|--------|
| Test count | 1,296 | 2,000+ |
| Branch coverage | Unknown | >80% overall, >95% critical |
| Known security issues | 4 medium | 0 medium |
| Documentation accuracy | 37 issues found | 0 high/medium |
| CI pipeline time | Unknown | <15 min |
| P99 API latency (populated DB) | Unknown | <50ms |
| Desktop app | Compiles, display issues | Ships on 3 platforms |
