import { Show, createResource, createSignal, For } from "solid-js";
import { resolveBaseUrl, getToken } from "@/lib/api/client";

/** Onboarding — Solid port (Phase B). Status/suggestions/complete. */

async function fetchJson<T>(path: string, fallback: T): Promise<T> {
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

export default function Onboarding() {
  const [status] = createResource(async () =>
    fetchJson<{ completed?: boolean; [k: string]: unknown }>("/onboarding/status", {
      completed: false,
    })
  );
  const [suggestions] = createResource(async () =>
    fetchJson<{ suggestions?: string[] }>("/onboarding/suggestions", { suggestions: [] })
  );
  const [done, setDone] = createSignal(false);
  const [busy, setBusy] = createSignal(false);

  const complete = async () => {
    setBusy(true);
    try {
      await fetchJson("/onboarding/complete", {});
      setDone(true);
    } finally {
      setBusy(false);
    }
  };

  return (
    <div class="mx-auto max-w-3xl px-4 py-6 sm:px-6 lg:px-8">
      <h1 class="font-display text-2xl font-bold text-gray-900 dark:text-white">Welcome to Tachyon</h1>
      <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
        A quick start guide for your knowledge base.
      </p>

      <Show when={done() || status()?.completed}>
        <div class="mt-6 rounded-none border border-green-300 bg-green-50 p-6 dark:border-green-700 dark:bg-green-900/20">
          <p class="font-medium text-green-700 dark:text-green-300">
            Onboarding complete — you're all set.
          </p>
        </div>
      </Show>

      <Show when={!done() && !status()?.completed}>
        <ol class="mt-6 space-y-3 text-sm">
          <li class="rounded-none border border-gray-300 bg-white p-4 dark:border-gray-600 dark:bg-gray-800">
            <strong>1.</strong> Create your first document from the dashboard.
          </li>
          <li class="rounded-none border border-gray-300 bg-white p-4 dark:border-gray-600 dark:bg-gray-800">
            <strong>2.</strong> Organize with spaces and tags.
          </li>
          <li class="rounded-none border border-gray-300 bg-white p-4 dark:border-gray-600 dark:bg-gray-800">
            <strong>3.</strong> Connect your graph — every wikilink builds the knowledge graph.
          </li>
        </ol>

        <Show when={(suggestions()?.suggestions ?? []).length > 0}>
          <h2 class="mt-6 text-lg font-semibold">Suggested next steps</h2>
          <ul class="mt-2 space-y-1 text-sm text-gray-600 dark:text-gray-400">
            <For each={suggestions()?.suggestions ?? []}>{(s) => <li>• {s}</li>}</For>
          </ul>
        </Show>

        <button
          class="mt-6 rounded-none bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700 disabled:opacity-50"
          disabled={busy()}
          onClick={() => void complete()}
        >
          {busy() ? "…" : "Mark onboarding complete"}
        </button>
      </Show>
    </div>
  );
}
