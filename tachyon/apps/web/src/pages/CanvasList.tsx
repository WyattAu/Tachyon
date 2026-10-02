import { For, Show, createResource, createSignal } from "solid-js";
import { A, useNavigate } from "@solidjs/router";
import { authStore } from "@/lib/auth";
import { resolveBaseUrl, getToken } from "@/lib/api/client";

/**
 * Canvas list — Solid port (Phase B). GET /canvases + create.
 * Canvas endpoints are untyped in the spec — hand-typed (tracked).
 */

interface CanvasItem {
  id: string;
  title: string;
  updated_at: string | null;
}

export default function CanvasList() {
  const navigate = useNavigate();
  const [canvases, { refetch }] = createResource(async () => {
    const res = await fetch(`${resolveBaseUrl()}/canvases`, {
      headers: { Authorization: `Bearer ${getToken() ?? ""}` },
    });
    if (!res.ok) return [];
    const body = (await res.json()) as unknown as { canvases?: CanvasItem[] };
    return body.canvases ?? [];
  });

  const [title, setTitle] = createSignal("");
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");

  const create = async () => {
    if (!title().trim()) return;
    setBusy(true);
    setError("");
    try {
      // owner_id = current user (server requires it in the create body)
      let ownerId = authStore.user()?.id ?? "";
      if (!ownerId) {
        const me = await (await fetch(`${resolveBaseUrl()}/auth/me`, {
          headers: { Authorization: `Bearer ${getToken() ?? ""}` },
        })).json();
        ownerId = me.id;
      }
      const res = await fetch(`${resolveBaseUrl()}/canvases`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${getToken() ?? ""}`,
        },
        body: JSON.stringify({ title: title(), owner_id: ownerId }),
      });
      if (!res.ok) throw new Error(`create failed (${res.status})`);
      const created = (await res.json()) as unknown as { canvas: CanvasItem };
      navigate(`/canvas/${created.canvas.id}`);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
      void refetch();
    }
  };

  return (
    <div class="mx-auto max-w-6xl px-4 py-6 sm:px-6 lg:px-8">
      <div class="mb-4 flex items-center justify-between">
        <div>
          <h1 class="font-display text-2xl font-bold text-gray-900 dark:text-white">Canvas</h1>
          <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
            Visual workspaces — nodes, edges, freeform thinking
          </p>
        </div>
      </div>

      <div class="mb-6 flex gap-2">
        <input
          placeholder="New canvas title"
          class="flex-1 max-w-md rounded-none border border-gray-300 px-3 py-2 text-sm dark:border-gray-600 dark:bg-gray-800 dark:text-white"
          value={title()}
          onInput={(e) => setTitle(e.currentTarget.value)}
          onKeyDown={(e) => e.key === "Enter" && void create()}
        />
        <button
          class="rounded-none bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700 disabled:opacity-50"
          disabled={busy() || !title().trim()}
          onClick={() => void create()}
        >
          {busy() ? "Creating…" : "Create Canvas"}
        </button>
      </div>
      <Show when={error()}>
        <p class="mb-3 text-sm text-red-600 dark:text-red-400">{error()}</p>
      </Show>

      <Show when={!canvases.loading} fallback={<div class="h-20 animate-pulse rounded-none bg-gray-200 dark:bg-gray-700" />}>
        <Show
          when={(canvases() ?? []).length > 0}
          fallback={
            <div class="rounded-none border border-gray-900 bg-white p-8 text-center dark:border-gray-100 dark:bg-gray-800">
              <p class="text-lg font-medium text-gray-700 dark:text-gray-300">No canvases yet</p>
              <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
                Create one above — nodes and edges persist to the server.
              </p>
            </div>
          }
        >
          <div class="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
            <For each={canvases()}>
              {(c) => (
                <A
                  href={`/canvas/${c.id}`}
                  class="block rounded-none border-2 border-gray-900 bg-white p-4 shadow-spatial-sm hover:shadow-spatial dark:border-gray-100 dark:bg-gray-800"
                >
                  <p class="font-semibold text-gray-900 dark:text-white">{c.title}</p>
                  <Show when={c.updated_at}>
                    <p class="mt-2 text-xs text-gray-400">
                      updated {new Date(c.updated_at as string).toLocaleDateString()}
                    </p>
                  </Show>
                </A>
              )}
            </For>
          </div>
        </Show>
      </Show>
    </div>
  );
}
