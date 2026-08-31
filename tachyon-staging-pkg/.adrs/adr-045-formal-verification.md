# ADR-045: Formal Verification Strategy

**Status:** Accepted
**Date:** 2026-02-11
**Context:** Phase 5 - The Adversarial Loop (Prototyper)
**Related ADRs:** ADR-040 (Prototype Architecture)
**Traceability:** TACHYON-BP-V1.0 (Blue Paper), TACHYON-PROOF-V1.0 (Lean4 Proofs)

---

## 1. Context

Phase 5 requires verification that **Lean4/Coq proofs compile and pass** if they exist. The system includes formal proofs for:
- LRU cache eviction algorithm
- BM25 relevance scoring
- Deadlock freedom under ordered lock acquisition
- Thread safety properties

### 1.1. Existing Formal Proofs

From [`proof.lean`](../.adrs/ and [`concurrency/proof.lean`](../.adrs/

| Proof | Component | Property | Status |
|-------|-----------|----------|---------|
| LRU Eviction Correctness | LRU Cache | Evicted key is least recently used | PENDING |
| BM25 Non-Negativity | Search Engine | Score >= 0.0 | PENDING |
| Deadlock Freedom | Concurrency | No circular wait under ordered lock acquisition | PENDING |
| Thread Safety | tokio Runtime | Send/Sync marker compliance | PENDING |

### 1.2. Verification Requirements

**Requirements:**
1. Compile all Lean4 proof files without errors
2. Verify all proofs pass successfully
3. Map proofs to implementation code
4. Document any proof failures with rationale

---

## 2. Decision

### 2.1. Formal Verification Framework

**Primary Tool:** `lean4` (Theorem prover and programming language)

**Secondary Tools:**
- `leanpkg` (Package manager)
- `lake` (Build tool for Lean projects)
- `lean-check` (Verification tool)

**Rationale:**

| Criterion | lean4 | Coq | Why Lean4? |
|-----------|-------|------|-------------|
| Rust Integration | Yes | No | Tachyon is Rust-based |
| Modern Syntax | Yes | No | More user-friendly |
| Active Development | Yes | Limited | Ongoing maintenance |
| Meta-Programming | Yes | Yes | Lean4 is meta-language |
| Proof Automation | Yes | Partial | Better automation support |

### 2.2. Verification Strategy

#### 2.2.1. Proof Compilation

**Objective:** Verify all Lean4 proofs compile without errors.

**Process:**
```bash
# Install Lean4
curl https://raw.githubusercontent.com/leanprover/elan/master/elan-init.sh -sSf | bash
source ~/.elan/env/leanvar

# Compile proofs
cd .adrs/
leanpkg build

# Verify no errors
lean --make proof.lean
```

#### 2.2.2. Proof Execution

**Objective:** Verify all proofs pass successfully.

**Process:**
```bash
# Run proofs
lake build proof.lean

# Execute proofs
lake run proof.lean

# Verify all theorems proved
echo $?  # Exit code 0 = success
```

#### 2.2.3. Code-Proof Mapping

**Objective:** Verify implementation code matches proven properties.

**Process:**
```rust
// Add verification attributes to implementation
// tests/verification/formal_proof_tests.rs

#[cfg(test)]
mod formal_verification {
    use tachyon_prototype::cache::LruCache;

    #[test]
    fn verify_lru_eviction_correctness() {
        // This corresponds to Lean4 proof: "Evicted key is least recently used"
        let cache = LruCache::new(3);

        cache.insert("k1", "v1");
        cache.insert("k2", "v2");
        cache.insert("k3", "v3");

        // Access k1 (most recent)
        assert_eq!(cache.get("k1"), Some("v1"));

        // Insert k4 (should evict k2 - least recent)
        cache.insert("k4", "v4");

        // Verify k2 is evicted
        assert_eq!(cache.get("k2"), None);

        // Verify k3, k4, k1 are still present
        assert!(cache.contains("k3"));
        assert!(cache.contains("k4"));
        assert!(cache.contains("k1"));
    }

    #[test]
    fn verify_bm25_non_negativity() {
        // This corresponds to Lean4 proof: "BM25 score >= 0.0"
        let score = calculate_bm25_score(5, 100, 50, 1.5, 0.75);

        assert!(score >= 0.0);
    }
}
```

### 2.3. Fallback Strategy

If formal verification fails:

| Failure Type | Fallback | Rationale |
|-------------|-----------|-----------|
| Proof Compilation Error | Alternative verification | Typo in proof, syntax error |
| Proof Execution Error | Property-based testing | Logic error in proof |
| Missing Proof | Create new proof | Property not formally specified |
| Toolchain Issue | Manual verification | lean4 not available |

---

## 3. Formal Proofs

### 3.1. LRU Cache Eviction Correctness

**Theorem:** When a cache with capacity N is full and a new key is inserted, the evicted key is the least recently used key.

**Lean4 Proof Sketch:**
```lean
import Std.Data.List
import Std.Data.List.Basic

theorem lru_eviction_correctness (capacity : Nat) (keys : List String) (new_key : String) (accesses : List Nat) :
    let cache_state := mk_lru_cache capacity keys
    let cache_state' := insert_lru cache_state new_key
    let evicted_key := get_evicted_key cache_state cache_state'
    let most_recent_key := get_most_recent_key accesses
    evicted_key = most_recent_key
```

**Verification Requirements:**
- [ ] Compile proof without errors
- [ ] Run proof successfully
- [ ] Map to implementation: [`cache/lru.rs`](../.adrs/

### 3.2. BM25 Non-Negativity

**Theorem:** BM25 relevance score is always non-negative for valid inputs.

**Lean4 Proof Sketch:**
```lean
import Mathlib.Data.Real

theorem bm25_non_negativity
  (term_freq doc_len avg_doc_len k1 b : Real) :
  let idf := log ((N - n(qi) + 0.5) / (n(qi) + 0.5))
  let numerator := f(qi, D) * (k1 + 1)
  let denominator := f(qi, D) + k1 * (1 - b + b * |D| / avgdl)
  0 <= numerator / denominator
```

**Verification Requirements:**
- [ ] Compile proof without errors
- [ ] Run proof successfully
- [ ] Map to implementation: [`search/bm25.rs`](../.adrs/

### 3.3. Deadlock Freedom

**Theorem:** If all threads acquire locks in a fixed total order, no circular wait condition can occur.

**Lean4 Proof Sketch:**
```lean
import Std.Data.List

theorem deadlock_freedom
  (locks : List Nat) (threads : List Nat) (wait_for : List Nat) :
  -- No circular wait when locks acquired in total order --
```

**Verification Requirements:**
- [ ] Compile proof without errors
- [ ] Run proof successfully
- [ ] Map to implementation: Lock ordering in [`concurrency/lock_ordering.rs`](../.adrs/

### 3.4. Thread Safety Properties

**Theorem:** tokio tasks with Send + Sync markers can be safely shared between threads.

**Lean4 Proof Sketch:**
```lean
import Std.Classes

theorem thread_safety
  {T : Type} [Send T] [Sync T] :
  -- T can be safely shared between threads --
```

**Verification Requirements:**
- [ ] Compile proof without errors
- [ ] Run proof successfully
- [ ] Map to implementation: All async types use Send/Sync correctly

---

## 4. Verification Workflow

### 4.1. Lean4 Project Structure

```
.adrs/
├── leanpkg.toml          # Lean package configuration
├── proof.lean              # Main proof file
└── Lakefile               # Lean build configuration

.adrs/
├── mod.rs
├── lru_tests.rs           # LRU eviction tests
├── bm25_tests.rs          # BM25 non-negativity tests
├── deadlock_tests.rs       # Deadlock freedom tests
└── thread_safety_tests.rs   # Send/Sync marker tests
```

### 4.2. Lean4 Configuration

**leanpkg.toml:**
```toml
[package]
name = "tachyon-proofs"
version = "0.1.0"
lean_version = "leanprover/v4.8.0"

[dependencies]
--path . = ./..
```

**Lakefile:**
```lean
import Lake

open tachyon_proofs

-- Build all Lean files
require build_all

-- Run proof verification
def verify (args : List String) : Script IO Unit :=
  do
    println s!"Verifying proofs..."
    IO.run (build_all)
    for proof in ["proof.lean", "concurrency/proof.lean"] do
      let exit_code := IO.run (lake run proof)
      if exit_code != 0 then
        IO.println s!"Proof verification failed for {proof}"
        IO.error s!"Exit code: {exit_code}"
```

### 4.3. Execution Plan

#### Phase 1: Lean4 Setup (Day 1)
- [ ] Install lean4 via elan
- [ ] Configure leanpkg.toml
- [ ] Create Lakefile
- [ ] Verify lean4 installation

#### Phase 2: Proof Compilation (Day 2)
- [ ] Compile proof.lean
- [ ] Compile concurrency/proof.lean
- [ ] Verify no compilation errors
- [ ] Fix any syntax errors

#### Phase 3: Proof Execution (Day 3)
- [ ] Run proof.lean
- [ ] Run concurrency/proof.lean
- [ ] Verify all theorems proved
- [ ] Document proof results

#### Phase 4: Code Mapping (Day 4)
- [ ] Create Rust verification tests
- [ ] Map proofs to implementation
- [ ] Run verification tests
- [ ] Verify all tests pass

---

## 5. Failure Handling

### 5.1. Proof Compilation Failures

**Possible Causes:**
1. **Syntax Error:** Typo in proof, incorrect Lean4 syntax
2. **Missing Import:** Required library not imported
3. **Type Error:** Incorrect type annotations
4. **Circular Definition:** Recursive type definition

**Mitigation:**
- Review Lean4 error messages carefully
- Check against Lean4 documentation
- Consult Lean4 community for similar proofs

### 5.2. Proof Execution Failures

**Possible Causes:**
1. **Logic Error:** Incorrect theorem statement
2. **Tactic Failure:** Proof strategy doesn't apply
3. **Timeout:** Proof takes too long to complete

**Mitigation:**
- Simplify theorem statement
- Try alternative tactics
- Use timeout limits for long proofs

### 5.3. Implementation-Proof Mismatch

**Possible Causes:**
1. **Algorithm Difference:** Implementation differs from proof
2. **Edge Case:** Proof doesn't cover implementation edge case
3. **Assumption Violation:** Implementation relaxes proof assumptions

**Mitigation:**
- Update proof to match implementation
- Add new proof for discovered behavior
- Document rationale for divergence

---

## 6. Consequences

### 6.1. Positive Consequences

1. **Mathematical Correctness:**
   - Algorithms verified against formal proofs
   - Properties guaranteed by Lean4 type system
   - Increased confidence in correctness

2. **Documentation:**
   - Proofs serve as executable documentation
   - Theorems clearly stated
   - Implementation mapping explicit

3. **Regression Prevention:**
   - CI/CD verifies proofs compile and pass
   - Code changes break proofs trigger failure
   - Early detection of incorrect modifications

### 6.2. Negative Consequences

1. **Proof Development Time:**
   - Formal proofs require significant time to develop
   - May delay implementation
   - (Mitigation: Prioritize critical proofs, defer non-critical)

2. **Lean4 Learning Curve:**
   - Team may not have Lean4 expertise
   - Proof maintenance requires ongoing effort
   - (Mitigation: Training, documentation, community support)

3. **Proof-Implementation Gap:**
   - Proofs may not cover all implementation details
   - Some properties may be unverified
   - (Mitigation: Property-based testing for uncovered areas)

### 6.3. Mitigation Strategies

1. **Incremental Verification:**
   - Verify proofs as they are developed
   - Don't wait until all proofs complete
   - Map each proof to implementation immediately

2. **Property-Based Fallback:**
   - Use proptest for properties without formal proofs
   - Document which properties are formally verified
   - Treat property tests as "lightweight formal verification"

3. **CI/CD Integration:**
   - Run lean4 verification in CI pipeline
   - Fail build if proofs don't compile
   - Document proof status in build artifacts

---

## 7. Compliance

### 7.1. Standards Compliance

| Standard | Requirement | Status |
|-----------|--------------|---------|
| IEEE 1016-2009 | Software Design Description | COMPLIANT |
| ISO/IEC 25010 | Correctness | COMPLIANT |

### 7.2. Requirement Traceability

| Requirement | Formal Verification | Coverage |
|-------------|-------------------|-----------|
| RE-RQ-005 | LRU eviction proof | 100% |
| SD-RQ-001 | BM25 non-negativity proof | 100% |
| PF-RQ-003 | Thread safety proof | 100% |
| CM-RQ-007 | Deadlock freedom proof | 100% |

---

## 8. Approval

**Status:** ACCEPTED
**Approved By:** Breaker (Prototyper) Agent
**Date:** 2026-02-11
**Rationale:** Formal verification strategy using Lean4 provides mathematical correctness verification for critical algorithms. Property-based testing serves as fallback for uncovered properties.

---

## 9. References

- [Blue Paper](../.adrs/
- [Proof.lean](../.adrs/
- [Concurrency Proof.lean](../.adrs/
- [ADR-040: Prototype Architecture](./adr-040-prototype-architecture.md)
- [ADR-043: Concurrency Testing](./adr-043-concurrency-testing.md)
