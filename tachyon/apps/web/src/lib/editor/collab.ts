import { onCleanup } from "solid-js";

/**
 * CRDT collaboration wiring (Phase E): connects the editor's yrs model to
 * the server's y-websocket room (/ws/{room}) using the binary protocol:
 *
 *   client → server: [0, ...yrs_update]  (server applies + relays)
 *   server → client: [0, ...yrs_update]  (from other clients / initial state)
 *
 * The model (editor-wasm) exposes incremental encode/apply in yrs wire
 * format — byte-compatible with the server's Yrs room document.
 *
 * Own messages are echoed back; yrs apply is idempotent so the echo is
 * harmless.
 */
export interface CollabPeer {
  clientId: number;
  name: string;
  color: string;
  cursor: { line: number; col: number } | null;
}

export interface CollabHandle {
  sendUpdate: () => void;
  close: () => void;
  connected: () => boolean;
  /** Publish this client's presence (identity + caret) to the room. */
  sendPresence: (cursor: { line: number; col: number } | null) => void;
  /** Peers currently in the room, excluding this client. */
  peers: () => CollabPeer[];
}

/**
 * Encode a lib0 varUint (LEB128, 7 bits per byte) — the framing yrs uses for
 * AwarenessUpdate. Hand-rolled because the JS side has no yrs decoder.
 */
function varUint(value: number): number[] {
  if (!Number.isSafeInteger(value) || value < 0) {
    throw new Error(`varUint: not a safe non-negative integer: ${value}`);
  }
  const out: number[] = [];
  let v = value;
  // Bitwise ops would truncate above 32 bits; yrs clientIDs go up to 2^53,
  // so this uses plain arithmetic. 2^n = 128^n groups, lowest first.
  while (v >= 128) {
    out.push((v % 128) + 128);
    v = Math.floor(v / 128);
  }
  out.push(v);
  return out;
}

function varString(s: string): number[] {
  const bytes = new TextEncoder().encode(s);
  return [...varUint(bytes.length), ...bytes];
}

/** Decode a lib0 varUint, returning the value and bytes consumed. */
function readVarUint(buf: Uint8Array, offset: number): [number, number] {
  let result = 0;
  let factor = 1;
  let i = offset;
  for (;;) {
    const byte = buf[i++];
    if (byte === undefined) return [0, 0];
    result += (byte & 0x7f) * factor;
    if ((byte & 0x80) === 0) break;
    factor *= 128;
    // yrs clientIDs are bounded by 2^53 (JS safe-integer range).
    if (factor > 2 ** 49) return [0, 0];
  }
  return [result, i - offset];
}

function readVarString(buf: Uint8Array, offset: number): [string, number] {
  const [len, n] = readVarUint(buf, offset);
  if (n === 0) return ["", 0];
  const end = offset + n + len;
  if (end > buf.length) return ["", 0];
  const text = new TextDecoder().decode(buf.subarray(offset + n, end));
  return [text, n + len];
}

/** Decode an AwarenessUpdate payload (len, then clientID/clock/json triples). */
export function decodeAwareness(buf: Uint8Array): Map<number, { clock: number; json: string }> {
  const out = new Map<number, { clock: number; json: string }>();
  let offset = 0;
  const [count, n0] = readVarUint(buf, offset);
  offset += n0;
  for (let i = 0; i < count; i++) {
    const [clientId, n1] = readVarUint(buf, offset);
    offset += n1;
    const [clock, n2] = readVarUint(buf, offset);
    offset += n2;
    const [json, n3] = readVarString(buf, offset);
    offset += n3;
    if (n1 === 0 || n2 === 0 || n3 === 0) break;
    out.set(clientId, { clock, json });
  }
  return out;
}

export function encodeAwareness(
  entries: Map<number, { clock: number; json: string }>
): Uint8Array {
  const parts: number[] = [];
  for (const [clientId, entry] of entries) {
    parts.push(...varUint(clientId), ...varUint(entry.clock), ...varString(entry.json));
  }
  return new Uint8Array([...varUint(entries.size), ...parts]);
}

