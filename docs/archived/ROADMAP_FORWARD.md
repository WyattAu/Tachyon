# Tachyon Roadmap Forward: v11.0.0 to Production and Beyond

**Date:** 2026-05-14
**Source:** Comprehensive audit findings
**Current Version:** 11.0.0
**Total Estimated Duration:** 15-24 weeks

---

## Current State Summary

| Metric | Value |
|--------|-------|
| Crates | 16 (`tachyon/crates/`) |
| Lines of Rust | 137,000+ |
| Tests | 1,395 (0 failures) |
| Stubs/todo!/unimplemented! | 0 |
| ADRs | 111 |
| CI/CD Workflows | 12 |
| E2E Tests (Playwright) | 113 (9 failing) |
| Formal Proofs (Lean 4) | Diff3Merge (complete), LCS (partial), GraphInvariants (axioms) |
| TLA+ Specs | 4 written, 0 verified through TLC |

---

## Phase 1: Foundation Hardening (2-3 weeks)

**Objective:** Eliminate correctness-affecting bugs and runtime inconsistencies that could cause production failures.

### 1.1 Resolve Async Runtime Conflict

**Problem:** `casbin` in `tachyon/crates/rbac/Cargo.toml:20` uses `runtime-async-std` feature while the entire project (workspace `Cargo.toml:10`) is built on tokio. Mixing async runtimes causes thread pool contention, potential deadlocks, and unpredictable behavior under load.

**Tasks:**
- Remove `runtime-async-std` feature from casbin dependency in `tachyon/crates/rbac/Cargo.toml:20`
- Add `runtime-tokio` feature if available; if casbin lacks native tokio support, replace casbin with a tokio-compatible RBAC engine (e.g., `casbin-rs` with adapter pattern or hand-rolled policy evaluator)
- Audit all `.await` call sites in `tachyon/crates/rbac/src/` to confirm tokio compatibility
- Run full test suite under tokio `current_thread` and `multi_thread` schedulers

**Effort:** 3-5 days
**Dependencies:** None
**Risk:** Medium -- casbin tokio support may require adapter rewrite
**Success Criteria:** `tokio::runtime::Handle::current()` returns the tokio runtime from all rbac code paths; zero async-std threads in thread dumps under load

### 1.2 Unify Database Backends

**Problem:** `tachyon/crates/database/Cargo.toml:16` configures sqlx with `postgres`, while `tachyon/crates/rbac/Cargo.toml:29` and `tachyon/crates/storage/Cargo.toml:23` both use `sqlite`. This means RBAC and storage cannot share the same database as the core database crate.

**Tasks:**
- Decide canonical backend: PostgreSQL (recommended for production) or SQLite (simpler deployment)
- If PostgreSQL: update `rbac/Cargo.toml:29` and `storage/Cargo.toml:23` to use `postgres` feature; migrate all `sqlite`-specific SQL to PostgreSQL-compatible syntax
- If SQLite: update `database/Cargo.toml:16` to use `sqlite`; adjust migration paths and utoipa schema generation
- Update `tachyon/crates/database/src/migrations/` accordingly
- Verify `docker-compose.yml` and `docker-compose.prod.yml` match the chosen backend
- Update all docs referencing the chosen backend

**Effort:** 5-7 days
**Dependencies:** Phase 1.1 (runtime must be consistent for integration tests)
**Risk:** High -- storage layer SQL may have SQLite-specific pragmas
**Success Criteria:** All three crates (`database`, `rbac`, `storage`) compile against the same sqlx feature flag; integration tests pass with a single database engine

### 1.3 Remove Dead Code

**Problem:** 22 files contain `allow(dead_code)`, indicating unused functions, structs, or modules that inflate the binary and confuse maintainers.

**Tasks:**
- Run `cargo +nightly udeps` to identify unused dependencies across all 16 crates
- For each `allow(dead_code)` occurrence, either remove the dead code or remove the allow attribute if the code is used via conditional compilation
- Remove `tree-sitter-sql = "0.0"` placeholder from `tachyon/crates/renderer/Cargo.toml:49` (no published version exists)
- Audit `tachyon/crates/testing/Cargo.toml:37` -- `tachyon-desktop-app` dependency pulls Tauri/GUI deps into test crate; isolate to feature-gated module or remove if unused

