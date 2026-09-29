import { For, Show, createResource } from "solid-js";
import { api } from "@/lib/api/client";

/** Plugins — Solid port (Phase B). Installed plugin list. */

interface Plugin {
  id: string;
  name?: string;
  description: string | null;
  author: string | null;
  enabled: boolean;
  version?: string;
}

export default function Plugins() {
  const [plugins] = createResource(async () => {
    const { data } = await api.GET("/plugins", { params: { query: {} } });
    return ((data ?? []) as unknown as Plugin[]) ?? [];
  });

  return (
    <div class="mx-auto max-w-6xl px-4 py-6 sm:px-6 lg:px-8">
      <div class="mb-4 flex items-center justify-between">
        <div>
          <h1 class="font-display text-2xl font-bold text-gray-900 dark:text-white">Plugins</h1>
          <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
            Extend Tachyon with community and custom plugins
          </p>
        </div>
        <button class="rounded-none bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700">
          + Install Plugin
        </button>
      </div>

      <Show when={!plugins.loading} fallback={<div class="h-24 animate-pulse rounded-none bg-gray-200 dark:bg-gray-700" />}>
        <Show
          when={(plugins() ?? []).length > 0}
          fallback={
            <div class="py-10 text-center">
              <p class="text-lg font-medium text-gray-700 dark:text-gray-300">No plugins installed</p>
              <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
                Install plugins to extend Tachyon's capabilities
              </p>
              <button class="mt-4 rounded-none bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700">
                Install Plugin
              </button>
            </div>
          }
        >
          <div class="space-y-2">
            <For each={plugins()}>
              {(pl) => (
                <div class="flex items-center justify-between rounded-none border-2 border-gray-900 bg-white px-4 py-3 dark:border-gray-100 dark:bg-gray-800">
                  <div>
                    <p class="font-medium text-gray-900 dark:text-white">
                      {pl.name ?? pl.id}{" "}
                      <span class={`ml-2 rounded-full px-2 py-0.5 text-xs ${pl.enabled ? "bg-green-100 text-green-700 dark:bg-green-900/40 dark:text-green-300" : "bg-gray-200 text-gray-600 dark:bg-gray-700 dark:text-gray-300"}`}>
                        {pl.enabled ? "enabled" : "disabled"}
                      </span>
                    </p>
                    <Show when={pl.description}>
                      <p class="text-xs text-gray-500 dark:text-gray-400">{pl.description}</p>
                    </Show>
                  </div>
                  <span class="text-xs text-gray-400">by {pl.author ?? "unknown"}</span>
                </div>
              )}
            </For>
          </div>
        </Show>
      </Show>
    </div>
  );
}
