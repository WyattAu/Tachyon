# TACHYON: CONTINUOUS INTEGRATION GUIDE

**Document ID:** TACHYON-QA-004-V1.0
**Date:** February 2026
**Status:** Approved for Implementation
**Classification:** Quality Assurance Documentation
**Compliance Level:** ISO/IEC 26514:2021, IEEE 829-2008

---

## TABLE OF CONTENTS

1. [Introduction](#1-introduction)
2. [CI Framework Overview](#2-ci-framework-overview)
3. [CI Pipeline Architecture](#3-ci-pipeline-architecture)
4. [Build Process](#4-build-process)
5. [Test Automation](#5-test-automation)
6. [Code Quality Checks](#6-code-quality-checks)
7. [Deployment Automation](#7-deployment-automation)
8. [CI Monitoring](#8-ci-monitoring)
9. [References](#9-references)

---

## 1. INTRODUCTION

### 1.1. Document Purpose

This document provides comprehensive guidance for implementing and maintaining Continuous Integration (CI) infrastructure for the Tachyon toolchain. It defines the CI framework, pipeline architecture, build processes, test automation, code quality checks, deployment automation, and monitoring procedures. This guide serves as the authoritative reference for establishing and operating CI systems that support Tachyon's development lifecycle.

### 1.2. Scope

This document covers:
- CI framework selection and configuration
- Pipeline architecture and workflow definitions
- Build procedures for all Tachyon components
- Automated testing integration
- Code quality enforcement mechanisms
- Deployment automation strategies
- CI monitoring and alerting
- Security considerations for CI systems

Out of scope:
- Detailed component-level implementation specifications
- Manual deployment procedures
- Local development environment setup
- Production incident response procedures

### 1.3. Document Dependencies

This document depends on the following documents:
- [TACHYON-STD-V1.0](../../.specs/01_standards/coding_standards.md) - Coding and Documentation Standards
- [TACHYON-REQ-BLD-V1.0](../../.specs/04_future_state/reqs/build_requirements.md) - Build and Deployment Requirements
- [TACHYON-REQ-SYS-V1.0](../../.specs/04_future_state/reqs/system_overview.md) - System Overview Requirements
- [TACHYON-ADR-001-V1.0](../../.specs/02_adrs/001_rust_as_primary_language.md) - Rust as Primary Language
- [TACHYON-ADR-010-V1.0](../../.specs/02_adrs/010_security_architecture.md) - Security Architecture
- [TACHYON-DSN-BLD-V1.0](../../.specs/04_future_state/design/build_design.md) - Build System Design
- [TACHYON-TST-V1.0](../../.specs/04_future_state/test_plan.md) - Test Plan

### 1.4. Terminology

| Term | Definition |
|-------|------------|
| **CI** | Continuous Integration - The practice of merging all developers' working copies to a shared mainline several times a day |
| **CD** | Continuous Deployment - The practice of automatically deploying code changes to production environments |
| **Pipeline** | A sequence of automated steps that code changes must pass before being merged |
| **Stage** | A logical grouping of pipeline steps (e.g., build, test, deploy) |
| **Artifact** | A build output produced by the CI system (e.g., binaries, packages) |
| **Matrix** | A configuration that runs the same pipeline with different parameter combinations |
| **Gate** | A condition that must be satisfied before proceeding to the next pipeline stage |
| **Quality Gate** | A set of automated checks that code must pass before being merged |
| **Orchestration** | The process of coordinating multiple CI jobs and stages |

### 1.5. CI Principles

The Tachyon CI system adheres to the following principles:

#### 1.5.1. Fast Feedback

**Principle:** Developers must receive feedback on code changes within 10 minutes of push.

**Implementation:**
- Parallel execution of independent pipeline stages
- Incremental builds leveraging Nix caching
- Early failure detection with fast unit tests
- Optimized test suites prioritizing critical paths

**Requirements:**
- REQ-BLD-005: Caching Strategy
- REQ-SYS-051: Rendering Latency

---

## 5. TEST AUTOMATION

### 5.1. Test Framework Overview

The Tachyon CI system implements comprehensive test automation across multiple test types, ensuring code quality and correctness through automated testing.

**Test Types:**

| Test Type | Purpose | Framework | Execution Time |
|------------|---------|-----------|---------------|
| **Unit Tests** | Test individual functions and modules | Rust built-in | 3-5 min |
| **Integration Tests** | Test component interactions | Rust built-in | 5-8 min |
| **End-to-End Tests** | Test complete user workflows | Custom | 10-20 min |
| **Performance Tests** | Benchmark performance characteristics | Criterion | 5-10 min |
| **Security Tests** | Test security controls | Custom | 2-5 min |

**Requirements:**
- REQ-BLD-067: Automated Testing
- REQ-SYS-052: Search Response Time
- REQ-SYS-054: Concurrent Users

### 5.2. Unit Testing

#### 5.2.1. Unit Test Framework

**Framework:** Rust built-in test framework

**Configuration:**
```toml
[dev-dependencies]
proptest = "1.5"
criterion = "0.5"
```

**Requirements:**
- REQ-BLD-067: Automated Testing
- REQ-SYS-051: Rendering Latency

#### 5.2.2. Unit Test Structure

**Test Organization:**
```
tachyon/
├── crates/
│   ├── desktop/
│   │   └── tests/
│   │       ├── mod.rs
│   │       ├── rendering_test.rs
│   │       ├── ipc_test.rs
│   │       └── ...
│   ├── server/
│   │   └── tests/
│   │       ├── mod.rs
│   │       ├── api_test.rs
│   │       ├── auth_test.rs
│   │       └── ...
│   └── shared/
│       └── tests/
│           ├── mod.rs
│           ├── markdown_test.rs
│           └── ...
```

**Test Module Example:**
```rust
#[cfg(test)]
mod rendering_test;

use tachyon_core::rendering::MarkdownRenderer;

#[test]
fn test_render_markdown_basic() {
    let renderer = MarkdownRenderer::new();
    let input = "# Hello World";
    let expected = "<h1>Hello World</h1>";
    let result = renderer.render(input);
    assert_eq!(result, expected);
}

#[test]
fn test_render_markdown_with_code() {
    let renderer = MarkdownRenderer::new();
    let input = "```rust\nfn main() {}\n```";
    let result = renderer.render(input);
    assert!(result.contains("<code>"));
    assert!(result.contains("fn main()"));
}

#[proptest]
fn test_render_markdown_various_inputs(input: String) {
    let renderer = MarkdownRenderer::new();
    let result = renderer.render(&input);
    assert!(!result.is_empty());
}
```

**Requirements:**
- REQ-BLD-067: Automated Testing
- TACHYON-STD-V1.0: Coding Standards

#### 5.2.3. Unit Test Execution

**CI Configuration:**
```yaml
  unit-tests:
    name: Unit Tests
    runs-on: ${{ matrix.os }}
    needs: [lint, format-check]
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Restore Nix store
        uses: actions/cache/restore@v4
        with:
          path: ~/.nix-store
          key: ${{ runner.os }}-nix-${{ hashFiles('**/*.nix') }}

      - name: Restore Cargo build
        uses: actions/cache/restore@v4
        with:
          path: |
            ~/.cargo/git
            ~/.cargo/bin
            target/
          key: ${{ runner.os }}-cargo-build-${{ hashFiles('**/Cargo.lock') }}

      - name: Run unit tests
        run: |
          nix develop -c "cargo test --lib --bins --no-fail-fast -- -Z unstable-options --report=html"

      - name: Generate coverage report
        run: |
          nix develop -c "cargo llvm-cov --html --no-fail-fast --lib --bins"

      - name: Upload coverage to Codecov
        uses: codecov/codecov-action@v4
        with:
          files: ./lcov.info
          flags: unittests
          name: codecov-${{ matrix.os }}

      - name: Upload coverage artifacts
        uses: actions/upload-artifact@v4
        with:
          name: coverage-${{ matrix.os }}
          path: |
            target/llvm-cov/
            lcov.info
```

**Requirements:**
- REQ-BLD-067: Automated Testing
- REQ-SYS-052: Search Response Time

### 5.3. Integration Testing

#### 5.3.1. Integration Test Framework

**Framework:** Rust built-in test framework with external services

**Configuration:**
```toml
[dev-dependencies]
tokio-test = "0.4"
sqlx = { version = "0.8", features = ["runtime-tokio-rustls", "sqlite"] }
```

**Requirements:**
- REQ-BLD-067: Automated Testing
- REQ-SYS-054: Concurrent Users

#### 5.3.2. Integration Test Structure

**Test Organization:**
```
tachyon/
└── tests/
    ├── integration/
    │   ├── mod.rs
    │   ├── api_integration_test.rs
    │   ├── auth_integration_test.rs
    │   ├── database_integration_test.rs
    │   └── ...
```

**Test Module Example:**
```rust
#[cfg(test)]
mod integration;

use tokio_test::tokio;
use tachyon_server::api::ApiServer;
use tachyon_server::database::Database;

#[tokio::test]
async fn test_api_create_document() {
    let db = Database::in_memory().await;
    let server = ApiServer::new(db).await;
    
    let response = server.create_document(
        "Test Document",
        "# Test Content"
    ).await;
    
    assert_eq!(response.status(), 201);
    assert_eq!(response.body().title, "Test Document");
}

#[tokio::test]
async fn test_api_get_document() {
    let db = Database::in_memory().await;
    let server = ApiServer::new(db).await;
    
    let doc_id = server.create_document(
        "Test Document",
        "# Test Content"
    ).await.body().id;
    
    let response = server.get_document(doc_id).await;
    assert_eq!(response.status(), 200);
    assert_eq!(response.body().title, "Test Document");
}
```

**Requirements:**
- REQ-BLD-067: Automated Testing
- REQ-SYS-054: Concurrent Users

#### 5.3.3. Integration Test Execution

**CI Configuration:**
```yaml
  integration-tests:
    name: Integration Tests
    runs-on: ubuntu-latest
    needs: unit-tests
    services:
      postgres:
        image: postgres:16
        env:
          POSTGRES_PASSWORD: postgres
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Restore Nix store
        uses: actions/cache/restore@v4
        with:
          path: ~/.nix-store
          key: ubuntu-latest-nix-${{ hashFiles('**/*.nix') }}

      - name: Restore Cargo build
        uses: actions/cache/restore@v4
        with:
          path: |
            ~/.cargo/git
            ~/.cargo/bin
            target/
          key: ubuntu-latest-cargo-build-${{ hashFiles('**/Cargo.lock') }}

      - name: Run integration tests
        run: |
          nix develop -c "cargo test --test-threads=1 --no-fail-fast"
        env:
          DATABASE_URL: postgresql://postgres:postgres@localhost:5432/tachyon_test

      - name: Upload test results
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: integration-test-results
          path: |
            target/test-results/
            target/nextest/
```

**Requirements:**
- REQ-BLD-067: Automated Testing
- REQ-SYS-054: Concurrent Users

### 5.4. End-to-End Testing

#### 5.4.1. E2E Test Framework

**Framework:** Custom E2E test framework with headless browser

**Configuration:**
```toml
[dev-dependencies]
headless-chrome = "1.0"
```

**Requirements:**
- REQ-BLD-067: Automated Testing
- REQ-SYS-051: Rendering Latency

#### 5.4.2. E2E Test Structure

**Test Organization:**
```
tachyon/
└── tests/
    └── e2e/
        ├── mod.rs
        ├── document_workflow_test.rs
        ├── search_workflow_test.rs
        └── ...
```

**Test Module Example:**
```rust
#[cfg(test)]
mod e2e;

use headless_chrome::{Browser, LaunchOptions};
use tachyon_desktop::ipc::IpcClient;

#[test]
fn test_document_creation_workflow() {
    let browser = Browser::new(LaunchOptions::default()).unwrap();
    let ipc = IpcClient::new().unwrap();
    
    // Navigate to application
    browser.goto("http://localhost:8080").unwrap();
    
    // Create new document
    ipc.send_command("create_document", "Test Document").unwrap();
    
    // Wait for document creation
    browser.wait_for_selector(".document-title").unwrap();
    
    // Verify document exists
    let title = browser.find(".document-title").unwrap();
    assert_eq!(title.text().unwrap(), "Test Document");
}
```

**Requirements:**
- REQ-BLD-067: Automated Testing
- REQ-SYS-051: Rendering Latency

#### 5.4.3. E2E Test Execution

**CI Configuration:**
```yaml
  e2e-tests:
    name: End-to-End Tests
    runs-on: ubuntu-latest
    needs: integration-tests
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Restore Nix store
        uses: actions/cache/restore@v4
        with:
          path: ~/.nix-store
          key: ubuntu-latest-nix-${{ hashFiles('**/*.nix') }}

      - name: Restore Cargo build
        uses: actions/cache/restore@v4
        with:
          path: |
            ~/.cargo/git
            ~/.cargo/bin
            target/
          key: ubuntu-latest-cargo-build-${{ hashFiles('**/Cargo.lock') }}

      - name: Start desktop application
        run: |
          nix develop -c "cargo run --bin tachyon-desktop &"
        env:
          RUST_LOG: debug

      - name: Run E2E tests
        run: |
          nix develop -c "cargo test --test e2e"

      - name: Upload test results
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: e2e-test-results
          path: |
            target/test-results/
            screenshots/
```

**Requirements:**
- REQ-BLD-067: Automated Testing
- REQ-SYS-051: Rendering Latency

### 5.5. Performance Testing

#### 5.5.1. Performance Test Framework

**Framework:** Criterion benchmarking framework

**Configuration:**
```toml
[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }
```

**Requirements:**
- REQ-BLD-067: Automated Testing
- REQ-SYS-051: Rendering Latency
- REQ-SYS-052: Search Response Time

#### 5.5.2. Performance Test Structure

**Test Organization:**
```
tachyon/
└── benches/
    ├── rendering_bench.rs
    ├── search_bench.rs
    └── ...
```

**Benchmark Example:**
```rust
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

criterion_group!(rendering);
fn bench_render_markdown(c: &mut Criterion) {
    c.bench_function("basic", |b| {
        let renderer = MarkdownRenderer::new();
        b.iter(|| {
            renderer.render("# Hello World");
        });
    });

    c.bench_function("with_code", |b| {
        let renderer = MarkdownRenderer::new();
        let input = "```rust\nfn main() {}\n```";
        b.iter(|| {
            renderer.render(black_box(input));
        });
    });
}

criterion_group!(search);
fn bench_search_documents(c: &mut Criterion) {
    let db = Database::in_memory().unwrap();
    let documents = create_test_documents(1000);
    
    c.bench_function("search", |b| {
        b.iter(|| {
            db.search("test query");
        });
    });
}

criterion_main!(rendering, search);
```

**Requirements:**
- REQ-BLD-067: Automated Testing
- REQ-SYS-051: Rendering Latency
- REQ-SYS-052: Search Response Time

#### 5.5.3. Performance Test Execution

**CI Configuration:**
```yaml
  performance-tests:
    name: Performance Tests
    runs-on: ubuntu-latest
    needs: unit-tests
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Restore Nix store
        uses: actions/cache/restore@v4
        with:
          path: ~/.nix-store
          key: ubuntu-latest-nix-${{ hashFiles('**/*.nix') }}

      - name: Restore Cargo build
        uses: actions/cache/restore@v4
        with:
          path: |
            ~/.cargo/git
            ~/.cargo/bin
            target/
          key: ubuntu-latest-cargo-build-${{ hashFiles('**/Cargo.lock') }}

      - name: Run benchmarks
        run: |
          nix develop -c "cargo bench -- --output-format bencher"

      - name: Upload benchmark results
        uses: actions/upload-artifact@v4
        with:
          name: benchmark-results
          path: target/criterion/

      - name: Store benchmark data
        uses: benchmark-action/github-action-benchmark@v1
        with:
          tool: 'cargo-criterion'
          output-file-path: target/criterion/
```

**Requirements:**
- REQ-BLD-067: Automated Testing
- REQ-SYS-051: Rendering Latency

### 5.6. Test Coverage

#### 5.6.1. Coverage Requirements

**Coverage Thresholds:**

| Component | Target Coverage | Minimum | Ideal |
|-----------|---------------|---------|--------|
| **Desktop** | Line coverage | 80% | 90% |
| **Server** | Line coverage | 80% | 90% |
| **Shared** | Line coverage | 85% | 95% |
| **Overall** | Line coverage | 80% | 90% |

**Requirements:**
- REQ-BLD-067: Automated Testing
- REQ-SYS-052: Search Response Time

#### 5.6.2. Coverage Reporting

**Coverage Tools:**
- **cargo-llvm-cov:** LLVM-based coverage for Rust
- **Codecov:** Coverage aggregation and reporting
- **GitHub Actions:** Native coverage reporting

**CI Configuration:**
```yaml
- name: Generate coverage report
  run: |
    nix develop -c "cargo llvm-cov --html --no-fail-fast --lib --bins"

- name: Upload coverage to Codecov
  uses: codecov/codecov-action@v4
  with:
    files: ./lcov.info
    flags: unittests
    name: codecov-${{ matrix.os }}

- name: Comment coverage on PR
  if: github.event_name == 'pull_request'
  uses: romeovs/coverage-comments@v1
  with:
    GITHUB_TOKEN: ${{ github.token }}
```

**Requirements:**
- REQ-BLD-067: Automated Testing
- REQ-SYS-052: Search Response Time

### 5.7. Quality Gates

#### 5.7.1. Quality Gate Definition

Quality gates enforce that code meets quality standards before being merged.

**Gate Categories:**

| Gate | Purpose | Threshold | Blocking |
|-------|---------|-----------|----------|
| **Test Coverage** | Ensure adequate test coverage | 80% line coverage | Yes |
| **Test Pass Rate** | Ensure tests pass | 100% pass rate | Yes |
| **Lint Clean** | Ensure no lint warnings | 0 warnings | Yes |
| **Security Scan** | Ensure no vulnerabilities | 0 critical/high | Yes |
| **Performance** | Ensure performance benchmarks pass | Within thresholds | Warning |

**Requirements:**
- REQ-BLD-067: Automated Testing
- REQ-SYS-052: Search Response Time

#### 5.7.2. Quality Gate Enforcement

**CI Configuration:**
```yaml
- name: Check test coverage
  run: |
    COVERAGE=$(cargo llvm-cov --summary --only --output-dir target/llvm-cov | grep "region" | awk '{print $2}' | sed 's/%//')
    echo "Coverage: $COVERAGE%"
    if (( $(echo "$COVERAGE < 80" | bc -l)); then
      echo "Coverage below threshold (80%)"
      exit 1
    fi

- name: Check test pass rate
  run: |
    cargo test --no-fail-fast -- -Z unstable-options --report=html
    if [ $? -ne 0 ]; then
      echo "Tests failed"
      exit 1
    fi

- name: Check lint clean
  run: |
    LINT_OUTPUT=$(cargo clippy --all-targets -- -D warnings 2>&1 || true)
    if [ -n "$LINT_OUTPUT" ]; then
      echo "Lint warnings found"
      exit 1
    fi
```

**Requirements:**
- REQ-BLD-067: Automated Testing
- REQ-SYS-052: Search Response Time

### 5.8. Test Result Reporting

#### 5.8.1. Test Result Storage

**Artifact Storage:**
- **Test Results:** JUnit XML format
- **Coverage Reports:** HTML and LCOV formats
- **Benchmark Results:** Criterion JSON format
- **Screenshots:** PNG format for E2E tests

**CI Configuration:**
```yaml
- name: Upload test results
  if: always()
  uses: actions/upload-artifact@v4
  with:
    name: test-results
    path: |
      target/test-results/
      target/nextest/

- name: Upload coverage reports
  if: always()
  uses: actions/upload-artifact@v4
  with:
    name: coverage-reports
    path: |
      target/llvm-cov/
      lcov.info

- name: Upload benchmark results
  if: always()
  uses: actions/upload-artifact@v4
  with:
    name: benchmark-results
    path: target/criterion/
```

**Requirements:**
- REQ-BLD-067: Automated Testing
- REQ-SYS-052: Search Response Time

#### 5.8.2. Test Result Notification

**Notification Channels:**
- **GitHub Status Checks:** Native GitHub status checks
- **Pull Request Comments:** Coverage and test results
- **Slack:** CI failure notifications
- **Email:** Critical failure notifications

**CI Configuration:**
```yaml
- name: Notify on failure
  if: failure()
  uses: 8398a7/action-slack@v3
  with:
    status: ${{ job.status }}
    text: 'Build failed: ${{ github.repository }} (${{ github.ref }})'
    webhook_url: ${{ secrets.SLACK_WEBHOOK }}

- name: Comment coverage on PR
  if: github.event_name == 'pull_request'
  uses: romeovs/coverage-comments@v1
  with:
    GITHUB_TOKEN: ${{ github.token }}
```

**Requirements:**
- REQ-BLD-067: Automated Testing
- REQ-SYS-052: Search Response Time

#### 1.5.2. Reproducibility

**Principle:** CI builds must be reproducible across different environments and over time.

**Implementation:**
- Nix flakes for hermetic build environments
- Pinned dependency versions via Cargo.lock and bun.lock
- Deterministic build configurations
- Containerized CI runners with consistent toolchains

**Requirements:**
- REQ-BLD-001: Nix Flakes
- REQ-BLD-004: Deterministic Builds
- REQ-BLD-014: Dependency Locking

#### 1.5.3. Security

**Principle:** CI systems must enforce security controls and prevent supply chain attacks.

**Implementation:**
- Dependency verification with SHA-256 checksums
- Security scanning of all dependencies
- Secret management through CI-provided mechanisms
- Code signing of release artifacts
- Vulnerability scanning on all builds

**Requirements:**
- REQ-BLD-072: Secret Management
- REQ-SYS-071: Authentication
- REQ-SYS-074: Input Validation

#### 1.5.4. Automation

**Principle:** All quality checks must be automated and enforced without manual intervention.

**Implementation:**
- Automated testing across all components
- Code quality checks (linting, formatting)
- Security scanning integration
- Automated deployment to staging environments
- Rollback automation on failure

**Requirements:**
- REQ-BLD-066: CI/CD Integration
- REQ-BLD-067: Automated Testing
- REQ-BLD-068: Automated Release

---

## 2. CI FRAMEWORK OVERVIEW

### 2.1. Framework Selection

**Decision:** GitHub Actions is selected as the primary CI framework for Tachyon toolchain.

**Rationale:**
1. **Native Git Integration:** Seamless integration with GitHub repositories and pull requests
2. **Workflow Definition:** YAML-based workflow definitions with clear syntax
3. **Matrix Support:** Built-in support for matrix builds across platforms and configurations
4. **Caching:** First-class caching support for Nix, Cargo, and other build artifacts
5. **Secret Management:** Secure secret management through GitHub Secrets
6. **Community:** Large community with extensive workflow examples and integrations
7. **Free Tier:** Generous free tier for open-source projects
8. **Self-Hosted Option:** Ability to self-host with GitHub Actions Runner for security-sensitive deployments

**Alternatives Considered:**
- **GitLab CI:** Excellent features but requires GitLab hosting
- **Jenkins:** Highly configurable but complex setup and maintenance
- **CircleCI:** Good features but limited free tier
- **Travis CI:** Declining support and limited features

### 2.2. CI Architecture

The Tachyon CI architecture follows a multi-tier approach:

```mermaid
graph TB
    subgraph "Source Control"
        GitHub[GitHub Repository]
        PR[Pull Requests]
        Push[Push Events]
    end
    
    subgraph "CI Orchestrator"
        Actions[GitHub Actions]
        Workflows[Workflow Definitions]
        Secrets[Secret Management]
        Cache[Artifact Caching]
    end
    
    subgraph "Build Matrix"
        Linux[Linux Builds]
        MacOS[macOS Builds]
        Windows[Windows Builds]
        WASM[WebAssembly Builds]
    end
    
    subgraph "Quality Gates"
        Unit[Unit Tests]
        Integration[Integration Tests]
        Lint[Linting]
        Security[Security Scanning]
    end
    
    subgraph "Artifacts"
        Binaries[Release Binaries]
        Packages[Platform Packages]
        Docs[Documentation]
        Coverage[Coverage Reports]
    end
    
    subgraph "Deployment"
        Staging[Staging Deploy]
        Registry[Package Registry]
        Release[GitHub Release]
    end
    
    GitHub --> Actions
    PR --> Actions
    Push --> Actions
    Actions --> Linux
    Actions --> MacOS
    Actions --> Windows
    Actions --> WASM
    
    Linux --> Unit
    MacOS --> Unit
    Windows --> Unit
    WASM --> Unit
    
    Unit --> Integration
    Integration --> Lint
    Lint --> Security
    
    Security --> Binaries
    Security --> Packages
    Security --> Docs
    Security --> Coverage
    
    Binaries --> Staging
    Packages --> Registry
    Docs --> Release
```

### 2.3. Workflow Structure

CI workflows are organized into logical categories:

| Workflow | Purpose | Triggers | Frequency |
|----------|---------|----------|
| **ci.yml** | Main CI pipeline for pull requests and pushes | On every push and pull request |
| **release.yml** | Release build and publish workflow | On tag creation |
| **nightly.yml** | Nightly builds and long-running tests | Scheduled daily at midnight UTC |
| **security.yml** | Security scanning and dependency audit | On every push and weekly scheduled |
| **performance.yml** | Performance benchmarking | On every push to main branch |
| **docs.yml** | Documentation build and deploy | On every push to docs branch |

### 2.4. Platform Support

The CI system supports the following platforms:

| Platform | Architectures | Status | Toolchain |
|----------|---------------|--------|-----------|
| **Linux** | x86_64, aarch64 | Tier 1 | GCC 14+, Clang 18+, Rust 1.82+ |
| **macOS** | x86_64, aarch64 | Tier 1 | Clang 18+, Rust 1.82+ |
| **Windows** | x86_64 | Tier 1 | MSVC 2022, Rust 1.82+ |
| **WebAssembly** | wasm32-unknown-unknown | Tier 1 | Rust 1.82+, emscripten |

**Platform Matrix:**

```yaml
strategy:
  matrix:
    os: [ubuntu-latest, macos-latest, windows-latest]
    rust: [stable]
    target:
      - x86_64-unknown-linux-gnu
      - x86_64-apple-darwin
      - x86_64-pc-windows-msvc
    include:
      - os: ubuntu-latest
        target: aarch64-unknown-linux-gnu
        rust: stable
      - os: macos-latest
        target: aarch64-apple-darwin
        rust: stable
```

### 2.5. Caching Strategy

The CI system implements multi-layer caching for optimal performance:

#### 2.5.1. Nix Store Cache

**Purpose:** Cache Nix store for reproducible builds.

**Configuration:**
```yaml
- name: Cache Nix store
  uses: actions/cache@v4
  with:
    path: ~/.nix-store
    key: ${{ runner.os }}-nix-${{ hashFiles('**/*.nix') }}
    restore-keys: |
      ${{ runner.os }}-nix-
```

**Benefits:**
- Reproducible builds across CI runs
- Reduced dependency resolution time
- Consistent build environments

#### 2.5.2. Cargo Registry Cache

**Purpose:** Cache Cargo registry for Rust dependencies.

**Configuration:**
```yaml
- name: Cache Cargo registry
  uses: actions/cache@v4
  with:
    path: ~/.cargo/registry
    key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}
```

**Benefits:**
- Faster dependency downloads
- Reduced network bandwidth
- Consistent dependency versions

#### 2.5.3. Cargo Build Cache

**Purpose:** Cache Cargo build artifacts for incremental compilation.

**Configuration:**
```yaml
- name: Cache Cargo build
  uses: actions/cache@v4
  with:
    path: |
      ~/.cargo/git
      ~/.cargo/bin
      target/
    key: ${{ runner.os }}-cargo-build-${{ hashFiles('**/Cargo.lock') }}
    restore-keys: |
      ${{ runner.os }}-cargo-build-
```

**Benefits:**
- Incremental compilation across CI runs
- Faster rebuild times
- Reduced compilation overhead

#### 2.5.4. Bun Cache

**Purpose:** Cache Bun packages for web frontend.

**Configuration:**
```yaml
- name: Cache Bun packages
  uses: actions/cache@v4
  with:
    path: |
      ~/.bun/install/cache
      node_modules
    key: ${{ runner.os }}-bun-${{ hashFiles('web/bun.lock') }}
```

**Benefits:**
- Faster dependency installation
- Reduced network requests
- Consistent dependency versions

### 2.6. Secret Management

The CI system uses GitHub Secrets for secure credential storage:

**Secret Categories:**

| Secret Name | Purpose | Required For |
|-------------|---------|--------------|
| **CARGO_REGISTRY_TOKEN** | Publishing to crates.io | Release workflow |
| **NPM_TOKEN** | Publishing to npm registry | Release workflow |
| **CODE_SIGNING_KEY** | Code signing for release artifacts | Release workflow |
| **SSH_DEPLOY_KEY** | Deployment to staging servers | Deploy workflow |
| **DOCKER_HUB_TOKEN** | Publishing to Docker Hub | Release workflow |
| **GITHUB_TOKEN** | GitHub API access for releases | Release workflow |
| **SLACK_WEBHOOK** | CI notifications | All workflows |

**Security Principles:**
1. **Least Privilege:** Secrets have minimum required permissions
2. **Rotation:** Secrets are rotated regularly (90-day maximum)
3. **Audit:** All secret access is logged and auditable
4. **Scope:** Secrets are scoped to specific repositories and workflows
5. **Encryption:** Secrets are encrypted at rest and in transit

**Requirements:**
- REQ-BLD-072: Secret Management
- REQ-SYS-071: Authentication
- REQ-SYS-075: Audit Logging

---

## 3. CI PIPELINE ARCHITECTURE

### 3.1. Main CI Workflow

The main CI workflow ([`.github/workflows/ci.yml`](../../.github/workflows/ci.yml)) executes on every push and pull request, providing comprehensive quality checks for all code changes.

#### 3.1.1. Workflow Triggers

```yaml
name: CI

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main, develop]
  workflow_dispatch:

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1
```

**Trigger Rationale:**
- **Push to main/develop:** Validates merges to main branches
- **Pull requests:** Provides feedback on proposed changes
- **Manual dispatch:** Enables on-demand CI runs for debugging

#### 3.1.2. Pipeline Stages

The CI pipeline is organized into sequential stages:

```mermaid
graph LR
    A[Setup] --> B[Lint]
    B --> C[Format Check]
    C --> D[Unit Tests]
    D --> E[Integration Tests]
    E --> F[Security Scan]
    F --> G[Build Artifacts]
    G --> H[Upload Artifacts]
    
    style A fill:#e1f5fe
    style B fill:#f39c12
    style C fill:#f39c12
    style D fill:#4caf50
    style E fill:#ff9800
    style F fill:#795548
    style G fill:#795548
    style H fill:#795548
```

**Stage Descriptions:**

| Stage | Purpose | Duration | Failure Impact |
|-------|---------|----------|---------------|
| **Setup** | Install dependencies and configure environment | 2-3 min | Blocks entire pipeline |
| **Lint** | Run Clippy lints | 1-2 min | Blocks subsequent stages |
| **Format Check** | Verify code formatting | 30 sec | Blocks subsequent stages |
| **Unit Tests** | Run unit test suite | 3-5 min | Blocks subsequent stages |
| **Integration Tests** | Run integration test suite | 5-8 min | Blocks subsequent stages |
| **Security Scan** | Scan for vulnerabilities | 2-3 min | Non-blocking but logged |
| **Build Artifacts** | Build release binaries | 3-5 min | Blocks artifact upload |
| **Upload Artifacts** | Store build artifacts | 1-2 min | Final stage |

#### 3.1.3. Stage 1: Setup

**Purpose:** Install Nix, configure Rust toolchain, and restore caches.

```yaml
jobs:
  setup:
    name: Setup
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Install Nix
        uses: cachix/install-nix-action@v24
        with:
          nix_path: nix
          extra_nix_config: |
            experimental-features = nix-command flakes

      - name: Setup Cachix
        uses: cachix/cachix-action@v12
        with:
          name: tachyon
          authToken: '${{ secrets.CACHIX_AUTH_TOKEN }}'

      - name: Cache Nix store
        uses: actions/cache@v4
        with:
          path: ~/.nix-store
          key: ${{ runner.os }}-nix-${{ hashFiles('**/*.nix') }}
          restore-keys: |
            ${{ runner.os }}-nix-

      - name: Cache Cargo registry
        uses: actions/cache@v4
        with:
          path: ~/.cargo/registry
          key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}

      - name: Cache Cargo build
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/git
            ~/.cargo/bin
            target/
          key: ${{ runner.os }}-cargo-build-${{ hashFiles('**/Cargo.lock') }}
          restore-keys: |
            ${{ runner.os }}-cargo-build-

      - name: Cache Bun packages
        uses: actions/cache@v4
        with:
          path: |
            ~/.bun/install/cache
            node_modules
          key: ${{ runner.os }}-bun-${{ hashFiles('web/bun.lock') }}
```

**Requirements:**
- REQ-BLD-001: Nix Flakes
- REQ-BLD-005: Caching Strategy

#### 3.1.4. Stage 2: Lint

**Purpose:** Run Clippy lints to catch code quality issues.

```yaml
  lint:
    name: Lint
    runs-on: ${{ matrix.os }}
    needs: setup
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Restore Nix store
        uses: actions/cache/restore@v4
        with:
          path: ~/.nix-store
          key: ${{ runner.os }}-nix-${{ hashFiles('**/*.nix') }}

      - name: Restore Cargo build
        uses: actions/cache/restore@v4
        with:
          path: |
            ~/.cargo/git
            ~/.cargo/bin
            target/
          key: ${{ runner.os }}-cargo-build-${{ hashFiles('**/Cargo.lock') }}

      - name: Run Clippy
        run: nix develop -c "cargo clippy --all-targets -- -D warnings"

      - name: Upload Clippy results
        if: failure()
        uses: actions/upload-artifact@v4
        with:
          name: clippy-results-${{ matrix.os }}
          path: target/
```

**Requirements:**
- REQ-BLD-050: Optimized Builds
- TACHYON-STD-V1.0: Coding Standards

#### 3.1.5. Stage 3: Format Check

**Purpose:** Verify code formatting with rustfmt.

```yaml
  format-check:
    name: Format Check
    runs-on: ubuntu-latest
    needs: setup
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Restore Nix store
        uses: actions/cache/restore@v4
        with:
          path: ~/.nix-store
          key: ubuntu-latest-nix-${{ hashFiles('**/*.nix') }}

      - name: Check formatting
        run: nix develop -c "cargo fmt -- --check"

      - name: Format diff
        if: failure()
        run: nix develop -c "cargo fmt -- -- --print-diff"
```

**Requirements:**
- TACHYON-STD-V1.0: Coding Standards

#### 3.1.6. Stage 4: Unit Tests

**Purpose:** Run unit test suite with coverage reporting.

```yaml
  unit-tests:
    name: Unit Tests
    runs-on: ${{ matrix.os }}
    needs: [lint, format-check]
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Restore Nix store
        uses: actions/cache/restore@v4
        with:
          path: ~/.nix-store
          key: ${{ runner.os }}-nix-${{ hashFiles('**/*.nix') }}

      - name: Restore Cargo build
        uses: actions/cache/restore@v4
        with:
          path: |
            ~/.cargo/git
            ~/.cargo/bin
            target/
          key: ${{ runner.os }}-cargo-build-${{ hashFiles('**/Cargo.lock') }}

      - name: Run unit tests
        run: |
          nix develop -c "cargo test --lib --bins --no-fail-fast -- -Z unstable-options --report=html"

      - name: Generate coverage report
        run: |
          nix develop -c "cargo llvm-cov --html --no-fail-fast --lib --bins"

      - name: Upload coverage to Codecov
        uses: codecov/codecov-action@v4
        with:
          files: ./lcov.info
          flags: unittests
          name: codecov-${{ matrix.os }}

      - name: Upload coverage artifacts
        uses: actions/upload-artifact@v4
        with:
          name: coverage-${{ matrix.os }}
          path: |
            target/llvm-cov/
            lcov.info
```

**Requirements:**
- REQ-BLD-067: Automated Testing
- REQ-SYS-052: Search Response Time

#### 3.1.7. Stage 5: Integration Tests

**Purpose:** Run integration test suite across all components.

```yaml
  integration-tests:
    name: Integration Tests
    runs-on: ubuntu-latest
    needs: unit-tests
    services:
      postgres:
        image: postgres:16
        env:
          POSTGRES_PASSWORD: postgres
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Restore Nix store
        uses: actions/cache/restore@v4
        with:
          path: ~/.nix-store
          key: ubuntu-latest-nix-${{ hashFiles('**/*.nix') }}

      - name: Restore Cargo build
        uses: actions/cache/restore@v4
        with:
          path: |
            ~/.cargo/git
            ~/.cargo/bin
            target/
          key: ubuntu-latest-cargo-build-${{ hashFiles('**/Cargo.lock') }}

      - name: Run integration tests
        run: |
          nix develop -c "cargo test --test-threads=1 --no-fail-fast"
        env:
          DATABASE_URL: postgresql://postgres:postgres@localhost:5432/tachyon_test

      - name: Upload test results
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: integration-test-results
          path: |
            target/test-results/
            target/nextest/
```

**Requirements:**
- REQ-BLD-067: Automated Testing
- REQ-SYS-054: Concurrent Users

#### 3.1.8. Stage 6: Security Scan

**Purpose:** Scan for security vulnerabilities in dependencies.

```yaml
  security-scan:
    name: Security Scan
    runs-on: ubuntu-latest
    needs: integration-tests
    continue-on-error: true
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Restore Nix store
        uses: actions/cache/restore@v4
        with:
          path: ~/.nix-store
          key: ubuntu-latest-nix-${{ hashFiles('**/*.nix') }}

      - name: Run cargo audit
        run: nix develop -c "cargo audit --json"
        continue-on-error: true

      - name: Run cargo deny
        run: nix develop -c "cargo deny --all-features"
        continue-on-error: true

      - name: Upload security report
        uses: actions/upload-artifact@v4
        with:
          name: security-report
          path: |
            advisory-db.json
            cargo-deny-report.json
```

**Requirements:**
- REQ-BLD-072: Secret Management
- REQ-SYS-074: Input Validation

#### 3.1.9. Stage 7: Build Artifacts

**Purpose:** Build release artifacts for all platforms.

```yaml
  build:
    name: Build
    runs-on: ${{ matrix.os }}
    needs: [unit-tests, integration-tests]
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
          - os: macos-latest
            target: x86_64-apple-darwin
          - os: windows-latest
            target: x86_64-pc-windows-msvc
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Restore Nix store
        uses: actions/cache/restore@v4
        with:
          path: ~/.nix-store
          key: ${{ runner.os }}-nix-${{ hashFiles('**/*.nix') }}

      - name: Restore Cargo build
        uses: actions/cache/restore@v4
        with:
          path: |
            ~/.cargo/git
            ~/.cargo/bin
            target/
          key: ${{ runner.os }}-cargo-build-${{ hashFiles('**/Cargo.lock') }}

      - name: Build release
        run: nix build .#tachyon-desktop

      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: tachyon-${{ matrix.os }}
          path: result/bin/
```

**Requirements:**
- REQ-BLD-046: Desktop Binary
- REQ-BLD-047: Server Binary
- REQ-BLD-051: Leptos Bundle

#### 3.1.10. Stage 8: Upload Artifacts

**Purpose:** Store build artifacts for deployment and release.

```yaml
  upload:
    name: Upload Artifacts
    runs-on: ubuntu-latest
    needs: build
    steps:
      - name: Download all artifacts
        uses: actions/download-artifact@v4

      - name: Upload to release
        if: startsWith(github.ref, 'refs/tags/')
        uses: softprops/action-gh-release@v1
        with:
          files: |
            tachyon-ubuntu-latest/*
            tachyon-macos-latest/*
            tachyon-windows-latest/*
          generate_release_notes: true
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

**Requirements:**
- REQ-BLD-068: Automated Release
- REQ-BLD-086: GitHub Releases

### 3.2. Release Workflow

The release workflow ([`.github/workflows/release.yml`](../../.github/workflows/release.yml)) executes on tag creation, building and publishing release artifacts.

#### 3.2.1. Workflow Triggers

```yaml
name: Release

on:
  push:
    tags:
      - 'v*.*'
```

**Tag Format:** Semantic versioning (MAJOR.MINOR.PATCH)

**Requirements:**
- REQ-BLD-076: Semantic Versioning
- REQ-BLD-077: Git Tags

#### 3.2.2. Release Stages

```mermaid
graph LR
    A[Tag Push] --> B[Validate Tag]
    B --> C[Build All Platforms]
    C --> D[Run Full Test Suite]
    D --> E[Security Audit]
    E --> F[Sign Artifacts]
    F --> G[Create GitHub Release]
    G --> H[Publish to Registry]
    H --> I[Deploy to Staging]
    
    style A fill:#e1f5fe
    style B fill:#f39c12
    style C fill:#4caf50
    style D fill:#ff9800
    style E fill:#ff9800
    style F fill:#795548
    style G fill:#795548
    style H fill:#795548
    style I fill:#795548
```

**Stage Descriptions:**

| Stage | Purpose | Duration | Failure Impact |
|-------|---------|----------|---------------|
| **Validate Tag** | Verify tag format and version | 30 sec | Blocks entire release |
| **Build All Platforms** | Build release binaries for all platforms | 10-15 min | Blocks subsequent stages |
| **Run Full Test Suite** | Execute comprehensive test suite | 15-20 min | Blocks subsequent stages |
| **Security Audit** | Full security scan and audit | 5-8 min | Non-blocking but logged |
| **Sign Artifacts** | Code sign all release artifacts | 2-3 min | Blocks release publishing |
| **Create GitHub Release** | Publish to GitHub Releases | 1-2 min | Final stage |
| **Publish to Registry** | Publish to package registries | 3-5 min | Optional stage |
| **Deploy to Staging** | Deploy to staging environment | 5-8 min | Optional stage |

#### 3.2.3. Release Build Matrix

```yaml
jobs:
  release:
    name: Release
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        target:
          - x86_64-unknown-linux-gnu
          - x86_64-apple-darwin
          - x86_64-pc-windows-msvc
        include:
          - os: ubuntu-latest
            target: aarch64-unknown-linux-gnu
          - os: macos-latest
            target: aarch64-apple-darwin
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Install Nix
        uses: cachix/install-nix-action@v24
        with:
          nix_path: nix

      - name: Setup Cachix
        uses: cachix/cachix-action@v12
        with:
          name: tachyon
          authToken: '${{ secrets.CACHIX_AUTH_TOKEN }}'

      - name: Build release
        run: nix build .#tachyon-desktop

      - name: Sign binary
        run: |
          gpg --default-key ${{ secrets.CODE_SIGNING_KEY_ID }} \
              --detach-sign \
              --armor \
              --yes \
              result/bin/tachyon-desktop

      - name: Upload to release
        uses: softprops/action-gh-release@v1
        with:
          files: result/bin/*
          generate_release_notes: true
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

**Requirements:**
- REQ-BLD-034: Code Signing
- REQ-BLD-039: Code Signing
- REQ-BLD-044: Code Signing

### 3.3. Nightly Workflow

The nightly workflow ([`.github/workflows/nightly.yml`](../../.github/workflows/nightly.yml)) executes on a schedule, running long-running tests and benchmarks.

#### 3.3.1. Schedule

```yaml
name: Nightly

on:
  schedule:
    - cron: '0 0 * * *'  # Daily at midnight UTC
  workflow_dispatch:
```

**Purpose:**
- Run long-running tests that are too slow for main CI
- Execute performance benchmarks
- Update dependency analysis
- Generate nightly build artifacts

**Requirements:**
- REQ-BLD-067: Automated Testing
- REQ-BLD-083: Release Testing

#### 3.3.2. Nightly Stages

```mermaid
graph LR
    A[Schedule] --> B[Checkout]
    B --> C[Run Long Tests]
    C --> D[Performance Benchmarks]
    D --> E[Dependency Analysis]
    E --> F[Generate Nightly Build]
    F --> G[Upload Artifacts]
    G --> H[Notify Status]
    
    style A fill:#e1f5fe
    style B fill:#f39c12
    style C fill:#4caf50
    style D fill:#ff9800
    style E fill:#795548
    style F fill:#795548
    style G fill:#795548
    style H fill:#795548
```

**Stage Descriptions:**

| Stage | Purpose | Duration | Failure Impact |
|-------|---------|----------|---------------|
| **Checkout** | Get latest code | 30 sec | Blocks entire workflow |
| **Run Long Tests** | Execute slow integration tests | 30-60 min | Non-blocking but logged |
| **Performance Benchmarks** | Run performance tests | 10-20 min | Non-blocking but logged |
| **Dependency Analysis** | Check for outdated dependencies | 5-10 min | Non-blocking but logged |
| **Generate Nightly Build** | Build latest artifacts | 10-15 min | Blocks artifact upload |
| **Upload Artifacts** | Store nightly artifacts | 2-3 min | Final stage |
| **Notify Status** | Send status notifications | 30 sec | Final stage |

### 3.4. Security Workflow

The security workflow ([`.github/workflows/security.yml`](../../.github/workflows/security.yml)) executes on every push and weekly, performing comprehensive security scanning.

#### 3.4.1. Schedule

```yaml
name: Security

on:
  push:
    branches: [main, develop]
  schedule:
    - cron: '0 0 * * 0'  # Weekly on Sunday at midnight UTC
  workflow_dispatch:
```

**Purpose:**
- Scan dependencies for vulnerabilities
- Check for supply chain attacks
- Validate code signing
- Generate security reports

**Requirements:**
- REQ-BLD-072: Secret Management
- REQ-SYS-074: Input Validation

#### 3.4.2. Security Stages

```mermaid
graph LR
    A[Trigger] --> B[Dependency Scan]
    B --> C[Code Analysis]
    C --> D[Secret Scan]
    D --> E[License Check]
    E --> F[Generate Report]
    F --> G[Upload Report]
    G --> H[Notify Team]
    
    style A fill:#e1f5fe
    style B fill:#ff9800
    style C fill:#ff9800
    style D fill:#ff9800
    style E fill:#ff9800
    style F fill:#795548
    style G fill:#795548
    style H fill:#795548
```

**Stage Descriptions:**

| Stage | Purpose | Duration | Failure Impact |
|-------|---------|----------|---------------|
| **Dependency Scan** | Scan for known vulnerabilities | 2-5 min | Non-blocking but logged |
| **Code Analysis** | Static analysis for security issues | 3-8 min | Non-blocking but logged |
| **Secret Scan** | Check for leaked secrets | 1-2 min | Blocking if secrets found |
| **License Check** | Verify license compliance | 1-2 min | Non-blocking but logged |
| **Generate Report** | Compile security findings | 30 sec | Blocks report upload |
| **Upload Report** | Store security artifacts | 1-2 min | Final stage |
| **Notify Team** | Send security alerts | 30 sec | Final stage |

---

## 4. BUILD PROCESS

### 4.1. Nix-Based Build System

The Tachyon build system uses Nix flakes for reproducible, hermetic, and cross-platform builds. All build artifacts are produced through Nix expressions, ensuring consistency across different environments and over time.

#### 4.1.1. Flake Structure

The Nix flake ([`flake.nix`](../../flake.nix)) defines all build targets and configurations:

**Structure:**
```nix
{
  description = "Tachyon - High-performance document management toolchain";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    crane.url = "github:ipetkov/crane";
    fenix.url = "github:nix-community/fenix";
    devshell.url = "github:numtide/devshell";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, crane, fenix, devshell, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" "clippy" ];
        };

        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        workspaceSrc = craneLib.cleanCargoSource ./.;

        commonArgs = {
          src = workspaceSrc;
          strictDeps = true;
          buildInputs = with pkgs; [
            pkg-config
            openssl
            sqlite
          ];
          nativeBuildInputs = with pkgs; [
            rustToolchain
            pkg-config
            cmake
          ];
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        tachyon-desktop = craneLib.buildPackage (commonArgs // {
          pname = "tachyon-desktop";
          version = "0.1.0";
          cargoArtifacts = craneLib.buildDepsOnly (commonArgs // {
            pname = "tachyon-desktop-deps";
          });
          doCheck = false;
        });

        tachyon-server = craneLib.buildPackage (commonArgs // {
          pname = "tachyon-server";
          version = "0.1.0";
          cargoArtifacts = craneLib.buildDepsOnly (commonArgs // {
            pname = "tachyon-server-deps";
          });
          doCheck = false;
        });

        tachyon-web = pkgs.mkYarnPackage {
          pname = "tachyon-web";
          version = "0.1.0";
          src = ./web;
          packageJSON = ./web/package.json;
          yarnLock = ./web/yarn.lock;
          yarnNix = ./web/yarn.nix;
          buildPhase = ''
            yarn build
          '';
          installPhase = ''
            mkdir -p $out
            cp -r dist/* $out/
          '';
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustToolchain
            cargo-watch
            cargo-edit
            rust-analyzer
            pkg-config
            openssl
            sqlite
            nodejs
            yarn
            bun
            devshell
          ];

          shellHook = ''
            export RUST_SRC_PATH="${pkgs.rustToolchain}/lib/rustlib/src/rust/library"
            export PATH="$PWD/target/debug:$PATH"
          '';
        };

      in {
        packages = {
          inherit tachyon-desktop tachyon-server tachyon-web;
          default = tachyon-desktop;
        };

        apps = {
          desktop = flake-utils.lib.mkApp {
            drv = tachyon-desktop;
            exePath = "/bin/tachyon-desktop";
          };
          server = flake-utils.lib.mkApp {
            drv = tachyon-server;
            exePath = "/bin/tachyon-server";
          };
        };

        devShells = devShells;

        checks = {
          inherit tachyon-desktop tachyon-server;
        };
      }
    );
}
```

**Requirements:**
- REQ-BLD-001: Nix Flakes
- REQ-BLD-002: Declarative Configuration
- REQ-BLD-003: Pure Evaluation
- REQ-BLD-004: Deterministic Builds

#### 4.1.2. Build Targets

The Nix flake provides the following build targets:

| Target | Description | Command | Output |
|--------|-------------|---------|--------|
| **tachyon-desktop** | Desktop application binary | `result/bin/tachyon-desktop` |
| **tachyon-server** | Server application binary | `result/bin/tachyon-server` |
| **tachyon-web** | Web frontend bundle | `result/dist/` |
| **default** | Default package (desktop) | `result/bin/tachyon-desktop` |

**Build Commands:**
```bash
# Build desktop application
nix build .#tachyon-desktop

# Build server application
nix build .#tachyon-server

# Build web frontend
nix build .#tachyon-web

# Build all packages
nix build

# Run checks (build + test)
nix flake check
```

**Requirements:**
- REQ-BLD-006: Desktop Target
- REQ-BLD-007: Server Target
- REQ-BLD-008: Web Target
- REQ-BLD-009: All Target
- REQ-BLD-010: Test Target

### 4.2. Cargo Configuration

The Cargo workspace ([`tachyon/Cargo.toml`](../../tachyon/Cargo.toml)) defines workspace configuration and dependencies:

**Workspace Configuration:**
```toml
[workspace]
members = [
    "crates/desktop",
    "crates/server",
    "crates/shared",
]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2024"
rust-version = "1.82"
authors = ["Tachyon Team"]
license = "MIT OR Apache-2.0"
repository = "https://github.com/tachyon/tachyon"
```

**Requirements:**
- REQ-BLD-011: Flake Definition
- REQ-BLD-012: Input Specification
- REQ-BLD-013: Output Specification
- REQ-BLD-014: Dependency Locking

### 4.3. Release Profiles

The Cargo configuration defines optimized build profiles for different use cases:

**Release Profile:**
```toml
[profile.release]
opt-level = 3           # Maximum optimization
lto = "fat"            # Link-time optimization
codegen-units = 1      # Single codegen unit for better optimization
panic = "abort"        # Abort on panic for smaller binaries
strip = true           # Strip symbols from binary
debug = false          # No debug info
```

**Requirements:**
- REQ-BLD-049: Strip Symbols
- REQ-BLD-050: Optimized Builds

### 4.4. Cross-Platform Builds

The CI system builds for all supported platforms using a matrix strategy:

**Platform Matrix:**
```yaml
strategy:
  matrix:
    os: [ubuntu-latest, macos-latest, windows-latest]
    target:
      - x86_64-unknown-linux-gnu
      - x86_64-apple-darwin
      - x86_64-pc-windows-msvc
    include:
      - os: ubuntu-latest
        target: aarch64-unknown-linux-gnu
      - os: macos-latest
        target: aarch64-apple-darwin
```

**Platform-Specific Build Steps:**

##### 4.4.1. Linux Build

**Toolchain:** GCC 14+ or Clang 18+

**Build Steps:**
```yaml
- name: Install build dependencies
  run: |
    sudo apt-get update
    sudo apt-get install -y \
      build-essential \
      pkg-config \
      libssl-dev \
      libsqlite3-dev

- name: Build for Linux
  run: |
    cargo build --release --target ${{ matrix.target }}
```

**Requirements:**
- REQ-BLD-028: Linux Support
- REQ-BLD-041: GCC/Clang Toolchain
- REQ-BLD-042: glibc Compatibility

##### 4.4.2. macOS Build

**Toolchain:** Clang 18+ with Xcode Command Line Tools

**Build Steps:**
```yaml
- name: Install build dependencies
  run: |
    brew install pkg-config openssl sqlite3

- name: Build for macOS
  run: |
    cargo build --release --target ${{ matrix.target }}
```

**Requirements:**
- REQ-BLD-027: macOS Support
- REQ-BLD-036: Clang Toolchain
- REQ-BLD-037: macOS SDK

##### 4.4.3. Windows Build

**Toolchain:** MSVC 2022 with Visual Studio Build Tools

**Build Steps:**
```yaml
- name: Setup MSVC
  uses: ilammy/msvc-dev-cmd@v1
  with:
    arch: ${{ matrix.arch }}

- name: Build for Windows
  run: |
    cargo build --release --target ${{ matrix.target }}
```

**Requirements:**
- REQ-BLD-026: Windows Support
- REQ-BLD-031: MSVC Toolchain
- REQ-BLD-032: Windows SDK

### 4.5. Build Artifacts

The CI system produces the following build artifacts:

#### 4.5.1. Rust Binaries

**Desktop Binary:**
- **Path:** `result/bin/tachyon-desktop`
- **Format:** Executable binary
- **Signing:** Code signed with GPG
- **Stripping:** Debug symbols removed
- **Compression:** Optional UPX compression

**Server Binary:**
- **Path:** `result/bin/tachyon-server`
- **Format:** Executable binary
- **Signing:** Code signed with GPG
- **Stripping:** Debug symbols removed
- **Compression:** Optional UPX compression

**Requirements:**
- REQ-BLD-046: Desktop Binary
- REQ-BLD-047: Server Binary
- REQ-BLD-049: Strip Symbols
- REQ-BLD-050: Optimized Builds

#### 4.5.2. Web Frontend Bundle

**Leptos Bundle:**
- **Path:** `result/dist/`
- **Format:** WASM bundle with HTML, CSS, JavaScript
- **Optimization:** Minified and tree-shaken
- **Source Maps:** Generated for debugging

**Requirements:**
- REQ-BLD-051: Leptos Bundle
- REQ-BLD-052: CSS Bundle
- REQ-BLD-053: JavaScript Bundle
- REQ-BLD-054: Asset Optimization
- REQ-BLD-055: Source Maps

#### 4.5.3. Platform Packages

**Linux Packages:**
- **DEB:** Debian package for Debian/Ubuntu
- **RPM:** RPM package for Fedora/RHEL
- **AppImage:** Self-contained AppImage

**macOS Packages:**
- **DMG:** Disk image for macOS
- **APP:** Application bundle

**Windows Packages:**
- **MSI:** Windows Installer
- **NSIS:** Nullsoft Scriptable Install System

**Requirements:**
- REQ-BLD-043: Package Generation
- REQ-BLD-033: Installer Generation
- REQ-BLD-038: Bundle Generation

### 4.6. Build Caching

The CI system implements multi-layer caching for optimal build performance:

#### 4.6.1. Nix Store Cache

**Purpose:** Cache Nix store for reproducible builds.

**Configuration:**
```yaml
- name: Cache Nix store
  uses: actions/cache@v4
  with:
    path: ~/.nix-store
    key: ${{ runner.os }}-nix-${{ hashFiles('**/*.nix') }}
    restore-keys: |
      ${{ runner.os }}-nix-
```

**Benefits:**
- Reproducible builds across CI runs
- Reduced dependency resolution time
- Consistent build environments

**Requirements:**
- REQ-BLD-001: Nix Flakes
- REQ-BLD-005: Caching Strategy

#### 4.6.2. Cargo Registry Cache

**Purpose:** Cache Cargo registry for Rust dependencies.

**Configuration:**
```yaml
- name: Cache Cargo registry
  uses: actions/cache@v4
  with:
    path: ~/.cargo/registry
    key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}
```

**Benefits:**
- Faster dependency downloads
- Reduced network bandwidth
- Consistent dependency versions

**Requirements:**
- REQ-BLD-014: Dependency Locking
- REQ-BLD-005: Caching Strategy

#### 4.6.3. Cargo Build Cache

**Purpose:** Cache Cargo build artifacts for incremental compilation.

**Configuration:**
```yaml
- name: Cache Cargo build
  uses: actions/cache@v4
  with:
    path: |
      ~/.cargo/git
      ~/.cargo/bin
      target/
    key: ${{ runner.os }}-cargo-build-${{ hashFiles('**/Cargo.lock') }}
    restore-keys: |
      ${{ runner.os }}-cargo-build-
```

**Benefits:**
- Incremental compilation across CI runs
- Faster rebuild times
- Reduced compilation overhead

**Requirements:**
- REQ-BLD-005: Caching Strategy
- REQ-BLD-050: Optimized Builds

#### 4.6.4. Bun Cache

**Purpose:** Cache Bun packages for web frontend.

**Configuration:**
```yaml
- name: Cache Bun packages
  uses: actions/cache@v4
  with:
    path: |
      ~/.bun/install/cache
      node_modules
    key: ${{ runner.os }}-bun-${{ hashFiles('web/bun.lock') }}
```

**Benefits:**
- Faster dependency installation
- Reduced network requests
- Consistent dependency versions

**Requirements:**
- REQ-BLD-005: Caching Strategy

### 4.7. Build Optimization

The CI system implements build optimizations for faster feedback:

#### 4.7.1. Parallel Execution

**Purpose:** Execute independent build steps in parallel.

**Configuration:**
```yaml
jobs:
  build-linux:
    name: Build Linux
    runs-on: ubuntu-latest

  build-macos:
    name: Build macOS
    runs-on: macos-latest

  build-windows:
    name: Build Windows
    runs-on: windows-latest
```

**Benefits:**
- Reduced total build time
- Faster feedback to developers
- Better resource utilization

**Requirements:**
- REQ-BLD-005: Caching Strategy

#### 4.7.2. Incremental Builds

**Purpose:** Build only changed components.

**Configuration:**
```yaml
- name: Detect changes
  id: changes
  run: |
    git diff --name-only HEAD~1 HEAD | grep -E '^tachyon/' | cut -d -f1 | sort -u > changed.txt

- name: Build changed components
  run: |
    for component in $(cat changed.txt); do
      nix build .#$component
    done
```

**Benefits:**
- Faster builds for small changes
- Reduced CI resource usage
- Faster feedback cycle

**Requirements:**
- REQ-BLD-005: Caching Strategy

#### 4.7.3. Build Matrix Optimization

**Purpose:** Optimize matrix for faster feedback.

**Configuration:**
```yaml
strategy:
  matrix:
    os: [ubuntu-latest]
    rust: [stable]
    fail-fast: false
  max-parallel: 4
```

**Benefits:**
- Faster initial feedback
- Better resource utilization
- Reduced queue time

**Requirements:**
- REQ-BLD-005: Caching Strategy
