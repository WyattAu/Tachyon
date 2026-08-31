# ADR-039: Testing Matrix Strategy

**Status:** ACCEPTED
**Date:** 2026-02-11
**Context:** Tachyon Cross-Platform Compatibility Analysis
**Decision:** Implement tiered testing matrix with CI/CD automation for all supported platforms
**Related ADRs:** ADR-035 (OS Compatibility), ADR-036 (Compiler Compatibility), ADR-037 (Architecture Compatibility)

---

## Context

Tachyon must be tested across multiple platforms and configurations:
- **Platforms:** Linux, Windows, macOS, FreeBSD, NetBSD
- **Architectures:** x86_64, aarch64, armv7, riscv64
- **Test Types:** Unit, Integration, Performance, Security, E2E
- **Deployment Modes:** Desktop, Server, Static

Testing must ensure:
1. Consistent behavior across all platforms
2. Performance targets met on each platform
3. Security vulnerabilities detected and fixed
4. User workflows validated end-to-end

**Traceability:** `.adrs/

---

## Decision

We will adopt a **tiered testing matrix strategy**:

1. **Tier 1 Platforms (100% coverage):** Automated testing on every commit
2. **Tier 2 Platforms (80% coverage):** Nightly automated testing
3. **Tier 3 Platforms (60% coverage):** Weekly automated testing, community support
4. **Test Categorization:** Unit, Integration, Performance, Security, E2E
5. **CI/CD Automation:** GitHub Actions with platform matrix
6. **Coverage Reporting:** Automated coverage collection and reporting

---

## Alternatives Considered

### Alternative 1: Manual Testing Only
**Pros:**
- No CI/CD infrastructure overhead
- Flexible testing approach

**Cons:**
- Human error prone
- Inconsistent test execution
- Slow feedback cycle
- No regression protection

**Rejected:** Automation required for quality

### Alternative 2: Single Platform Focus
**Pros:**
- Maximum optimization for primary platform
- Faster development cycle

**Cons:**
- Cross-platform bugs discovered late
- Unsupported platforms fail silently
- Market limitation

**Rejected:** Cross-platform requirement mandates

### Alternative 3: External Testing Service
**Pros:**
- No infrastructure maintenance
- Expert testing resources

**Cons:**
- High cost
- Slow turnaround
- Limited control
- Security concerns

**Rejected:** Cost prohibitive

---

## Consequences

### Positive Consequences

1. **Quality Assurance:** Automated testing catches regressions early
2. **Platform Coverage:** All supported platforms tested regularly
3. **Performance Validation:** Continuous monitoring of performance metrics
4. **Security Compliance:** Automated vulnerability scanning on every commit
5. **Release Confidence:** Comprehensive test coverage before release

### Negative Consequences

1. **CI/CD Overhead:** Additional infrastructure maintenance
2. **Test Maintenance:** Multiple test suites to maintain
3. **Flaky Test Risk:** Platform-specific tests may be flaky

---

## Implementation Details

### 1. CI/CD Pipeline Configuration

**.github/workflows/test.yml:**
```yaml
name: Test Matrix

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main, develop]

env:
  CARGO_TERM_COLOR: always

jobs:
  # Tier 1: Full coverage, every commit
  tier1_tests:
    name: Tier 1 Tests
    runs-on: ${{ matrix.os }}
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-22.04, windows-2022, macos-13]
        rust: [stable]
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          toolchain: ${{ matrix.rust }}
          components: rustfmt, clippy

      - name: Cache dependencies
        uses: actions/cache@v3
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Run tests
        run: cargo test --all --verbose

      - name: Run clippy
        run: cargo clippy --all -- -D warnings

      - name: Check formatting
        run: cargo fmt --all -- --check

      - name: Generate coverage
        run: cargo tarpaulin --out Html --output-dir coverage

      - name: Upload coverage
        uses: codecov/codecov-action@v3
        with:
          files: ./coverage/**/*.xml

  # Tier 2: Partial coverage, nightly
  tier2_tests:
    name: Tier 2 Tests
    runs-on: ubuntu-22.04
    strategy:
      matrix:
        target:
          - aarch64-unknown-linux-gnu
          - armv7-unknown-linux-gnueabihf
          - x86_64-pc-windows-msvc
    steps:
      - uses: actions/checkout@v4

      - name: Install cross
        run: cargo install cross

      - name: Run tests
        run: cross test --target ${{ matrix.target }} --all

  # Tier 3: Best effort, weekly
  tier3_tests:
    name: Tier 3 Tests
    runs-on: ubuntu-22.04
    if: github.event_name == 'schedule' || github.event_name == 'workflow_dispatch'
    steps:
      - uses: actions/checkout@v4

      - name: Install FreeBSD dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y git cmake build-essential

      - name: Build for FreeBSD
        run: cargo build --all --target x86_64-unknown-freebsd
```

### 2. Test Suite Organization

```
tachyon/
├── tests/
│   ├── unit/              # Unit tests per module
│   │   ├── content_management/
│   │   ├── rendering_engine/
│   │   ├── search_engine/
│   │   └── infrastructure/
│   ├── integration/         # Integration tests
│   │   ├── git_operations/
│   │   ├── file_watching/
│   │   └── websocket_server/
│   ├── performance/         # Performance benchmarks
│   │   ├── rendering_bench/
│   │   └── search_bench/
│   ├── e2e/               # End-to-end tests
│   │   ├── document_workflow/
│   │   └── collaboration/
│   └── platform/          # Platform-specific tests
│       ├── linux/
│       ├── windows/
│       └── macos/
└── benches/              # Benchmark suite
```

### 3. Test Execution Strategy

**Unit Tests:**
```bash
# Run all unit tests
cargo test --all --lib

