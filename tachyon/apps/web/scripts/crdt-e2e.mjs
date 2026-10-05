// 2-client CRDT convergence E2E — two browser contexts editing the same
// document concurrently over the y-websocket room.
// Run: node tachyon/apps/web/scripts/crdt-e2e.mjs (repo root)
import { chromium } from "playwright";

const BASE = process.env.PROBE_URL ?? "http://127.0.0.1:4561/";
const DOC = process.env.CRDIT_DOC ?? process.env.CRD_DOC ?? "01a10a30-8a66-7550-a805-703a3c7bc8ca";
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

async function bootEditor(base, docId, token) {
  const b = await chromium.launch({ args: ["--disable-dev-shm-usage"] });
  const p = await b.newPage({ viewport: { width: 1100, height: 800 } });
  await p.addInitScript((t) => localStorage.setItem("tachyon_token", t), token);
  p.on("pageerror", (e) => console.log("[pageerror]", String(e).slice(0, 160)));
  p.on("console", (m) => { if (m.type() === "error") console.log("[console]", m.text().slice(0, 140)); });
  await p.goto(`${base}documents/${docId}/edit`, {
    waitUntil: "commit",
    timeout: 90000,
  });
  await p.getByTestId("editor-surface").waitFor({ timeout: 90000 });
  await sleep(1500); // collab ws connect + full-state announce
  const collab = await p.getByText("collab:").innerText().catch(() => "collab:off");
  return { b, p, collab };
}

async function type(p, text) {
  await p.getByTestId("editor-surface").click();
  await p.keyboard.press("Control+End").catch(() => {});
  await p.keyboard.press("End");
  await p.keyboard.type(text, { delay: 20 });
  await sleep(1200); // debounced broadcast
}

async function main() {
  const token = process.env.CRDT_TOKEN ?? "";
  const A = await bootEditor(BASE, DOC, token);
  console.log("client A:", A.collab);
  const B = await bootEditor(BASE, DOC, token);
  console.log("client B:", B.collab);

  // A types first
  await type(A.p, "ALPHA-");
  await sleep(1500);

  // B types concurrently
  await type(B.p, "BETA-");
  await sleep(2500); // convergence window

  // Both should show merged content
  const textA = await A.p.getByTestId("editor-surface").innerText();
  const textB = await B.p.getByTestId("editor-surface").innerText();
  const aHasBoth = textA.includes("ALPHA") && textA.includes("BETA");
  const bHasBoth = textB.includes("ALPHA") && textB.includes("BETA");
  console.log("A sees merged:", aHasBoth ? "PASS" : "FAIL — " + JSON.stringify(textA.slice(0, 120)));
  console.log("B sees merged:", bHasBoth ? "PASS" : "FAIL — " + JSON.stringify(textB.slice(0, 120)));

  // server convergence: the room flushes to documents.content on a 30s
  // timer — poll the REST doc until the merged text is materialized.
  let persisted = false;
  for (let i = 0; i < 24; i++) {
    const res = await fetch(`${BASE.replace(/\/$/, "")}/api/v1/documents/${DOC}`, {
      headers: { Authorization: `Bearer ${token}` },
    });
    const body = await res.json().catch(() => ({}));
    const c = body?.content ?? "";
    if (c.includes("ALPHA") && c.includes("BETA")) {
      persisted = true;
      console.log(`server persisted merged content after ~${i * 3}s:`, JSON.stringify(c));
      break;
    }
    await sleep(3000);
  }
  if (!persisted) console.log("server persistence: FAIL (no merged content in REST doc)");

  const pre = await fetch(`${BASE.replace(/\/$/, "")}/api/v1/documents/${DOC}`, {
    headers: { Authorization: `Bearer ${token}` },
  }).then((r) => r.json()).catch((e) => ({ err: String(e) }));
  console.log("REST content before reload:", JSON.stringify(pre?.content ?? pre));

  // bypass the HTTP cache so the reload re-reads documents.content
  const cdp = await A.p.context().newCDPSession(A.p);
  await cdp.send("Network.enable");
  await cdp.send("Network.setCacheDisabled", { cacheDisabled: true });
  await A.p.reload({ waitUntil: "commit" });
  await A.p.getByTestId("editor-surface").waitFor({ timeout: 60000 });
  await sleep(6000);
  const textA2 = await A.p.getByTestId("editor-surface").innerText();
  console.log("A after reload:", textA2.includes("ALPHA") && textA2.includes("BETA") ? "PASS" : "FAIL");
  if (!(textA2.includes("ALPHA") && textA2.includes("BETA")))
    console.log("A reloaded text:", JSON.stringify(textA2.slice(0, 160)));

  await A.b.close();
  await B.b.close();
  process.exit(aHasBoth && bHasBoth && persisted ? 0 : 1);
}

main().catch((e) => { console.log("ERR", String(e).slice(0, 200)); process.exit(1); });
