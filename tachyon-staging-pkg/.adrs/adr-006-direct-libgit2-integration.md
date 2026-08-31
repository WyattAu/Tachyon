# ADR-006: Direct libgit2 Integration for Git Operations

## Status

| Status | Accepted |
|---------|----------|
| Date | 2026-02-11 |
| Decision | Use direct libgit2 bindings via git2-rs instead of shelling out to git CLI |
| Context | Git Integration and Version Control Architecture |

---

## Context and Problem Statement

### Current Situation

Tachyon requires Git operations for content versioning, history tracking, and collaboration support. The system must:
- Read and write Git repositories directly
- Commit changes with metadata
- Retrieve file history and diffs
- Support concurrent repository operations
- Achieve sub-10ms commit operations

### Problem

Shelling out to git CLI introduces several issues:
- Performance overhead: Process spawning, IPC, and command parsing
- Error handling: Complex parsing of git CLI output required
- Security: Shell injection vulnerabilities if not properly sanitized
- Cross-platform: Different git CLI behavior across platforms
- Reliability: Process failures, timeouts, and race conditions

### Constraints

| Constraint | Value | Source |
|------------|--------|---------|
| Commit operation latency | <10ms | requirements.md:103-117 |
| Auto-save interval | 2000ms | requirements.md:148-162 |
| Concurrent operations | Supported | requirements.md:163-177 |
| Cross-platform support | Linux, macOS, Windows | requirements.md:103-117 |

### Research Findings

Multi-lingual research (78 sources across 15 languages) confirms:
- Direct libgit2 integration provides 3-5x performance improvement over git CLI
- Chinese (ZH) and Russian (RU) sources document performance benefits
- Better error handling through Rust type system
- Thread-safe operations with native C library

Source: integrated_findings.md:77-99 (libgit2 Architecture)

---

## Decision Drivers

| Factor | Impact | Weight |
|---------|--------|--------|
| Performance | CRITICAL | 40% |
| Reliability | HIGH | 30% |
| Security | HIGH | 20% |
| Cross-Platform | MEDIUM | 10% |

---

## Considered Alternatives

### Alternative 1: Git CLI Shelling

**Description:** Execute git commands via shell (e.g., `git add`, `git commit`).

**Pros:**
- Simple implementation
- Git-native behavior
- No additional dependencies

**Cons:**
- 3-5x performance overhead from process spawning
- Complex error parsing required
- Shell injection vulnerabilities if inputs not sanitized
- Different behavior across platforms
- Reliability issues (process failures, timeouts)

**Evaluation:** REJECTED - Too many performance and security issues

### Alternative 2: Gitoxide (Pure Rust)

**Description:** Use pure Rust Git implementation.

**Pros:**
- 100% Rust implementation
- Type-safe operations
- No native dependencies

**Cons:**
- Less mature than libgit2/git2-rs
- Smaller ecosystem and community
- Limited feature set compared to libgit2
- Potential compatibility issues with existing Git servers

**Evaluation:** REJECTED - Less mature, potential compatibility risks

### Alternative 3: libgit2-sys (Low-Level Bindings)

**Description:** Use low-level C bindings directly without git2-rs wrapper.

**Pros:**
- Maximum control and performance
- Direct access to all libgit2 features

**Cons:**
- Unsafe Rust code required
- Complex memory management
- Higher development effort
- Difficult to maintain

**Evaluation:** REJECTED - Too complex, unsafe code

### Alternative 4: git2-rs (SELECTED)

**Description:** Use Rust bindings to libgit2 with safe Rust API.

**Configuration:**
- git2-rs 0.18.3 with default features
- Direct libgit2 integration via libgit2-sys
- Thread-safe operations with Rust wrappers
- High-level API for common operations

**Pros:**
- 3-5x performance improvement over git CLI (research validated)
- Type-safe Rust API prevents memory errors
- Better error handling via Result types
- Cross-platform support (Linux, macOS, Windows)
- Mature ecosystem with extensive documentation
- Active maintenance (quarterly releases)
- Compatible with existing Git infrastructure

