# Real-Time Collaboration v2: CRDT Persistence Design (G.2)

## 1. Overview

The current collaboration system operates entirely in-memory through WebSocket connections. This document specifies the transition to a CRDT-based persistence layer backed by PostgreSQL, enabling offline-first editing, automatic conflict resolution, real-time cursor and presence sharing, and full document history. The goal is to decouple document state from session lifetime while preserving sub-100ms latency for concurrent edits.

## 2. Current State

- **Sync mechanism**: WebSocket-based operational transform (OT)
- **Document state**: Held in-memory per server instance; lost on restart
- **Persistence**: Only the final document text is saved to PostgreSQL on explicit save/close
- **Offline support**: None. Disconnection loses unsynchronized edits.
- **Presence**: Basic awareness of connected users, no cursor positions or selections
- **History**: No operation log; diff is computed on demand against last saved snapshot

Key limitations: OT requires a central authority server, does not handle offline merges, and cannot reconstruct edit history without an operation log.

## 3. CRDT Selection

### Candidates Evaluated

| Criterion            | Yjs                                  | Automerge                                | Automerge-repo                        |
|----------------------|--------------------------------------|------------------------------------------|---------------------------------------|
| Language             | JavaScript                           | Rust (with WASM/JS bindings)            | Rust (network layer for Automerge)    |
| Maturity             | Production-proven (Figma, Notion)    | Stable, academic lineage                 | Emerging, tightly coupled to Automerge|
| Binary format size   | ~30% overhead over plaintext         | ~3-5x overhead for small docs, improves  | Same as Automerge                     |
| Performance          | Fast in JS runtime                   | Fast in Rust; WASM has bridging cost     | Depends on Automerge performance      |
| Offline merge        | Supported via encoding/decoding      | First-class support                      | Built-in sync protocol                |
| Ecosystem            | Rich bindings (React, ProseMirror)   | automerge-wasm, automerge-codemirror6    | Storage adapters, sync protocols      |

### Recommendation

- **Backend (Rust)**: Automerge. Native Rust integration avoids WASM bridging overhead on the server. Binary document format maps directly to PostgreSQL `bytea` storage.
- **Frontend (TypeScript)**: Yjs for the editor binding layer (ProseMirror/CodeMirror). Bridge Yjs document state to Automerge on the server via a binary encoding translation layer.
- **Rationale**: Yjs has superior editor integrations and lower memory overhead in the browser. Automerge provides stronger guarantees on the server and a compact binary format for storage. The bridge layer converts between the two CRDT encodings at the sync boundary.

## 4. Persistence Model

### Storage Schema

```sql
CREATE TABLE crdt_documents (
    document_id   UUID PRIMARY KEY,
    workspace_id  UUID NOT NULL REFERENCES workspaces(id),
    binary_state  bytea NOT NULL,
    version       bigint NOT NULL DEFAULT 0,
    snapshot_at   bigint NOT NULL DEFAULT 0,
    created_at    timestamptz NOT NULL DEFAULT now(),
    updated_at    timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE crdt_operations (
    id            BIGSERIAL PRIMARY KEY,
    document_id   UUID NOT NULL REFERENCES crdt_documents(document_id),
    op_data       bytea NOT NULL,
    seq           bigint NOT NULL,
    actor_id      text NOT NULL,
    created_at    timestamptz NOT NULL DEFAULT now(),
    UNIQUE (document_id, seq)
);

CREATE INDEX idx_crdt_ops_doc_seq ON crdt_operations (document_id, seq);
```

### Write Path

1. Client sends CRDT incremental update (binary patch) to server.
2. Server applies patch to in-memory Automerge document.
3. Server appends operation to `crdt_operations` (append-only log).
4. If operation count since last snapshot exceeds threshold N (configurable, default 1000):
   - Serialize full Automerge document to binary.
   - Update `binary_state` and `snapshot_at` in `crdt_documents`.
   - Mark operations prior to `snapshot_at` as eligible for compaction.