**Effort:** 2-3 days
**Dependencies:** None
**Risk:** Low
**Success Criteria:** Zero `allow(dead_code)` in release builds; `cargo udeps` reports no unused deps; `tachyon-testing` does not depend on `tachyon-desktop-app` without explicit feature flag

### 1.4 Verify TLA+ Specs Through TLC

**Problem:** Four TLA+ specs exist in `tachyon/specs/tla/` (Diff3Merge.tla, GraphInvariants.tla, OperationalTransform.tla, ReviewStateMachine.tla) but none have been verified through the TLC model checker.

**Tasks:**
- Install TLC (Java-based model checker)
- Write TLC configuration files for each spec: `Diff3Merge.cfg`, `GraphInvariants.cfg`, `OperationalTransform.cfg`, `ReviewStateMachine.cfg`
- Define model constraints (state space bounds, invariants to check)
- Run TLC on each spec; document results in `tachyon/specs/tla/README.md`
- Fix any specification errors discovered during verification

**Effort:** 3-4 days
**Dependencies:** None
**Risk:** Medium -- specs may have bugs requiring re-verification cycles
**Success Criteria:** All 4 specs pass TLC without invariant violations; results documented with trace counts

### 1.5 Fix Semgrep SAST Blocking Findings

**Problem:** `security-new.yml:68-74` runs Semgrep with `p/security-audit`, `p/secrets`, `p/owasp-top-ten` rules. 35 findings are currently blocking.

**Tasks:**
- Run Semgrep locally with the same config; export findings as JSON
- Triage each finding: false positive (add baseline suppression), true positive (fix), or acceptable risk (document and suppress with justification)
- For true positives, fix the vulnerable code pattern
- Create `.semgrepignore` or `semgrep_baseline.sarif` for false positives
- Update `security-new.yml` to fail only on new findings (baseline mode)

**Effort:** 3-5 days
**Dependencies:** None
**Risk:** Low
**Success Criteria:** Semgrep CI job passes with zero new findings; all suppressions have documented justification in ADR format

---

## Phase 2: CI/CD Consolidation (1-2 weeks)

**Objective:** Eliminate overlapping deployment workflows and establish a single, reliable pipeline.

### 2.1 Consolidate CD Workflows

**Problem:** Three overlapping CD workflows exist:
- `.github/workflows/cd.yml` (541 lines) -- full pipeline with Docker, WASM, staging, production, rollback
- `.github/workflows/cd-new.yml` (130 lines) -- stub version with placeholder deploys
- `.github/workflows/deploy.yml` (57 lines) -- minimal single-job deploy on main push

**Tasks:**
- Delete `.github/workflows/cd-new.yml` (it is a stub with `echo "This would run..."` placeholders)
- Merge `.github/workflows/deploy.yml` functionality into `.github/workflows/cd.yml` (cd.yml already covers all deploy.yml cases)
- Delete `.github/workflows/deploy.yml`
- Update `.github/workflows/deploy-staging.yml` if it overlaps with the staging job in cd.yml
- Verify no other workflows reference the deleted files

**Effort:** 1-2 days
**Dependencies:** None
**Risk:** Low
**Success Criteria:** Single cd.yml workflow handles all deployment scenarios; `deploy-staging.yml` exists only if it provides distinct functionality

### 2.2 Fix E2E Test Failures

**Problem:** 9 of 113 Playwright tests fail. Some tests use `|| true` vacuous assertions that mask real failures.

**Tasks:**
- Identify the 9 failing tests from the Playwright suite (likely in `tachyon/crates/frontend/` or top-level `e2e/`)
- Remove all `|| true` fallback patterns from test assertions
- Fix or update tests for actual API/behavior changes since tests were written
- Add retry-on-flaky-test configuration for known intermittent failures (with upper bound)
- Run full E2E suite in CI to confirm green

**Effort:** 3-4 days
**Dependencies:** Phase 1.2 (database backend must match what E2E tests expect)
**Risk:** Medium -- some failures may require backend changes
**Success Criteria:** 0 test failures; 0 vacuous assertions; CI E2E job passes consistently

### 2.3 Harden CI Matrix

**Tasks:**
- Verify all 12 workflows trigger correctly and do not conflict (check `concurrency` groups)
- Add required-status-checks to branch protection: CI, security, E2E
- Ensure workflow permissions follow least-privilege principle
- Add workflow timeout limits (max 60 minutes per job)

