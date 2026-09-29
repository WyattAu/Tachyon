import { For, Show, createResource, createSignal } from "solid-js";
import { resolveBaseUrl, getToken } from "@/lib/api/client";

/**
 * Webhooks settings — Solid port (Phase B). List + create + delete.
 * GET/POST /webhooks, DELETE /webhooks/{id} — hand-typed (spec gap).
 */

interface Webhook {
  id: string;
  url: string;
  events?: string[];
  active?: boolean;
}

export default function WebhooksSettings() {
  const [hooks, { refetch }] = createResource(async () => {
    const res = await fetch(`${resolveBaseUrl()}/webhooks`, {
      headers: { Authorization: `Bearer ${getToken() ?? ""}` },
    });
    if (!res.ok) return [];
    const body = (await res.json()) as unknown as { webhooks?: Webhook[] } | Webhook[];
    return Array.isArray(body) ? body : (body.webhooks ?? []);
  });

  const [url, setUrl] = createSignal("");
  const [busy, setBusy] = createSignal(false);

  const add = async () => {
    if (!url().trim()) return;
    setBusy(true);
    try {
      await fetch(`${resolveBaseUrl()}/webhooks`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${getToken() ?? ""}`,
        },
        body: JSON.stringify({ url: url() }),
      });
      setUrl("");
      void refetch();
    } finally {
      setBusy(false);
    }
  };

  const remove = async (id: string) => {
    await fetch(`${resolveBaseUrl()}/webhooks/${id}`, {
      method: "DELETE",
      headers: { Authorization: `Bearer ${getToken() ?? ""}` },
    });
    void refetch();
  };

  return (
    <div class="mx-auto max-w-4xl px-4 py-6 sm:px-6 lg:px-8">
      <h1 class="font-display text-2xl font-bold text-gray-900 dark:text-white">Webhooks</h1>
      <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
        HTTP callbacks for document and collaboration events.
      </p>

      <div class="mt-4 flex gap-2">
        <input
          placeholder="https://example.com/hook"
          class="flex-1 rounded-none border border-gray-300 px-3 py-2 text-sm dark:border-gray-600 dark:bg-gray-800 dark:text-white"
          value={url()}
          onInput={(e) => setUrl(e.currentTarget.value)}
        />
        <button
          class="rounded-none bg-blue-600 px-4 py-2 text-sm text-white hover:bg-blue-700 disabled:opacity-50"
          disabled={busy()}
          onClick={() => void add()}
        >
          Add
        </button>
      </div>

      <Show when={!hooks.loading} fallback={<div class="mt-4 h-16 animate-pulse rounded-none bg-gray-200 dark:bg-gray-700" />}>
        <Show
          when={(hooks() ?? []).length > 0}
          fallback={<p class="mt-4 text-sm text-gray-500 dark:text-gray-400">No webhooks configured.</p>}
        >
          <div class="mt-4 space-y-2">
            <For each={hooks()}>
              {(h) => (
                <div class="flex items-center justify-between rounded-none border border-gray-300 bg-white px-4 py-2 dark:border-gray-600 dark:bg-gray-800">
                  <span class="font-mono text-xs text-gray-700 dark:text-gray-300">{h.url}</span>
                  <button class="text-sm text-red-600 hover:underline" onClick={() => void remove(h.id)}>
                    Remove
                  </button>
                </div>
              )}
            </For>
          </div>
        </Show>
      </Show>
    </div>
  );
}
