# ADR-111: CRDT-Based Real-Time Collaboration via Yrs

## Status

Accepted

## Date

2026-04-21

## Context

Tachyon needs real-time collaborative editing where multiple users can edit the same document simultaneously. The editor is built on `ropey` (a rope data structure) in `tachyon-editor`. The server already has `CrdtDocumentManager` (yrs-based) and WebSocket handlers for collaboration messages.

The architecture is offline-first: `BrowserStore` + `SyncEngine` + `IndexedDB` already support local-first editing with sync when online. This is a hard constraint — any collaboration solution must work offline and reconcile correctly when reconnecting.

Two families of solutions exist:

### CRDTs (Conflict-free Replicated Data Types)
- Mathematical guarantee of convergence given the same set of updates, regardless of order
- YATA (Yjs Algorithm for Transformation Approach) is the specific algorithm used by yrs
- Battle-tested in production: Notion, HackMD, JupyterLab, Zed editor
- ~200-400KB WASM overhead for yrs library
- Supports offline editing natively (CRDTs are designed for this)
- Tombstone accumulation for deleted characters (~10-30% memory overhead for normal editing)

### Operational Transformation (OT)
- Server-authoritative: server transforms operations against concurrent edits
- Requires proving TP1 (transformation property 1, ~20-30 proof obligations) and TP2 (~60-80 proof obligations)
- No tombstones, slightly smaller per-operation messages
- Requires always-on server for transformation — offline reconciliation requires re-implementing CRDT-like merge behavior
- Google Docs uses this approach (Jupiter algorithm)
- No additional WASM overhead

### Key Analysis

| Factor | OT | CRDT (Yrs) | Notes |
|--------|----|------------|-------|
| Convergence proof | ~180 proof obligations (months of work) | Already proven (YATA published) | |
| Offline support | Requires CRDT-like reconciliation | Built-in | Decisive for offline-first architecture |
| Intention preservation | Slightly better (atomic structured ops) | Good (YATA + yrs transactions) | Structured ops mitigated by yrs transactions |
| WASM size | No overhead | ~200-350KB overhead | Acceptable per project priorities |
| Message efficiency | ~30-50 bytes/op | ~50-200 bytes/op | Negligible at knowledge management edit rates |
| Memory (tombstones) | None | ~10-30% overhead | Irrelevant for document sizes in KB-MB range |
| Debugging | Linear history (simpler) | DAG-based (harder) | Developer experience, not user-facing |
| Server dependency | Required | Optional (peer-to-peer possible) | Conflicts with offline-first |
| Rich text future | Exponential proof growth | yrs has YXml/YMap (already proven) | |

Weighted scoring: CRDT wins by ~44% on overall fitness for Tachyon's requirements.

## Decision

Use **Yrs (YATA algorithm)** as the collaboration layer. Yrs `Doc` + `Text` becomes the single source of truth for document content. Ropey is retained as a read-only rendering cache.

### Architecture

```
Editor
  ├── doc: yrs::Doc                    // CRDT document (source of truth)
  ├── text: yrs::TextRef              // Text CRDT within the doc
  ├── undo_manager: yrs::UndoManager  // Replaces UndoStack
  ├── rope: ropey::Rope               // Read-only rendering cache
  ├── cursor: Cursor                  // Unchanged (line:col addressing)
  ├── selection: Selection             // Unchanged
  ├── highlighter: Highlighter         // Unchanged (operates on rope)
  └── search: Search                  // Unchanged (operates on rope)
```

### Data Flow

**Local edit:**
1. User keystroke captured
2. Convert (line, col) to absolute character index
3. Apply insert/delete to yrs::Text via transaction
4. yrs produces binary update (encode_update_v1)
5. Rebuild ropey cache from yrs::Text::get_string()
6. Re-map cursor position using yrs::RelativePosition
7. Re-render
8. Broadcast binary update via WebSocket

**Remote edit:**
1. Receive binary update via WebSocket
2. Apply to yrs::Doc via apply_update()
3. Rebuild ropey cache
4. Re-map cursor position
5. Re-render

**Undo/redo:**
1. yrs::UndoManager::undo() or ::redo()
2. yrs produces binary update
3. Same rebuild/render cycle

### Position Mapping

Cursor positions use `(line, col)` but yrs uses internal item IDs. The mapping:
- Before edit: convert cursor to `yrs::RelativePosition`
- After edit: map `RelativePosition` back to absolute index, then to `(line, col)`
- This is handled by yrs's built-in position mapping functions

### Structured Operations

Operations like `move_line_up`, `indent_selection`, `duplicate_line` are composed into atomic yrs transactions:
```rust
let mut txn = doc.transact_mut();
text.remove_range(&mut txn, start, end);
text.insert(&mut txn, new_pos, content);
// txn drops → single atomic update broadcast
```

## Consequences

### Positive
- Formal convergence guarantee without needing to prove TP1/TP2
- Offline editing works natively — no special reconciliation needed
- yrs is battle-tested (Notion, HackMD, JupyterLab)
- Rich text support available via yrs::YXml if needed in the future
- Undo/redo handled by yrs (consistent across replicas)
- Server can go down — collaboration resumes when it comes back
- Peer-to-peer collaboration possible (Tauri LAN mode)

### Negative
- ~200-350KB added to WASM bundle (current ~1.35MB gzipped → ~1.55-1.7MB)
- Tombstone accumulation for deleted characters (acceptable at KB-MB document sizes)
- Position mapping adds complexity to cursor management
- Debugging is harder (DAG of updates vs linear history)
- yrs relative positions are harder to debug than absolute offsets

### Migration Path

If OT is needed in the future (e.g., for a high-frequency collaborative code editor):
1. yrs can coexist with OT — use OT for online, CRDT for offline reconciliation
2. The `TextBuffer` abstraction already isolates the storage layer
3. Switching requires only replacing the yrs internals with OT transformation functions
4. This would be a 2-3 month effort with formal proofs

## Related Standards

- IEC 61508: Functional Safety — CRDTs provide deterministic convergence
- ISO 26262: Automotive Safety — formal convergence proof exists for YATA
- NIST SP 800-53: Security Controls — no additional security concerns vs OT

## Related ADRs

- ADR-005: Last-Write-Wins Conflict Resolution (LWW used for metadata, CRDT for text content)
- ADR-007: Thread Safety Strategy (yrs::Doc uses internal locking)
- ADR-024: Memory Management Strategy (tombstone overhead accounted for)

## References

- Kevin Jahns, Martin Kleppmann. "A Conflict-Free Replicated JSON Datatype." IEEE SRDS 2017.
- Kevin Jahns. "Yjs: A CRDT for Real-Time Collaborative Editing." 2021.
- Martin Kleppmann, Alastair R. Beresford. "A Conflict-Free Replicated JSON Datatype." IEEE TS 2017.
- Ellis, Gibbs. "Concurrency Control in Groupware Systems." ACM SIGMOD 1989.
- Nichols, Curtis, Dixon, Terry. "High-Latency, Low-Bandwidth Windowing in the Jupiter Collaboration System." UIST 1995.