**Cons:**
- Native dependency (libgit2) with C code
- Requires OpenSSL/libgit2 compilation (native dependency)
- Larger dependency tree than pure Rust
- Potential security vulnerabilities in native library

**Evaluation:** ACCEPTED - Best balance of performance, safety, and maturity

---

## Decision

**Use direct libgit2 bindings via git2-rs 0.18.3 for all Git operations.**

### Rationale

1. **Performance Improvement:**
   - 3-5x performance improvement over git CLI (research validated)
   - Sub-10ms commit operations achievable
   - Chinese (ZH) and Russian (RU) sources document 3-5x gains
   - Direct C library calls eliminate process spawning overhead

2. **Type Safety:**
   - Result types force error handling
   - Rust's ownership model prevents memory errors
   - No unsafe code required (unlike direct libgit2-sys)
   - Compiler guarantees memory safety

3. **Error Handling:**
   - Structured error types (git2::Error)
   - No CLI output parsing required
   - Easier to test and debug
   - Better error messages for users

4. **Cross-Platform Support:**
   - Linux: Native libgit2 support (yellow_paper.md:145-172)
   - macOS: Native libgit2 support
   - Windows: Native libgit2 support
   - Consistent behavior across platforms

5. **Maturity and Ecosystem:**
   - git2-rs 0.18.3 is well-maintained (quarterly releases)
   - Extensive documentation and examples
   - Large Rust community support
   - Used by major projects (cargo, rust-analyzer)

6. **Security:**
   - No shell injection vulnerabilities (no CLI shelling)
   - Type-safe API prevents memory corruption
   - libgit2-sys maintained by Git project (security patches)

7. **Feature Coverage:**
   - Complete libgit2 API access
   - Support for all Tachyon Git operations
   - Future-proof for new Git features

### Trade-offs

| Aspect | Benefit | Cost | Mitigation |
|---------|--------|---------|-------------|
| Performance | 3-5x faster | Native dependency | Regular updates from Git project |
| Type Safety | Result types | Larger dependency tree | Mature git2-rs ecosystem |
| Security | No shell injection | OpenSSL compilation | Supply chain monitoring |

---

## Implementation Plan

### Phase 1: Core Git Operations

**Tasks:**
- Implement Repository struct wrapping git2-rs Repository
- Implement commit operations with custom metadata
- Implement file history retrieval
- Implement diff operations for version comparison
- Add error handling and conversion to Tachyon error types

**Traceability:** blue_paper.md:103-117 (CM-RQ-003)
- dep_spec/git2-rs/dep_spec.toml:1-77

**Core API:**
```rust
use git2::{Repository, Oid, Time, Commit, ObjectType};

struct GitRepository {
    repo: Repository,
    path: PathBuf,
}

impl GitRepository {
    fn commit(&self, message: &str, files: &[&str]) -> Result<String, GitError> {
        let mut index = self.repo.index()?;
        for file in files {
            let mut oid = index.add_path(Path::new(file))?;
            index.write(&mut oid, fs::Metadata::default())?;
        }
        
        let tree_id = index.write_tree()?;
        let tree = self.repo.find_tree(tree_id)?;
        
        let signature = self.repo.signature_default()?;
        let commit_id = self.repo.commit(
            Some("HEAD"),
            Some(&signature),
            Some("HEAD"),
            None,
            Some(message),
            Some(&tree),
        )?;
        
        Ok(commit_id.to_string())
    }
    
    fn get_file_at_commit(&self, commit_hash: &str, file_path: &str) 
        -> Result<String, GitError> {
        let obj = self.repo.find_commit(commit_hash)?;
        let commit = obj.into_commit()?;
        let tree = commit.tree()?;
        let entry = tree.get_name(file_path)?;
        let object = self.repo.find_object(entry.id())?;
        match object {
            Some(ObjectType::Blob(blob)) => Ok(blob.content()?),
            _ => Err(GitError::ObjectType),
        }
    }
}
```

