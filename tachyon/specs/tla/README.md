# TLA+ Formal Specifications — Tachyon v0.17.0

Formal verification specs for the Tachyon knowledge management system.
These specs are artifacts for future verification with TLC and TLAPS.

## Verification Status

**All specs are unverified.** TLA+ tools (TLC, TLAPS) are not installed.
To install: <https://github.com/tlaplus/tlaplus/releases>

## Specifications

### 1. OperationalTransform.tla

Real-time collaborative editing via operational transforms.

| Property | Description |
|---|---|
| Convergence | All clients eventually see the same document state |
| Intention Preservation | Transformed ops preserve logical insertion intent |
| Causality | Causal ordering of operations is respected at all clients |
| No Data Loss | All inserted characters appear in the final document |

```bash
tlc -deadlock -config OperationalTransform.cfg OperationalTransform.tla
```

### 2. ReviewStateMachine.tla

Document review workflow state machine.

| Property | Description |
|---|---|
| No Invalid Transitions | Terminal states have no outgoing transitions |
| Idempotent Cancel | Cancelling a cancelled review is a no-op |
| Approval Chain | Must go through Pending before Approved |
| Single Resolution | Only one non-terminal state at a time |

```bash
tlc -config ReviewStateMachine.cfg ReviewStateMachine.tla
```

### 3. GraphInvariants.tla

Knowledge graph structural invariants.

| Invariant | Description |
|---|---|
| INV001 No Self-Loops | `EdgeSrc(e) /= EdgeTgt(e)` |
| INV002 Weight Bounds | `0.0 <= Weight(e) <= 1.0` |
| INV003 Reversal Idempotent | Reversing an edge twice returns to original |
| INV007 Component Partition | Active nodes partition into connected components |
| INV018 Degree Sum | `Sum(degrees) = 2 * |Edges|` |

```bash
tlc -config GraphInvariants.cfg GraphInvariants.tla
```

### 4. Diff3Merge.tla

Three-way merge algorithm for conflict resolution.

| Property | Description |
|---|---|
| No Data Loss | One-sided changes always appear in merge |
| Both Same | Identical changes merge cleanly |
| Conflict Detection | Divergent same-region changes produce conflicts |
| Idempotent | `Merge(base, base, base) == base` |

```bash
tlc -config Diff3Merge.cfg Diff3Merge.tla
```

## Running TLC

Each spec needs a `.cfg` file to define model parameters:

```
---- MODULE OperationalTransform ----
CONSTANTS MaxPendingOps = 5
           MaxDocLen = 10
           CharSet = {"a", "b", "c"}
SPECIFICATION Spec
PROPERTIES TypeInvariant
```

```bash
java -jar tla2tools.jar OperationalTransform.tla
```

## Running TLAPS

TLAPS provides mechanical proofs:

```bash
tlapm OperationalTransform.tla
```

Proofs are not yet written. Each spec contains temporal property definitions
that can serve as proof obligations.
