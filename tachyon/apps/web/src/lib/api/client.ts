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

export const api = createClient<paths>({
  baseUrl: resolveBaseUrl(),
  fetch: (input, init) => {
    // openapi-fetch passes a Request whose headers carry Content-Type (and
    // everything else); `init.headers` holds only extra custom options and
    // may be absent. Headers in init would REPLACE the request's, so merge:
    // request headers first, init headers win on conflict, then auth token.
    const headers = new Headers(
      input instanceof Request ? input.headers : undefined
    );
    if (init?.headers) {
      new Headers(init.headers).forEach((v, k) => headers.set(k, v));
    }
    const token = getToken();
    if (token) {
      headers.set("Authorization", `Bearer ${token}`);
    }
    return fetch(input, { ...init, headers });
  },
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

export async function me() {
  const { data, error, response } = await api.GET("/auth/me");
  if (error) throw new Error(`/auth/me ${response.status}`);
  return data;
}
