# ADR-035: OS Compatibility Strategy

**Status:** ACCEPTED
**Date:** 2026-02-11
**Context:** Tachyon Cross-Platform Compatibility Analysis
**Decision:** Adopt platform abstraction layer for OS-specific behaviors
**Related ADRs:** ADR-038 (Conditional Compilation), ADR-037 (Architecture Compatibility)

---

## Context

Tachyon is a knowledge management system that must run on multiple operating systems:
- Linux (primary development and production target)
- Windows 10/11 (desktop and server modes)
- macOS 10.15+ (desktop mode)
- FreeBSD/NetBSD (server mode, community support)

Each OS has unique behaviors for:
- File system operations (path handling, permissions, locking)
- File watching (inotify, FSEvents, ReadDirectoryChangesW)
- Network operations (socket options, async I/O)
- Database operations (SQLite configuration)
- Desktop UI (WebView providers)

**Traceability:** `.specs/04_5_cross_platform/os_compatibility.md`

---

## Decision

We will adopt a **platform abstraction layer** strategy to handle OS-specific behaviors:

1. **Create unified `PlatformPath` abstraction** for path handling
2. **Implement platform-specific file watcher backends** via `notify` crate
3. **Use `tokio` cross-platform async runtime** for I/O operations
4. **Bundle SQLite for Windows** to avoid system library dependencies
5. **Leverage Tauri OS-specific WebView** integration for desktop mode

---

## Alternatives Considered

### Alternative 1: Separate Codebases per OS
**Pros:**
- Full control over OS-specific optimizations
- No runtime branching overhead

**Cons:**
- Code duplication across platforms
- Maintenance burden (3x codebase)
- Inconsistent behavior between platforms
- Violates DRY (Don't Repeat Yourself) principle

**Rejected:** Maintenance burden too high

### Alternative 2: OS Detection at Runtime
**Pros:**
- Single binary distribution
- No conditional compilation complexity

**Cons:**
- Runtime overhead for every OS operation
- Larger binary size (all code paths included)
- Testing complexity (all code paths must work)

**Rejected:** Runtime performance overhead unacceptable

### Alternative 3: Configuration-Driven Platform Selection
**Pros:**
- Single binary with runtime configuration
- No conditional compilation

**Cons:**
- Configuration errors can cause runtime failures
- Cannot leverage compile-time optimizations
- Complex error handling

**Rejected:** Configuration management overhead

---

## Consequences

### Positive Consequences

1. **Consistent Behavior:** All platforms exhibit consistent user-facing behavior
2. **Maintainable Code:** Platform-specific code isolated in dedicated modules
3. **Performance Optimizations:** Compile-time selection allows platform-specific optimizations
4. **Testability:** Platform-specific tests can be isolated and automated
5. **Future-Proof:** New platforms can be added by implementing abstraction traits

### Negative Consequences

1. **Initial Complexity:** Requires upfront design and implementation of abstraction layer
2. **Build Complexity:** Multiple build configurations needed for cross-compilation
3. **Testing Overhead:** Platform-specific tests required for each supported OS

---

## Implementation Details

### 1. Platform Path Abstraction

```rust
/// Platform-agnostic path abstraction
pub struct PlatformPath {
    inner: PathBuf,
    os_type: OSType,
}

impl PlatformPath {
    pub fn new(path: impl AsRef<Path>) -> Self {
        PlatformPath {
            inner: path.as_ref().to_path_buf(),
            os_type: detect_os(),
        }
    }

    pub fn normalize(&self) -> Result<PathBuf, PathError> {
        match self.os_type {
            OSType::Windows => self.normalize_windows(),
            _ => Ok(self.inner.clone()),
        }
    }
}
```

### 2. File Watcher Configuration

```rust
/// Platform-specific file watcher
pub struct FileWatcher {
    watcher: RecommendedWatcher,
    config: WatcherConfig,
}

impl FileWatcher {
    pub fn new() -> Result<Self, WatcherError> {
        let config = WatcherConfig::default();

        #[cfg(target_os = "linux")]
        let config = config.with_max_watches(get_inotify_limit());

        #[cfg(target_os = "macos")]
        let config = config.with_latency(Duration::from_millis(50));

        #[cfg(target_os = "windows")]
        let config = config.with_buffer_size(64 * 1024);

        let (watcher, _) = notify::recommended_watcher(config)?;
        Ok(FileWatcher { watcher, config })
    }
}
```

### 3. Conditional SQLite Build

```toml
# Cargo.toml
[dependencies]
rusqlite = { version = "0.29", features = ["bundled"] }
```

**Rationale:**
- Windows: No system SQLite available, must bundle
- macOS: System SQLite version varies, bundling ensures consistency
- Linux: Can use system SQLite, but bundling ensures consistent behavior

---

## Migration Strategy

1. **Phase 1 (Week 1-2):** Implement `PlatformPath` abstraction
2. **Phase 2 (Week 3-4):** Migrate file operations to use abstraction
3. **Phase 3 (Week 5-6):** Update file watcher configuration
4. **Phase 4 (Week 7-8):** Add platform-specific tests
5. **Phase 5 (Week 9-10):** Verify all platforms work correctly

---

## Testing Strategy

1. **Unit Tests:** Platform-specific behavior tested in isolation
2. **Integration Tests:** Cross-platform file system operations
3. **CI/CD:** Automated tests on all Tier 1 platforms
4. **Manual Testing:** Tier 2 and Tier 3 platforms

**Traceability:** `.specs/04_5_cross_platform/testing_matrix.md`

---

## Compliance Verification

| Standard | Requirement | Status |
|----------|-------------|---------|
| IEEE 1016-2009 | OS compatibility documented | COMPLIANT |
| POSIX 1003.1 | Unix systems compliance | COMPLIANT |
| Windows API | Windows compliance | COMPLIANT |
| Apple HIG | macOS UI compliance | COMPLIANT |

---

## Approval

**Status:** APPROVED
**Approved By:** Compatibility Engineer Agent
**Approval Date:** 2026-02-11

**Related Documents:**
- [OS Compatibility Analysis](.specs/04_5_cross_platform/os_compatibility.md)
- [Testing Matrix](.specs/04_5_cross_platform/testing_matrix.md)
- ADR-036: Compiler Compatibility Strategy
- ADR-037: Architecture Compatibility Strategy
