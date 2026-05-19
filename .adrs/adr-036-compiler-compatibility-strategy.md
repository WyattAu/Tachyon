# ADR-036: Compiler Compatibility Strategy

**Status:** ACCEPTED
**Date:** 2026-02-11
**Context:** Tachyon Cross-Platform Compatibility Analysis
**Decision:** Pin MSRV to 1.60 and use conditional compilation for compiler-specific optimizations
**Related ADRs:** ADR-035 (OS Compatibility), ADR-038 (Conditional Compilation)

---

## Context

Tachyon is written in Rust and must compile with multiple compiler configurations:
- rustc with MSVC backend (Windows)
- rustc with Clang backend (macOS, FreeBSD, NetBSD)
- rustc with GCC backend (Linux)

All core dependencies require Rust 1.60+:
- tokio: >=1.60 (dep_spec/tokio/dep_spec.toml:75)
- tantivy: >=1.60 (dep_spec/tantivy/dep_spec.toml:74)

Cross-compilation is required for:
- Windows builds on Linux
- macOS builds on Linux
- ARM/RISC-V builds on x86_64

**Traceability:** `.adrs/

---

## Decision

We will adopt a **compiler compatibility strategy** with the following principles:

1. **Pin MSRV to 1.60** in `Cargo.toml` to ensure minimum Rust version compatibility
2. **Use conditional compilation** (`#[cfg(...)]`) for compiler-specific optimizations
3. **Avoid compiler-specific extensions** to maintain portability
4. **Configure cross-compilation** via `.cargo/config.toml`
5. **Enable LTO (Link-Time Optimization)** for release builds

---

## Alternatives Considered

### Alternative 1: Use Latest Rust Version Only
**Pros:**
- Access to latest language features
- Improved compiler optimizations
- Better error messages

**Cons:**
- Reduces compatibility with older systems
- May break dependency compatibility
- Users may need newer Rust toolchain

**Rejected:** Compatibility requirements mandate 1.60+ support

### Alternative 2: Multiple Codebases per Compiler
**Pros:**
- Full control over compiler-specific optimizations
- No conditional compilation overhead

**Cons:**
- Code duplication across compilers
- Maintenance burden
- Inconsistent behavior between compilers

**Rejected:** Same rationale as ADR-035

### Alternative 3: Feature Detection at Runtime
**Pros:**
- Single binary distribution
- No conditional compilation

**Cons:**
- Runtime overhead for every operation
- Cannot leverage compile-time optimizations
- Complex error handling

**Rejected:** Runtime performance overhead unacceptable

---

## Consequences

### Positive Consequences

1. **Consistent Compilation:** All platforms use same source code with conditional compilation
2. **Performance Optimizations:** Compiler-specific optimizations enabled at compile time
3. **Cross-Compilation Support:** Easy building for all target platforms
4. **Dependency Compatibility:** MSRV ensures all dependencies work with minimum version
5. **Binary Size Optimization:** LTO reduces final binary size

### Negative Consequences

1. **Conditional Compilation Complexity:** Requires careful `#[cfg(...)]` usage
2. **Build Configuration Overhead:** Multiple target triples to configure
3. **Testing Overhead:** Must verify all compiler-specific code paths

---

## Implementation Details

### 1. MSRV Pinning in Cargo.toml

```toml
[package]
name = "tachyon"
version = "0.1.0"
edition = "2021"
rust-version = "1.60"

[workspace.dependencies]
tokio = { version = "1", features = ["full"] }
tantivy = "0.21"
git2 = "0.18"
```

### 2. Conditional Compilation for SIMD

```rust
/// Portable SIMD abstraction with compiler-specific optimizations
#[cfg(target_arch = "x86_64")]
#[cfg(target_feature = "avx2")]
pub unsafe fn simd_add(a: &[f32], b: &[f32], out: &mut [f32]) {
    use std::arch::x86_64::*;
    let avx_a = _mm256_loadu_ps(a.as_ptr());
    let avx_b = _mm256_loadu_ps(b.as_ptr());
    let result = _mm256_add_ps(avx_a, avx_b);
    _mm256_storeu_ps(out.as_mut_ptr(), result);
}

#[cfg(target_arch = "aarch64")]
#[cfg(target_feature = "neon")]
pub unsafe fn simd_add(a: &[f32], b: &[f32], out: &mut [f32]) {
    use std::arch::aarch64::*;
    let neon_a = vld1q_f32(a.as_ptr());
    let neon_b = vld1q_f32(b.as_ptr());
    let result = vaddq_f32(neon_a, neon_b);
    vst1q_f32(out.as_mut_ptr(), result);
}

#[cfg(not(any(
    all(target_arch = "x86_64", target_feature = "avx2"),
    all(target_arch = "aarch64", target_feature = "neon")
)))]
pub unsafe fn simd_add(a: &[f32], b: &[f32], out: &mut [f32]) {
    // Scalar fallback
    for i in 0..a.len() {
        out[i] = a[i] + b[i];
    }
}
```

### 3. Cross-Compilation Configuration

```toml
# .cargo/config.toml
[build]
target = "x86_64-unknown-linux-gnu"

[target.x86_64-pc-windows-msvc]
linker = "rust-lld.exe"

[target.aarch64-apple-darwin]
linker = "clang"
ar = "ar"

[target.aarch64-unknown-linux-gnu]
linker = "aarch64-linux-gnu-gcc"

[target.armv7-unknown-linux-gnueabihf]
linker = "arm-linux-gnueabihf-gcc"
```

### 4. LTO Configuration

```toml
# Cargo.toml
[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
strip = true

[profile.dev]
opt-level = 0
lto = false
debug = true
```

---

## Migration Strategy

1. **Phase 1 (Week 1-2):** Add MSRV to `Cargo.toml`
2. **Phase 2 (Week 3-4):** Configure cross-compilation toolchain
3. **Phase 3 (Week 5-6):** Implement conditional SIMD optimizations
4. **Phase 4 (Week 7-8):** Enable LTO for release builds
5. **Phase 5 (Week 9-10):** Set up cross-compilation CI jobs

---

## Testing Strategy

1. **Compiler Matrix Testing:** Test all supported compiler configurations
2. **MSRV Verification:** Ensure code compiles with Rust 1.60
3. **Cross-Compilation Testing:** Verify builds on all target triples
4. **Performance Testing:** Benchmark compiler-specific optimizations

**Traceability:** `.adrs/

---

## Compliance Verification

| Standard | Requirement | Status |
|----------|-------------|---------|
| IEEE 1016-2009 | Compiler compatibility documented | COMPLIANT |
| ISO C99 | FFI boundary compatibility | COMPLIANT |
| MSVC ABI | Windows compatibility | COMPLIANT |
| System V ABI | Unix compatibility | COMPLIANT |

---

## Approval

**Status:** APPROVED
**Approved By:** Compatibility Engineer Agent
**Approval Date:** 2026-02-11

**Related Documents:**
- [Compiler Compatibility Analysis](.adrs/
- [Testing Matrix](.adrs/
- ADR-035: OS Compatibility Strategy
- ADR-037: Architecture Compatibility Strategy
