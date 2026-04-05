# Rust Anti-Patterns

This document contains Rust-specific anti-patterns to avoid in Tachyon project development.

## Concurrency Anti-Patterns

### AP-RUST-001: Blocking Async Operations

**Category:** Concurrency
**Severity:** High

**Problem:** Using blocking operations in async context blocks the entire thread executor, reducing concurrency.

**Anti-Pattern Example:**
```rust
// BAD: Blocking in async context
async fn bad_blocking() {
    let result = std::fs::read_to_string("file.txt").unwrap();  // Blocks!
}

// GOOD: Non-blocking in async context
async fn good_non_blocking() {
    let result = tokio::fs::read_to_string("file.txt").await?;
    Ok(result)
}
```

**Consequences:**
- Reduced throughput
- Thread starvation
- Poor scalability

**Solution:** Use tokio's async I/O operations instead of std I/O.

**Traceability:** P-RUST-001

---

### AP-RUST-002: Unbounded Channel Buffer

**Category:** Concurrency
**Severity:** High

**Problem:** Unbounded channels grow indefinitely under producer-consumer imbalance.

**Anti-Pattern Example:**
```rust
// BAD: Unbounded channel
let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

// GOOD: Bounded channel with backpressure
let (tx, rx) = tokio::sync::mpsc::channel(1000);
```

**Consequences:**
- Memory exhaustion
- Unbounded growth
- System instability

**Solution:** Always use bounded channels with appropriate buffer sizes.

**Traceability:** AP-RUST-002

## I/O Anti-Patterns

### AP-RUST-007: Synchronous File Operations in Async Context

**Category:** I/O
**Severity:** High

**Problem:** Blocking file operations block the async executor.

**Anti-Pattern Example:**
```rust
// BAD: Blocking in async context
async fn bad_read(path: &Path) -> Result<String> {
    let content = std::fs::read_to_string(path)?;  // Blocks!
    Ok(content)
}

// GOOD: Async file operations
async fn good_read(path: &Path) -> Result<String> {
    let content = tokio::fs::read_to_string(path).await?;
    Ok(content)
}
```

**Consequences:**
- Reduced throughput
- Thread blocking
- Poor scalability

**Solution:** Use tokio's async file operations in async contexts.

**Traceability:** AP-PUST-002

## Security Anti-Patterns

### AP-RUST-009: Security Through Obscurity

**Category:** Security
**Severity:** Critical

**Problem:** Assuming secret endpoints or hidden parameters provide security.

**Anti-Pattern Example:**
```rust
// BAD: Security through obscurity
async fn get_internal_document(user_role: &str, secret_param: &str) -> Result<String> {
    // Wrong: Secret parameter provides access!
    if secret_param == "admin_override" {
        return get_restricted_document();
    }
}
```

**Consequences:**
- False security
- Secret leakage
- Bypass vulnerability

**Solution:** Implement proper RBAC, never rely on secret parameters.

**Traceability:** LL-SEC-001

### AP-RUST-011: Hardcoded Credentials

**Category:** Security
**Severity:** Critical

**Problem:** Hardcoded credentials are exposed in version control and binaries.

**Anti-Pattern Example:**
```rust
// BAD: Hardcoded credentials
const DB_PASSWORD: &str = "SuperSecret123";  // Leaked to VCS!
```

**Consequences:**
- Credential leakage
- Security breach
- Compliance violation

**Solution:** Always use environment variables or secret managers for credentials.

**Traceability:** LL-SEC-003

## References

- [Rust Patterns](.patterns/rust_patterns.md)
- [Pattern Library Specification](.specs/08_5_knowledge_base/pattern_library.md)
- [Anti-Pattern Library Specification](.specs/08_5_knowledge_base/anti_patterns.md)
