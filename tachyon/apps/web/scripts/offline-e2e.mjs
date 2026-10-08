// Offline queue E2E — the W1.4 verification.
//
// Scenario: editor is live, socket is force-closed, user types offline
// (updates queue to localStorage), the page is RELOADED (model is gone,
// queue must survive), the socket reconnects, and the server must
// eventually materialize the offline text into documents.content.
//
// Run: CRDT_TOKEN=<jwt> CRD_DOC=<uuid> node scripts/offline-e2e.mjs
import { chromium } from "playwright";

const BASE = process.env.PROBE_URL ?? "http://127.0.0.1:18080/";
const DOC = process.env.CRD_DOC;
const TOKEN = process.env.CRDT_TOKEN ?? "";
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

if (!DOC) {
  console.error("CRD_DOC required");
  process.exit(1);
}

const b = await chromium.launch({ args: ["--disable-dev-shm-usage", "--no-sandbox"] });
const p = await b.newPage({ viewport: { width: 1100, height: 800 } });
await p.addInitScript((t) => localStorage.setItem("tachyon_token", t), TOKEN);
p.on("pageerror", (e) => console.log("[pageerror]", String(e).slice(0, 160)));

await p.goto(`${BASE}documents/${DOC}/edit`, { waitUntil: "commit", timeout: 90000 });
await p.getByTestId("editor-surface").waitFor({ timeout: 90000 });
await p.waitForTimeout(2500); // connect + settle

const live = await p.getByText("collab:live").count();
console.log("initially live:", live === 1 ? "PASS" : "FAIL");

// Suspend the socket (no auto-reconnect) so the offline phase is isolated
// from the 1s backoff reconnect.
await p.evaluate(() => window.__tachyonCollab.suspend());
await p.waitForTimeout(500);
const off = await p.getByText("collab:off").count();
console.log("suspended offline:", off === 1 ? "PASS" : "FAIL");

// Type offline. The debounced broadcaster queues frames to localStorage.
await p.getByTestId("editor-surface").click();
await p.keyboard.press("Control+End").catch(() => {});
await p.keyboard.press("End");
await p.keyboard.type("OFFLINE-", { delay: 25 });
await sleep(1200); // let the debounce land the frames in the queue

const queued = await p.evaluate(() => window.__tachyonCollab.queued());
console.log("queued while offline:", queued > 0 ? `PASS (${queued} frames)` : "FAIL (0)");
const persistedLocally = await p.evaluate(() =>
  Object.keys(localStorage).some((k) => k.startsWith("crdt-q:"))
);
console.log("queue persisted to localStorage:", persistedLocally ? "PASS" : "FAIL");

// Reload: the model is gone, only the persisted queue carries the offline
// work. The fresh page auto-connects and replays it — nothing else could
// have delivered it, since the pre-reload socket was suspended.
await p.reload({ waitUntil: "commit" });
await p.getByTestId("editor-surface").waitFor({ timeout: 90000 });

// Reconnect with backoff (1s first attempt) then flush the queue.
let reconnected = false;
for (let i = 0; i < 20; i++) {
  await sleep(1000);
  if ((await p.getByText("collab:live").count()) === 1) {
    reconnected = true;
    break;
  }
}
console.log("reconnected after reload:", reconnected ? "PASS" : "FAIL");
await p.waitForTimeout(2000);

// The server must materialize the offline text (30s flush timer).
let serverContent = "";
let materialized = false;
for (let i = 0; i < 24; i++) {
  const res = await fetch(`${BASE.replace(/\/$/, "")}/api/v1/documents/${DOC}`, {
    headers: { Authorization: `Bearer ${TOKEN}` },
  });
  const body = await res.json().catch(() => ({}));
  serverContent = body?.content ?? "";
  if (serverContent.includes("OFFLINE-")) {
    materialized = true;
    console.log(`server materialized offline edits after ~${i * 3}s`);
    break;
  }
  await sleep(3000);
}
console.log("offline edits persisted:", materialized ? "PASS" : `FAIL (content: ${JSON.stringify(serverContent.slice(0, 80))})`);

// Duplication guard: the offline text must appear exactly once.
const once = materialized && (serverContent.match(/OFFLINE-/g) ?? []).length === 1;
console.log("offline edits appear exactly once:", once ? "PASS" : "FAIL");

await b.close();
process.exit(materialized && once && queued > 0 && persistedLocally && reconnected ? 0 : 1);
