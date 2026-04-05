# Global Anti-Pattern Library

**Document ID:** TACHYON-GAPL-V1.0
**Date:** 2026-02-12
**Phase:** 12 (Knowledge Transfer)
**Status:** Approved
**Standard:** IEEE 1016-2009

---

## 1. Introduction

This document contains anti-patterns identified in the Tachyon project. Anti-patterns are common pitfalls and suboptimal approaches that should be avoided in future development. Each anti-pattern includes description, consequences, prevention strategies, and related patterns to use instead.

**Anti-Pattern Sources:**
- Tachyon Project Phase 12 Knowledge Transfer
- Architecture Reviews (.adrs/)
- Post-Mortem Analysis (.specs/10_metrics/post_mortem.md)

---

## 2. Concurrency Anti-Patterns

### 2.1. AP-SYNC-BLOCKING: Synchronous Blocking Operations

**Category:** Concurrency
**Severity:** Critical
**Frequency:** Common in async Rust codebases

**Description:** Using synchronous blocking operations (e.g., `std::fs::File::read`, blocking `thread::sleep`) within async tokio runtime contexts causes the entire runtime to block, preventing other tasks from executing and leading to thread starvation.

**Consequences:**
- Blocks async runtime threads
- Causes thread starvation
- Degrades performance significantly
- Leads to poor user experience
- Can cause deadlock scenarios

**Examples:**
```rust
// BAD: Blocking file read in async context
async fn process_file(path: &Path) {
    // This blocks the entire runtime
    let content = std::fs::read_to_string(path).unwrap();
    process(&content);
}

// GOOD: Use async file operations
async fn process_file(path: &Path) {
    let content = tokio::fs::read_to_string(path).await?;
    process(&content);
    Ok(())
}
```

**Prevention Strategies:**
1. Use tokio::fs for async file operations
2. Use tokio::time for async delays
3. Use tokio::sync::Mutex instead of std::sync::Mutex in async contexts
4. Prefer lock-free data structures (DashMap) over blocking operations

**Related Pattern:** P-RUST-001 (Tokio Multi-Threaded Scheduler)

---

### 2.2. AP-MUTEX-CONTENTION: Mutex Contention

**Category:** Concurrency
**Severity:** Critical
**Frequency:** Common in high-throughput systems

**Description:** Using std::sync::Mutex for frequently accessed shared resources causes excessive lock contention under high load, resulting in performance degradation as threads compete for lock acquisition.

**Consequences:**
- Severe performance degradation
- Thread blocking and starvation
- Increased latency under load
- Scalability bottlenecks
- Poor CPU utilization

**Examples:**
```rust
// BAD: Mutex for high-contention resource
use std::sync::Mutex;

struct Cache {
    data: Mutex<HashMap<String, String>>,
}

impl Cache {
    fn get(&self, key: &str) -> Option<String> {
        let guard = self.data.lock().unwrap();
        guard.get(key).cloned()
    }
}
```

**Prevention Strategies:**
1. Use lock-free data structures (DashMap)
2. Use read-write locks (RwLock) for read-heavy workloads
3. Use async-aware synchronization primitives
4. Limit critical section duration

**Related Pattern:** P-RUST-002 (DashMap for Concurrent Caching)

---

## 3. Architecture Anti-Patterns

### 3.1. AP-GOD-MODULE: God Module

**Category:** Architecture
**Severity:** High
**Frequency:** Common in large codebases

**Description:** Creating a single module with too many responsibilities violates the Single Responsibility Principle. God modules have excessive complexity, making them difficult to understand, test, and maintain.

**Consequences:**
- Low cohesion (unrelated functionality)
- High coupling (hard to modify)
- Difficult to test
- Difficult to maintain
- Hard to understand for new developers

**Examples:**
```rust
// BAD: God module with too many responsibilities
struct Application {
    content_manager: ContentManager,
    renderer: Renderer,
    searcher: Searcher,
    cache: Cache,
    auth: AuthManager,
    ui: UIManager,
    network: NetworkManager,
    database: DatabaseManager,
    // ... 20+ more responsibilities
}
```

**Prevention Strategies:**
1. Apply Single Responsibility Principle
2. Extract cohesive sub-modules
3. Define clear interfaces between modules
4. Keep modules focused and small

**Related Module:** N/A (Apply SRP consistently)

---

## 4. Security Anti-Patterns

### 4.1. AP-IMPLICIT-AUTH: Implicit Authentication

**Category:** Security
**Severity:** Critical
**Frequency:** Common in web applications

**Description:** Implicitly trusting user input or session data without proper authentication and authorization checks. This anti-pattern assumes that if a request is made, it must be from a legitimate, authenticated user.

