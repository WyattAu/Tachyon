import { For, Show } from "solid-js";
import { themeStore, type ThemeMode } from "@/lib/theme";

/** Themes — Solid port (Phase C). Functional: mode applied app-wide. */

const MODES: Array<{ id: ThemeMode; label: string; desc: string }> = [
  { id: "dark", label: "Dark", desc: "Tachyon default — spatial materialism" },
  { id: "light", label: "Light", desc: "High-contrast daylight" },
  { id: "system", label: "System", desc: "Follow your OS preference" },
];

export default function Themes() {
  return (
    <div class="mx-auto max-w-3xl px-4 py-6 sm:px-6 lg:px-8">
      <h1 class="font-display text-2xl font-bold text-gray-900 dark:text-white">Themes</h1>
      <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
        Appearance applies instantly and persists on this device.
      </p>

      <div class="mt-6 space-y-3">
        <For each={MODES}>
          {(m) => (
            <button
              class={`flex w-full items-center justify-between rounded-none border-2 px-5 py-4 text-left ${
                themeStore.mode() === m.id
                  ? "border-blue-600 bg-blue-50/50 dark:bg-blue-900/20"
                  : "border-gray-300 bg-white hover:border-gray-400 dark:border-gray-600 dark:bg-gray-800 dark:hover:border-gray-500"
              }`}
              onClick={() => themeStore.set(m.id)}
            >
              <div>
                <p class="font-medium text-gray-900 dark:text-white">{m.label}</p>
                <p class="text-xs text-gray-500 dark:text-gray-400">{m.desc}</p>
              </div>
              <Show when={themeStore.mode() === m.id}>
                <span class="rounded-full bg-blue-600 px-2 py-0.5 text-xs text-white">active</span>
              </Show>
            </button>
          )}
        </For>
      </div>

      <p class="mt-6 text-xs text-gray-400">
        Custom accent colors + brand theming: see Branding (server-side brand
        settings API is a tracked Phase C server gap — the old Leptos client
        called a /settings/brand route that has no server implementation).
      </p>
    </div>
  );
}