**Effort:** 2-3 days
**Dependencies:** Phase 2.1
**Risk:** Low
**Success Criteria:** `gh workflow list` shows active workflows; branch protection requires CI+security+E2E

---

## Phase 3: Test Coverage and Quality (2-3 weeks)

**Objective:** Increase meaningful test coverage, add property-based testing, and establish mutation testing.

### 3.1 Align Coverage Expectations

**Problem:** CHANGELOG claims 1,589 tests; actual count is 1,395. Coverage reports claim 95% but `.coverage.toml` sets minimum at 60%.

**Tasks:**
- Run `cargo test --workspace 2>&1 | tail -5` to get authoritative test count
- Update CHANGELOG and VERSION.md with accurate count
- Set realistic coverage targets in `.coverage.toml`: 80% minimum (current setting), 95% aspirational
- Generate actual coverage report with `cargo-llvm-cov` and publish to CI artifact

**Effort:** 1-2 days
**Dependencies:** None
**Risk:** Low
**Success Criteria:** Reported test count matches `cargo test` output; coverage report generated per CI run

### 3.2 Property-Based Testing for Core Algorithms

**Tasks:**
- Add proptest properties to `tachyon/crates/core/src/` for:
  - Document merge operations (commutativity, associativity, idempotency)
  - CRDT operations (convergence property)
  - Permission evaluation (no permission escalation)
- Add quickcheck properties to `tachyon/crates/renderer/src/` for:
  - Markdown round-trip: `parse(render(html)) == html` for well-formed input
  - HTML sanitization: output never contains `<script>`, `javascript:`, `onerror`
- Target: 20 new property-based tests

**Effort:** 4-5 days
**Dependencies:** Phase 1.1, 1.2
**Risk:** Low
**Success Criteria:** `cargo test -p tachyon-core -- properties` passes; `cargo test -p tachyon-renderer -- properties` passes

### 3.3 Mutation Testing

**Tasks:**
- Configure `cargo-mutants` for workspace
- Run mutation testing on `tachyon/crates/core/` and `tachyon/crates/rbac/` (highest-value crates)
- Aim for 80% mutation score on tested code
- Document surviving mutants and create issues for each

**Effort:** 3-4 days
**Dependencies:** Phase 3.2
**Risk:** Low
**Success Criteria:** Mutation score >= 80% on core and rbac; surviving mutants tracked as issues

### 3.4 Integration Test Gap Analysis

**Tasks:**
- Map each public API endpoint in `tachyon/crates/server/src/` to at least one integration test
- Identify untested middleware paths (auth, rate limiting, CORS)
- Add tests for error paths (401, 403, 404, 422, 500)
- Add WebSocket integration tests for CRDT sync scenarios

**Effort:** 4-5 days
**Dependencies:** Phase 1.2
**Risk:** Medium
**Success Criteria:** Every route handler has at least one integration test; error path coverage >= 90%

---

## Phase 4: Documentation Remediation (2-3 weeks)

**Objective:** Clean up stale, incorrect, and duplicated documentation.

### 4.1 Fix Repository URL References

**Problem:** Multiple docs reference `tachyon-org/tachyon` instead of the actual `WyattAu/Tachyon`.

**Tasks:**
- Search all `.md` files for `tachyon-org/tachyon` and replace with `WyattAu/Tachyon`
- Search for fictional `tachyon.io` domain references; replace with `github.com/WyattAu/Tachyon` or remove
- Update `.github/workflows/cd.yml` and `cd-new.yml` hardcoded URLs if present

**Effort:** 1 day
**Dependencies:** None
**Risk:** Low
**Success Criteria:** Zero occurrences of `tachyon-org` or `tachyon.io` in documentation

### 4.2 Consolidate Duplicate Documentation Directories

**Problem:** Four overlapping directory pairs:
- `docs/user/` (19 files) vs `docs/user-guide/` (8 files)
- `docs/developer/` (13 files) vs `docs/dev/` (7 files)

**Tasks:**
- Compare file contents between each pair
- Merge unique content into the canonical location: `docs/user/` and `docs/developer/`
- Remove `docs/user-guide/` and `docs/dev/`
- Update all internal links (`[text](../user-guide/foo.md)` -> `[text](../user/foo.md)`)