### Phase 2: Auto-Save Integration

**Tasks:**
- Implement 2-second debounce for auto-save (CM-RQ-006)
- Integrate with file watcher for change detection
- Implement commit history tracking per document
- Add auto-save configuration (interval, commit message template)
- Add metrics for auto-save success/failure

**Traceability:** blue_paper.md:148-162 (CM-RQ-006)
- adr-004: File watching integration

**Auto-Save Logic:**
```rust
struct AutoSaveConfig {
    interval_ms: u64,         // 2000ms from CM-RQ-006
    debounce_ms: u64,          // Align with file watcher
    commit_message: String,    // Template for commits
}

struct AutoSaveManager {
    config: AutoSaveConfig,
    pending_changes: HashMap<PathBuf, Instant>,
    last_commit: HashMap<PathBuf, String>,
}

impl AutoSaveManager {
    fn on_file_change(&mut self, path: PathBuf) {
        let now = Instant::now();
        self.pending_changes.insert(path.clone(), now);
        
        // Debounce for 2000ms
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(self.config.debounce_ms)).await;
            self.process_pending_changes();
        });
    }
    
    fn process_pending_changes(&mut self) {
        for (path, timestamp) in self.pending_changes.drain() {
            if timestamp.elapsed() >= Duration::from_millis(self.config.interval_ms) {
                self.commit_document(&path)?;
            }
        }
    }
}
```

### Phase 3: Concurrent Operations

**Tasks:**
- Implement thread-safe repository access
- Implement concurrent commit handling
- Add conflict detection and resolution (ADR-005)
- Implement atomic operations for consistency
- Add operation queuing for high load

**Traceability:** adr-005: Last-Write-Wins conflict resolution

**Thread Safety:**
```rust
use std::sync::{Arc, RwLock};

struct ThreadSafeGitRepository {
    repo: Arc<RwLock<Repository>>,
}

impl ThreadSafeGitRepository {
    fn with_read<T, F>(&self, f: F) -> T {
        let guard = self.repo.read().unwrap();
        f(&*guard)
    }
    
    fn with_write<T, F>(&self, f: F) -> T {
        let mut guard = self.repo.write().unwrap();
        f(&mut guard)
    }
}
```

### Phase 4: Error Handling and Monitoring

**Tasks:**
- Convert git2::Error to Tachyon::GitError
- Implement detailed error logging
- Add Git operation metrics (commits, reads, failures)
- Implement health checks (repository validity, disk space)
- Add telemetry for Git operation performance

**Error Mapping:**
```rust
#[derive(Debug, thiserror::Error)]
pub enum GitError {
    RepositoryNotFound { path: PathBuf },
    InvalidCommitHash { hash: String },
    FileNotFound { path: PathBuf },
    GitOperationFailed { operation: String, reason: String },
    ConflictDetected { document_path: PathBuf },
}

impl From<git2::Error> for GitError {
    fn from(err: git2::Error) -> Self {
        match err.class() {
            git2::ErrorClass::Repository => GitError::RepositoryNotFound,
            git2::ErrorClass::Oid => GitError::InvalidCommitHash,
            git2::ErrorClass::Reference => GitError::FileNotFound,
            _ => GitError::GitOperationFailed {
                operation: format!("{:?}", err),
                reason: err.message().to_string(),
            },
        }
    }
}
```

---

## Consequences

### Positive Consequences

1. **Performance:**
   - 3-5x performance improvement over git CLI
   - Sub-10ms commit operations achievable
   - Efficient batch operations with direct API
   - Meets CM-RQ-003 (Git Integration) requirements

2. **Type Safety:**
   - Result types prevent runtime panics
   - Rust ownership model prevents memory errors
   - Compiler-verified memory safety
   - Easier to test and debug

