import { For, Show, createResource } from "solid-js";
import { api, resolveBaseUrl, getToken } from "@/lib/api/client";

/**
 * Catalog — Solid port (Phase B). Projects + components + stats.
 * /projects + /catalog/stats are untyped in the current OpenAPI spec —
 * hand-typed here (tracked for server annotation cleanup).
 */

interface Project {
  id: string;
  name: string;
  description: string | null;
  status?: string;
}

async function plainGet<T>(path: string, fallback: T): Promise<T> {
  try {
    const res = await fetch(`${resolveBaseUrl()}${path}`, {
      headers: { Authorization: `Bearer ${getToken() ?? ""}` },
    });
    if (!res.ok) return fallback;
    return (await res.json()) as unknown as T;
  } catch {
    return fallback;
  }
}

export default function Catalog() {
  const [stats] = createResource(async () =>
    plainGet<Record<string, number>>("/catalog/stats", {
      projects: 0,
      components: 0,
      members: 0,
    })
  );
  const [projects] = createResource(async () =>
    plainGet<Project[]>("/projects", [])
  );

  const stat = (k: string) => stats()?.[k] ?? 0;

  return (
    <div class="mx-auto max-w-6xl px-4 py-6 sm:px-6 lg:px-8">
      <h1 class="font-display text-2xl font-bold text-gray-900 dark:text-white">Catalog</h1>

      <div class="mt-4 grid gap-4 md:grid-cols-3">
        <div class="rounded-none border border-gray-900 bg-white p-5 dark:border-gray-100 dark:bg-gray-800">
          <p class="text-sm text-gray-500 dark:text-gray-400">Projects</p>
          <p class="text-3xl font-bold">{stat("projects")}</p>
        </div>
        <div class="rounded-none border border-gray-900 bg-white p-5 dark:border-gray-100 dark:bg-gray-800">
          <p class="text-sm text-gray-500 dark:text-gray-400">Components</p>
          <p class="text-3xl font-bold">{stat("components")}</p>
        </div>
        <div class="rounded-none border border-gray-900 bg-white p-5 dark:border-gray-100 dark:bg-gray-800">
          <p class="text-sm text-gray-500 dark:text-gray-400">Members</p>
          <p class="text-3xl font-bold">{stat("members")}</p>
        </div>
      </div>

      <div class="mt-6">
        <h2 class="mb-3 text-lg font-semibold">Projects</h2>
        <Show when={!projects.loading} fallback={<div class="h-16 animate-pulse rounded-none bg-gray-200 dark:bg-gray-700" />}>
          <Show
            when={(projects() ?? []).length > 0}
            fallback={
              <div class="rounded-none border border-gray-900 bg-white p-6 text-center text-sm text-gray-500 dark:border-gray-100 dark:bg-gray-800 dark:text-gray-400">
                No projects found. Create your first project to get started!
              </div>
            }
          >
            <div class="space-y-2">
              <For each={projects()}>
                {(pr) => (
                  <div class="rounded-none border border-gray-300 bg-white px-4 py-3 dark:border-gray-600 dark:bg-gray-800">
                    <p class="font-medium text-gray-900 dark:text-white">{pr.name}</p>
                    <Show when={pr.description}>
                      <p class="text-xs text-gray-500 dark:text-gray-400">{pr.description}</p>
                    </Show>
                  </div>
                )}
              </For>
            </div>
          </Show>
        </Show>
      </div>
    </div>
  );
}
