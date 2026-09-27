/**
 * `window.__tachyonEditor` — FROZEN BRIDGE CONTRACT (ADR-113 Phase A)
 * ================================================================
 *
 * This is the seam between the editor MODEL (Rust, compiled to WASM —
 * currently the Leptos NativeEditor, future engine-wasm/editor-wasm) and
 * any VIEW (Leptos today, SolidJS tomorrow). The WASM split must keep this
 * contract bit-compatible; `scripts/editor-contract.mjs` runs these
 * assertions against whatever implementation is live.
 *
 * History: this exact bridge is what let the agent controller drive the
 * editor end-to-end (typing, auto-save round-trip to Postgres verified).
 *
 * CONSUMER: apps/web Solid Editor view (Phase B documents migration)
 * IMPLEMENTER: crates/frontend native_editor.rs (today) → engine-wasm
 *
 * ## Methods
 *
 * - `insertText(text: string): void`
 *     Insert at caret, advance caret past the insertion.
 *     MUST fire the app's change/dirty pipeline (auto-save must trigger).
 * - `key(key: string): void`
 *     Named-key dispatch. Recognized: Enter (auto-indent), Backspace,
 *     Delete, Tab (indent), Home, End, ArrowLeft/Right/Up/Down.
 *     Single characters insert literally.
 *     UNRECOGNIZED multi-char names MUST be ignored (never typed verbatim —
 *     regression: "ControlEnd" once landed in a production document).
 * - `content(): string`
 *     Full document text, including trailing state. Round-trips exactly
 *     through server persistence.
 * - `toggleFind(): void`
 * - `find(query: string): void` — start search
 * - `findNext(): void` / `findPrevious(): void`
 *
 * ## Lifecycle
 *
 * - The object appears on `window` when an editor mounts; disappear/rebuild
 *   on unmount/remount is allowed (controllers must re-read `window.__tachyonEditor`).
 * - Only ONE live instance per window (last mount wins).
 *
 * ## Out of scope (do NOT add here)
 * - Selection read/write, undo/redo, decorations — these need a designed
 *   v2 contract; extensions get a new versioned object (`__tachyonEditor2`).
 */

export interface TachyonEditorBridge {
  insertText(text: string): void;
  key(key: string): void;
  content(): string;
  toggleFind(): void;
  find(query: string): void;
  findNext(): void;
  findPrevious(): void;
}

/** Named keys with dedicated editor behavior (single chars type literally). */
export const BRIDGE_KEYS = [
  "Enter",
  "Backspace",
  "Delete",
  "Tab",
  "Home",
  "End",
  "ArrowLeft",
  "ArrowRight",
  "ArrowUp",
  "ArrowDown",
] as const;

export type BridgeKey = (typeof BRIDGE_KEYS)[number];

/** Runtime guard for controllers/tests. */
export function isBridge(value: unknown): value is TachyonEditorBridge {
  if (typeof value !== "object" || value === null) return false;
  const v = value as Record<string, unknown>;
  return (
    typeof v.insertText === "function" &&
    typeof v.key === "function" &&
    typeof v.content === "function" &&
    typeof v.toggleFind === "function" &&
    typeof v.find === "function" &&
    typeof v.findNext === "function" &&
    typeof v.findPrevious === "function"
  );
}