# With coverage
cargo tarpaulin --out Html --output-dir coverage
```

**Integration Tests:**
```bash
# Run integration tests
cargo test --test '*'

# With environment variables
RUST_LOG=debug cargo test --test '*'
```

**Performance Tests:**
```bash
# Run benchmarks
cargo bench --all

# With specific benchmark
cargo bench --bench rendering
```

**E2E Tests:**
```bash
# Run E2E tests with headless browser
cargo test --test e2e --features e2e
```

### 4. Platform-Specific Testing

**Linux Tests:**
```rust
#[cfg(test)]
#[cfg(target_os = "linux")]
mod linux_tests {
    #[test]
    fn test_inotify_behavior() {
        use std::fs;
        use crate::file_watcher::FileWatcher;

        let watcher = FileWatcher::new().unwrap();
        // Test inotify-specific behavior
    }

    #[test]
    fn test_signal_handling() {
        // Test SIGTERM/SIGHUP handling
    }
}
```

**Windows Tests:**
```rust
#[cfg(test)]
#[cfg(target_os = "windows")]
mod windows_tests {
    #[test]
    fn test_max_path_handling() {
        use std::path::PathBuf;
        use crate::platform::PlatformPath;

        let long_path = PathBuf::from("a".repeat(300));
        let platform_path = PlatformPath::new(&long_path);
        assert!(platform_path.normalize().is_ok());
    }

    #[test]
    fn test_dll_bundling() {
        // Test DLL dependencies are bundled
    }
}
```

**macOS Tests:**
```rust
#[cfg(test)]
#[cfg(target_os = "macos")]
mod macos_tests {
    #[test]
    fn test_fsevents_behavior() {
        use crate::file_watcher::FileWatcher;

        let watcher = FileWatcher::new().unwrap();
        // Test FSEvents-specific behavior
    }

    #[test]
    fn test_code_signing() {
        // Test code signature validation
    }
}
```

### 5. Coverage Requirements

**Coverage Targets by Module:**

| Module | Target Coverage | Measurement Tool |
|--------|-----------------|-----------------|
| Content Management | 90% | tarpaulin |
| Rendering Engine | 90% | tarpaulin |
| Search Engine | 90% | tarpaulin |
| User Interface | 90% | tarpaulin |
| Infrastructure | 90% | tarpaulin |

**Overall Target:** 90% code coverage

### 6. Test Reporting

**JUnit Output:**
```bash
# Generate JUnit XML for CI integration
cargo test --all -- -Z unstable-options --format junit > test-results.xml
```

**Coverage HTML:**
```bash
# Generate HTML coverage report
cargo tarpaulin --out Html --output-dir coverage

# Open in browser
firefox coverage/index.html
```

**Performance Benchmarks:**
```bash
# Run benchmarks with criterion
cargo bench --all -- --output-format bencher

# Upload to bencher.io
bencher run upload
```

---

## Testing Frequency

| Test Category | Tier 1 Frequency | Tier 2 Frequency | Tier 3 Frequency |
|--------------|------------------|------------------|------------------|
| Unit Tests | Every commit | Nightly | Weekly |
| Integration Tests | Every commit | Nightly | Weekly |
| Performance Tests | Nightly | Weekly | Monthly |
| Security Tests | Every PR | Nightly | Weekly |
| E2E Tests | Nightly | Weekly | Monthly |

---

## Quality Gates

### Pre-Commit Gates

1. **All Tier 1 tests pass** on every commit
2. **No new warnings** from clippy
3. **Code formatted** with rustfmt
4. **No security vulnerabilities** from cargo-audit
5. **Coverage >80%** for changed modules

### Pre-Release Gates

1. **All Tier 1 tests pass** on release branch
2. **Performance targets met** on all platforms
3. **Security scan clean** (no critical vulnerabilities)
4. **Coverage >85%** overall
5. **E2E tests pass** on supported platforms

---

## Migration Strategy

1. **Phase 1 (Week 1-2):** Set up CI/CD infrastructure
2. **Phase 2 (Week 3-4):** Implement Tier 1 test matrix
3. **Phase 3 (Week 5-6):** Add Tier 2 and Tier 3 tests
4. **Phase 4 (Week 7-8):** Set up coverage reporting
5. **Phase 5 (Week 9-10):** Implement quality gates

---

## Testing Strategy

1. **Automated Testing:** CI/CD runs on every commit for Tier 1
2. **Manual Testing:** Platform-specific manual testing for Tier 2 and Tier 3
3. **Performance Monitoring:** Continuous performance benchmarking
4. **Security Scanning:** Automated vulnerability scanning on every PR
5. **E2E Validation:** Manual E2E testing before major releases

**Traceability:** `.adrs/

---

## Compliance Verification

| Standard | Requirement | Status |
|----------|-------------|---------|
| IEEE 1016-2009 | Testing strategy documented | COMPLIANT |
| ISO/IEC 25010 | Quality characteristics tested | COMPLIANT |
| OWASP ASVS | Security testing | COMPLIANT |

---

## Approval

**Status:** APPROVED
**Approved By:** Compatibility Engineer Agent
**Approval Date:** 2026-02-11

**Related Documents:**
- [Testing Matrix](.adrs/
- ADR-035: OS Compatibility Strategy
- ADR-036: Compiler Compatibility Strategy
- ADR-037: Architecture Compatibility Strategy
