import { Show, createResource, createSignal } from "solid-js";
import { api, resolveBaseUrl, getToken } from "@/lib/api/client";

/** SSG — Solid port (Phase B). Site config + build + download. */

interface SsgConfig {
  site_title: string;
  site_description: string;
  base_url: string;
  theme: string;
}

export default function Ssg() {
  const [config, { refetch }] = createResource(async () => {
    const { data } = await api.GET("/ssg/config", {});
    return (data ?? {
      site_title: "Tachyon Docs",
      site_description: "A knowledge base built with Tachyon",
      base_url: "https://docs.example.com",
      theme: "auto",
    }) as unknown as SsgConfig;
  });

  const [buildState, setBuildState] = createSignal<"idle" | "building" | "done" | "error">("idle");
  const [buildMsg, setBuildMsg] = createSignal("");

  const build = async () => {
    setBuildState("building");
    try {
      const res = await fetch(`${resolveBaseUrl()}/ssg/build`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${getToken() ?? ""}`,
        },
        body: JSON.stringify({}),
      });
      if (!res.ok) throw new Error(`build failed (${res.status})`);
      setBuildMsg("Site built — use Download Latest to fetch it.");
      setBuildState("done");
    } catch (e) {
      setBuildMsg(e instanceof Error ? e.message : String(e));
      setBuildState("error");
    }
  };

  return (
    <div class="mx-auto max-w-6xl px-4 py-6 sm:px-6 lg:px-8">
      <h1 class="font-display text-2xl font-bold text-gray-900 dark:text-white">
        Static Site Generator
      </h1>
      <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
        Generate a static site from your published documents
      </p>

      <div class="mt-6 rounded-none border border-gray-900 bg-white p-6 dark:border-gray-100 dark:bg-gray-800">
        <h2 class="mb-4 text-lg font-semibold">Site Configuration</h2>

        <Show when={!config.loading} fallback={<div class="h-40 animate-pulse rounded bg-gray-200 dark:bg-gray-700" />}>
          <label class="mb-1 block text-sm font-medium" for="ssg-title">
            Site Title
          </label>
          <input
            id="ssg-title"
            class="mb-3 w-full rounded-none border border-gray-300 bg-white px-3 py-2 dark:border-gray-600 dark:bg-gray-700 dark:text-white"
            value={config()?.site_title ?? ""}
            readOnly
          />
          <label class="mb-1 block text-sm font-medium" for="ssg-desc">
            Description
          </label>
          <input
            id="ssg-desc"
            class="mb-3 w-full rounded-none border border-gray-300 bg-white px-3 py-2 dark:border-gray-600 dark:bg-gray-700 dark:text-white"
            value={config()?.site_description ?? ""}
            readOnly
          />
          <label class="mb-1 block text-sm font-medium" for="ssg-url">
            Base URL
          </label>
          <input
            id="ssg-url"
            class="mb-3 w-full rounded-none border border-gray-300 bg-white px-3 py-2 dark:border-gray-600 dark:bg-gray-700 dark:text-white"
            value={config()?.base_url ?? ""}
            readOnly
          />
          <label class="mb-1 block text-sm font-medium" for="ssg-theme">
            Theme
          </label>
          <select
            id="ssg-theme"
            class="mb-4 w-full rounded-none border border-gray-300 px-3 py-2 dark:border-gray-600"
            value={config()?.theme ?? "auto"}
          >
            <option value="auto">Auto (system preference)</option>
            <option value="light">Light</option>
            <option value="dark">Dark</option>
          </select>

          <div class="flex items-center gap-3">
            <button
              class="rounded-none bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700"
              disabled={buildState() === "building"}
              onClick={() => void build()}
            >
              {buildState() === "building" ? "Building…" : "</> Build Site"}
            </button>
            <button class="rounded-none border border-gray-300 px-4 py-2 text-sm hover:bg-gray-100 dark:border-gray-600 dark:hover:bg-gray-700">
              Download Latest
            </button>
            <Show when={buildMsg()}>
              <span class={`text-sm ${buildState() === "error" ? "text-red-600" : "text-green-600"}`}>
                {buildMsg()}
              </span>
            </Show>
          </div>
        </Show>
      </div>
    </div>
  );
}