**Consequences:**
- Unauthorized access to sensitive data
- Session hijacking
- Privilege escalation
- Data exposure
- Compliance violations

**Examples:**
```rust
// BAD: Implicitly trusting user from session
async fn get_document(user_id: Option<&str>) -> Result<Document> {
    let user_id = user_id.unwrap(); // Trusts implicitly
    get_document_by_id(user_id)
}

// GOOD: Always authenticate
async fn get_document(user_id: Option<&str>) -> Result<Document> {
    let user_id = user_id.ok_or_else(|| anyhow!("Authentication required"))?;
    authenticate_user(user_id)?;
    get_document_by_id(user_id)
}
```

**Prevention Strategies:**
1. Implement RBAC with explicit role checking
2. Validate all user inputs at trust boundaries
3. Use least-privilege principle by default
4. Log all authorization decisions
5. Use secure session management

**Related Pattern:** P-SEC-001 (Trust Boundary Validation)

---

## 5. Performance Anti-Patterns

### 5.1. AP-CACHE-MISS-STORM: Cache Miss Storm

**Category:** Performance
**Severity:** High
**Frequency:** Common in caching implementations

**Description:** Cache key design that causes repeated cache misses, leading to excessive load on backend systems and poor user experience. This often occurs when cache keys don't include sufficient context or when cache eviction policies are poorly designed.

**Consequences:**
- Severe performance degradation
- Increased backend load
- Poor user experience
- Resource exhaustion
- Cascading failures

**Examples:**
```rust
// BAD: Cache key missing role context
fn cache_key(path: &Path) -> String {
    let commit = get_current_commit(path);
    format!("{}{}", path.display()) // Missing role
}
```

**Prevention Strategies:**
1. Include all relevant context in cache keys
2. Use role-based cache isolation
3. Implement proper cache eviction policies
4. Monitor cache hit rates
5. Design cache warming strategies

**Related Pattern:** P-ARCH-002 (LRU Cache with Role-Based Keys)

---

## 6. Anti-Pattern Analysis Framework

### 6.1. Detection Checklist

| Anti-Pattern | Early Detection Signs | Detection Methods |
|--------------|---------------------|-----------------|
| AP-SYNC-BLOCKING | Thread starvation, blocking runtime | Performance profiling, async task monitoring |
| AP-MUTEX-CONTENTION | High lock wait times | Lock contention profiling, thread state monitoring |
| AP-GOD-MODULE | Large files, many dependencies | Code complexity analysis, dependency graph analysis |
| AP-IMPLICIT-AUTH | Unauthorized access attempts | Access log analysis, audit trails |
| AP-CACHE-MISS-STORM | High backend load, poor cache hit rate | Cache metrics, backend monitoring |

### 6.2. Prevention Process

1. **Identification:** Recognize anti-patterns during code review
2. **Documentation:** Document in anti-pattern library with examples
3. **Training:** Educate team on anti-patterns and alternatives
4. **Linting:** Use automated linters to detect common anti-patterns
5. **Review:** Regular architecture reviews to catch violations
6. **Testing:** Write tests specifically for anti-pattern scenarios

### 6.3. Remediation

When anti-pattern is detected:
1. **Assess Impact:** Determine severity and affected components
2. **Plan Fix:** Design refactoring approach
3. **Implement:** Apply appropriate patterns
4. **Verify:** Test to ensure anti-pattern is resolved
5. **Document:** Record lessons learned

---

## 7. Anti-Pattern vs Pattern Mapping

| Anti-Pattern | Related Pattern | Reason for Pattern |
|--------------|----------------|-------------------|
| AP-SYNC-BLOCKING | P-RUST-001 | Use async runtime properly |
| AP-MUTEX-CONTENTION | P-RUST-002 | Use lock-free structures |
| AP-GOD-MODULE | N/A | Apply SRP consistently |
| AP-IMPLICIT-AUTH | P-SEC-001 | Always validate inputs |
| AP-CACHE-MISS-STORM | P-ARCH-002 | Design cache keys carefully |

---

## 8. Anti-Pattern Evolution

### 8.1. Version History

| Version | Date | Changes |
|---------|-------|---------|
| 1.0.0 | 2026-02-12 | Initial release from Tachyon project |

### 8.2. Future Enhancements

- **ML Detection:** Machine learning models to automatically detect anti-patterns
- **Real-time Analysis:** IDE plugins to warn about anti-patterns during development
- **Community Contributions:** Crowdsourced anti-pattern database
- **Cross-Language Support:** Anti-patterns for multiple programming languages

---

**Document Status:** COMPLETE
**Owner:** Knowledge Manager
**Reviewers:** TBD
**Approved By:** TBD
