//! Formal verification proof sketches
//!
//! Contains Lean4 proof sketches for key system invariants.
//! These are NOT compiled as part of the Rust crate (Lean4 is a separate tool).
//! They serve as documentation and can be verified independently with:
//!
//! ```bash
//! lean --make .adrs/
//! ```
//!
//! VERIFICATION PENDING: Environment missing Lean 4
//!
//! Available proofs:
//! - `crdt_convergence.lean` -- Proves that concurrent CRDT edits converge to identical state
