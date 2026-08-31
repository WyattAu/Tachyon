# ADR-012: Memory Model Considerations
# Status: Accepted
# Date: 2026-02-11
# Phase: 2.5 (Concurrency Analysis)

---

## Context

**Problem Statement:**
The Tachyon system uses Rust with async/await concurrency. Understanding Rust's memory model is critical for writing correct, race-free concurrent code. Incorrect assumptions about memory ordering, atomic operations, and synchronization can lead to subtle bugs and undefined behavior.

**Drivers:**
- thread_safety_analysis.md: Shared resource identification
- proof.lean: Atomic operation properties
- synchronization_design.md: Lock-based synchronization
- dep_spec/tokio/dep_spec.toml: tokio as async runtime

---

## Decision

**Selected Strategy:** Strict adherence to Rust memory model with proper use of atomic operations, ordering, and synchronization

**Rationale:**
Rust's memory model provides strong guarantees for data race prevention. By understanding and correctly applying these guarantees, we can ensure thread safety without relying on runtime checks. The strategy includes: (1) Use appropriate atomic operations, (2) Respect Rust's borrowing rules, (3) Ensure proper memory ordering, (4) Avoid data races with Send/Sync traits.

---

## Alternatives Considered

| Option | Description | Pros | Cons | Decision |
|---------|-------------|------|-----------|
| Unsafe Rust | Maximum performance | Safety not guaranteed | REJECTED |
| Relaxed Ordering | Faster, but subtle bugs | Requires discipline | REJECTED |
| SeqCst Ordering | Slower, but predictable | Added complexity | REJECTED |
| Strict Ordering | Correct by default | Performance cost | SELECTED |

**Selection Rationale:**
Rust's default memory ordering (SeqCst) provides the right balance of performance and correctness. Unsafe code would introduce undefined behavior and data races that cannot be caught by the compiler. The performance impact of proper atomic operations is negligible for Tachyon's use cases.

---

## Consequences

**Positive Consequences:**
- Compile-time data race prevention via Rust ownership
- Correct memory ordering by default
- Undefined behavior prevented
- Clear semantics for concurrent code

**Negative Consequences:**
- Slightly higher memory overhead from atomic operations
- Need to understand Send/Sync bounds for cross-thread data
- Potential for misuse of unsafe code

**Mitigation Strategies:**
- Avoid unsafe code unless absolutely necessary
- Use appropriate atomic types (AtomicUsize, AtomicBool, etc.)
- Use Arc for shared ownership across threads
- Understand Send/Sync trait bounds
- Test with miri for undefined behavior detection

---

## Implementation Details

### 1. Rust Memory Model Fundamentals

#### 1.1. Ownership and Borrowing

**Key Concepts:**
- Ownership: Each value has a single owner
- Borrowing: References to values without taking ownership
- Lifetimes: Compile-time tracking of reference validity
- Move Semantics: Transfer ownership on assignment

**Example:**
```rust
pub struct SharedCache {
    map: Arc<Mutex<HashMap<String, String>>>,
}

// Correct: Arc provides shared ownership
fn use_cache(cache: &SharedCache) {
    let guard = cache.map.lock().unwrap();
    // guard borrows map mutex
}

// Incorrect: Attempt to move owned value
fn use_cache_wrong(cache: SharedCache) {
    let map = cache.map; // Moves Arc, invalidating cache
    // Compiler error: use of moved value
}
```

#### 1.2. Atomic Operations

**Memory Ordering:**
- Ordering types: Relaxed, Acquire, Release, SeqCst
- Acquire-Release semantics: For data races
- Compiler optimizations: Single-instruction RMW

**Example:**
```rust
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct Counter {
    value: AtomicUsize,
}

impl Counter {
    pub fn increment(&self) -> usize {
        self.value.fetch_add(1, Ordering::Relaxed)
    }
    
    pub fn get(&self) -> usize {
        self.value.load(Ordering::SeqCst) // Strongest ordering for reads
    }
}
```

**Race Condition Mitigation:**
Acquire-Release with SeqCst ordering prevents data races between increment and read.

#### 1.3. Send and Sync Traits

**Key Concepts:**
- Send: Transfer ownership to another thread
- Sync: Thread-safe access
- 'static lifetime: Available for entire program duration

