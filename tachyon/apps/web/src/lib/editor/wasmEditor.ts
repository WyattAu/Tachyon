import { createSignal } from "solid-js";
import { TachyonEditor, initSync, type InitOutput } from "@/lib/editor/wasm/tachyon_editor_wasm.js";
import wasmUrl from "@/lib/editor/wasm/tachyon_editor_wasm_bg.wasm?url";
import type { TachyonEditorBridge } from "./contract";

// Generated bindings (cargo build -p tachyon-editor-wasm --target
// wasm32-unknown-unknown --release && wasm-bindgen --target web).
// Regenerate: see docs/feature-complete-v1.md Phase A.

let initPromise: Promise<InitOutput> | null = null;

async function ensureInit(): Promise<InitOutput> {
  if (!initPromise) {
    initPromise = (async () => {
      const wasm = await fetch(wasmUrl);
      const out = initSync({ module: await wasm.arrayBuffer() });
      return out;
    })();
  }
  return initPromise;
}

export type EditorHandle = TachyonEditor & TachyonEditorBridge;

/** Load the wasm module and construct an editor over `content`. */
export async function createEditor(content: string): Promise<EditorHandle> {
  await ensureInit();
  return new TachyonEditor(content) as unknown as EditorHandle;
}

/**
 * Thin reactive state bridge for the Solid view. The model lives in Rust
 * (wasm); Solid holds only render-mirrors of the fields the view shows.
 */
export function createEditorState() {
  const [lines, setLines] = createSignal<string[]>([]);
  const [cursor, setCursor] = createSignal<[number, number]>([0, 0]);
  const [dirty, setDirty] = createSignal(false);
  const [words, setWords] = createSignal(0);
  const [chars, setChars] = createSignal(0);

  return {
    lines,
    cursor,
    dirty,
    words,
    chars,
    syncFrom(handle: EditorHandle) {
      const n = handle.lineCount();
      const out: string[] = new Array(n);
      for (let i = 0; i < n; i++) out[i] = handle.line(i);
      setLines(out);
      setCursor(Array.from(handle.cursor()) as [number, number]);
      setDirty(handle.isDirty());
      setWords(handle.wordCount());
      setChars(handle.charCount());
    },
  };
}
