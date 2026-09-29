import { For, Show, createSignal } from "solid-js";
import { api } from "@/lib/api/client";

/**
 * Import — Solid port (Phase B). Obsidian vault import via the multipart
 * endpoint; Notion/Confluence land in Phase C with their callback flows.
 */

export default function Import() {
  const [file, setFile] = createSignal<File | null>(null);
  const [state, setState] = createSignal<"idle" | "importing" | "done" | "error">("idle");
  const [message, setMessage] = createSignal("");

  const importVault = async () => {
    const f = file();
    if (!f) return;
    setState("importing");
    try {
      const body = new FormData();
      body.append("file", f);
      const res = await fetch("/api/v1/import/obsidian", {
        method: "POST",
        headers: { Authorization: `Bearer ${localStorage.getItem("tachyon_token") ?? ""}` },
        body,
      });
      if (!res.ok) throw new Error(`import failed (${res.status})`);
      setState("done");
      setMessage(`Imported ${f.name}.`);
    } catch (e) {
      setState("error");
      setMessage(e instanceof Error ? e.message : String(e));
    }
  };

  return (
    <div class="mx-auto max-w-3xl px-4 py-6 sm:px-6 lg:px-8">
      <h1 class="font-display text-2xl font-bold text-gray-900 dark:text-white">Import</h1>
      <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
        Bring your existing notes into Tachyon.
      </p>

      <div class="mt-6 space-y-4">
        {/* Obsidian */}
        <div class="rounded-none border-2 border-gray-900 bg-white p-6 dark:border-gray-100 dark:bg-gray-800">
          <h2 class="font-semibold text-gray-900 dark:text-white">Obsidian Vault</h2>
          <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
            Upload a .zip export of your vault. Folders become spaces, markdown files become
            documents, wikilinks become graph edges.
          </p>
          <div class="mt-3 flex items-center gap-3">
            <input
              type="file"
              accept=".zip"
              class="text-sm"
              onChange={(e) => setFile(e.currentTarget.files?.[0] ?? null)}
            />
            <button
              class="rounded-none bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700 disabled:opacity-50"
              disabled={!file() || state() === "importing"}
              onClick={() => void importVault()}
            >
              {state() === "importing" ? "Importing…" : "Import Vault"}
            </button>
          </div>
          <Show when={message()}>
            <p class={`mt-3 text-sm ${state() === "error" ? "text-red-600" : "text-green-600"}`}>
              {message()}
            </p>
          </Show>
        </div>

        {/* Coming in Phase C */}
        <For each={["Notion", "Confluence", "Docusaurus / Markdown zip"]}>
          {(name) => (
            <div class="rounded-none border border-gray-300 bg-white p-4 opacity-60 dark:border-gray-600 dark:bg-gray-800">
              <p class="font-medium text-gray-700 dark:text-gray-300">{name}</p>
              <p class="text-xs text-gray-400">Phase C — server endpoints exist, wiring pending</p>
            </div>
          )}
        </For>
      </div>
    </div>
  );
}