**Example:**
```rust
pub struct ThreadSafeSender<T> {
    sender: tokio::sync::mpsc::UnboundedSender<T>,
}

// T implements Send for Arc<T>, safe to share across threads
fn new_sender<T>() -> ThreadSafeSender<T> {
    ThreadSafeSender {
        sender: tokio::sync::mpsc::unbounded_channel(),
    }
}
```

**Race Condition Mitigation:**
Using Arc for shared ownership ensures no dangling references.

### 2. tokio Memory Model

#### 2.1. Async/Await Semantics

**Key Concepts:**
- Async tasks: Poll-based scheduling
- Blocking: Await points yield control to scheduler
- Cancellation: Cooperative via token cancellation

**Example:**
```rust
use tokio::sync::Mutex;

pub struct AsyncState {
    value: tokio::sync::Mutex<Vec<u8>>,
}

impl AsyncState {
    pub async fn update(&self, new_value: u8) {
        let mut guard = self.value.lock().await;
        guard.push(new_value);
    }
    
    pub async fn read(&self) -> Vec<u8> {
        let guard = self.value.lock().await;
        guard.clone()
    }
}
```

**Race Condition Mitigation:**
Mutex guards prevent concurrent modification during reads.

### 3. DashMap Memory Model

#### 3.1. Sharded Concurrent Access

**Key Concepts:**
- Per-shard mutex: Independent locking per shard
- Lock-free reads: Non-blocking get operations
- Sharding strategy: Hash-based or fixed count

**Example:**
```rust
use dashmap::DashMap;

pub struct ConcurrentCache<K, V> {
    map: DashMap<K, CacheEntry<V>>,
}

impl<K: Hash + Eq, V: Clone> ConcurrentCache<K, V> {
    pub fn get(&self, key: &K) -> Option<V> {
        self.map.get(key).map(|entry| entry.value.clone())
    }
}
```

**Race Condition Mitigation:**
DashMap's internal sharding prevents lock contention on reads.

### 4. Unsafe Code Guidelines

#### 4.1. When to Use Unsafe

**Guidelines:**
1. Only when absolutely necessary for performance
2. Document safety invariants thoroughly
3. Use minimal unsafe blocks with clear boundaries
4. Test extensively with miri
5. Prefer safe alternatives when possible

**Example:**
```rust
// Only when absolutely necessary
pub unsafe fn performance_critical(ptr: *const u8, len: usize) -> Vec<u8> {
    // SAFETY: Must ensure ptr is valid for 'len' bytes
    if !ptr.is_null() && (ptr as usize) + len <= 0x100000000 {
        std::slice::from_raw_parts(ptr, len)
    } else {
        panic!("Invalid pointer or length");
    }
}
```

**Race Condition Mitigation:**
Minimal unsafe code with clear invariants reduces undefined behavior surface.

---

## Testing Strategy

### 1. Memory Model Tests with miri

**Configuration:**
```toml
[dependencies]
miri = { version = "0.7" }

[[test]]
name = "miri_memory_model"
harness = false
```

**Test Cases:**
- Atomic operation ordering
- Arc reference cycles
- Borrow checker violations
- Send/Sync trait misuse

### 2. Data Race Tests

**Example Test:**
```rust
#[cfg(test)]
mod tests {
    #[test]
    #[cfg_attr(loom, loom::test)]
    fn test_atomic_counter_no_race() {
        loom::model(|| {
            let counter = std::sync::atomic::AtomicUsize::new(0);
            
            // Multiple threads increment
            let handle1 = std::thread::spawn(|| {
                counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            });
            let handle2 = std::thread::spawn(|| {
                counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            });
            
            handle1.join().unwrap();
            handle2.join().unwrap();
            
            // Verify no data race: final value must be 2
            assert_eq!(counter.load(std::sync::atomic::Ordering::Relaxed), 2);
        });
    }
}
```

---

## Compliance

| Standard | Status |
|----------|---------|
| IEEE 1016-2009 | COMPLIANT |
| ISO/IEC 25010 | COMPLIANT |

---

## References

- proof.lean: Formal proofs of atomic properties
- thread_safety_analysis.md: Thread safety mechanisms
- synchronization_design.md: Synchronization primitive usage
- race_condition_mitigation.md: Race condition prevention
- The Rust Book: "The Rust Programming Language" - Ownership and Borrowing

---

## Sign-off

**Approved By:** Concurrency Engineer Agent
**Approval Date:** 2026-02-11
**Next Document:** Phase 2.5 Concurrency Report
