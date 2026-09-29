import { For, Show, createResource } from "solid-js";
import { useParams } from "@solidjs/router";
import { api } from "@/lib/api/client";

/** Team detail — Solid port (Phase B). Team info + members list. */

interface Team {
  id: string;
  name: string;
  slug: string;
  description: string | null;
  member_count: number;
}
interface Member {
  user_id: string;
  role?: string;
  username?: string;
}

export default function TeamDetail() {
  const params = useParams();
  const teamId = () => params.team_id ?? "";

  const [team] = createResource(teamId, async (id) => {
    const { data } = await api.GET("/teams/{team_id}", {
      params: { path: { team_id: id } },
    });
    return (data ?? null) as unknown as Team | null;
  });

  const [members] = createResource(teamId, async (id) => {
    const { data } = await api.GET("/teams/{team_id}/members", {
      params: { path: { team_id: id } },
    });
    return ((data ?? []) as unknown as Member[]) ?? [];
  });

  return (
    <div class="mx-auto max-w-4xl px-4 py-6 sm:px-6 lg:px-8">
      <Show when={team()} fallback={<p class="text-sm text-gray-500">Loading…</p>}>
        {(t) => (
          <>
            <h1 class="font-display text-2xl font-bold text-gray-900 dark:text-white">{t().name}</h1>
            <Show when={t().description}>
              <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">{t().description}</p>
            </Show>
            <p class="mt-2 text-xs text-gray-400">
              @{t().slug} · {t().member_count} members
            </p>

            <h2 class="mt-6 mb-3 text-lg font-semibold">Members</h2>
            <div class="space-y-2">
              <For each={members()}>
                {(m) => (
                  <div class="flex items-center justify-between rounded-none border border-gray-300 bg-white px-4 py-2 dark:border-gray-600 dark:bg-gray-800">
                    <span class="font-mono text-xs text-gray-700 dark:text-gray-300">
                      {m.username ?? m.user_id}
                    </span>
                    <span class="rounded-full bg-gray-100 px-2 py-0.5 text-xs text-gray-600 dark:bg-gray-700 dark:text-gray-300">
                      {m.role ?? "member"}
                    </span>
                  </div>
                )}
              </For>
            </div>
          </>
        )}
      </Show>
    </div>
  );
}
