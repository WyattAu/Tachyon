import { Show } from "solid-js";

/** Branding settings — Solid port (Phase B). Server brand settings. */

export default function BrandingSettings() {
  return (
    <div class="mx-auto max-w-4xl px-4 py-6 sm:px-6 lg:px-8">
      <h1 class="font-display text-2xl font-bold text-gray-900 dark:text-white">
        Branding
      </h1>
      <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
        Customize your instance identity. Server settings API wiring lands in
        the Phase C settings pass (GET/PUT brand settings exist server-side).
      </p>
      <div class="mt-6 rounded-none border border-gray-900 bg-white p-6 opacity-70 dark:border-gray-100 dark:bg-gray-800">
        <p class="text-sm text-gray-500 dark:text-gray-400">
          Site name, logo, and accent color configuration.
        </p>
      </div>
    </div>
  );
}
