/* tslint:disable */
/* eslint-disable */

export function engine_version(): string;

/**
 * Extract wiki-link targets from markdown content.
 *
 * Matches `[[target]]` and `[[target|display]]`; returns unique targets
 * in order of first appearance.
 */
export function extract_wikilinks(content: string): any[];

/**
 * Render markdown to sanitized HTML.
 *
 * Pipeline: wikilink pre-processing -> pulldown-cmark -> ammonia sanitize.
 */
export function render_markdown(content: string): string;

/**
 * Convert arbitrary text into a URL-safe slug.
 *
 * Mirrors `tachyon_core::util::slugify` so client and server produce
 * identical slugs — keep the two in sync.
 */
export function slugify(input: string): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly engine_version: (a: number) => void;
    readonly extract_wikilinks: (a: number, b: number, c: number) => void;
    readonly render_markdown: (a: number, b: number, c: number) => void;
    readonly slugify: (a: number, b: number, c: number) => void;
    readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
    readonly __wbindgen_export: (a: number, b: number, c: number) => void;
    readonly __wbindgen_export2: (a: number, b: number) => number;
    readonly __wbindgen_export3: (a: number, b: number, c: number, d: number) => number;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
