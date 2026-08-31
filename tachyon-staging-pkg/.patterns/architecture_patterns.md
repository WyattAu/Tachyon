# Architecture Patterns

This document contains architecture patterns and best practices identified during Tachyon project development.

## System Architecture Patterns

### P-ARCH-001: Three-Tier JIT Compilation

**Category:** Rendering
**Complexity:** Medium
**Context:** Sub-15ms rendering latency requirement for real-time editing.

**Problem:** Single-tier caching is too slow for real-time editing.

**Solution:** Three-tier compilation: cache lookup > template rendering > baseline parsing.

**Implementation:**
```rust
async fn render_document_tiered(path: &Path) -> Result<String> {
    let key = cache_key(path);
    
    // Tier 3: Cache lookup (fastest)
    if let Some(html) = cache.get(&key) {
        return Ok(html);
    }
    
    // Tier 1: Baseline parsing (slowest)
    let markdown = std::fs::read_to_string(path)?;
    let ast = parse_markdown(&markdown)?;
    
    // Tier 2: Template rendering (medium)
    let html = render_template("base.html", &ast)?;
    
    cache.insert(key, html.clone());
    Ok(html)
}
```

**Benefits:**
- Cache hit: <1ms
- Template: <5ms
- Baseline: <10ms
- Total (cache miss): <15ms

**Traceability:** LL-ARCH-001

---

### P-ARCH-002: LRU Cache with Role-Based Keys

**Category:** Caching
**Complexity:** Medium
**Context:** Read-heavy workloads with role-based access control.

**Problem:** Standard HashMap locks degrade concurrency and doesn't account for roles.

**Solution:** LRU cache with DashMap for lock-free reads and role-based keys.

**Implementation:**
```rust
use dashmap::DashMap;

fn cache_key(path: &Path, role: Role) -> String {
    format!("{}:{}", path.display(), role)
}

struct Cache<K, V> {
    map: DashMap<K, V>,
    capacity: usize,
}
```

**Benefits:**
- Lock-free reads
- Role-aware caching
- Linear scalability

**Traceability:** LL-ARCH-002

---

### P-ARCH-003: BM25 Search with Inverted Index

**Category:** Search
**Complexity:** Medium
**Context:** Full-text search needs relevance scoring and fast lookups.

**Problem:** Linear search is too slow and lacks relevance.

**Solution:** Tantivy inverted index with BM25 relevance scoring.

**Implementation:**
```toml
[dependencies]
tantivy = "0.21"
```

**Benefits:**
- Relevance-ranked results
- Fast term lookup
- Sub-100ms search latency

**Traceability:** LL-ARCH-003

## Concurrency Patterns

### P-ARCH-004: Lock-Free Data Structures

**Category:** Concurrency
**Complexity:** Medium
**Context:** High concurrency requires lock-free access patterns.

**Problem:** Mutex locks cause contention under parallel access.

**Solution:** Use DashMap for lock-free concurrent data structures.

**Implementation:**
```toml
[dependencies]
dashmap = "5.5"
```

```rust
use dashmap::DashMap;

let cache: DashMap<String, String> = DashMap::new();

// Lock-free read
if let Some(value) = cache.get(&key) {
    // Access without lock
}

// Sharded write
cache.insert(key, value);
```

**Benefits:**
- Lock-free reads
- Sharded writes
- Linear scalability

**Traceability:** LL-ARCH-004

---

### P-ARCH-005: Semaphore-Based Concurrency Limits

**Category:** Concurrency
**Complexity:** Medium
**Context:** Unbounded concurrency can overwhelm resources.

**Problem:** Unbounded parallel requests cause resource exhaustion.

**Solution:** Semaphore-based concurrency control.

**Implementation:**
```rust
use tokio::sync::Semaphore;

const MAX_CONCURRENT_REQUESTS: usize = 100;

async fn handle_requests_concurrently() -> Result<()> {
    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_REQUESTS));
    // Implementation
}
```

**Benefits:**
- Controlled resource usage
- Prevents overload
- Predictable performance

**Traceability:** LL-ARCH-005

## File System Patterns

### P-ARCH-006: Debounced File Watching

**Category:** File System
**Complexity:** Simple
**Context:** File watcher generates events for every write operation.

**Problem:** Rapid successive writes cause redundant processing.

**Solution:** Debounce events with 50ms window.

**Implementation:**
```rust
async fn debounced_events(mut rx: Receiver<Event>) -> Vec<Event> {
    let mut batch = Vec::new();
    let mut deadline: Option<Instant> = None;
    let debounce_duration = Duration::from_millis(50);
    
    while let Ok(event) = rx.recv().await {
        batch.push(event);
        
        match deadline {
            None => {
                deadline = Some(Instant::now() + debounce_duration);
            }
            Some(d) if Instant::now() >= d => {
                break;
            }
            Some(_) => {}
        }
    }
    
    batch
}
```

