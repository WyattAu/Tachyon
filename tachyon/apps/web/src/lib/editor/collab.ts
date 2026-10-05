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
export interface CollabHandle {
  sendUpdate: () => void;
  close: () => void;
  connected: () => boolean;
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
  serverOriginOverride?: string
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

  ws.onopen = () => {
    statusChanged?.(true);
    // announce our full state so the room converges with this client
    sendFrame(handle.encodeFullState());
    // flush queued local updates
    for (const p of pending) sendFrame(p);
    pending = [];
  };
  ws.onmessage = (ev) => {
    const buf = ev.data instanceof ArrayBuffer ? new Uint8Array(ev.data) : null;
    if (!buf || buf.length < 2 || buf[0] !== 0) return;
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

  return {
    sendUpdate,
    close: () => {
      alive = false;
      ws.close();
    },
    connected: () => ws.readyState === WebSocket.OPEN,
  };
}
