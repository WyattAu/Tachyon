import { For, Show, createResource } from "solid-js";
import { A } from "@solidjs/router";
import { api } from "@/lib/api/client";

/** Teams — Solid port (Phase B). Team list with member counts. */

interface Team {
  id: string;
  name: string;
  slug: string;
  description: string | null;
  avatar_url: string | null;
  member_count: number;
}

export default function Teams() {
  const [teams] = createResource(async () => {
    const { data } = await api.GET("/teams", {});
    return ((data ?? []) as unknown as Team[]) ?? [];
  });

  return (
    <div class="mx-auto max-w-6xl px-4 py-6 sm:px-6 lg:px-8">
      <div class="mb-4 flex items-center justify-between">
        <div>
          <h1 class="font-display text-2xl font-bold text-gray-900 dark:text-white">Teams</h1>
          <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">Manage your teams and team members</p>
        </div>
        <button class="rounded-none bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700">
          Create Team
        </button>
      </div>

      <Show when={!teams.loading} fallback={<div class="h-24 animate-pulse rounded-none bg-gray-200 dark:bg-gray-700" />}>
        <Show
          when={(teams() ?? []).length > 0}
          fallback={
            <div class="py-10 text-center">
              <p class="text-lg font-medium text-gray-700 dark:text-gray-300">No teams</p>
              <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
                Get started by creating a new team.
              </p>
            </div>
          }
        >
          <div class="space-y-2">
            <For each={teams()}>
              {(t) => (
                <A
                  href={`/teams/${t.id}`}
                  class="flex items-center justify-between rounded-none border-2 border-gray-900 bg-white px-4 py-3 shadow-spatial-sm hover:shadow-spatial dark:border-gray-100 dark:bg-gray-800"
                >
                  <div>
                    <p class="font-medium text-gray-900 dark:text-white">{t.name}</p>
                    <Show when={t.description}>
                      <p class="text-xs text-gray-500 dark:text-gray-400">{t.description}</p>
                    </Show>
                  </div>
                  <span class="text-sm text-gray-400">{t.member_count} members</span>
                </A>
              )}
            </For>
          </div>
        </Show>
      </Show>
    </div>
  );
}
