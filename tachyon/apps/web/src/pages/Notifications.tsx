import { For, Show, onMount, createSignal } from "solid-js";
import { A } from "@solidjs/router";
import { notificationsStore } from "@/lib/notifications";

/** Notifications — Solid port (Phase C). SSE-live list with read management. */

export default function Notifications() {
  onMount(() => {
    void notificationsStore.load();
    notificationsStore.connectStream();
  });

  const [filter, setFilter] = createSignal<"all" | "unread">("all");
  const list = () => {
    const ns = notificationsStore.notifications();
    return filter() === "unread" ? ns.filter((n) => !n.read) : ns;
  };

  return (
    <div class="mx-auto max-w-4xl px-4 py-6 sm:px-6 lg:px-8">
      <div class="mb-4 flex items-center justify-between">
        <div>
          <h1 class="font-display text-2xl font-bold text-gray-900 dark:text-white">
            Notifications
          </h1>
          <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
            {notificationsStore.unread()} unread ·{" "}
            <span class={notificationsStore.connected() ? "text-green-600" : "text-gray-400"}>
              {notificationsStore.connected() ? "live" : "polling"}
            </span>
          </p>
        </div>
        <div class="flex gap-2">
          <button
            class={`rounded-full px-3 py-1 text-sm ${filter() === "all" ? "bg-blue-600 text-white" : "border border-gray-300 dark:border-gray-600"}`}
            onClick={() => setFilter("all")}
          >
            All
          </button>
          <button
            class={`rounded-full px-3 py-1 text-sm ${filter() === "unread" ? "bg-blue-600 text-white" : "border border-gray-300 dark:border-gray-600"}`}
            onClick={() => setFilter("unread")}
          >
            Unread
          </button>
          <button
            class="rounded-none border border-gray-300 px-3 py-1 text-sm hover:bg-gray-100 dark:border-gray-600 dark:hover:bg-gray-700"
            onClick={() => void notificationsStore.markAllRead()}
          >
            Mark all read
          </button>
        </div>
      </div>

      <Show
        when={list().length > 0}
        fallback={
          <div class="rounded-none border border-gray-900 bg-white p-10 text-center dark:border-gray-100 dark:bg-gray-800">
            <p class="text-lg font-medium text-gray-700 dark:text-gray-300">
              You're all caught up
            </p>
          </div>
        }
      >
        <div class="space-y-2">
          <For each={list()}>
            {(n) => (
              <div
                class={`flex items-start justify-between rounded-none border px-4 py-3 ${n.read ? "border-gray-200 bg-white opacity-70 dark:border-gray-700 dark:bg-gray-800" : "border-blue-300 bg-blue-50/50 dark:border-blue-800 dark:bg-blue-900/20"}`}
              >
                <div>
                  <p class="font-medium text-gray-900 dark:text-white">
                    <Show when={!n.read}>
                      <span class="mr-2 inline-block h-2 w-2 rounded-full bg-blue-600" />
                    </Show>
                    {n.title}
                  </p>
                  <Show when={n.body}>
                    <p class="mt-0.5 text-sm text-gray-600 dark:text-gray-400">{n.body}</p>
                  </Show>
                  <p class="mt-1 text-xs text-gray-400">
                    {n.type ?? "notification"}
                    <Show when={n.created_at}>
                      {" · "}
                      {new Date(n.created_at as string).toLocaleString()}
                    </Show>
                  </p>
                </div>
                <div class="flex flex-col items-end gap-1">
                  <Show when={!n.read}>
                    <button
                      class="text-xs text-blue-600 hover:underline dark:text-blue-400"
                      onClick={() => void notificationsStore.markRead(n.id)}
                    >
                      Mark read
                    </button>
                  </Show>
                  <Show when={n.link}>
                    <A href={n.link as string} class="text-xs text-gray-500 hover:underline">
                      Open
                    </A>
                  </Show>
                </div>
              </div>
            )}
          </For>
        </div>
      </Show>
    </div>
  );
}