3. **Reliability:**
   - Better error handling with structured types
   - Thread-safe operations for concurrent access
   - No process spawning reliability issues
   - Mature, well-tested library

4. **Cross-Platform:**
   - Native libgit2 support for Linux, macOS, Windows
   - Consistent behavior across platforms
   - No platform-specific workarounds required

### Negative Consequences

1. **Native Dependency:**
   - libgit2 is a C library with potential security vulnerabilities
   - Requires OpenSSL/libgit2 compilation (native dependency)
   - Larger dependency tree than pure Rust
   - Supply chain monitoring required (already in Phase 1.5)

2. **Complexity:**
   - git2-rs API is extensive but complex
   - Requires understanding of Git internals
   - Error handling conversion required

3. **Build Complexity:**
   - Native library compilation required
   - Potential cross-compilation issues
   - Slower build than pure Rust alternatives

---

## Monitoring and Validation

### Success Criteria

| Metric | Target | Measurement Method |
|---------|--------|-------------------|
| Commit latency | <10ms P95 | Performance benchmarks |
| Auto-save accuracy | <200ms variance | Integration tests |
| Concurrent operations | No data corruption | Thread safety tests |
| Error handling | 100% git2 errors converted | Unit tests |
| Repository operations | Success rate >99.5% | Integration tests |
| Memory safety | Zero memory safety violations | Rust compiler checks |

### Testing Strategy

1. **Unit Tests:**
   - Test all Git operations (commit, read, diff, history)
   - Test error handling and conversion
   - Test thread-safe concurrent access
   - Test auto-save debounce logic

2. **Integration Tests:**
   - Test full Git workflow (edit -> commit -> detect conflict)
   - Test with real Git repository
   - Test file watcher integration
   - Test concurrent editing scenarios

3. **Performance Tests:**
   - Benchmark commit operation latency
   - Measure overhead vs. git CLI baseline
   - Profile memory usage during large operations
   - Test with 100 concurrent commits

4. **Cross-Platform Tests:**
   - Test on Linux (libgit2 native)
   - Test on macOS (libgit2 native)
   - Test on Windows (libgit2 native)
   - Verify consistent behavior

### Rollback Plan

If git2-rs fails to meet requirements:

1. **Evaluate Gitoxide:** Re-assess pure Rust Git implementation
2. **Fallback to Git CLI:** Implement sanitized shelling with strict input validation
3. **Hybrid Approach:** Use git2-rs for core operations, CLI for edge cases
4. **Optimize Native Calls:** Profile and optimize libgit2 usage
5. **Partial Feature Set:** Use git2-rs for common operations, CLI for advanced features

---

## Related Decisions

- blue_paper.md:103-117 (CM-RQ-003 - Git Integration)
- blue_paper.md:148-162 (CM-RQ-006 - Auto-Save)
- adr-005: Last-Write-Wins conflict resolution (Git operations)
- dep_spec/git2-rs/dep_spec.toml:1-77 (git2-rs specification)

---

## References

1. **Research Sources:**
   - integrated_findings.md:77-99 (libgit2 Architecture)
   - yellow_paper.md:145-172 (libgit2 Architecture)
   - sbom.spdx:66-76 (git2-rs dependency)
   - sbom.spdx:232-239 (wasm-bindgen with OpenSSL)

2. **Requirements:**
   - requirements.md:103-117 (CM-RQ-003)
   - requirements.md:148-162 (CM-RQ-006)
   - requirements.md:163-177 (CM-RQ-007 - Concurrent Operations)

3. **Domain Constraints:**
   - domain_constraints.toml:74-84 (Concurrent users)

4. **Architecture:**
   - blue_paper.md:103-117 (Git Operations)
   - blue_paper.md:163-177 (Conflict Resolution)

---

**Document Revision History:**

| Version | Date | Author | Changes |
|---------|-------|--------|---------|
| 1.0 | 2026-02-11 | Initial ADR creation |
