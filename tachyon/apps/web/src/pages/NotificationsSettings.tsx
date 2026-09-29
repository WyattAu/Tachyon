import { Show } from "solid-js";

/** Notifications settings — Solid port (Phase B). Full flow in Phase C
 * (VAPID push subscribe exists in push.rs; preferences endpoint is v2). */

export default function NotificationsSettings() {
  return (
    <div class="mx-auto max-w-4xl px-4 py-6 sm:px-6 lg:px-8">
      <h1 class="font-display text-2xl font-bold text-gray-900 dark:text-white">
        Notification Preferences
      </h1>
      <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
        Push notifications (SSE + VAPID web push) land in Phase C.
      </p>
      <div class="mt-6 rounded-none border border-gray-900 bg-white p-6 opacity-70 dark:border-gray-100 dark:bg-gray-800">
        <p class="text-sm text-gray-500 dark:text-gray-400">
          Per-event preferences (comments, mentions, review requests, digests).
        </p>
      </div>
    </div>
  );
}
