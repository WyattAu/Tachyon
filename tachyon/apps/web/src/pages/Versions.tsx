import { For, Show, createResource, createSignal } from "solid-js";
import { api } from "@/lib/api/client";

/** Versions — Solid port (Phase B). Pick a document, browse its versions. */

interface Doc {
  id: string;
  title: string;
}
interface Version {
  id: string;
  version_number: number;
  commit_message: string | null;
  created_by: string | null;
  created_at: string | null;
}

export default function Versions() {
  const [docId, setDocId] = createSignal("");
  const [versions] = createResource(docId, async (id) => {
    if (!id) return [];
    const { data } = await api.GET("/documents/{document_id}/versions", {
      params: { path: { document_id: id } },
    });
    return ((data ?? []) as unknown as Version[]) ?? [];
  });

  const [docs] = createResource(async () => {
    const { data } = await api.GET("/documents", {
      params: { query: { page: 1, page_size: 20 } },
    });
    return ((data as unknown as { results?: Doc[] })?.results ?? []) as Doc[];
  });

  return (
    <div class="mx-auto max-w-4xl px-4 py-6 sm:px-6 lg:px-8">
      <h1 class="font-display text-2xl font-bold text-gray-900 dark:text-white">Versions</h1>
      <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
        Browse a document's version history.
      </p>

      <select
        class="mt-4 w-full rounded-none border border-gray-300 bg-white px-3 py-2 dark:border-gray-600 dark:bg-gray-800 dark:text-white"
        value={docId()}
        onChange={(e) => setDocId(e.currentTarget.value)}
      >
        <option value="">Select a document…</option>
        <For each={docs()}>
          {(d) => <option value={d.id}>{d.title || d.id}</option>}
        </For>
      </select>

      <Show when={docId()}>
        <Show when={!versions.loading} fallback={<div class="mt-4 h-20 animate-pulse rounded-none bg-gray-200 dark:bg-gray-700" />}>
          <Show
            when={(versions() ?? []).length > 0}
            fallback={
              <p class="mt-4 text-sm text-gray-500 dark:text-gray-400">No versions recorded yet.</p>
            }
          >
            <div class="mt-4 space-y-2">
              <For each={versions()}>
                {(v) => (
                  <div class="rounded-none border border-gray-300 bg-white px-4 py-3 dark:border-gray-600 dark:bg-gray-800">
                    <div class="flex items-center justify-between">
                      <span class="font-medium text-gray-900 dark:text-white">
                        v{v.version_number}
                      </span>
                      <span class="text-xs text-gray-400">
                        {v.created_at ? new Date(v.created_at).toLocaleString() : ""}
                      </span>
                    </div>
                    <Show when={v.commit_message}>
                      <p class="mt-1 text-sm text-gray-600 dark:text-gray-400">{v.commit_message}</p>
                    </Show>
                  </div>
                )}
              </For>
            </div>
          </Show>
        </Show>
      </Show>
    </div>
  );
}