**Effort:** 3-4 days
**Dependencies:** Phase 4.1
**Risk:** Medium -- broken links may cascade
**Success Criteria:** No duplicate directory pairs; `grep -r "user-guide\|/dev/" docs/` returns zero hits for old paths

### 4.3 Complete API Documentation

**Problem:** API docs cover approximately 30% of actual endpoints.

**Tasks:**
- Extract all route definitions from `tachyon/crates/server/src/` (search for `.route(`, `.get(`, `.post(`, etc.)
- Generate OpenAPI spec from utoipa annotations in `tachyon/crates/database/Cargo.toml:38` (`utoipa` dependency)
- For each undocumented endpoint, add request/response examples to `docs/api/`
- Cross-reference with `docs/user/api_reference.md`

**Effort:** 5-7 days
**Dependencies:** None
**Risk:** Low
**Success Criteria:** API docs cover 100% of route handlers; OpenAPI spec validates successfully

### 4.4 Fix Stale Reference Files

**Tasks:**
- Populate `TRACEABILITY_MATRIX.md` with actual requirement-to-implementation mappings
- Populate `STANDARD_CONFLICTS.md` if conflicts exist, or add a note documenting that no conflicts were identified
- Update `VERSION.md` to reflect v11.0.0 accurately (fix outdated sections)
- Fix CHANGELOG version ordering (v1.1.0 appearing after v0.2.0 if present)
- Populate `.specs/` directory with spec files referenced by ADRs
- Resolve SQLite/PostgreSQL confusion across docs files

**Effort:** 3-4 days
**Dependencies:** Phase 1.2 (database decision)
**Risk:** Low
**Success Criteria:** No empty stub matrices; VERSION.md matches v11.0.0 reality; CHANGELOG versions in semver order

---

## Phase 5: Performance and Scalability (2-4 weeks)

**Objective:** Establish performance baselines, identify bottlenecks, and prepare for horizontal scaling.

### 5.1 Benchmarking Baseline

**Tasks:**
- Run existing Criterion benchmarks in `tachyon/crates/benchmarks/` and `tachyon/crates/renderer/benches/`
- Run k6 load tests from `load-tests/k6-load.js` against staging
- Document baseline metrics: p50/p95/p99 latency, throughput (req/s), memory footprint, CPU utilization
- Store results in `.reports/benchmark-baseline-v11.md`

**Effort:** 2-3 days
**Dependencies:** Phase 2.2 (E2E green), staging deployment
**Risk:** Low
**Success Criteria:** Baseline report with quantified metrics for all critical paths

### 5.2 Profiling and Optimization

**Tasks:**
- Profile under load using `perf`, `flamegraph`, and `tokio-console`
- Identify top 5 hotspots
- Optimize: consider connection pooling tuning, query optimization, cache hit rates
- Re-benchmark after each optimization

**Effort:** 5-7 days
**Dependencies:** Phase 5.1
**Risk:** Medium -- optimizations may introduce regressions
**Success Criteria:** >= 20% improvement in p95 latency on at least 2 critical paths; no regression in correctness tests

### 5.3 Horizontal Scaling Preparation

**Tasks:**
- Ensure all session state is externalized (Redis, database) -- no in-memory-only state that breaks with multiple server instances
- Verify WebSocket relay works with sticky sessions or shared state
- Add health check endpoints to all services (`/health`, `/ready`)
- Document scaling topology in `docs/architecture/`

**Effort:** 3-5 days
**Dependencies:** Phase 5.2
**Risk:** Medium -- CRDT sync may require architectural changes for multi-instance
**Success Criteria:** Application runs correctly with 2+ backend instances behind a load balancer

---

## Phase 6: Security Certification (2-3 weeks)

**Objective:** Achieve a defensible security posture suitable for production deployment.

### 6.1 Semgrep Zero-Defect Baseline

**Tasks:**
- All 35 Semgrep findings from Phase 1.5 resolved and verified
- Enable `block_on: true` in `security-new.yml` SAST job
- Add SARIF upload to GitHub Security tab

**Effort:** 1-2 days
**Dependencies:** Phase 1.5
**Risk:** Low
**Success Criteria:** Security tab shows zero open Semgrep findings

### 6.2 Dependency Vulnerability Remediation

