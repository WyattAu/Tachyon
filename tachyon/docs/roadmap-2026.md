# Engineering Roadmap 2026 (loop-derived)

> Derived from a repo audit + a literature/protocol research pass (Oct 2026).
> Supersedes the frozen `feature-complete-v1.md` ordering, which had drifted:
> several rows it lists as open were already shipped, while the collaboration
> core was marked done on the strength of a *schema and a route* that no client
> ever used. This file is the working roadmap; `feature-complete-v1.md` stays as
> the FC gate record.

## Loop

`ANALYZE → RESEARCH → ROADMAP → IMPLEMENT → TEST → VERIFY → DOGFOOD → repeat`

A row closes only with a verification artifact (test output, E2E run, or
measured number) recorded next to it. No exceptions for "it compiles".

## Findings from the audit

- Collab was **structurally present, functionally absent**: rooms existed, the
  text projection worked, but there was no awareness, no offline path, no
  per-client undo scoping, and connect shipped *full* state (O(doc) per
  connection — state-based anti-entropy where the protocol calls for a
  state-vector diff).
- The FC checklist had rows marked open that were in fact done (graph, canvas,
  admin pages, blog, onboarding, flashcards), and rows marked done whose
  evidence was weak. Roadmap rebuilt from code, not from the checklist.
- Stack is *behind* 2026 SOTA in Postgres (16 vs 18), Tauri (pre-2.12),
  Playwright, and testing discipline (no CRDT property/simulation testing).

## Research inputs

- yrs ships `sync::Awareness` and `sync::Protocol` in-core: presence is a
  capability we already own the hard half of. Loro offers nothing equivalent
  in Rust — staying on yrs is the right call for a collab product.
- `y-protocols/PROTOCOL.md` is normative: `messageSync=0`,
  `messageAwareness=1`; sync subtypes `SyncStep1=0` (state vector),
  `SyncStep2=1` (diff vs peer SV), `Update=2`. Awareness is a *separate*
  state-based CRDT: apply iff incoming clock > local, 30s expiry, ≤15s
  re-broadcast, `state=null` on disconnect.
- Awareness payloads are **unauthenticated**; identity must be bound at the
  handshake, above the protocol. Read-only enforcement is a two-byte check and
  is UX, not security.
- CRDT text overhead is 14–42× raw text before tombstones; `encode_v2` /
  `decode_v2` roughly halves it. `gc=true` (the default) discards deleted
  content, so version history and cross-restart undo require `gc=false` rooms
  **plus** compaction you own.
- `clientID` collisions permanently corrupt a document — never derive clientID
  from user/session identity.
- CRDT correctness in 2026 is tested with proptest convergence invariants,
  deterministic simulation against a shadow model with per-step assertions, and
  bounded proofs for merge commutativity/idempotence. `proptest-lockstep`
  (Lean-verified `EventualConsistencyModel`) is purpose-built for this.
- Projecting text into `documents.content` is a *snapshot*, not state: search,
  export and agents read that column and cannot participate in merges. Agent
  writes must go through the CRDT.
- E2EE and CRDT compose only via encrypted snapshots + encrypted updates with
  plaintext causal metadata (secsync). Server-side search over encrypted docs is
  forfeited by construction — a product decision, not an oversight.

## Roadmap

### W1 — Collaboration core made correct (P0) — CLOSED
1. [x] **Presence/awareness, end to end.** Server-owned per-room presence
   (clock ordering, `null`-state removal, per-connection clientID ownership,
   30s expiry sweep), client avatars + caret publishing, wire codec verified
   byte-identical to yrs'. Evidence: `a078453`, `864589b`, `b4b50c0` (the
   get_or_load pool stall that delayed frames by seconds), 9 presence tests +
   `verify-awareness-codec.mjs` + `presence-proto.mjs`. Handshake authz is the
   open residue (see below).
