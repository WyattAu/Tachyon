import { createSignal } from "solid-js";
import { createMemo } from "solid-js";
import { api, me, getToken, setToken, clearToken } from "@/lib/api/client";

/**
 * Auth store (Phase A). Token in localStorage; user profile fetched from
 * /auth/me on boot when a token exists. Same storage key as the desktop app
 * ("tachyon_token") so both clients can share a browser profile if needed.
 */

export interface TachyonUser {
  id: string;
  username: string;
  display_name?: string | null;
  email?: string | null;
  [k: string]: unknown;
}

const [user, setUser] = createSignal<TachyonUser | null>(null);
const [authState, setAuthState] = createSignal<
  "unknown" | "authenticated" | "anonymous"
>("unknown");

export const authStore = {
  user,
  state: authState,
  isAuthenticated: createMemo(() => authState() === "authenticated"),

  /** Boot-time restore: if a token exists, fetch /auth/me to validate it. */
  async restore(): Promise<void> {
    if (!getToken()) {
      setAuthState("anonymous");
      return;
    }
    try {
      const profile = await me();
      if (profile) {
        setUser(profile);
        setAuthState("authenticated");
        return;
      }
    } catch {
      // token invalid/expired
    }
    clearToken();
    setAuthState("anonymous");
  },

  async login(username: string, password: string): Promise<void> {
    const body = (await api.POST("/auth/login", {
      body: { username, password } as never,
    })).data as unknown as { access_token?: string; error?: string } | undefined;
    const token = body?.access_token;
    if (!token) {
      throw new Error(body?.error ?? "login failed");
    }
    setToken(token);
    const profile = await me();
    setUser(profile);
    setAuthState("authenticated");
  },

  async guestLogin(): Promise<void> {
    const body = (await api.POST("/auth/guest", {} as never))
      .data as unknown as { access_token?: string } | undefined;
    const token = body?.access_token;
    if (!token) throw new Error("guest login failed");
    setToken(token);
    const profile = await me();
    setUser(profile);
    setAuthState("authenticated");
  },

  logout(): void {
    clearToken();
    setUser(null);
    setAuthState("anonymous");
  },
};
