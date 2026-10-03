import { A } from "@solidjs/router";
import {
  createEffect,
  createResource,
  createSignal,
  For,
  Show,
} from "solid-js";
import { useNavigate, useParams } from "@solidjs/router";
import { api } from "@/lib/api/client";
import {
  generateDocumentKey,
  keyFingerprint,
  encryptContent,
  decryptContent,
  exportKey,
  importKey,
  type EncryptedPayload,
} from "@/lib/e2ee";
import { e2eApi } from "@/lib/e2eApi";
import { createEditor, createEditorState, type EditorHandle } from "@/lib/editor/wasmEditor";

/**
 * Document editor — Solid view over the WASM editor model (Phase B).
 *
 * Model: tachyon-editor compiled to wasm (crates/editor-wasm), implementing
 * the FROZEN bridge contract (src/lib/editor/contract.ts). The view renders
 * model mirrors and routes key events into the model; every mutation is a
 * model call, then a state re-sync. Save: PUT /documents/:id (staging),
 * Ctrl+S manual, autosave to follow.
 */

export default function DocumentEditor() {
  const params = useParams();
  const navigate = useNavigate();
  const docId = () => params.id ?? "";

  const [handle, setHandle] = createSignal<EditorHandle | null>(null);
  const state = createEditorState();
  const [loadError, setLoadError] = createSignal<string | null>(null);
  const [saveState, setSaveState] = createSignal<"idle" | "saving" | "saved" | "error">("idle");
  const [saveMessage, setSaveMessage] = createSignal("");
  const [booted, setBooted] = createSignal(false);
  const [encrypted, setEncrypted] = createSignal(false);
  const [docKey, setDocKey] = createSignal<CryptoKey | null>(null);

  // Load document content, then boot the wasm model over it.
  const [doc] = createResource(docId, async (id) => {
    const { data, error, response } = await api.GET("/documents/{document_id}", {
      params: { path: { document_id: id } },
    });
    if (error || !data) throw new Error(`load failed (${response.status})`);
    return data as unknown as { title: string; content: string };
  });

  // Boot the wasm editor once the document resource resolves — sourced on
  // `doc` so it cannot race the fetch (an onMount read returns undefined).
  const [boot] = createResource(doc, async (d) => {
    let content = d?.content ?? "";
    if (content.startsWith('{"iv"')) {
      const key = await loadKeyFromLocal();
      const payload = JSON.parse(content) as EncryptedPayload;
      setEncrypted(true);
      if (key) {
        content = await decryptContent(key, payload);
      } else {
        content = "";
        setLoadError("Document is encrypted and the key is not on this device.");
        setBooted(true);
        return null;
      }
    } else {
      // server-side status check (registered key without local payload state)
      const status = await e2eApi.status(docId());
      setEncrypted(status.encrypted);
    }
    const h = await createEditor(content);
    setHandle(h);
    state.syncFrom(h);
    setBooted(true);
    return h;
  });

  const resync = () => {
    const h = handle();
    if (h) state.syncFrom(h);
  };

  const jwkKey = () => `e2e:doc:${docId()}`;
  const loadKeyFromLocal = async (): Promise<CryptoKey | null> => {
    const raw = localStorage.getItem(jwkKey());
    if (!raw) return null;
    try {
      return await importKey(JSON.parse(raw));
    } catch {
      return null;
    }
  };

  /** Enable encryption: generate key, register fingerprint, store JWK locally. */
  const enableEncryption = async () => {
    const h = handle();
    if (!h) return;
    const key = await generateDocumentKey();
    const fingerprint = await keyFingerprint(key);
    const meta = await e2eApi.registerKey(docId(), "AES-256-GCM", fingerprint);
    if (!meta) {
      setSaveState("error");
      setSaveMessage("key registration failed — is the server e2e API up?");
      return;
    }
    localStorage.setItem(jwkKey(), JSON.stringify(await exportKey(key)));
    setDocKey(key);
    setEncrypted(true);
  };

  const disableEncryption = async () => {
    const h = handle();
    if (!h) return;
    // decrypt in place (content already plaintext in the model), drop key
    await e2eApi.deleteKey(docId());
    localStorage.removeItem(jwkKey());
    setDocKey(null);
    setEncrypted(false);
  };

  /** Returns content to persist — encrypted payload JSON when locked. */
  const contentToSave = async (): Promise<string> => {
    const h = handle();
    if (!h) return "";
    const plain = h.content();
    let key = docKey();
    if (!encrypted()) return plain;
    if (!key) {
      key = await loadKeyFromLocal();
      if (!key) throw new Error("encryption key missing on this device");
      setDocKey(key);
    }
    const payload = await encryptContent(key, plain);
    return JSON.stringify(payload);
  };

  /** Route a KeyboardEvent into the model per the frozen contract. */
  const onKey = (e: KeyboardEvent) => {
    const h = handle();
    if (!h) return;
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "s") {
      e.preventDefault();
      void save();
      return;
    }
    const named = [
      "Enter", "Backspace", "Delete", "Tab",
      "Home", "End", "ArrowLeft", "ArrowRight", "ArrowUp", "ArrowDown",
    ];
    if (e.ctrlKey || e.metaKey || e.altKey) return; // browser shortcuts pass through (v1)
    if (named.includes(e.key)) {
      e.preventDefault();
      h.key(e.key);
      resync();
    } else if (e.key.length === 1) {
      e.preventDefault();
      h.insertText(e.key);
      resync();
    }
    // multi-char non-named keys (IME, dead keys): browser default (v1)
  };

  const save = async () => {
    const h = handle();
    if (!h) return;
    setSaveState("saving");
    try {
      const d = doc();
      const { error, response } = await api.PUT("/documents/{document_id}", {
        params: { path: { document_id: docId() } },
        body: { title: d?.title ?? "Untitled", content: await contentToSave() } as never,
      });
      if (error) throw new Error(`save failed (${response.status})`);
      h.markSaved();
      resync();
      setSaveState("saved");
    } catch {
      setSaveState("error");
    }
  };

  // Keep the title in the tab while editing
  createEffect(() => {
    if (doc()) document.title = `${doc()!.title} — Tachyon`;
  });

  return (
    <div class="mx-auto max-w-4xl px-4 py-4">
      <Show when={loadError() || (boot.error ? String(boot.error) : null)}>
        <div class="rounded-none border border-red-300 bg-red-50 p-3 text-sm text-red-700 dark:border-red-800 dark:bg-red-900/20 dark:text-red-300">
          {loadError() || String(boot.error)} —{" "}
          <A class="underline" href="/documents">
            back to documents
          </A>
        </div>
      </Show>

      <Show when={booted()}>
        <div class="mb-2 flex items-center justify-between">
          <h1 class="font-display truncate text-xl font-bold text-gray-900 dark:text-white">
            {doc()?.title ?? "Untitled"}
          </h1>
          <button
            class={`rounded-none px-3 py-1.5 text-sm ${encrypted() ? "bg-green-700 text-white" : "border border-gray-300 hover:bg-gray-100 dark:border-gray-600 dark:hover:bg-gray-700"}`}
            title={encrypted() ? "Encryption ON (key on this device)" : "Enable E2E encryption"}
            onClick={() => void (encrypted() ? disableEncryption() : enableEncryption())}
          >
            {encrypted() ? "🔒 Encrypted" : "🔓 Encrypt"}
          </button>
          <button
            class="rounded-none bg-blue-600 px-3 py-1.5 text-sm text-white hover:bg-blue-700"
            onClick={() => void save()}
          >
            Save
          </button>
        </div>

        {/* editor surface: line numbers + lines + cursor */}
        <div
          class="overflow-auto rounded-none border-2 border-gray-900 bg-gray-950 p-3 font-mono text-sm text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500 dark:border-gray-100"
          tabindex="0"
          data-testid="editor-surface"
          onKeyDown={onKey}
        >
          <div class="flex">
            <div class="w-10 flex-shrink-0 select-none pr-2 text-right text-gray-600">
              <For each={state.lines()}>{() => <div>&nbsp;</div>}</For>
            </div>
            <div class="flex-1 whitespace-pre-wrap break-words">
              <For each={state.lines()}>
                {(line, i) => (
                  <div class="min-h-[1.4em]">
                    <span class="mr-3 inline-block w-6 select-none text-right text-gray-600">
                      {i() + 1}
                    </span>
                    <Show
                      when={state.cursor()[0] === i()}
                      fallback={<span>{line}</span>}
                    >
                      <span>{line.slice(0, state.cursor()[1])}</span>
                      <span class="animate-pulse bg-blue-400 text-blue-400">|</span>
                      <span>{line.slice(state.cursor()[1])}</span>
                    </Show>
                    <Show when={line.length === 0 && state.cursor()[0] !== i()}>
                      &nbsp;
                    </Show>
                  </div>
                )}
              </For>
            </div>
          </div>
        </div>

        {/* status bar */}
        <div data-testid="editor-status" class="mt-1 flex items-center justify-between rounded-none border border-gray-700 bg-gray-900 px-3 py-1.5 text-xs text-gray-400">
          <span class="flex items-center gap-2">
            <span
              class={`inline-block h-2 w-2 rounded-full ${state.dirty() ? "bg-yellow-400" : "bg-green-500"}`}
            />
            Ln {state.cursor()[0] + 1}, Col {state.cursor()[1] + 1}
          </span>
          <span>
            {state.words()} words · {state.chars()} chars
          </span>
          <span>
            {saveState() === "saving"
              ? "Saving…"
              : saveState() === "saved"
                ? "Saved"
                : saveState() === "error"
                  ? saveMessage() || "Save failed"
                  : "Ctrl+S to save"}
          </span>
        </div>

        <p class="mt-2 text-center text-xs text-gray-500">
          Model: tachyon-editor compiled to WASM · contract:{" "}
          <code>src/lib/editor/contract.ts</code> ·{" "}
          <button class="underline" onClick={() => navigate(`/documents/${docId()}`)}>
            view document
          </button>
        </p>
      </Show>
    </div>
  );
}

