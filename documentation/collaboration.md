---
title: Collaboration
description: Real-time collaboration with CRDTs and WebSockets
order: 10
tags: [collaboration, reference]
---

# Collaboration

Tachyon provides real-time collaborative editing using CRDTs (Conflict-free Replicated Data Types) over WebSocket connections.

## How It Works

```
User A                 Server                  User B
  |                      |                        |
  |-- Edit (insert) ---->|                        |
  |                      |-- Broadcast --------->|
  |                      |                        |
  |<--- Edit (delete) ---|                        |
  |                      |                        |
  |                      |<--- Edit (insert) -----|
  |<---- Broadcast ------|                        |
```

All edits are expressed as operations on a Yrs (Yjs Rust port) document. Yrs guarantees convergence: all clients eventually reach the same document state, regardless of network ordering.

## Connecting

```
WS /ws?token=<jwt>
```

Messages use JSON encoding:

```json
{
  "type": "edit",
  "document_id": "uuid",
  "update": "base64-encoded-yrs-update"
}
```

## Message Types

| Type | Direction | Description |
|------|-----------|-------------|
| `subscribe` | Client to Server | Join a document channel |
| `unsubscribe` | Client to Server | Leave a document channel |
| `edit` | Bidirectional | CRDT update (Yrs binary) |
| `cursor` | Bidirectional | Cursor position broadcast |
| `presence` | Server to Client | Online user list |
| `error` | Server to Client | Error notification |

## Operational Transform

For documents not using Yrs, Tachyon provides an operational transform (OT) engine as a fallback:

- Insert and delete operations
- Concurrent edit transformation
- UTF-8 aware offset management
- Convergence guarantees for all operation sequences

## Formal Properties

The CRDT implementation satisfies:

- **Commutativity**: Operations can be applied in any order
- **Associativity**: Grouping of operations does not affect result
- **Idempotence**: Applying an operation twice yields the same result as once

Formal proofs are available in `tachyon/specs/lean/`.

## Presence Detection

When multiple users are editing the same document, the server broadcasts presence updates:

```json
{
  "type": "presence",
  "document_id": "uuid",
  "users": [
    { "user_id": "uuid", "username": "alice", "cursor": { "line": 5, "col": 12 } },
    { "user_id": "uuid", "username": "bob", "cursor": { "line": 5, "col": 28 } }
  ]
}
```

## Further Reading

- [Architecture](architecture.html) - System design
- [Editor Guide](editor-guide.html) - Keyboard shortcuts
- [API Reference](api-reference.html) - WebSocket endpoints
