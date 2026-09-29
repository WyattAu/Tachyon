import { For, Show, createResource } from "solid-js";
import { api } from "@/lib/api/client";

/** Admin: Role Management — Solid port (Phase B). Roles + permissions. */

interface Role {
  id: string;
  name: string;
  description: string | null;
  permissions: string[] | null;
  is_system: boolean;
}

export default function AdminRoles() {
  const [roles] = createResource(async () => {
    const { data } = await api.GET("/roles", {});
    return ((data ?? []) as unknown as Role[]) ?? [];
  });

  return (
    <div class="mx-auto max-w-5xl px-4 py-6 sm:px-6 lg:px-8">
      <div class="mb-4 flex items-center justify-between">
        <div>
          <h1 class="font-display text-2xl font-bold text-gray-900 dark:text-white">
            Role Management
          </h1>
          <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
            Manage roles and permissions for your team
          </p>
        </div>
        <button class="rounded-none bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700">
          Create Role
        </button>
      </div>

      <Show when={!roles.loading} fallback={<div class="h-20 animate-pulse rounded-none bg-gray-200 dark:bg-gray-700" />}>
        <Show
          when={(roles() ?? []).length > 0}
          fallback={<p class="text-sm text-gray-500 dark:text-gray-400">No roles defined.</p>}
        >
          <div class="overflow-x-auto rounded-none border-2 border-gray-900 bg-white dark:border-gray-100 dark:bg-gray-800">
            <table class="w-full text-left text-sm">
              <thead class="border-b border-gray-200 text-xs tracking-wider text-gray-500 uppercase dark:border-gray-700">
                <tr>
                  <th class="px-4 py-2">Name</th>
                  <th class="px-4 py-2">Description</th>
                  <th class="px-4 py-2">Permissions</th>
                  <th class="px-4 py-2">Type</th>
                </tr>
              </thead>
              <tbody class="divide-y divide-gray-200 dark:divide-gray-700">
                <For each={roles()}>
                  {(r) => (
                    <tr>
                      <td class="px-4 py-2 font-medium text-gray-900 dark:text-white">{r.name}</td>
                      <td class="px-4 py-2 text-gray-600 dark:text-gray-400">{r.description ?? "—"}</td>
                      <td class="px-4 py-2 font-mono text-xs text-gray-500">
                        {(r.permissions ?? []).join(", ") || "—"}
                      </td>
                      <td class="px-4 py-2">
                        <span class={`rounded-full px-2 py-0.5 text-xs ${r.is_system ? "bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-300" : "bg-gray-100 text-gray-600 dark:bg-gray-700 dark:text-gray-300"}`}>
                          {r.is_system ? "system" : "custom"}
                        </span>
                      </td>
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
