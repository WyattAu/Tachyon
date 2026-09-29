import { For, Show, createResource } from "solid-js";
import { A } from "@solidjs/router";
import { api } from "@/lib/api/client";

/** Spaces — Solid port (Phase B). Hierarchy list with doc counts. */

interface Space {
  id: string;
  name: string;
  description: string | null;
  icon: string | null;
  color: string | null;
  document_count: number;
  is_default: boolean;
  parent_id: string | null;
}

export default function Spaces() {
  const [spaces] = createResource(async () => {
    const { data } = await api.GET("/spaces", { params: { query: {} } });
    return ((data ?? []) as unknown as Space[]) ?? [];
  });

  return (
    <div class="mx-auto max-w-6xl px-4 py-6 sm:px-6 lg:px-8">
      <div class="mb-4 flex items-center justify-between">
        <div>
          <h1 class="font-display text-2xl font-bold text-gray-900 dark:text-white">Spaces</h1>
          <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
            Organize your documents into spaces
          </p>
        </div>
        <button class="rounded-none bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700">
          + New Space
        </button>
      </div>

      <Show when={!spaces.loading} fallback={<div class="h-24 animate-pulse rounded-none bg-gray-200 dark:bg-gray-700" />}>
        <Show
          when={(spaces() ?? []).length > 0}
          fallback={
            <div class="rounded-none border border-gray-900 bg-white p-8 text-center dark:border-gray-100 dark:bg-gray-800">
              <p class="font-medium text-gray-700 dark:text-gray-300">No spaces yet</p>
            </div>
          }
        >
          <p class="mb-3 text-sm text-gray-500 dark:text-gray-400">
            {spaces()!.length} space{spaces()!.length === 1 ? "" : "s"}
          </p>
          <div class="space-y-2">
            <For each={spaces()}>
              {(sp) => (
                <A
                  href={`/spaces`}
                  class="flex items-center justify-between rounded-none border-2 border-gray-900 bg-white px-4 py-3 shadow-spatial-sm hover:shadow-spatial dark:border-gray-100 dark:bg-gray-800"
                >
                  <div class="flex items-center gap-3">
                    <span
                      class="flex h-8 w-8 items-center justify-center rounded bg-blue-600 text-sm text-white"
                      style={sp.color ? `background:${sp.color}` : ""}
                    >
                      {sp.icon || "📁"}
                    </span>
                    <div>
                      <p class="font-medium text-gray-900 dark:text-white">{sp.name}</p>
                      <Show when={sp.description}>
                        <p class="text-xs text-gray-500 dark:text-gray-400">{sp.description}</p>
                      </Show>
                    </div>
                    <Show when={sp.is_default}>
                      <span class="rounded-full bg-gray-200 px-2 py-0.5 text-xs text-gray-600 dark:bg-gray-700 dark:text-gray-300">
                        default
                      </span>
                    </Show>
                  </div>
                  <span class="text-sm text-gray-400">{sp.document_count ?? 0}</span>
                </A>
              )}
            </For>
          </div>
        </Show>
      </Show>
    </div>
  );
}
