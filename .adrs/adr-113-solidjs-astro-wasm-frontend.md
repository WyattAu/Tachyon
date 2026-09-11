# ADR-113: Frontend Migration to SolidJS + Astro with Dedicated WASM Engine

## Status
Accepted

## Context

The current frontend is Leptos 0.8 (Rust compiled to WASM, built with Trunk).
Operational experience has exposed structural costs that compound over time:

| Dimension | Leptos (current) | SolidJS + Astro + WASM engine |
|---|---|---|
| Bundle size | ~24 MB WASM | ~7 KB SolidJS + 1.9 MB engine WASM |
| Iteration speed | 5-6 min trunk builds, no HMR | Vite HMR (instant) |
| Ecosystem | Small, niche | Full npm |
| Cross-project reuse | None (isolated) | Shared UI + shared engine packages |
| Type safety | End-to-end Rust | Rust-to-TS via codegen (ts-rs/specta) |

The Rust *core* is the project's actual differentiator: the editor
(`tachyon-editor`), CRDT sync (yrs), markdown pipeline, and graph algorithms.
The view layer is not. Separating them lets the view layer adopt the same stack
used by the other WyattAu frontends (SolidJS + Astro + dedicated WASM), enabling
shared component libraries and a consistent pattern across projects.

A working proof exists: `tachyon-engine-wasm` compiles to a 1.9 MB WASM binary
with a fully typed TS API (`render_markdown`, `extract_wikilinks`, `slugify`),
demonstrating the engine-boundary approach.

## Decision

1. **Keep the Rust core in Rust.** The editor, CRDT, markdown pipeline, and
   graph algorithms stay in Rust, exposed to the web via `wasm-bindgen` as the
   `tachyon-engine-wasm` crate (npm name: `@wyattau/tachyon-engine`).

2. **Migrate the view layer to SolidJS**, hosted in an Astro application shell
   (`apps/web` in this repository):
   - Astro (SSG) for public surfaces: landing page, docs, blog, download links.
     Static output is deployable to CloudFlare Pages.
   - SolidJS SPA segments for the authenticated application: documents, editor,
     search, graph.
   - The same SolidJS UI is embedded in the Tauri desktop shell (replacing the
     current Leptos-in-webview arrangement).

3. **Strangler-fig migration, not big-bang.** The Leptos frontend remains
   functional until the SolidJS frontend reaches feature parity:
   - Phase 1: Engine boundary (`tachyon-engine-wasm`) — done.
   - Phase 2: `apps/web` Astro shell + landing/docs (parallel to existing site).
   - Phase 3: Typed API clients (ts-rs or specta-typescript on server route types).
   - Phase 4: Port app pages in order: login -> documents -> search -> graph ->
     remaining pages.
   - Phase 5: Retire Leptos frontend; Tauri switches to SolidJS build.

4. **Known engine-boundary gaps (tracked):**
   - `docs-pipeline`'s `katex` dependency requires an engine feature
     (quick-js native / wasm-js on wasm32) that cannot be split per-target from
     the consumer side. Until docs-pipeline gates its katex engine per-target,
     the WASM engine renders markdown without server-side LaTeX; LaTeX is left
     in the HTML for client-side KaTeX JS (standard frontend pattern).
   - `tachyon-core` is wasm-hostile (git2/tokio/notify). `slugify` is mirrored
     in the engine crate; extract to a tiny shared crate when feasible.

## Consequences

- Two languages at the view layer (TypeScript + Rust WASM); API types must be
  generated (Phase 3) to preserve type safety at the boundary.
- Build iteration for frontend work drops from minutes to milliseconds.
- Bundle size drops from ~24 MB to ~2 MB total.
- Public site becomes CloudFlare Pages-deployable static output; the
  authenticated app remains self-hosted (Docker/native).
- Component patterns become reusable across WyattAu projects.
- Two frontends must be maintained during the transition window.

## Alternatives Considered

- **Stay on Leptos:** avoids migration cost but keeps 24 MB bundles, minute-scale
  builds, and zero ecosystem reuse. Rejected.
- **Big-bang rewrite:** cleaner end state but long period with no shippable
  frontend. Rejected in favor of strangler-fig.
- **React/Vue instead of SolidJS:** larger ecosystems but diverges from the
  SolidJS pattern already established across WyattAu projects. Rejected.