**Tasks:**
- Run `cargo audit` and fix all RUSTSEC advisories
- Run `trivy` container scan and fix all CRITICAL/HIGH findings
- Establish weekly automated audit in CI (`security-new.yml` schedule: Mondays at 06:00)
- Add Dependabot or Renovate for automated dependency updates

**Effort:** 2-3 days
**Dependencies:** None
**Risk:** Low
**Success Criteria:** `cargo audit` exits 0; Trivy finds zero CRITICAL/HIGH; weekly audit runs green

### 6.3 Lean 4 Proof Completion

**Tasks:**
- Complete partial LCS proof in `tachyon/specs/lean/TachyonProofs/`
- Convert GraphInvariants from axioms to proved theorems where feasible
- Add proofs for OperationalTransform correctness properties
- Document proof coverage in `tachyon/specs/lean/README.md`

**Effort:** 5-7 days
**Dependencies:** Phase 1.4 (TLA+ verification may inform Lean proofs)
**Risk:** High -- some properties may be undecidable or require significant axiom restructuring
**Success Criteria:** LCS proof complete; at least 2 GraphInvariants proved; proof coverage documented

### 6.4 Penetration Testing Preparation

**Tasks:**
- Create threat model document based on STRIDE methodology
- Document attack surface: API endpoints, WebSocket, file upload, authentication flows
- Prepare scope document for external penetration test
- Set up staging environment that mirrors production

**Effort:** 3-4 days
**Dependencies:** Phase 6.1, 6.2
**Risk:** Low
**Success Criteria:** Threat model document complete; staging mirrors production topology

---

## Phase 7: Production Release (1-2 weeks)

**Objective:** Final QA, deployment automation, and operational readiness.

### 7.1 Release Candidate QA

**Tasks:**
- Create v12.0.0-rc.1 tag from main
- Run full CI pipeline (CI + security + E2E + mutation)
- Run smoke tests against staging deployment
- Verify all monitoring dashboards populate correctly (Grafana provisioning in `monitoring/grafana/`)
- Test rollback procedure using cd.yml `rollback: true` input

**Effort:** 2-3 days
**Dependencies:** Phases 1-6 complete
**Risk:** Medium
**Success Criteria:** All CI jobs green; staging deployment healthy; rollback tested and verified

### 7.2 Deployment Automation Verification

**Tasks:**
- Verify cd.yml handles: tag-triggered production deploy, main-branch staging deploy, manual deploy, rollback
- Verify GitHub Release creation with auto-generated release notes
- Verify Slack notifications fire correctly
- Verify Docker images are multi-arch (amd64 + arm64)
- Test blue-green or canary deployment if infrastructure supports it

**Effort:** 2-3 days
**Dependencies:** Phase 7.1
**Risk:** Medium
**Success Criteria:** Full deployment lifecycle tested end-to-end; rollback completes in < 5 minutes

### 7.3 Operational Readiness

**Tasks:**
- Verify all 6 runbooks in `docs/runbooks/` are accurate and actionable
- Test backup/restore procedure from `scripts/backup.sh`
- Verify alerting rules in `monitoring/grafana/provisioning/` fire correctly
- Create incident response template
- Document SLOs: 99.9% uptime, p99 latency < 500ms, error rate < 0.1%

**Effort:** 2-3 days
**Dependencies:** Phase 7.2
**Risk:** Low
**Success Criteria:** Runbooks tested against staging; backup/restore verified; alerting confirmed

---

## Phase 8: Post-Production Evolution (Ongoing)

### 8.1 Nix Flake Completeness

**Problem:** `flake.nix` is missing Lean 4, TLA+, and Playwright tooling.

**Tasks:**
- Add `lean4` package to flake inputs
- Add `tlaplus` (TLC model checker) to flake
- Add Playwright tooling for E2E tests
- Verify `nix develop` provides complete development environment

**Effort:** 2-3 days
**Dependencies:** None (can run in parallel with other phases)
**Risk:** Low
**Success Criteria:** `nix develop` provides all tools needed for development, testing, and verification

### 8.2 Mobile Applications

- Evaluate Tauri mobile support for iOS/Android
- Design offline-first sync architecture leveraging existing CRDT layer
- Implement mobile-specific UI patterns

**Effort:** 8-12 weeks
**Dependencies:** Phase 7

### 8.3 AI-Powered Features

