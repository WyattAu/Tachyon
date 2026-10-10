import { For, Show, createResource, createSignal } from "solid-js";
import { api, resolveBaseUrl, getToken } from "@/lib/api/client";

/**
 * Plugins — Solid port (Phase C) with the OSS marketplace.
 * "Installed" tab: GET /plugins (typed). "Marketplace" tab: curated
 * community registry (public/plugin-registry.json — OSS v1 ships it with
 * the app; remote registry URL later). Install = POST /plugins with the
 * registry manifest (CreatePluginBody).
 */

interface Plugin {
  id: string;
  name: string;
  description: string | null;
  version?: string;
  author: string | null;
  homepage: string | null;
  license: string | null;
  enabled: boolean;
}

interface RegistryEntry {
  name: string;
  description: string;
  version: string;
  author: string;
  homepage: string;
  license: string;
  extension_points: string[];
  runtime_type: string;
  entry_point: string;
}

export default function Plugins() {
  const [tab, setTab] = createSignal<"installed" | "marketplace">("installed");
  const [plugins, { refetch }] = createResource(async () => {
    const { data } = await api.GET("/plugins", { params: { query: {} } });
    return ((data ?? []) as unknown as Plugin[]) ?? [];
  });
  // Registry source: a remote URL when configured (localStorage override —
  // the deployment-level knob is the same contract served by host env), with
  // the bundled copy as offline fallback. Dogfooding happens at arm's length:
  // the tachyon-plugins repo has its own cadence and must not require a
  // Tachyon release to add a plugin.
  const [registrySource, setRegistrySource] = createSignal<"remote" | "bundled">("bundled");
  const [registry] = createResource(async () => {
    const remote =
      localStorage.getItem("tachyon_registry_url")?.trim() ||
      (window as unknown as { __TACHYON_REGISTRY_URL__?: string }).__TACHYON_REGISTRY_URL__ ||
      "";
    if (remote) {
      try {
        const res = await fetch(remote);
        if (res.ok) {
          const body = (await res.json()) as { plugins: RegistryEntry[] };
          if (Array.isArray(body.plugins)) {
            setRegistrySource("remote");
            return body.plugins;
          }
        }
      } catch {
        // remote unreachable — fall through to the bundled copy
      }
    }
    const res = await fetch("/plugin-registry.json");
    if (!res.ok) return [];
    return ((await res.json()) as unknown as { plugins: RegistryEntry[] }).plugins ?? [];
  });

  const [busyName, setBusyName] = createSignal<string | null>(null);
  const [message, setMessage] = createSignal("");

  const installedNames = () => new Set((plugins() ?? []).map((p) => p.name));
  const marketplaceEntries = () =>
    (registry() ?? []).filter((r) => !installedNames().has(r.name));

  const install = async (entry: RegistryEntry) => {
    setBusyName(entry.name);
    setMessage("");
    try {
      const res = await fetch(`${resolveBaseUrl()}/plugins`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${getToken() ?? ""}`,
        },
        body: JSON.stringify({
          name: entry.name,
          description: entry.description,
          version: entry.version,
          author: entry.author,
          homepage: entry.homepage,
          license: entry.license,
          extension_points: entry.extension_points,
          runtime_type: entry.runtime_type,
          entry_point: entry.entry_point,
          enabled: true,
        }),
      });
      if (!res.ok) throw new Error(`install failed (${res.status})`);
      setMessage(`Installed ${entry.name}.`);
      void refetch();
    } catch (e) {
      setMessage(e instanceof Error ? e.message : String(e));
    } finally {
      setBusyName(null);
    }
  };

  return (
    <div class="mx-auto max-w-6xl px-4 py-6 sm:px-6 lg:px-8">
      <div class="mb-4 flex items-center justify-between">
        <div>
          <h1 class="font-display text-2xl font-bold text-gray-900 dark:text-white">Plugins</h1>
          <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
            Extend Tachyon with community and custom plugins
          </p>
        </div>
        <div class="flex rounded-none border border-gray-300 dark:border-gray-600">
          <button
            class={`px-4 py-2 text-sm ${tab() === "installed" ? "bg-blue-600 text-white" : "bg-white hover:bg-gray-50 dark:bg-gray-800 dark:hover:bg-gray-700"}`}
            onClick={() => setTab("installed")}
          >
            Installed
          </button>
          <button
            class={`px-4 py-2 text-sm ${tab() === "marketplace" ? "bg-blue-600 text-white" : "bg-white hover:bg-gray-50 dark:bg-gray-800 dark:hover:bg-gray-700"}`}
            onClick={() => setTab("marketplace")}
          >
            Marketplace
          </button>
        </div>
      </div>

      <Show when={message()}>
        <p class="mb-3 text-sm text-green-600 dark:text-green-400">{message()}</p>
      </Show>

      <Show
        when={tab() === "installed"}
        fallback={
          <div>
            <p class="mb-3 text-xs text-gray-500" data-testid="registry-source">
              {registrySource() === "remote"
                ? "registry: remote (tachyon-plugins)"
                : "registry: bundled"}
            </p>
            <MarketplaceList entries={marketplaceEntries()} busyName={busyName()} onInstall={install} />
          </div>
        }
      >
        <Show when={!plugins.loading} fallback={<div class="h-24 animate-pulse rounded-none bg-gray-200 dark:bg-gray-700" />}>
          <Show
            when={(plugins() ?? []).length > 0}
            fallback={
              <div class="py-10 text-center">
                <p class="text-lg font-medium text-gray-700 dark:text-gray-300">No plugins installed</p>
                <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
                  Browse the Marketplace tab to install community plugins.
                </p>
              </div>
            }
          >
            <div class="space-y-2">
              <For each={plugins()}>
                {(pl) => (
                  <div class="flex items-center justify-between rounded-none border-2 border-gray-900 bg-white px-4 py-3 dark:border-gray-100 dark:bg-gray-800">
                    <div>
                      <p class="font-medium text-gray-900 dark:text-white">
                        {pl.name}{" "}
                        <span class="ml-1 text-xs text-gray-400">v{pl.version ?? "?"}</span>{" "}
                        <span class={`ml-2 rounded-full px-2 py-0.5 text-xs ${pl.enabled ? "bg-green-100 text-green-700 dark:bg-green-900/40 dark:text-green-300" : "bg-gray-200 text-gray-600 dark:bg-gray-700 dark:text-gray-300"}`}>
                          {pl.enabled ? "enabled" : "disabled"}
                        </span>
                      </p>
                      <Show when={pl.description}>
                        <p class="text-xs text-gray-500 dark:text-gray-400">{pl.description}</p>
                      </Show>
                    </div>
                    <span class="text-xs text-gray-400">by {pl.author ?? "unknown"}</span>
                  </div>
                )}
              </For>
            </div>
          </Show>
        </Show>
      </Show>
    </div>
  );
}

function MarketplaceList(props: {
  entries: RegistryEntry[];
  busyName: string | null;
  onInstall: (entry: RegistryEntry) => Promise<void>;
}) {
  return (
    <Show
      when={props.entries.length > 0}
      fallback={
        <div class="py-10 text-center">
          <p class="text-lg font-medium text-gray-700 dark:text-gray-300">Marketplace is up to date</p>
          <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
            All registry plugins are installed.
          </p>
        </div>
      }
    >
      <div class="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
        <For each={props.entries}>
          {(entry) => (
            <div class="rounded-none border-2 border-gray-900 bg-white p-4 shadow-spatial-sm dark:border-gray-100 dark:bg-gray-800">
              <div class="flex items-start justify-between">
                <p class="font-semibold text-gray-900 dark:text-white">{entry.name}</p>
                <span class="text-xs text-gray-400">v{entry.version}</span>
              </div>
              <p class="mt-1 text-sm text-gray-600 dark:text-gray-400">{entry.description}</p>
              <div class="mt-2 flex flex-wrap gap-1">
                <For each={entry.extension_points}>
                  {(ep) => (
                    <span class="rounded-full bg-gray-100 px-2 py-0.5 text-xs text-gray-600 dark:bg-gray-700 dark:text-gray-300">
                      {ep}
                    </span>
                  )}
                </For>
              </div>
              <div class="mt-3 flex items-center justify-between">
                <span class="text-xs text-gray-400">
                  {entry.author} · {entry.license}
                </span>
                <button
                  class="rounded-none bg-blue-600 px-3 py-1.5 text-sm text-white hover:bg-blue-700 disabled:opacity-50"
                  disabled={props.busyName === entry.name}
                  onClick={() => void props.onInstall(entry)}
                >
                  {props.busyName === entry.name ? "Installing…" : "Install"}
                </button>
              </div>
            </div>
          )}
        </For>
      </div>
    </Show>
  );
}
