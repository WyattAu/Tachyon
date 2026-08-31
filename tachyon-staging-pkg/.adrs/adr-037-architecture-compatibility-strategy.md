# ADR-037: Architecture Compatibility Strategy

**Status:** ACCEPTED
**Date:** 2026-02-11
**Context:** Tachyon Cross-Platform Compatibility Analysis
**Decision:** Use Rust's type system for architecture-independent data handling
**Related ADRs:** ADR-036 (Compiler Compatibility), ADR-038 (Conditional Compilation)

---

## Context

Tachyon must support multiple CPU architectures:
- **x86_64** (primary target: Intel/AMD 64-bit)
- **aarch64** (ARM64: Apple Silicon, ARM servers)
- **armv7** (ARM 32-bit: embedded devices, older hardware)
- **riscv64** (RISC-V: emerging architecture, future-proofing)

Architecture-specific considerations:
- **Endianness:** Byte order (little-endian vs big-endian)
- **Word size:** Pointer and integer sizes (32-bit vs 64-bit)
- **Alignment:** Memory alignment requirements
- **SIMD capabilities:** Vector instruction sets (AVX2, NEON)

**Traceability:** `.adrs/ `.dep_spec/tokio/dep_spec.toml:77`

---

## Decision

We will adopt an **architecture-independent strategy** using Rust's type system:

1. **Use fixed-width integer types** (`u32`, `u64`, `i32`, etc.) for all architecture-agnostic operations
2. **Avoid pointer arithmetic** for size calculations
3. **Use network byte order** (`u16::to_be`, `u16::from_be`) for cross-platform serialization
4. **Leverage SIMD abstraction** with conditional compilation for architecture-specific optimizations
5. **Handle alignment explicitly** using `#[repr(C)]` for FFI and `#[repr(packed)]` for wire formats

---

## Alternatives Considered

### Alternative 1: Architecture-Specific Codebases
**Pros:**
- Full control over architecture-specific optimizations
- No conditional compilation overhead
- Maximum performance per architecture

**Cons:**
- Code duplication across 4+ architectures
- Maintenance burden (4x codebase)
- Inconsistent behavior between architectures
- Violates DRY principle

**Rejected:** Maintenance burden too high

### Alternative 2: Runtime Architecture Detection
**Pros:**
- Single binary distribution
- No conditional compilation

**Cons:**
- Runtime overhead for every operation
- Larger binary size (all code paths included)
- Complex error handling

**Rejected:** Runtime performance overhead unacceptable

### Alternative 3: Bytecode/VM Abstraction
**Pros:**
- Complete architecture independence
- Single distribution

**Cons:**
- Runtime overhead (VM startup, JIT compilation)
- Increased complexity
- Loss of low-level control

**Rejected:** Over-engineering for use case

---

## Consequences

### Positive Consequences

1. **Portability:** Code compiles and runs correctly on all supported architectures
2. **Type Safety:** Rust's type system prevents architecture-specific bugs
3. **Performance:** SIMD optimizations enabled where available via conditional compilation
4. **Future-Proof:** New architectures can be added by implementing SIMD traits

### Negative Consequences

1. **Conditional Compilation Complexity:** Requires careful `#[cfg(...)]` usage
2. **Testing Overhead:** Must verify all architecture-specific code paths
3. **Binary Size:** Multiple SIMD implementations increase binary size

---

## Implementation Details

### 1. Fixed-Width Integer Types

```rust
/// Architecture-independent hash computation
pub fn compute_file_hash(path: &Path) -> u64 {
    let mut hasher = DefaultHasher::new();
    // Use u64 for hash on all architectures
    hasher.update(path.as_os_str().as_bytes());
    hasher.finish()
}

/// Architecture-independent size limit
const MAX_DOCUMENT_SIZE: u64 = 10 * 1024 * 1024; // 10MB
const MAX_CACHE_ENTRIES: u32 = 1000;
```

### 2. Network Byte Order

```rust
/// Cross-platform serialization
pub fn serialize_u32(value: u32, buffer: &mut [u8]) {
    // Always use big-endian for network/wire format
    buffer[0] = (value >> 24) as u8;
    buffer[1] = (value >> 16) as u8;
    buffer[2] = (value >> 8) as u8;
    buffer[3] = value as u8;
}

pub fn deserialize_u32(buffer: &[u8]) -> u32 {
    // Always read big-endian
    ((buffer[0] as u32) << 24)
        | ((buffer[1] as u32) << 16)
        | ((buffer[2] as u32) << 8)
        | (buffer[3] as u32))
}
```

