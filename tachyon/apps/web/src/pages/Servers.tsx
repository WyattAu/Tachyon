import { For, Show, createSignal, onMount } from "solid-js";

/**
 * Server Connections — Solid port (Phase B). Pure client-side (localStorage
 * registry, same "tachyon_servers" key contract as the desktop).
 */

interface ServerEntry {
  id: string;
  name: string;
  url: string;
}

const KEY = "tachyon_servers";

export default function Servers() {
  const [servers, setServers] = createSignal<ServerEntry[]>([]);
  const [name, setName] = createSignal("");
  const [url, setUrl] = createSignal("");

  onMount(() => {
    try {
      setServers(JSON.parse(localStorage.getItem(KEY) ?? "[]"));
    } catch {
      setServers([]);
    }
  });

  const persist = (list: ServerEntry[]) => {
    setServers(list);
    localStorage.setItem(KEY, JSON.stringify(list));
  };

  const add = () => {
    if (!name().trim() || !url().trim()) return;
    persist([...servers(), { id: crypto.randomUUID(), name: name().trim(), url: url().trim() }]);
    setName("");
    setUrl("");
  };

  const remove = (id: string) => persist(servers().filter((s) => s.id !== id));

  return (
    <div class="mx-auto max-w-4xl px-4 py-6 sm:px-6 lg:px-8">
      <h1 class="font-display text-2xl font-bold text-gray-900 dark:text-white">Server Connections</h1>
      <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
        Manage your Tachyon server connections. Each server has its own documents and authentication.
      </p>

      <div class="mt-4 space-y-2">
        <For each={servers()}>
          {(s) => (
            <div class="flex items-center justify-between rounded-none border-2 border-blue-900 bg-blue-50/40 px-4 py-3 dark:border-blue-200 dark:bg-blue-900/20">
              <div>
                <p class="font-medium text-gray-900 dark:text-white">{s.name}</p>
                <p class="text-xs text-gray-500 dark:text-gray-400">{s.url}</p>
              </div>
              <button class="text-sm text-red-600 hover:underline" onClick={() => remove(s.id)}>
                Remove
              </button>
            </div>
          )}
        </For>
      </div>

      <div class="mt-6 flex gap-2">
        <input
          placeholder="Name"
          class="w-40 rounded-none border border-gray-300 px-3 py-2 text-sm dark:border-gray-600 dark:bg-gray-800 dark:text-white"
          value={name()}
          onInput={(e) => setName(e.currentTarget.value)}
        />
        <input
          placeholder="http://192.168.1.191:18080"
          class="flex-1 rounded-none border border-gray-300 px-3 py-2 text-sm dark:border-gray-600 dark:bg-gray-800 dark:text-white"
          value={url()}
          onInput={(e) => setUrl(e.currentTarget.value)}
        />
        <button class="rounded-none bg-blue-600 px-4 py-2 text-sm text-white hover:bg-blue-700" onClick={add}>
          Add Server
        </button>
      </div>
    </div>
  );
}
