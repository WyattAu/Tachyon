import { For, Show, createSignal, createMemo, createResource } from "solid-js";
import { A, useNavigate } from "@solidjs/router";
import { api } from "@/lib/api/client";

/**
 * Documents list — Solid port of the Leptos documents page, list half
 * (Phase B). Search (server-side), status filter (client-side), grid/list
 * views, pagination, and the create-document modal.
 * The editor half lives at /documents/:id/edit (separate port).
 */

interface Doc {
  id: string;
  title: string;
  status: string;
  tags: string[];
  word_count: number;
  updated_at: string | null;
  created_at: string | null;
  [k: string]: unknown;
}

const PAGE_SIZE = 12;

export default function DocumentsList() {
  const navigate = useNavigate();
  const [search, setSearch] = createSignal("");
  const [statusFilter, setStatusFilter] = createSignal("");
  const [viewMode, setViewMode] = createSignal<"grid" | "list">("grid");
  const [page, setPage] = createSignal(1);

  const [data, { refetch }] = createResource(
    () => ({ page: page(), search: search() }),
    async ({ page: p, search: q }) => {
      const { data } = await api.GET("/documents", {
        params: { query: { page: p, page_size: PAGE_SIZE, search: q || undefined } },
      });
      const body = (data ?? {}) as unknown as {
        results?: Doc[];
        total?: number;
        page?: number;
        page_size?: number;
      };
      return {
        results: body.results ?? [],
        total: body.total ?? 0,
        totalPages: Math.max(1, Math.ceil((body.total ?? 0) / (body.page_size || PAGE_SIZE))),
      };
    }
  );

  // Status filter is client-side (mirrors Leptos behavior)
  const filtered = createMemo(() => {
    const f = statusFilter();
    if (!f) return data()?.results ?? [];
    return (data()?.results ?? []).filter((d) => d.status === f);
  });

  // --- create modal ---
  const [showCreate, setShowCreate] = createSignal(false);
  const [newTitle, setNewTitle] = createSignal("");
  const [createError, setCreateError] = createSignal<string | null>(null);
  const [creating, setCreating] = createSignal(false);

  const createDoc = async (e: SubmitEvent) => {
    e.preventDefault();
    setCreating(true);
    setCreateError(null);
    try {
      const { data: created, error, response } = await api.POST("/documents", {
        body: { title: newTitle(), content: "" } as never,
      });
      if (error || !created) throw new Error(`create failed (${response.status})`);
      setShowCreate(false);
      navigate(`/documents/${(created as unknown as { id: string }).id}/edit`);
    } catch (err) {
      setCreateError(err instanceof Error ? err.message : String(err));
    } finally {
      setCreating(false);
    }
  };

  return (
    <div class="mx-auto max-w-6xl px-4 py-6 sm:px-6 lg:px-8">
      {/* header */}
      <div class="mb-4 flex items-center justify-between">
        <div class="flex items-center gap-2">
          <h1 class="font-display text-2xl font-bold text-gray-900 dark:text-white">Documents</h1>
          <span class="inline-flex items-center gap-1 rounded-full bg-green-100 px-2 py-0.5 text-xs text-green-700 dark:bg-green-900/40 dark:text-green-300">
            ● Synced
          </span>
        </div>
        <button
          class="rounded-none bg-blue-600 px-4 py-2 text-white hover:bg-blue-700"
          onClick={() => setShowCreate(true)}
        >
          + New Document
        </button>
      </div>

      {/* controls */}
      <div class="mb-4 flex flex-col gap-3 sm:flex-row">
        <div class="relative flex-1">
          <input
            type="text"
            placeholder="Search documents..."
            class="w-full rounded-none border border-gray-300 bg-white px-4 py-2 pl-10 dark:border-gray-600 dark:bg-gray-800 dark:text-white"
            value={search()}
            onInput={(e) => {
              setSearch(e.currentTarget.value);
              setPage(1);
            }}
            onKeyDown={(e) => {
              if (e.key === "Enter") void refetch();
            }}
          />
          <svg
            class="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-gray-400"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            viewBox="0 0 24 24"
            aria-hidden="true"
          >
            <path stroke-linecap="round" stroke-linejoin="round" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
          </svg>
        </div>
        <select
          class="rounded-none border border-gray-300 bg-white px-3 py-2 dark:border-gray-600 dark:bg-gray-800 dark:text-white"
          value={statusFilter()}
          onChange={(e) => setStatusFilter(e.currentTarget.value)}
          aria-label="Filter by status"
        >
          <option value="">All Status</option>
          <option value="draft">Draft</option>
          <option value="published">Published</option>
          <option value="archived">Archived</option>
        </select>
        <div class="flex overflow-hidden rounded-none border border-gray-300 dark:border-gray-600">
          <button
            class={`px-3 py-2 ${viewMode() === "grid" ? "bg-blue-600 text-white" : "bg-white text-gray-700 hover:bg-gray-50 dark:bg-gray-800 dark:text-gray-300 dark:hover:bg-gray-700"}`}
            onClick={() => setViewMode("grid")}
            aria-label="Grid view"
            aria-pressed={viewMode() === "grid"}
          >
            ▦
          </button>
          <button
            class={`px-3 py-2 ${viewMode() === "list" ? "bg-blue-600 text-white" : "bg-white text-gray-700 hover:bg-gray-50 dark:bg-gray-800 dark:text-gray-300 dark:hover:bg-gray-700"}`}
            onClick={() => setViewMode("list")}
            aria-label="List view"
            aria-pressed={viewMode() === "list"}
          >
            ☰
          </button>
        </div>
      </div>

      {/* body */}
      <Show
        when={!data.loading && filtered().length > 0}
        fallback={
          <Show
            when={!data.loading}
            fallback={
              <div class="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
                {Array.from({ length: 3 }).map(() => (
                  <div class="h-40 animate-pulse rounded-none bg-gray-200 dark:bg-gray-700" />
                ))}
              </div>
            }
          >
            <div class="rounded-none border border-gray-900 bg-white p-10 text-center dark:border-gray-100 dark:bg-gray-800">
              <p class="text-lg font-medium text-gray-700 dark:text-gray-300">No documents yet</p>
              <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
                Create your first document to get started.
              </p>
              <button
                class="mt-4 rounded-none bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700"
                onClick={() => setShowCreate(true)}
              >
                + New Document
              </button>
            </div>
          </Show>
        }
      >
        <div>
          <p class="mb-4 text-sm text-gray-500 dark:text-gray-400">
            {filtered().length} document{filtered().length === 1 ? "" : "s"}
          </p>
          <Show
            when={viewMode() === "grid"}
            fallback={
              <div class="divide-y divide-gray-200 rounded-none border-2 border-gray-900 bg-white dark:divide-gray-700 dark:border-gray-100 dark:bg-gray-800">
                <For each={filtered()}>
                  {(doc) => (
                    <A
                      href={`/documents/${doc.id}`}
                      class="flex items-center justify-between px-4 py-3 hover:bg-gray-50 dark:hover:bg-gray-700/50"
                    >
                      <div>
                        <p class="font-medium text-gray-900 dark:text-white">{doc.title || "Untitled"}</p>
                        <p class="text-xs text-gray-500 dark:text-gray-400">
                          {doc.word_count ?? 0} words · {doc.status}
                        </p>
                      </div>
                      <Show when={doc.updated_at}>
                        <span class="text-xs text-gray-400">
                          {new Date(doc.updated_at as string).toLocaleDateString()}
                        </span>
                      </Show>
                    </A>
                  )}
                </For>
              </div>
            }
          >
            <div class="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
              <For each={filtered()}>
                {(doc) => (
                  <A
                    href={`/documents/${doc.id}`}
                    class="block rounded-none border-2 border-gray-900 bg-white p-4 shadow-spatial-sm transition-shadow hover:shadow-spatial dark:border-gray-100 dark:bg-gray-800"
                  >
                    <p class="font-semibold text-gray-900 dark:text-white">{doc.title || "Untitled"}</p>
                    <div class="mt-2 flex flex-wrap gap-1">
                      <For each={(doc.tags ?? []).slice(0, 3)}>
                        {(tag) => (
                          <span class="rounded-full bg-gray-100 px-2 py-0.5 text-xs text-gray-600 dark:bg-gray-700 dark:text-gray-300">
                            {tag}
                          </span>
                        )}
                      </For>
                    </div>
                    <div class="mt-3 flex items-center justify-between text-xs text-gray-400">
                      <span>{doc.word_count ?? 0} words · {doc.status}</span>
                      <Show when={doc.updated_at}>
                        <span>{new Date(doc.updated_at as string).toLocaleDateString()}</span>
                      </Show>
                    </div>
                  </A>
                )}
              </For>
            </div>
          </Show>
        </div>
      </Show>

      {/* pagination */}
      <Show when={(data()?.total ?? 0) > (data()?.results ?? []).length || page() > 1}>
        <div class="mt-6 flex items-center justify-center gap-2">
          <button
            class="rounded-none border border-gray-300 bg-white px-3 py-2 text-sm text-gray-700 hover:bg-gray-50 disabled:cursor-not-allowed disabled:opacity-50 dark:border-gray-600 dark:bg-gray-800 dark:text-gray-300 dark:hover:bg-gray-700"
            disabled={page() <= 1}
            onClick={() => setPage(page() - 1)}
          >
            Previous
          </button>
          <span class="px-3 py-2 text-sm text-gray-700 dark:text-gray-300">
            Page {page()} of {data()?.totalPages ?? 1}
          </span>
          <button
            class="rounded-none border border-gray-300 bg-white px-3 py-2 text-sm text-gray-700 hover:bg-gray-50 disabled:cursor-not-allowed disabled:opacity-50 dark:border-gray-600 dark:bg-gray-800 dark:text-gray-300 dark:hover:bg-gray-700"
            disabled={page() >= (data()?.totalPages ?? 1)}
            onClick={() => setPage(page() + 1)}
          >
            Next
          </button>
        </div>
      </Show>

      {/* create modal */}
      <Show when={showCreate()}>
        <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
          <form
            class="w-full max-w-md rounded-none bg-white p-6 dark:bg-gray-800"
            role="dialog"
            aria-modal="true"
            aria-labelledby="new-doc-title"
            onSubmit={createDoc}
          >
            <h2 id="new-doc-title" class="mb-4 text-xl font-bold text-gray-900 dark:text-white">
              New Document
            </h2>
            <Show when={createError()}>
              <div class="mb-4 rounded border border-red-200 bg-red-50 p-3 text-sm text-red-700 dark:border-red-800 dark:bg-red-900/20 dark:text-red-300">
                {createError()}
              </div>
            </Show>
            <label class="mb-1 block text-sm font-medium text-gray-700 dark:text-gray-300" for="doc-title">
              Title
            </label>
            <input
              id="doc-title"
              type="text"
              required
              placeholder="Enter document title"
              class="mb-4 w-full rounded-none border border-gray-300 px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700 dark:text-white"
              value={newTitle()}
              onInput={(e) => setNewTitle(e.currentTarget.value)}
            />
            <div class="flex justify-end gap-2">
              <button
                type="button"
                class="rounded-none border border-gray-300 px-4 py-2 text-sm hover:bg-gray-100 dark:border-gray-600 dark:hover:bg-gray-700"
                onClick={() => setShowCreate(false)}
              >
                Cancel
              </button>
              <button
                type="submit"
                disabled={creating()}
                class="rounded-none bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700 disabled:opacity-50"
              >
                {creating() ? "Creating…" : "Create"}
              </button>
            </div>
          </form>
        </div>
      </Show>
    </div>
  );
}
