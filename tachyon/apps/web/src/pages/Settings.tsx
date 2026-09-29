import { Show, createResource } from "solid-js";
import { A } from "@solidjs/router";
import { authStore } from "@/lib/auth";

/** Settings — Solid port (Phase B). Profile summary + section links. */

export default function Settings() {
  const [me] = createResource(async () => authStore.user() ?? undefined);

  return (
    <div class="mx-auto max-w-4xl px-4 py-6 sm:px-6 lg:px-8">
      <h1 class="font-display text-2xl font-bold text-gray-900 dark:text-white">Settings</h1>

      <div class="mt-4 border-b border-gray-200 text-sm dark:border-gray-700">
        <span class="inline-block border-b-2 border-blue-600 pb-2 font-medium text-blue-600">
          Profile
        </span>
        <A href="/settings/branding" class="ml-4 inline-block pb-2 text-gray-500 hover:text-gray-700 dark:hover:text-gray-300">Account</A>
        <A href="/settings/webhooks" class="ml-4 inline-block pb-2 text-gray-500 hover:text-gray-700 dark:hover:text-gray-300">Preferences</A>
        <A href="/settings/notifications" class="ml-4 inline-block pb-2 text-gray-500 hover:text-gray-700 dark:hover:text-gray-300">Notifications</A>
      </div>

      <div class="mt-6 rounded-none border border-gray-900 bg-white p-6 dark:border-gray-100 dark:bg-gray-800">
        <h2 class="text-lg font-semibold">Profile</h2>
        <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">Your account information</p>

        <Show when={me()} fallback={<p class="mt-4 text-sm text-gray-500">Loading…</p>}>
          {(u) => (
            <div class="mt-4 space-y-4">
              <div>
                <label class="mb-1 block text-sm font-medium">Display Name</label>
                <input
                  class="w-full rounded-none border border-gray-300 bg-gray-100 px-3 py-2 dark:border-gray-600 dark:bg-gray-700 dark:text-white"
                  value={u().display_name ?? u().username}
                  readOnly
                />
              </div>
              <div>
                <label class="mb-1 block text-sm font-medium">Email</label>
                <input
                  class="w-full rounded-none border border-gray-300 bg-gray-100 px-3 py-2 text-gray-500 dark:border-gray-600 dark:bg-gray-700"
                  value={u().email ?? "—"}
                  readOnly
                />
                <p class="mt-1 text-xs text-gray-400">
                  Email is read-only and managed by your authentication provider.
                </p>
              </div>
            </div>
          )}
        </Show>
      </div>
    </div>
  );
}

