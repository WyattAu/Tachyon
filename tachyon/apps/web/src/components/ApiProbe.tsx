import { createSignal, onMount, For, Show } from "solid-js";
import { health, login, me, getToken } from "@/lib/api/client";

/**
 * Phase A verification island: proves the GENERATED typed client talks to
 * staging end-to-end (health → login → /auth/me) from the Solid runtime.
 * This page is scaffolding — replaced by the real shell in Phase B.
 */
export default function ApiProbe() {
  const [status, setStatus] = createSignal("checking…");
  const [lines, setLines] = createSignal<string[]>([]);

  const log = (s: string) => setLines((prev) => [...prev, s]);

  onMount(async () => {
    try {
      const h = await health();
      log(`health: ${h.ok ? "OK" : "FAIL"}`);
      if (!getToken()) {
        const r = await login("admin", "admin123");
        log(`login: token ${r.access_token ? "received" : "MISSING"}`);
      } else {
        log("login: token already present");
      }
      const user = await me();
      log(`auth/me: ${JSON.stringify(user).slice(0, 120)}`);
      setStatus("all green");
    } catch (e) {
      log(`ERROR: ${e instanceof Error ? e.message : String(e)}`);
      setStatus("failed");
    }
  });

  return (
    <div class="mx-auto max-w-xl p-8">
      <h1 class="font-display text-2xl font-bold">Tachyon web — API probe</h1>
      <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
        Typed OpenAPI client → staging 192.168.1.191:18080
      </p>
      <p
        class="mt-4 inline-block rounded px-3 py-1 font-mono text-sm"
        classList={{
          "bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200": status() === "all green",
          "bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200": status() === "failed",
          "bg-gray-200 text-gray-700 dark:bg-gray-800 dark:text-gray-300": status() !== "all green" && status() !== "failed",
        }}
      >
        {status()}
      </p>
      <pre class="mt-4 overflow-auto rounded border border-gray-200 bg-white p-4 text-xs dark:border-gray-700 dark:bg-gray-800">
        <For each={lines()} fallback={"…"}>
          {(line) => <div>{line}</div>}
        </For>
      </pre>
    </div>
  );
}