- Add semantic search using embeddings (extend existing Tantivy full-text search)
- Implement AI-assisted document summarization
- Add intelligent linking suggestions based on content similarity
- Integrate with LLM APIs for content generation assistance

**Effort:** 6-10 weeks
**Dependencies:** Phase 7

### 8.4 Plugin Marketplace

- Extend `tachyon/crates/plugin-runtime/` with sandboxed WASM execution
- Create plugin SDK and documentation
- Build marketplace infrastructure (registry, versioning, signing)

**Effort:** 10-15 weeks
**Dependencies:** Phase 7

### 8.5 Enterprise Features

- SAML/OIDC SSO integration
- Audit logging with tamper-evident storage
- Data retention policies and legal hold
- Multi-tenancy with resource isolation
- SCIM provisioning

**Effort:** 12-16 weeks
**Dependencies:** Phase 7

### 8.6 Community Building

- Publish crates to crates.io
- Create contribution guidelines and good-first-issue labels
- Set up community chat (Discord/Zulip)
- Write blog posts on architecture decisions and formal verification approach
- Present at Rust conferences

**Effort:** Ongoing
**Dependencies:** Phase 7

---

## Dependency Graph

```
Phase 1 (Foundation)
  1.1 (async runtime) -----> 1.2 (database unification)
  1.3 (dead code)           1.4 (TLA+)
  1.5 (Semgrep)
        |
        v
Phase 2 (CI/CD)             Phase 8.1 (Nix -- parallel)
  2.1 (consolidate) ---> 2.2 (E2E fix) ---> 2.3 (harden)
        |
        v
Phase 3 (Testing)
  3.1 (coverage) ---> 3.2 (PBT) ---> 3.3 (mutation) ---> 3.4 (integration)
        |
        v
Phase 4 (Docs) [can run in parallel with Phase 3]
  4.1 (URLs) ---> 4.2 (consolidate dirs) ---> 4.3 (API docs) ---> 4.4 (stale files)
        |
        v
Phase 5 (Performance)
  5.1 (baseline) ---> 5.2 (optimize) ---> 5.3 (scaling)
        |
        v
Phase 6 (Security)
  6.1 (Semgrep baseline) ---> 6.2 (deps) ---> 6.4 (pentest prep)
  6.3 (Lean proofs) [parallel track]
        |
        v
Phase 7 (Release)
  7.1 (RC QA) ---> 7.2 (deploy verify) ---> 7.3 (ops readiness)
        |
        v
Phase 8 (Evolution) [ongoing]
```

---

## Risk Register

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| casbin tokio migration breaks RBAC | Medium | High | Feature-flag new engine; keep old as fallback |
| Database unification requires SQL rewrite | High | Medium | Incremental migration with dual-write period |
| TLA+ specs have fundamental errors | Medium | Low | Specs are documentation; errors caught before proof |
| Semgrep findings reveal real vulnerabilities | Medium | High | Prioritize by CVSS; fix before any deployment |
| E2E tests reveal backend regressions | High | Medium | Fix backend; do not weaken test assertions |
| Lean proofs are infeasible for some properties | Medium | Low | Document axioms as assumptions; prove what is possible |
| Performance regression during optimization | Low | Medium | Benchmark after each change; revert on regression |

---

## Summary Timeline

| Phase | Duration | Start | End |
|-------|----------|-------|-----|
| 1: Foundation Hardening | 2-3 weeks | Week 1 | Week 3 |
| 2: CI/CD Consolidation | 1-2 weeks | Week 3 | Week 5 |
| 3: Test Coverage | 2-3 weeks | Week 5 | Week 8 |
| 4: Documentation | 2-3 weeks | Week 3 (parallel) | Week 8 |
| 5: Performance | 2-4 weeks | Week 8 | Week 12 |
| 6: Security Certification | 2-3 weeks | Week 10 | Week 13 |
| 7: Production Release | 1-2 weeks | Week 13 | Week 15 |
| 8: Post-Production | Ongoing | Week 15+ | -- |

**Critical path:** Phase 1 -> Phase 2 -> Phase 3 -> Phase 5 -> Phase 6 -> Phase 7 = 13-15 weeks

Phase 4 (Documentation) runs in parallel with Phase 3, reducing the wall-clock timeline. Phase 8.1 (Nix) can begin at any time.