2. [x] **Room lineage is the single content authority** (this absorbed the
   SyncStep2 row's real risk): clients boot empty from the room, the server
   seeds cold rooms from `documents.content` exactly once, and nothing ever
   pushes a full-state doc into the room again — the duplication bug is dead
   and asserted against by exact-equality E2E. Evidence: `1d5608c`. A true
   SV-diff exchange remains as a *bandwidth* optimization only.
3. [x] **Edits and undo land as incremental CRDT ops.** The whole-rope mirror
   (remove-all + insert-all per keystroke) is gone; undo/redo inverses go
   through the room. A latent word-delete transaction-origin bug was caught by
   the property suite. Evidence: `8f81e2a`, `94a1e4b`, 167 unit + 5 property
   tests (`crdt_simulation.rs`). yrs::undo::UndoManager (origin-scoped
   collaborative undo) is the deeper follow-up.
4. [x] **Offline queue.** Frames persist to localStorage (2 MiB cap),
   exponential-backoff reconnect, suspend/resume, reload-survival E2E. Evidence:
   `d368b06`, `offline-e2e.mjs` 9/9.
5. [x] **Durability on last disconnect.** Room flushes immediately when the
   last client leaves (timer remains the safety net): REST/search/SSG lag
   dropped from ~27s to <3s. Evidence: `28fd7ea`.

Residue resolved: WS handshake authz closed in `8d19318` (research pitfall
#7) — the upgrade requires and validates a JWT, verified 401-without /
presence-with. Still open: `yrs::undo::UndoManager` for origin-scoped
collaborative undo, and the SyncStep2 SV-diff exchange as a bandwidth
optimization.

### W2 — Correctness gates (P0, cheap, prevents data loss)
5. **CRDT simulation tests.** proptest convergence invariants + a shadow-model
   DST harness over `[Insert, Delete, Sync, Disconnect, Reconnect]` with
   per-step assertions, plus retained regression cases for shrunk failures.
6. **v2 update encoding** for ~2× state size, with a size-ratio metric.
7. **Room lifecycle + compaction.** `gc=false` rooms, periodic snapshot rows
   (additive-row model), bounded room count, flush-on-last-disconnect instead of
   a bare 30s timer.

### W3 — Documents domain gaps the FC list exposes
8. Markdown preview + TOC (every keystroke feature still missing).
9. Versions list/diff/restore from room snapshots.
10. Comments + reviews, anchored via relative positions (`StickyIndex` /
    `Quotable`) — non-negotiable before comment anchoring ships.
11. Attachments upload UI (server side is fixed; the client path is not proven).

### W4 — Stack currency (P0 by value/churn ratio)
12. [x] **Postgres 16 → 18.3** — executed as dump/restore to a fresh cluster
    (`tachyon-pg18`), verified 145 documents + users + crdt state before the
    swap, service unit repointed. Server needed zero code changes; health,
    login and document totals verified live against 18.3. (pg_dumpall needed
    `-U tachyon` — the app's DB role — and the restore must run as the new
    cluster's superuser because the dump creates the role itself.)
13. [x] **Tauri 2.12** — resolved 2.12.1 from the old 2.0.0-beta pin,
    desktop crate checks clean (`e5d9c2b`).
14. [x] **Playwright 1.64** — trace retain-on-failure + trace CLI for flake
    diagnosis; collab E2E 8/8 green on the new headless shell (`ad1eb5e`,
    `2ce3076`). `oha` beside k6 still to add.
15. [x] **Tailwind 4.3.3** — pinned explicitly; AOT CSS compiles (28.7 kB),
    E2E green (`2ce3076`). `@container-size` adoption for editor panes still
    to do.
16. Astro → 7.x for Vite 8/Rolldown build speed. Deferred behind W1–W4 so a
    regression is easy to bisect.

### W5 — Release (Phase F/G)
17. k6: 100 VUs sustained, p95 target, 0 errors on core paths.
18. Backup/restore DRILL executed (restore.sh has never been run).
19. Docker image builds; desktop installers (deb/AppImage/NSIS); CLI binaries.
20. Docs site dogfooding our own SSG; CHANGELOG, semver, signed v1.0.0.

## Deferred on purpose (and why)

- Solid 2.0 (RC, not stable), SolidStart (retired), axum 0.9 (no timeline),
  PG 19 (not GA, actively shrinking), Tauri 3 ("long time off"), Component
  Model 1.0 (needs ≥2 browser engines; also Memory64 is Safari-blocked and
  threads need COOP/COEP). Re-review triggers: **Solid 2.0 stable + start
  mode**, **PG 19 GA**.
- Loro/Diamond Types migration: justified only if we need movable blocks
  (Loro) or plain-text-only Eg-walker docs (Diamond Types, unlicensed). Not now.
- Dropping Astro for Solid-on-Vite-8: the right target architecture, but only
  once Solid 2 start mode is stable.

## Companion repos (dogfooding)

Goal: other projects consume Tachyon so the stack is exercised by real callers,
not only by our own fixtures.

- `tachyon-plugin-registry` — remote marketplace registry, consumed by the
  marketplace UI (also closes the "marketplace is an unmounted stub" row).
- `tachyon-templates` — template packs applied through the templates API.
- `tachyon-import-fixtures` — Obsidian vault / Notion / Confluence fixtures used
  by import E2E.