import createClient from "openapi-fetch";
import type { paths } from "./schema";

/**
 * Tachyon API client (ADR-113 Phase A).
 *
 * Types are GENERATED from the server's OpenAPI spec (schema.d.ts) — never
 * hand-edit. Regenerate: `bun run gen:api` (CI gate: regeneration must be
 * a no-op diff, otherwise server/client types have drifted).
 *
 * SAME-ORIGIN architecture — no CORS, ever:
 * - dev:  `astro dev` proxies /api → TACHYON_DEV_TARGET (see astro.config.mjs)
 * - prod: the Tachyon server serves this build; API is same-origin
 * - PUBLIC_TACHYON_API overrides only for split deployments (CloudFlare etc.)
 */

const BUILD_API = import.meta.env.PUBLIC_TACHYON_API ?? "";

export function resolveBaseUrl(): string {
  if (BUILD_API) return BUILD_API;
  if (typeof window !== "undefined") {
    return `${window.location.origin}/api/v1`;
  }
  return "http://localhost:18080/api/v1";
}

const TOKEN_KEY = "tachyon_token";

export function getToken(): string | null {
  if (typeof localStorage === "undefined") return null;
  return localStorage.getItem(TOKEN_KEY);
}

export function setToken(token: string): void {
  localStorage.setItem(TOKEN_KEY, token);
}

export function clearToken(): void {
  localStorage.removeItem(TOKEN_KEY);
}

// --- Tauri transport (desktop) ---------------------------------------------
// Inside the Tauri webview (tauri://localhost) plain fetch cannot reach the
// API server (origin restrictions), so requests route through the api_proxy
// IPC command — the same bridge the desktop Leptos client used.
function isTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI__" in window;
}

async function tauriFetch(input: Request, init?: RequestInit): Promise<Response> {
  const core = (
    window as unknown as {
      __TAURI__: { core: { invoke: (cmd: string, args: unknown) => Promise<unknown> } };
    }
  ).__TAURI__.core;
  const url = new URL(input.url);
  // path includes /api/v1 — the desktop api_proxy does base + path
  const path = url.pathname + url.search;
  const method = (init?.method ?? input.method ?? "GET").toUpperCase();
  let body: unknown = undefined;
  if (init?.body && typeof init.body === "string") {
    try {
      body = JSON.parse(init.body);
    } catch {
      body = init.body;
    }
  } else if (init?.body) {
    body = init.body;
  } else if (method !== "GET" && method !== "HEAD" && input.body) {
    // openapi-fetch may pass the payload on the Request itself (init.body
    // undefined) — drain the stream and parse it, or api_proxy sends no
    // body at all and the server answers 400.
    try {
      const text = await input.clone().text();
      body = text ? JSON.parse(text) : undefined;
    } catch {
      body = undefined;
    }
  }
  const headers: Record<string, string> = {};
  new Headers(init?.headers ?? input.headers).forEach((v, k) => {
    headers[k] = v;
  });
  // auth injection — the non-Tauri branch injects after we return, so the
  // Tauri branch must do its own (regression: every authed call 401'd)
  const token = getToken();
  if (token && !headers["authorization"] && !headers["Authorization"]) {
    headers["Authorization"] = `Bearer ${token}`;
  }
  const result = (await core.invoke("api_proxy", { method, path, body, headers })) as {
    status: number;
    body: unknown;
  };
  return new Response(JSON.stringify(result.body ?? null), {
    status: result.status,
    headers: { "content-type": "application/json" },
  });
}

export const api = createClient<paths>({
  baseUrl: resolveBaseUrl(),
  // openapi-fetch passes a Request whose headers carry Content-Type (and
  // everything else); `init.headers` holds only extra custom options and
  // may be absent. Headers in init would REPLACE the request's, so merge:
  // request headers first, init headers win on conflict, then auth token.
  // (The runtime really does pass two arguments — the published type
  // under-declares it.)
  fetch: ((input: Request, init?: RequestInit) => {
    if (isTauri()) return tauriFetch(input as Request, init);
    const headers = new Headers(input.headers);
    if (init?.headers) {
      new Headers(init.headers).forEach((v, k) => headers.set(k, v));
    }
    const token = getToken();
    if (token) {
      headers.set("Authorization", `Bearer ${token}`);
    }
    return fetch(input, { ...init, headers });
  }) as unknown as (input: Request, init?: RequestInit) => Promise<Response>,
});

/** Typed helpers for the hottest paths — thin, all types flow from schema. */

export async function login(username: string, password: string) {
  const { data, error, response } = await api.POST("/auth/login", {
    body: { username, password } as never,
  });
  if (error || !data) throw new Error(`login failed: ${response.status}`);
  // Server shape: { success, access_token, ... } — narrow via the schema type
  const body = data as unknown as { access_token?: string };
  if (body.access_token) setToken(body.access_token);
  return body;
}

export async function health() {
  // Health lives at the server ROOT (/health), not under /api/v1 — it is not
  // part of the utoipa schema. Untyped by design; it's an ops probe.
  const base = resolveBaseUrl().replace(/\/api\/v1\/?$/, "");
  try {
    const res = await fetch(`${base}/health`);
    return { ok: res.ok, data: res.ok ? await res.json() : null };
  } catch {
    return { ok: false, data: null };
  }
}

export async function me(): Promise<TachyonUser> {
  // NOTE: the OpenAPI spec double-prefixes this path ("/api/v1/auth/me" —
  // utoipa annotation bug, server annotation fix tracked) so the typed
  // client can't express it. Route through the api transport manually:
  // same-origin fetch in web, api_proxy IPC in Tauri.
  const path = "/auth/me";
  if (isTauri()) {
    const core = (
      window as unknown as {
        __TAURI__: { core: { invoke: (cmd: string, args: unknown) => Promise<unknown> } };
      }
    ).__TAURI__.core;
    const result = (await core.invoke("api_proxy", {
      method: "GET",
      path,
      headers: { Authorization: `Bearer ${getToken() ?? ""}` },
    })) as { status: number; body: unknown };
    if (result.status !== 200 || !result.body) throw new Error(`/auth/me ${result.status}`);
    return result.body as unknown as TachyonUser;
  }
  const res = await fetch(`${resolveBaseUrl()}${path}`, {
    headers: getToken() ? { Authorization: `Bearer ${getToken()}` } : undefined,
  });
  if (!res.ok) throw new Error(`/auth/me ${res.status}`);
  return (await res.json()) as TachyonUser;
}

interface TachyonUser {
  id: string;
  username: string;
  display_name?: string | null;
  email?: string | null;
  [k: string]: unknown;
}