export function connectCollab(
  docId: string,
  handle: {
    encodeUpdate: () => Uint8Array;
    applyRemoteUpdate: (u: Uint8Array) => boolean;
    encodeFullState: () => Uint8Array;
  },
  onRemote: () => void,
  statusChanged?: (connected: boolean) => void,
  serverOriginOverride?: string,
  identity?: { name: string; color: string },
  onPeers?: (peers: CollabPeer[]) => void
): CollabHandle {
  const proto = serverOriginOverride
    ? serverOriginOverride.replace(/^http/, "ws")
    : typeof window !== "undefined"
      ? window.location.protocol === "https:"
        ? "wss:"
        : "ws:"
      : "ws:";
  const host = serverOriginOverride
    ? serverOriginOverride.replace(/^\w+:\/\//, "")
    : typeof window !== "undefined"
      ? window.location.host
      : "127.0.0.1:18080";

  let ws: WebSocket | null = null;
  let alive = true;
  let pending: Uint8Array[] = [];
  let receivedRoomState = false;
  let reconnectAttempts = 0;
  let reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  // Suspended = deliberately offline (airplane mode, test isolation). The
  // queue keeps collecting; nothing reconnects until resumed.
  let suspended = false;

  // ─── Offline queue ─────────────────────────────────────────────────────
  // In-memory `pending` dies with the page; localStorage does not. Every
  // queued frame is a self-contained yrs update, so replaying the whole
  // queue on any future connection (this session or the next page load)
  // converges: update N only references items from the room base or from
  // updates 1..N-1, i.e. replay order preserves causal order.
  const queueKey = `crdt-q:${docId}`;
  const QUEUE_LIMIT = 2 * 1024 * 1024; // 2 MiB of binary updates

  const loadQueue = (): Uint8Array[] => {
    try {
      const raw = localStorage.getItem(queueKey);
      if (!raw) return [];
      const arr = JSON.parse(raw) as number[][];
      return arr.map((a) => new Uint8Array(a));
    } catch {
      return [];
    }
  };

  const saveQueue = () => {
    try {
      // Serialize oldest-first; if the queue outgrows the cap, drop the
      // OLDEST updates — they are the most likely to already be on the
      // server (the tail is the freshest local work).
      let arr = pending;
      let serialized = JSON.stringify(arr.map((u) => Array.from(u)));
      while (serialized.length > QUEUE_LIMIT && arr.length > 1) {
        arr = arr.slice(1);
        serialized = JSON.stringify(arr.map((u) => Array.from(u)));
      }
      if (serialized.length <= QUEUE_LIMIT) localStorage.setItem(queueKey, serialized);
    } catch {
      // Persistence is best-effort: quota errors must not take down editing.
    }
  };

  const clearQueue = () => {
    pending = [];
    try {
      localStorage.removeItem(queueKey);
    } catch {
      /* ignore */
    }
  };

  pending = loadQueue();

  const sendFrame = (payload: Uint8Array) => {
    if (ws && ws.readyState === WebSocket.OPEN) {
      const frame = new Uint8Array(payload.length + 1);
      frame[0] = 0; // sync message type
      frame.set(payload, 1);
      ws.send(frame);
    } else {
      pending.push(payload); // queue until open (also persisted)
      saveQueue();
    }
  };

  const connect = () => {
    if (!alive || suspended) return;
    reconnectAttempts += 1;
    const socket = new WebSocket(`${proto}//${host}/ws/${encodeURIComponent(docId)}`);
    socket.binaryType = "arraybuffer";
    ws = socket;
    wire(socket);
  };

  const scheduleReconnect = () => {
    if (!alive || suspended) return;
    if (reconnectTimer) clearTimeout(reconnectTimer);
    // Exponential backoff, capped: 1s, 2s, 4s, ... 15s.
    const delay = Math.min(1000 * 2 ** Math.max(0, reconnectAttempts - 1), 15_000);
    reconnectTimer = setTimeout(connect, delay);
  };

  const onopen = () => {
    statusChanged?.(true);
    // Announce immediately so a peer who is already here sees us without
    // waiting for the first caret move.
    sendPresence(lastCursor);
    // Deliberately NOT announcing our full state: the local model may have
    // been bootstrapped from the stored markdown, which carries a different
    // CRDT lineage than the room for identical characters. Pushing it would
    // append the document a second time. The server seeds cold rooms itself,
    // and this client renders whatever the room holds.
    // Local updates queued while the socket was still opening are incremental
    // and lineage-consistent, so they are safe to flush.
    // Flush updates queued offline (this session or a previous page load)
    // before anything new: replay order preserves causal order.
    const offline = loadQueue();
    if (offline.length > 0) {
      for (const payload of offline) {
        if (!ws || ws.readyState !== WebSocket.OPEN) break;
        const frame = new Uint8Array(payload.length + 1);
        frame[0] = 0;
        frame.set(payload, 1);
        ws.send(frame);
      }
      clearQueue();
    }
    for (const p of pending) sendFrame(p);
    pending = [];
  };
  const onmessage = (ev: MessageEvent) => {
    const buf = ev.data instanceof ArrayBuffer ? new Uint8Array(ev.data) : null;
    if (!buf || buf.length < 2) return;
    // Message type 1 = awareness (presence), type 0 = yrs sync.
    if (buf[0] === 1) {
      applyAwareness(buf.slice(1));
      return;
    }
    if (buf[0] !== 0) return;
    receivedRoomState = true;
    // The server uses two framings on the same wire format:
    //   [0, 1, ...update] initial full state pushed on connect
    //   [0, 2, ...diff]   delta-sync response
    //   [0, ...update]    relayed peer frame (no sub-type byte)
    // A sub-type byte is only present in the first two, so try the
    // sub-type framing first and fall back to the raw payload. Both are
    // idempotent yrs merges, so a speculative apply is harmless.
    if (handle.applyRemoteUpdate(buf.slice(2))) {
      onRemote();
      return;
    }
    if (handle.applyRemoteUpdate(buf.slice(1))) onRemote();
  };
  const onclose = () => {
    statusChanged?.(false);
    scheduleReconnect();
  };

  let onerror: ((e: unknown) => void) | null = (e) => {
    // Close handling drives reconnect; error just logs context.
    console.warn("[collab] socket error", e);
  };

  const wire = (socket: WebSocket) => {
    socket.onopen = () => {
      reconnectAttempts = 0; // healthy connection resets the backoff
      onopen();
    };
    socket.onmessage = onmessage;
    socket.onclose = onclose;
    socket.onerror = onerror;
  };

  connect();

  // Test/ops hook: the editor contract precedent (`__tachyonEditor`). Lets
  // E2E force disconnect/reconnect without browser devtools tricks, and
  // operators inspect queue depth.
  if (typeof window !== "undefined") {
    (window as unknown as Record<string, unknown>).__tachyonCollab = {
      disconnect: () => ws?.close(),
      reconnect: () => {
        if (reconnectTimer) clearTimeout(reconnectTimer);
        connect();
      },
      suspend: () => {
        suspended = true;
        if (reconnectTimer) clearTimeout(reconnectTimer);
        ws?.close();
      },
      resume: () => {
        suspended = false;
        if (reconnectTimer) clearTimeout(reconnectTimer);
        connect();
      },
      queued: () => pending.length + loadQueue().length,
    };
  }

  const sendUpdate = () => {
    if (!alive) return;
    sendFrame(handle.encodeUpdate());
  };

  // ─── Presence ──────────────────────────────────────────────────────────
  // clientID is random per page load and never derived from user identity:
  // a reused clientID permanently corrupts a CRDT document.
  const clientId = Math.floor(Math.random() * 0x7fffffff) + 1;
  const clocks = new Map<number, number>();
  const raw = new Map<number, string>();
  let presenceClock = 0;
  let lastCursor: { line: number; col: number } | null = null;

  const collectPeers = (): CollabPeer[] => {
    const peers: CollabPeer[] = [];
    for (const [id, json] of raw) {
      if (id === clientId) continue;
      try {
        const parsed = JSON.parse(json) as {
          user?: { name?: string; color?: string };
          cursor?: { line?: number; col?: number } | null;
        };
        peers.push({
          clientId: id,
          name: parsed.user?.name ?? "anonymous",
          color: parsed.user?.color ?? "#888",
          cursor:
            typeof parsed.cursor?.line === "number"
              ? { line: parsed.cursor.line ?? 0, col: parsed.cursor.col ?? 0 }
              : null,
        });
      } catch {
        // A malformed payload must not take down the socket.
      }
    }
    return peers;
  };

  const emitPeers = () => onPeers?.(collectPeers());

  const applyAwareness = (payload: Uint8Array) => {
    for (const [id, entry] of decodeAwareness(payload)) {
      if (entry.json === "null") {
        raw.delete(id);
        clocks.delete(id);
        continue;
      }
      const known = clocks.get(id);
      // State-based CRDT: only a strictly newer clock wins.
      if (known !== undefined && known >= entry.clock) continue;
      clocks.set(id, entry.clock);
      raw.set(id, entry.json);
    }
    emitPeers();
  };

  const sendAwarenessFrame = (payload: Uint8Array) => {
    if (!ws || ws.readyState !== WebSocket.OPEN) return;
    const frame = new Uint8Array(payload.length + 1);
    frame[0] = 1; // message type: awareness
    frame.set(payload, 1);
    ws.send(frame);
  };

  const sendPresence = (cursor: { line: number; col: number } | null) => {
    if (!alive) return;
    lastCursor = cursor;
    presenceClock += 1;
    const state = JSON.stringify({
      user: identity ?? { name: "anonymous", color: "#888" },
      cursor,
    });
    const entries = new Map<number, { clock: number; json: string }>([
      [clientId, { clock: presenceClock, json: state }],
    ]);
    sendAwarenessFrame(encodeAwareness(entries));
  };

  return {
    sendUpdate,
    sendPresence,
    peers: collectPeers,
    close: () => {
      if (!alive) return;
      alive = false;
      // Announce departure so peers drop the avatar immediately instead of
      // waiting for the server's expiry sweep.
      presenceClock += 1;
      const nullState = new Map<number, { clock: number; json: string }>([
        [clientId, { clock: presenceClock, json: "null" }],
      ]);
      sendAwarenessFrame(encodeAwareness(nullState));
      ws?.close();
    },
    connected: () => (ws ? ws.readyState === WebSocket.OPEN : false),
  };
}