**Benefits:**
- Reduces redundant processing
- Improves cache hit rate
- Stabilizes event stream

**Traceability:** LL-ARCH-006

---

### P-ARCH-007: Request Coalescing

**Category:** File System
**Complexity:** Medium
**Context:** Simultaneous requests for same uncached data overwhelm backing store.

**Problem:** Multiple requests for same uncached data cause thundering herd.

**Solution:** Use request coalescing (singleflight) to prevent cache stampede.

**Implementation:**
```toml
[dependencies]
singleflight = "0.10"
```

```rust
use singleflight::Group;

async fn get_document_coalesced(path: &Path) -> Result<String> {
    let key = cache_key(path);
    let group = Group::new(key);
    
    let html = group
        .work(|| async {
            if let Some(cached) = cache.get(path) {
                return Ok(cached.clone());
            }
            let html = render_document(path).await?;
            cache.insert(path, html.clone());
            Ok(html)
        })
        .await?;
    
    Ok(html)
}
```

**Benefits:**
- Prevents backend overload
- Reduces redundant processing
- Improved cache effectiveness

**Traceability:** LL-ARCH-007

## Integration Patterns

### P-ARCH-008: Git Operations via git2-rs

**Category:** Integration
**Complexity:** Medium
**Context:** Need reliable Git operations for version control.

**Problem:** Shell commands to Git have inconsistent error handling.

**Solution:** Use git2-rs direct libgit2 bindings.

**Implementation:**
```toml
[dependencies]
git2 = "0.18"
```

```rust
use git2::{Repository, ObjectType};

async fn get_file_commit(path: &Path, file: &str) -> Result<String> {
    let repo = Repository::open(path)?;
    let head = repo.head()?;
    let commit = head.peel_to_commit()?;
    let tree = commit.tree()?;
    
    let object = repo.find_object(tree.get_path(Path::new(file))?, Some(ObjectType::Blob))?;
    let blob = object.as_blob().ok_or_else(|| anyhow::anyhow!("Not a blob"))?;
    
    let content = String::from_utf8(blob.content().to_vec())?;
    Ok(content)
}
```

**Benefits:**
- Direct library bindings
- Consistent error handling
- Cross-platform compatibility

**Traceability:** LL-ARCH-008

## Hardware Abstraction Patterns

### P-ARCH-009: Hardware Abstraction Layer (HAL)

**Category:** Hardware
**Complexity:** High
**Context:** Hardware-specific code must be isolated for cross-platform support.

**Problem:** Hardware-specific code scattered throughout codebase.

**Solution:** Hardware Abstraction Layer with trait-based abstraction.

**Implementation:**
```rust
/// Hardware Abstraction Layer
pub trait Hardware {
    fn get_cpu_count(&self) -> usize;
    fn get_memory_total(&self) -> u64;
    fn get_disk_space(&self, path: &Path) -> Result<u64>;
}

/// POSIX implementation
pub struct PosixHardware;

impl Hardware for PosixHardware {
    fn get_cpu_count(&self) -> usize {
        num_cpus::get()
    }
    
    fn get_memory_total(&self) -> u64 {
        // POSIX-specific implementation
        // ...
    }
    
    fn get_disk_space(&self, path: &Path) -> Result<u64> {
        // POSIX-specific implementation
        // ...
    }
}
```

**Benefits:**
- Cross-platform support
- Hardware isolation
- Testable architecture

**Traceability:** LL-ARCH-009

## Formal Verification Patterns

### P-ARCH-010: Lean4 Formal Verification

**Category:** Formal Methods
**Complexity:** High
**Context:** Need mathematical confidence in correctness for critical concurrent algorithms.

**Problem:** Testing alone cannot guarantee correctness for concurrent code.

**Solution:** Use Lean4 formal verification for thread safety invariants.

**Implementation:**
```lean
-- Lean4 proof of cache invariants
theorem cache_thread_safe : cache_invariant_preserved ∀ k v, k', by
  sorry

theorem cache_lock_free : no_mutex_required ∀ k v, by
  sorry
```

**Benefits:**
- Mathematical confidence in correctness
- Proven thread safety
- Reduced race conditions

**Traceability:** LL-ARCH-010

## References

- [Blue Paper: Tachyon System Architecture Specification](.adrs/
- [Hardware Abstraction Layer Specification](.adrs/
- [Concurrency Analysis](.adrs/
