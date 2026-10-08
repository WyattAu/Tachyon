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

  const ws = new WebSocket(`${proto}//${host}/ws/${encodeURIComponent(docId)}`);
  ws.binaryType = "arraybuffer";

  let alive = true;
  let pending: Uint8Array[] = [];

  const sendFrame = (payload: Uint8Array) => {
    if (ws.readyState === WebSocket.OPEN) {
      const frame = new Uint8Array(payload.length + 1);
      frame[0] = 0; // sync message type
      frame.set(payload, 1);
      ws.send(frame);
    } else {
      pending.push(payload); // queue until open
    }
  };

  let receivedRoomState = false;

  ws.onopen = () => {
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
    for (const p of pending) sendFrame(p);
    pending = [];
  };
  ws.onmessage = (ev) => {
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
  ws.onclose = () => {
    statusChanged?.(false);
  };

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
    if (ws.readyState !== WebSocket.OPEN) return;
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
      ws.close();
    },
    connected: () => ws.readyState === WebSocket.OPEN,
  };
}
