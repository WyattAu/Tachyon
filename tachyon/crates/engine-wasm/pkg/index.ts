// Re-export the wasm-bindgen generated bindings with a typed init helper.
// Usage:
//   import { initTachyonEngine, renderMarkdown } from "@wyattau/tachyon-engine";
//   await initTachyonEngine();            // fetches/instantiates the .wasm
//   const html = renderMarkdown("# Hi");  // sync after init
import init, {
  engine_version,
  extract_wikilinks,
  render_markdown,
  slugify,
} from "./tachyon_engine_wasm.js";

export { engine_version, extract_wikilinks, render_markdown, slugify };

let initialized = false;

export async function initTachyonEngine(input?: RequestInfo | URL | Response | BufferSource): Promise<void> {
  if (initialized) return;
  await init(input);
  initialized = true;
}

export function isInitialized(): boolean {
  return initialized;
}
