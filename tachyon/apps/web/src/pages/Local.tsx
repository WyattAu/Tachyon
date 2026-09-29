import { A } from "@solidjs/router";

/** Local Documents — Solid port (Phase B). Offline store browser. */

export default function Local() {
  return (
    <div class="mx-auto max-w-4xl px-4 py-6 sm:px-6 lg:px-8">
      <h1 class="font-display text-2xl font-bold text-gray-900 dark:text-white">
        Local Documents
      </h1>
      <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
        Documents stored only on this device. These have not been synced to any server.
      </p>
      <div class="mt-8 rounded-none border border-gray-900 bg-white p-10 text-center dark:border-gray-100 dark:bg-gray-800">
        <p class="text-lg font-medium text-gray-700 dark:text-gray-300">No local documents</p>
        <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
          Documents you create offline will appear here.
        </p>
        <A
          href="/documents"
          class="mt-4 inline-block text-sm text-blue-600 hover:underline dark:text-blue-400"
        >
          Go to server documents →
        </A>
      </div>
    </div>
  );
}
