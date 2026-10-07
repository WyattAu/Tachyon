// Protocol-level presence check: two raw WebSocket clients against the real
// server. A joins and announces; B joins later and must receive a snapshot
// containing A. No browser required.
// Run: node scripts/presence-proto.mjs <docId>
import { encodeAwareness, decodeAwareness } from "../src/lib/editor/collab.ts";

const DOC = process.argv[2] ?? "test-room";
const URL = `ws://127.0.0.1:18080/ws/${DOC}`;
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

function connect(label, onFrame) {
  return new Promise((resolve, reject) => {
    const ws = new WebSocket(URL);
    ws.binaryType = "arraybuffer";
    const frames = [];
    ws.onopen = () => {
      console.log(`${label}: open @${Date.now() % 100000}`);
      resolve({ ws, frames });
    };
    ws.onerror = (e) => reject(new Error(`${label} error: ${e.message ?? e}`));
    ws.onmessage = (ev) => {
      const buf = new Uint8Array(ev.data);
      frames.push([...buf.slice(0, 12)]);
      if (buf[0] === 1) {
        const peers = [...decodeAwareness(buf.slice(1)).entries()];
        console.log(`${label}: awareness @${Date.now() % 100000} ->`, peers.map(([id, e]) => `${id}@${e.clock}:${e.json}`));
      } else {
        console.log(`${label}: frame type=${buf[0]} len=${buf.length}`);
      }
      onFrame?.(buf);
    };
    ws.onclose = () => console.log(`${label}: closed`);
  });
}

const announce = (ws, clientId, name) => {
  const payload = encodeAwareness(
    new Map([[clientId, { clock: 1, json: JSON.stringify({ user: { name, color: "#f00" } }) }]])
  );
  const frame = new Uint8Array(payload.length + 1);
  frame[0] = 1;
  frame.set(payload, 1);
  ws.send(frame);
};

const a = await connect("A");
announce(a.ws, 111, "alice");
await sleep(800);

const b = await connect("B");
announce(b.ws, 222, "bob");
await sleep(800);

// Second announce from A: distinguishes "snapshot path broken" from
// "A's frames never stored". If storage works, B receives alice via relay.
console.log("-- A re-announces (heartbeat) --");
announce(a.ws, 111, "alice2");
await sleep(1200);

console.log("\nverdict: B received frames:", b.frames.length > 0 ? "yes" : "NO");
a.ws.close();
b.ws.close();
process.exit(0);