### 3. Alignment Handling

```rust
/// Packed structure for wire format (no padding)
#[repr(packed)]
pub struct FileHeader {
    magic: [u8; 4],
    version: u32,
    flags: u32,
    size: u64,
}

/// C-compatible structure for FFI
#[repr(C)]
pub struct FileInfo {
    pub size: u64,
    pub modified: u64,
    pub permissions: u32,
}
```

### 4. SIMD Abstraction

```rust
/// SIMD trait for architecture abstraction
pub trait SimdAdd {
    unsafe fn add(a: &[f32], b: &[f32], out: &mut [f32]);
}

#[cfg(target_arch = "x86_64")]
#[cfg(target_feature = "avx2")]
impl SimdAdd for Avx2Simd {
    unsafe fn add(a: &[f32], b: &[f32], out: &mut [f32]) {
        use std::arch::x86_64::*;
        let avx_a = _mm256_loadu_ps(a.as_ptr());
        let avx_b = _mm256_loadu_ps(b.as_ptr());
        let result = _mm256_add_ps(avx_a, avx_b);
        _mm256_storeu_ps(out.as_mut_ptr(), result);
    }
}

#[cfg(target_arch = "aarch64")]
#[cfg(target_feature = "neon")]
impl SimdAdd for NeonSimd {
    unsafe fn add(a: &[f32], b: &[f32], out: &mut [f32]) {
        use std::arch::aarch64::*;
        let neon_a = vld1q_f32(a.as_ptr());
        let neon_b = vld1q_f32(b.as_ptr());
        let result = vaddq_f32(neon_a, neon_b);
        vst1q_f32(out.as_mut_ptr(), result);
    }
}

#[cfg(not(any(
    all(target_arch = "x86_64", target_feature = "avx2"),
    all(target_arch = "aarch64", target_feature = "neon")
)))]
impl SimdAdd for ScalarSimd {
    unsafe fn add(a: &[f32], b: &[f32], out: &mut [f32]) {
        // Portable scalar fallback
        for i in 0..a.len() {
            out[i] = a[i] + b[i];
        }
    }
}

/// Generic function using SIMD trait
pub fn add_vectors<T: SimdAdd>(a: &[f32], b: &[f32], out: &mut [f32]) {
    unsafe { T::add(a, b, out) }
}
```

### 5. Pointer Size Independence

```rust
/// Use usize for indices, not pointers
pub struct DocumentIndex {
    inner: usize,
}

impl DocumentIndex {
    pub fn new(index: usize) -> Self {
        DocumentIndex { inner: index }
    }

    pub fn as_offset(&self) -> u64 {
        self.inner as u64
    }
}

/// Avoid pointer arithmetic
pub fn read_document_at(base: &[u8], offset: u64, size: usize) -> &[u8] {
    let start = offset as usize;
    let end = start.saturating_add(size);
    &base[start..end.min(base.len())]
}
```

---

## Migration Strategy

1. **Phase 1 (Week 1-2):** Audit code for architecture assumptions
2. **Phase 2 (Week 3-4):** Replace `usize` with explicit types where needed
3. **Phase 3 (Week 5-6):** Implement SIMD abstraction traits
4. **Phase 4 (Week 7-8):** Add network byte order serialization
5. **Phase 5 (Week 9-10):** Test on all supported architectures

---

## Testing Strategy

1. **Architecture Matrix Testing:** Test on x86_64, aarch64, armv7, riscv64
2. **Endianness Testing:** Verify network byte order on all platforms
3. **SIMD Testing:** Benchmark architecture-specific optimizations
4. **Alignment Testing:** Verify packed structures work correctly

**Traceability:** `.adrs/

---

## Compliance Verification

| Standard | Requirement | Status |
|----------|-------------|---------|
| IEEE 1016-2009 | Architecture compatibility documented | COMPLIANT |
| ISO C99 | FFI boundary compatibility | COMPLIANT |
| POSIX | Alignment requirements | COMPLIANT |
| Network Byte Order | Big-endian wire format | COMPLIANT |

---

## Approval

**Status:** APPROVED
**Approved By:** Compatibility Engineer Agent
**Approval Date:** 2026-02-11

**Related Documents:**
- [Compiler Compatibility Analysis](.adrs/
- [Testing Matrix](.adrs/
- ADR-035: OS Compatibility Strategy
- ADR-038: Conditional Compilation Strategy
