import { createResource, For, Show } from "solid-js";
import { A } from "@solidjs/router";
import { api } from "@/lib/api/client";
import { authStore } from "@/lib/auth";

/**
 * Dashboard — Solid port of the Leptos dashboard (Phase B).
 * Data: GET /documents?page=1&page_size=8, GET /spaces, GET /activity?limit=10.
 */

interface DashboardDoc {
  id: string;
  title: string;
  updated_at?: string | null;
  [k: string]: unknown;
}

function greeting(): string {
  const h = new Date().getHours();
  if (h < 12) return "Good morning";
  if (h < 17) return "Good afternoon";
  return "Good evening";
}

const [docs] = createResource(async () => {
  const { data } = await api.GET("/documents", {
    params: { query: { page: 1, page_size: 8 } },
  });
  const body = data as unknown as { results?: DashboardDoc[] } | undefined;
  return body?.results ?? [];
});

const [spaces] = createResource(async () => {
  const { data } = await api.GET("/spaces", {
    params: {
      // list is filterable; defaults fetch everything visible to the user
      query: {
        parent_id: null,
        visibility: null,
        owner_id: null,
        limit: null,
        offset: null,
      },
    },
  });
  return (data as unknown as unknown[] | undefined) ?? [];
});

const [activity] = createResource(async () => {
  const { data } = await api.GET("/activity", {
    params: { query: { limit: 10 } },
  } as never);
  const body = data as unknown as { activities?: Array<Record<string, unknown>> } | undefined;
  return body?.activities ?? [];
});

function StatCard(props: { label: string; value: string | number; loading: boolean }) {
  return (
    <div class="rounded-none border border-gray-900 bg-white p-5 dark:border-gray-100 dark:bg-gray-800">
      <p class="text-sm text-gray-600 dark:text-gray-400">{props.label}</p>
      <Show
        when={!props.loading}
        fallback={<div class="mt-1 h-7 w-12 animate-pulse rounded bg-gray-200 dark:bg-gray-700" />}
      >
        <p class="text-3xl font-bold text-gray-900 dark:text-white">{props.value}</p>
      </Show>
    </div>
  );
}

export default function Dashboard() {
  return (
    <div class="mx-auto max-w-6xl px-4 py-6 sm:px-6 lg:px-8">
      <div class="mb-8">
        <h1 class="font-display text-2xl font-bold text-gray-900 dark:text-white">
          {greeting()}, welcome back
          {authStore.user()?.display_name ? `, ${authStore.user()?.display_name}` : ""}
        </h1>
        <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
          Here's what's happening with your knowledge base.
        </p>
      </div>

      <div class="mb-8 flex flex-wrap gap-3">
        <A
          href="/documents"
          class="inline-flex items-center gap-2 rounded-none bg-blue-600 px-4 py-2.5 text-sm font-medium text-white shadow-sm hover:bg-blue-700"
        >
          + New Document
        </A>
        <A
          href="/spaces"
          class="inline-flex items-center gap-2 rounded-none border border-gray-300 bg-white px-4 py-2.5 text-sm font-medium hover:bg-gray-100 dark:border-gray-600 dark:bg-gray-800 dark:hover:bg-gray-700"
        >
          New Space
        </A>
        <A
          href="/import"
          class="inline-flex items-center gap-2 rounded-none border border-gray-300 bg-white px-4 py-2.5 text-sm font-medium hover:bg-gray-100 dark:border-gray-600 dark:bg-gray-800 dark:hover:bg-gray-700"
        >
          Import
        </A>
      </div>

      <div class="mb-8 grid gap-4 sm:grid-cols-3">
        <StatCard label="Total Documents" value={docs()?.length ?? 0} loading={docs.loading} />
        <StatCard label="Spaces" value={spaces()?.length ?? 0} loading={spaces.loading} />
        <StatCard label="Recent Activity" value={activity()?.length ?? 0} loading={activity.loading} />
      </div>

      <div class="mb-3 flex items-center justify-between">
        <h2 class="text-lg font-semibold">Recent Documents</h2>
        <A href="/documents" class="text-sm text-blue-600 hover:underline dark:text-blue-400">
          View all
        </A>
      </div>

      <div class="rounded-none border border-gray-900 bg-white p-6 dark:border-gray-100 dark:bg-gray-800">
        <Show
          when={!docs.loading && (docs()?.length ?? 0) > 0}
          fallback={
            <div class="py-8 text-center">
              <p class="font-medium text-gray-700 dark:text-gray-300">No documents yet</p>
              <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
                Create your first document to get started.
              </p>
              <A
                href="/documents"
                class="mt-4 inline-block rounded-none bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700"
              >
                + New Document
              </A>
            </div>
          }
        >
          <ul class="divide-y divide-gray-200 dark:divide-gray-700">
            <For each={docs()}>
              {(doc) => (
                <li>
                  <A
                    href={`/documents/${doc.id}`}
                    class="flex items-center justify-between px-2 py-3 hover:bg-gray-50 dark:hover:bg-gray-700/50"
                  >
                    <span class="font-medium text-gray-900 dark:text-white">
                      {doc.title || "Untitled"}
                    </span>
                    <Show when={doc.updated_at}>
                      <span class="text-xs text-gray-400">
                        {new Date(doc.updated_at as string).toLocaleDateString()}
                      </span>
                    </Show>
                  </A>
                </li>
              )}
            </For>
          </ul>
        </Show>
      </div>
    </div>
  );
}
