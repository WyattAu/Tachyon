import { initSync, render_markdown } from "@/lib/engine/tachyon_engine_wasm.js";
import wasmUrl from "@/lib/engine/tachyon_engine_wasm_bg.wasm?url";

/**
 * Markdown rendering via engine-wasm (tachyon-engine-wasm crate) — the SAME
 * renderer the SSG pipeline uses (pulldown-cmark + ammonia sanitization),
 * giving exact render parity between document view, preview, and static
 * sites. Replaces the interim regex renderer in DocumentView.
 *
 * Regenerate: cargo build -p tachyon-engine-wasm --target
 * wasm32-unknown-unknown --release && wasm-bindgen --target web --out-dir
 * crates/engine-wasm/pkg target/.../tachyon_engine_wasm.wasm
 */

let initPromise: Promise<unknown> | null = null;

function ensureInit(): Promise<unknown> {
  if (!initPromise) {
    initPromise = (async () => {
      const wasm = await fetch(wasmUrl);
      return initSync({ module: await wasm.arrayBuffer() });
    })();
  }
  return initPromise;
}

/** Sanitized markdown -> HTML, identical to the SSG pipeline. */
export async function renderMarkdown(content: string): Promise<string> {
  await ensureInit();
  return render_markdown(content);
}
