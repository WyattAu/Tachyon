import { For, Show, createResource, createSignal } from "solid-js";
import { api } from "@/lib/api/client";

/** Templates — Solid port (Phase B). Category filter + template cards. */

interface Template {
  id: string;
  name: string;
  description: string | null;
  category: string | null;
  tags: string[] | null;
}

export default function Templates() {
  const [category, setCategory] = createSignal("");
  const [templates] = createResource(async () => {
    const { data } = await api.GET("/templates", { params: { query: {} } });
    return ((data ?? []) as unknown as Template[]) ?? [];
  });

  const categories = () => {
    const set = new Set<string>();
    for (const t of templates() ?? []) if (t.category) set.add(t.category);
    return Array.from(set).sort();
  };

  const filtered = () => {
    const c = category();
    if (!c) return templates() ?? [];
    return (templates() ?? []).filter((t) => t.category === c);
  };

  return (
    <div class="mx-auto max-w-6xl px-4 py-6 sm:px-6 lg:px-8">
      <div class="mb-4 flex items-center justify-between">
        <div>
          <h1 class="font-display text-2xl font-bold text-gray-900 dark:text-white">Templates</h1>
          <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
            Manage document templates to standardize your workflow
          </p>
        </div>
        <button class="rounded-none bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700">
          + New Template
        </button>
      </div>

      <Show when={!templates.loading} fallback={<div class="h-24 animate-pulse rounded-none bg-gray-200 dark:bg-gray-700" />}>
        <Show
          when={(templates() ?? []).length > 0}
          fallback={
            <div class="py-10 text-center">
              <p class="text-lg font-medium text-gray-700 dark:text-gray-300">No templates yet</p>
              <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
                Create your first template to get started
              </p>
              <button class="mt-4 rounded-none bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700">
                Create Template
              </button>
            </div>
          }
        >
          <div class="flex flex-wrap gap-2">
            <button
              class={`rounded-full px-3 py-1 text-sm ${category() === "" ? "bg-blue-600 text-white" : "border border-gray-300 bg-white text-gray-700 dark:border-gray-600 dark:bg-gray-800 dark:text-gray-300"}`}
              onClick={() => setCategory("")}
            >
              All
            </button>
            <For each={categories()}>
              {(c) => (
                <button
                  class={`rounded-full px-3 py-1 text-sm ${category() === c ? "bg-blue-600 text-white" : "border border-gray-300 bg-white text-gray-700 dark:border-gray-600 dark:bg-gray-800 dark:text-gray-300"}`}
                  onClick={() => setCategory(c)}
                >
                  {c}
                </button>
              )}
            </For>
          </div>

          <div class="mt-4 grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
            <For each={filtered()}>
              {(t) => (
                <div class="rounded-none border-2 border-gray-900 bg-white p-4 shadow-spatial-sm dark:border-gray-100 dark:bg-gray-800">
                  <p class="font-semibold text-gray-900 dark:text-white">{t.name}</p>
                  <Show when={t.description}>
                    <p class="mt-1 line-clamp-2 text-sm text-gray-500 dark:text-gray-400">{t.description}</p>
                  </Show>
                  <div class="mt-3 flex flex-wrap gap-1">
                    <Show when={t.category}>
                      <span class="rounded-full bg-blue-100 px-2 py-0.5 text-xs text-blue-700 dark:bg-blue-900 dark:text-blue-300">
                        {t.category}
                      </span>
                    </Show>
                    <For each={(t.tags ?? []).slice(0, 2)}>
                      {(tag) => (
                        <span class="rounded-full bg-gray-100 px-2 py-0.5 text-xs text-gray-600 dark:bg-gray-700 dark:text-gray-300">
                          {tag}
                        </span>
                      )}
                    </For>
                  </div>
                </div>
              )}
            </For>
          </div>
        </Show>
      </Show>
    </div>
  );
}
