import { createSignal } from "solid-js";
import { api, resolveBaseUrl } from "@/lib/api/client";

/**
 * Notifications store (Phase C): list + unread count + SSE live stream.
 * Stream: GET /notifications/stream (server-sent events). Reconnects with
 * backoff; new notifications are upserted into the list signal.
 */

export interface Notification {
  id: string;
  title: string;
  body: string | null;
  type: string | null;
  link: string | null;
  read: boolean;
  created_at: string | null;
}

const [notifications, setNotifications] = createSignal<Notification[]>([]);
const [unread, setUnread] = createSignal(0);
const [connected, setConnected] = createSignal(false);

let evtSource: EventSource | null = null;
let retryTimer: ReturnType<typeof setTimeout> | null = null;
let pollTimer: ReturnType<typeof setInterval> | null = null;

function startPolling(): void {
  if (pollTimer) return;
  pollTimer = setInterval(() => {
    void notificationsStore.load();
  }, 30000);
}

export const notificationsStore = {
  notifications,
  unread,
  connected,

  async load(): Promise<void> {
    const [{ data }, count] = await Promise.all([
      api.GET("/notifications", { params: { query: {} } }),
      api.GET("/notifications/unread-count", {}),
    ]);
    const list = (data as unknown as { notifications?: Notification[] } | undefined)
      ?.notifications ?? [];
    setNotifications(list);
    const c = (count.data as unknown as { count?: number } | undefined)?.count ?? 0;
    setUnread(c);
  },

  async markAllRead(): Promise<void> {
    await api.POST("/notifications/read-all", {});
    setNotifications((ns) => ns.map((n) => ({ ...n, read: true })));
    setUnread(0);
  },

  async markRead(id: string): Promise<void> {
    await api.POST("/notifications/{id}/read", {
      params: { path: { id } },
    });
    setNotifications((ns) => ns.map((n) => (n.id === id ? { ...n, read: true } : n)));
    setUnread((u) => Math.max(0, u - 1));
  },

  /** Subscribe to the SSE stream with a polling fallback.
   *
   * The SSE endpoint sits behind bearer auth, which native EventSource
   * cannot send — if the stream 401s, we fall back to 30s polling so the
   * page still works (server-side token-query support tracked for Phase C
   * server pass). Safe to call once per app boot.
   */
  connectStream(): void {
    if (evtSource || pollTimer || typeof window === "undefined") return;
    const base = resolveBaseUrl();
    const es = new EventSource(`${base}/notifications/stream`);
    es.onopen = () => {
      setConnected(true);
      if (pollTimer) {
        clearInterval(pollTimer);
        pollTimer = null;
      }
    };
    es.onerror = () => {
      setConnected(false);
      es.close();
      evtSource = null;
      startPolling();
    };
    es.onmessage = (ev) => {
      try {
        const n = JSON.parse(ev.data) as Notification;
        setNotifications((ns) => [n, ...ns].slice(0, 100));
        if (!n.read) setUnread((u) => u + 1);
      } catch {
        // non-JSON keepalive — ignore
      }
    };
    evtSource = es;
    // if neither open nor error fires quickly (auth rejection may not fire
    // onerror on some proxies), poll until we see an open
    setTimeout(() => {
      if (!connected()) startPolling();
    }, 5000);
  },

  disconnectStream(): void {
    evtSource?.close();
    evtSource = null;
    if (retryTimer) clearTimeout(retryTimer);
    if (pollTimer) clearInterval(pollTimer);
  },
};
