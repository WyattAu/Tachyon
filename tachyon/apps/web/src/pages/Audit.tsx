import { For, Show, createResource, createSignal } from "solid-js";
import { resolveBaseUrl, getToken } from "@/lib/api/client";

/**
 * Audit Log — Solid port (Phase B). GET /admin/audit (hand-typed — the
 * endpoint is not yet in the OpenAPI spec; tracked for annotation cleanup).
 */

interface AuditEntry {
  id?: string;
  timestamp?: string;
  action?: string;
  user_id?: string;
  resource?: string;
  details?: string;
}

async function fetchAudit(): Promise<AuditEntry[]> {
  try {
    const res = await fetch(`${resolveBaseUrl()}/admin/audit`, {
      headers: { Authorization: `Bearer ${getToken() ?? ""}` },
    });
    if (!res.ok) return [];
    const body = (await res.json()) as unknown as {
      entries?: AuditEntry[];
      audit?: AuditEntry[];
    } | AuditEntry[];
    return Array.isArray(body) ? body : (body.entries ?? body.audit ?? []);
  } catch {
    return [];
  }
}

export default function Audit() {
  const [entries] = createResource(fetchAudit);
  const [actionFilter, setActionFilter] = createSignal("");

  const filtered = () => {
    const f = actionFilter().toLowerCase();
    const list = entries() ?? [];
    if (!f) return list;
    return list.filter((e) => (e.action ?? "").toLowerCase().includes(f));
  };

  return (
    <div class="mx-auto max-w-6xl px-4 py-6 sm:px-6 lg:px-8">
      <div class="mb-4 flex items-center justify-between">
        <div>
          <h1 class="font-display text-2xl font-bold text-gray-900 dark:text-white">Audit Log</h1>
          <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
            Track all actions and changes across your organization.
          </p>
        </div>
        <button class="rounded-none border border-gray-300 px-4 py-2 text-sm hover:bg-gray-100 dark:border-gray-600 dark:hover:bg-gray-700">
          Export CSV
        </button>
      </div>

      <div class="rounded-none border-2 border-gray-900 bg-white p-4 dark:border-gray-100 dark:bg-gray-800">
        <div class="flex flex-col gap-2 sm:flex-row">
          <input
            type="text"
            placeholder="e.g. document.create"
            class="flex-1 rounded-none border border-gray-300 bg-white px-3 py-2 text-sm dark:border-gray-600 dark:bg-gray-700 dark:text-white"
            value={actionFilter()}
            onInput={(e) => setActionFilter(e.currentTarget.value)}
          />
          <span class="rounded-none bg-blue-600 px-4 py-2 text-center text-sm text-white">Filter</span>
          <button
            class="rounded-none px-3 py-2 text-sm text-gray-500 hover:underline"
            onClick={() => setActionFilter("")}
          >
            Clear
          </button>
        </div>
      </div>

      <Show when={!entries.loading} fallback={<div class="mt-4 h-20 animate-pulse rounded-none bg-gray-200 dark:bg-gray-700" />}>
        <Show
          when={filtered().length > 0}
          fallback={
            <div class="mt-4 rounded-none border border-gray-900 bg-white p-8 text-center text-sm text-gray-500 dark:border-gray-100 dark:bg-gray-800 dark:text-gray-400">
              No audit log entries found.
            </div>
          }
        >
          <div class="mt-4 overflow-x-auto rounded-none border-2 border-gray-900 bg-white dark:border-gray-100 dark:bg-gray-800">
            <table class="w-full text-left text-sm">
              <thead class="border-b border-gray-200 text-xs tracking-wider text-gray-500 uppercase dark:border-gray-700">
                <tr>
                  <th class="px-4 py-2">Time</th>
                  <th class="px-4 py-2">Action</th>
                  <th class="px-4 py-2">Actor</th>
                  <th class="px-4 py-2">Resource</th>
                </tr>
              </thead>
              <tbody class="divide-y divide-gray-200 dark:divide-gray-700">
                <For each={filtered()}>
                  {(e) => (
                    <tr>
                      <td class="px-4 py-2 text-xs text-gray-500">
                        {e.timestamp ? new Date(e.timestamp).toLocaleString() : "—"}
                      </td>
                      <td class="px-4 py-2 font-mono text-xs">{e.action ?? "—"}</td>
                      <td class="px-4 py-2">{e.user_id ?? "—"}</td>
                      <td class="px-4 py-2">{e.resource ?? "—"}</td>
                    </tr>
                  )}
                </For>
              </tbody>
            </table>
          </div>
        </Show>
      </Show>
    </div>
  );
}
