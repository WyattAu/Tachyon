import { For, Show, createResource, createSignal } from "solid-js";
import { api } from "@/lib/api/client";

/** Tags — Solid port (Phase B). Browse documents by tag. */

interface TagInfo {
  tag: string;
  count: number;
}

export default function Tags() {
  const [filter, setFilter] = createSignal("");
  const [tags] = createResource(async () => {
    const { data } = await api.GET("/tags", {});
    const body = (data ?? {}) as unknown as { tags?: TagInfo[] };
    return body.tags ?? [];
  });

  const filtered = () =>
    (tags() ?? []).filter((t) => t.tag.toLowerCase().includes(filter().toLowerCase()));

  return (
    <div class="mx-auto max-w-6xl px-4 py-6 sm:px-6 lg:px-8">
      <h1 class="font-display text-2xl font-bold text-gray-900 dark:text-white">Tags</h1>
      <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">Browse and filter documents by tag.</p>

      <input
        type="text"
        placeholder="Filter tags..."
        class="mt-4 w-full max-w-md rounded-none border border-gray-300 bg-white px-3 py-2 dark:border-gray-600 dark:bg-gray-800 dark:text-white"
        value={filter()}
        onInput={(e) => setFilter(e.currentTarget.value)}
      />

      <Show when={!tags.loading} fallback={<div class="mt-4 h-16 animate-pulse rounded-none bg-gray-200 dark:bg-gray-700" />}>
        <Show
          when={filtered().length > 0}
          fallback={
            <p class="mt-6 text-sm text-gray-500 dark:text-gray-400">
              No tags found. Add tags to your documents to see them here.
            </p>
          }
        >
          <div class="mt-6 flex flex-wrap gap-2">
            <For each={filtered()}>
              {(t) => (
                <span class="inline-flex items-center gap-1 rounded-full border border-gray-300 bg-white px-3 py-1 text-sm dark:border-gray-600 dark:bg-gray-800 dark:text-gray-300">
                  {t.tag}
                  <span class="rounded-full bg-blue-100 px-1.5 text-xs text-blue-700 dark:bg-blue-900 dark:text-blue-300">
                    {t.count}
                  </span>
                </span>
              )}
            </For>
          </div>
        </Show>
      </Show>
    </div>
  );
}
