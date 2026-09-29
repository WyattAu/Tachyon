import { Show, createSignal } from "solid-js";
import { authStore } from "@/lib/auth";

/**
 * Profile — Solid port (Phase B). Displays /auth/me fields; email is
 * read-only (server-managed). Display name edit lands with the PUT /auth/me
 * wiring in Phase C.
 */

export default function Profile() {
  const [me] = createSignal(authStore.user());

  return (
    <div class="mx-auto max-w-3xl px-4 py-6 sm:px-6 lg:px-8">
      <h1 class="font-display text-2xl font-bold text-gray-900 dark:text-white">Profile</h1>
      <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">Your account details</p>

      <Show
        when={me()}
        fallback={
          <div class="mt-6 h-24 animate-pulse rounded-none bg-gray-200 dark:bg-gray-700" />
        }
      >
        {(u) => (
          <div class="mt-6 rounded-none border border-gray-900 bg-white p-6 dark:border-gray-100 dark:bg-gray-800">
            <div class="flex items-center gap-4">
              <div class="flex h-14 w-14 items-center justify-center rounded-full bg-blue-600 text-xl font-bold text-white">
                {(u().display_name ?? u().username ?? "?").charAt(0).toUpperCase()}
              </div>
              <div>
                <p class="text-lg font-semibold text-gray-900 dark:text-white">
                  {u().display_name ?? u().username}
                </p>
                <p class="text-sm text-gray-500 dark:text-gray-400">@{u().username}</p>
              </div>
            </div>
            <dl class="mt-6 space-y-3 text-sm">
              <div class="flex justify-between border-b border-gray-200 pb-2 dark:border-gray-700">
                <dt class="text-gray-500 dark:text-gray-400">User ID</dt>
                <dd class="font-mono text-xs text-gray-700 dark:text-gray-300">{u().id}</dd>
              </div>
              <div class="flex justify-between border-b border-gray-200 pb-2 dark:border-gray-700">
                <dt class="text-gray-500 dark:text-gray-400">Email</dt>
                <dd class="text-gray-700 dark:text-gray-300">{u().email ?? "—"}</dd>
              </div>
            </dl>
          </div>
        )}
      </Show>
    </div>
  );
}