### Read Path

1. Load `binary_state` from `crdt_documents`.
2. Load all operations from `crdt_operations` where `seq > snapshot_at`.
3. Reapply operations to reconstruct current state.
4. If no operations pending, use `binary_state` directly (zero-replay path).

### Blob Size Management

- Snapshot threshold N is configurable per workspace.
- Background compaction job runs daily to prune operations older than 7 days that precede a snapshot.
- Expected steady-state: `binary_state` at 2-5x plaintext size; operations log bounded by snapshot frequency.

## 5. Offline-First Architecture

### Client State

- Local CRDT document maintained in browser IndexedDB via Automerge-repo or Yjs IndexedDB provider.
- All edits applied locally first, then queued for sync.

### Sync Protocol

```
Client (reconnects)                    Server
     |                                    |
     |--- sync-state (client hash) ----->|
     |                                    |
     |<-- server-state (server hash) ----|
     |                                    |
     |--- missing-ops (binary patches) ->|
     |<-- missing-ops (binary patches) --|
     |                                    |
     |----------- ACK ------------------>|
```

1. On reconnect, client sends its document hash and vector clock.
2. Server compares with its vector clock; both sides exchange missing operations.
3. Automerge merge on both sides converges to identical state.
4. ACK confirms sync completion.

### Conflict Resolution Strategy

- **Text content**: Automerge handles conflicts via concurrent character insertion with stable ordering. No data loss.
- **Metadata (title, tags, permissions)**: Last-writer-wins (LWW) register. Timestamp from the authoritative server clock.
- **Structural operations (heading changes, list reordering)**: Automerge sequence semantics preserve intent under concurrent edits.

## 6. Cursor and Presence

Presence data is ephemeral and NOT persisted to PostgreSQL. It uses a separate PubSub channel to avoid coupling with the CRDT document state.

### Protocol

- **Cursor update**: `{ actor_id, position: { line, column }, selection?: { start, end } }`
- **Debounce**: Client-side 50ms debounce before sending cursor updates to avoid flooding the channel.
- **Presence heartbeat**: Client sends heartbeat every 5 seconds. Server marks actor as absent after 15 seconds without heartbeat (3x tolerance).
- **Broadcast**: Server fans out presence updates to all subscribers of the document channel.
- **Storage**: In-memory on server (Redis in multi-instance deployments) with TTL matching the heartbeat timeout.

### Separation from CRDT

Cursors and presence are independent of the CRDT document state. They do not affect the binary document blob and are not included in snapshots or operation logs.

## 7. Document History

The append-only operations log in `crdt_operations` provides the foundation for history features.

### Capabilities

- **Full replay**: Reconstruct document state at any point by loading the snapshot preceding that point and replaying operations up to the target sequence number.
- **Diff generation**: Use Automerge's `automerge-diff` to compute changes between any two sequence numbers, producing a human-readable diff.
- **Point-in-time restoration**: Fork a document at a given sequence number, creating a new CRDT document branch.
- **Actor attribution**: Each operation is tagged with `actor_id`, enabling per-user edit tracking.

### Storage Trade-off

History retention is bounded by the compaction policy. Operations older than 7 days that precede a snapshot are pruned. For longer retention, snapshots must be preserved (incremental cost in storage). Configurable per workspace.

## 8. Implementation Priority

| Phase | Feature               | Duration | Dependencies                         |
|-------|-----------------------|----------|--------------------------------------|
| P1    | CRDT persistence      | 2 weeks  | PostgreSQL schema, Automerge backend |
| P2    | Offline sync          | 1.5 weeks| P1, IndexedDB client adapter         |
| P3    | Cursor and presence   | 1 week   | P1, PubSub channel                    |
| P4    | Document history      | 1.5 weeks| P1, diff/replay implementation       |

**Total estimated duration**: 6 weeks.

P1 is the critical path. All subsequent phases depend on a stable CRDT persistence layer. P3 can proceed in parallel with P2 since presence is decoupled from the CRDT document state.
