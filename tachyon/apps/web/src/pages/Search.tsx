import { For, Show, createSignal } from "solid-js";
import { A, useNavigate } from "@solidjs/router";
import { api } from "@/lib/api/client";

/** Search — Solid port (Phase B). Full-text search via GET /search. */

interface SearchResultItem {
  id: string;
  title: string;
  headline: string | null;
  tags: string[] | null;
  status: string;
  word_count: number;
}

export default function Search() {
  const navigate = useNavigate();
  const [query, setQuery] = createSignal("");
  const [searching, setSearching] = createSignal(false);
  const [searched, setSearched] = createSignal(false);
  const [results, setResults] = createSignal<SearchResultItem[]>([]);
  const [total, setTotal] = createSignal(0);
  const [error, setError] = createSignal<string | null>(null);

  const run = async (e: SubmitEvent) => {
    e.preventDefault();
    if (!query().trim()) return;
    setSearching(true);
    setError(null);
    try {
      const { data } = await api.GET("/search", {
        params: { query: { query: query() } },
      } as never);
      const body = (data ?? {}) as unknown as {
        results?: SearchResultItem[];
        total?: number;
      };
      setResults(body.results ?? []);
      setTotal(body.total ?? 0);
      setSearched(true);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setSearching(false);
    }
  };

  return (
    <div class="mx-auto max-w-4xl px-4 py-6 sm:px-6 lg:px-8">
      <h1 class="font-display text-2xl font-bold text-gray-900 dark:text-white">Search</h1>

      <form class="mt-4 flex gap-2" onSubmit={run}>
        <input
          type="text"
          placeholder="Search documents and projects..."
          class="flex-1 rounded-none border border-gray-300 bg-white px-3 py-2 dark:border-gray-600 dark:bg-gray-800 dark:text-white"
          value={query()}
          onInput={(e) => setQuery(e.currentTarget.value)}
        />
        <button
          type="submit"
          disabled={searching()}
          class="rounded-none bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700 disabled:opacity-50"
        >
          {searching() ? "…" : "Search"}
        </button>
      </form>

      <Show when={error()}>
        <p class="mt-3 text-sm text-red-600 dark:text-red-400">{error()}</p>
      </Show>

      <Show when={searched()}>
        <p class="mt-4 text-sm text-gray-500 dark:text-gray-400">
          {total()} result{total() === 1 ? "" : "s"} for "{query()}"
        </p>
        <div class="mt-2 space-y-2">
          <For each={results()}>
            {(r) => (
              <A
                href={`/documents/${r.id}`}
                class="block rounded-none border border-gray-300 bg-white p-4 hover:shadow-spatial-sm dark:border-gray-600 dark:bg-gray-800"
              >
                <p class="font-semibold text-gray-900 dark:text-white">{r.title}</p>
                <Show when={r.headline}>
                  <p
                    class="mt-1 text-sm text-gray-600 dark:text-gray-400 [&_mark]:bg-yellow-200 [&_mark]:text-yellow-900"
                    innerHTML={r.headline ?? ""}
                  />
                </Show>
                <div class="mt-2 flex gap-2 text-xs text-gray-400">
                  <span>{r.status}</span>
                  <span>{r.word_count} words</span>
                </div>
              </A>
            )}
          </For>
        </div>
      </Show>
    </div>
  );
}
