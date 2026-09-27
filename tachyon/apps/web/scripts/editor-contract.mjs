// Editor bridge contract test — runs the frozen contract (contract.ts)
// against ANY live implementation of window.__tachyonEditor.
//
// Requires: the Tachyon desktop app running with TACHYON_DEBUG>=1 (control
// plane on :17890) and staging reachable. Creates a scratch document, drives
// the editor through the control plane, verifies, deletes the doc.
//
// Run: node tachyon/apps/web/scripts/editor-contract.mjs

import { chromium } from "playwright";
import { readFileSync } from "node:fs";

const CONTROL = "http://127.0.0.1:17890";
const TOKEN = readFileSync("/tmp/tachyon-control.token", "utf8").trim();
const API = process.env.TACHYON_API ?? "http://192.168.1.191:18080";
const H = { "Content-Type": "application/json", Authorization: `Bearer x` };

async function apiLogin() {
  const r = await fetch(`${API}/api/v1/auth/login`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ username: "admin", password: "admin123" }),
  });
  const d = await r.json();
  if (!d.access_token) throw new Error("staging login failed");
  return d.access_token;
}

const control = async (method, path, body) => {
  const r = await fetch(`${CONTROL}${path}`, {
    method,
    headers: { Authorization: `Bearer ${TOKEN}`, "Content-Type": "application/json" },
    body: body ? JSON.stringify(body) : undefined,
  });
  return r.json();
};

const evalJs = async (js) => {
  const { eval_id } = await control("POST", "/control/eval", { js });
  if (!eval_id) throw new Error("eval dispatch failed");
  // warmup quirk: first eval after load drops its callback — handled by caller
  for (let i = 0; i < 30; i++) {
    await new Promise((r) => setTimeout(r, 500));
    const { events } = await control("GET", "/control/events?since=0");
    const hit = (events ?? []).find(
      (e) => e.kind === "eval_result" && e.eval_id === eval_id
    );
    if (hit) return hit.ok ? String(hit.result) : `EVAL_ERROR: ${hit.result}`;
  }
  return "EVAL_TIMEOUT";
};

// The contract assertions, executed INSIDE the page.
const CONTRACT_JS = `
(async () => {
  const out = { checks: [], pass: 0, fail: 0 };
  const check = (name, ok, detail) => {
    out.checks.push(\`[\${ok ? "PASS" : "FAIL"}] \${name}\${detail ? " — " + detail : ""}\`);
    ok ? out.pass++ : out.fail++;
  };
  const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

  const bridge = window.__tachyonEditor;
  check("bridge exists", !!bridge);
  if (!bridge) return JSON.stringify(out);

  for (const m of ["insertText","key","content","toggleFind","find","findNext","findPrevious"]) {
    check(\`method \${m} is function\`, typeof bridge[m] === "function");
  }

  // Semantic: insertText advances content
  const before = bridge.content();
  bridge.insertText("CONTRACT_MARKER_");
  await sleep(300);
  check("insertText lands", bridge.content().includes("CONTRACT_MARKER_"));

  // Semantic: unknown multi-char key names are IGNORED (not typed)
  const mid = bridge.content();
  bridge.key("ControlEnd");
  bridge.key("ShiftHome");
  await sleep(300);
  check("unknown key names ignored", bridge.content() === mid,
    bridge.content().length !== mid.length ? "length changed" : "");

  // Semantic: recognized keys do not type literally
  bridge.key("End");
  bridge.key("ArrowDown");
  bridge.key("Home");
  await sleep(200);
  check("named keys move, not type", !bridge.content().includes("EndArrowDownHome"));

  return JSON.stringify(out);
})()
`;

async function main() {
  const token = await apiLogin();
  const auth = { "Content-Type": "application/json", Authorization: `Bearer ${token}` };

  // 1. scratch doc
  let docId;
  {
    const r = await fetch(`${API}/api/v1/documents`, {
      method: "POST",
      headers: auth,
      body: JSON.stringify({
        title: `Bridge contract ${new Date().toISOString().slice(11, 19)}`,
        content: "# contract test\n\nscratch document — safe to delete\n",
      }),
    });
    docId = (await r.json()).id;
  }
  console.log(`scratch doc: ${docId}`);

  // 2. app state + navigate
  const state = await control("GET", "/control/state");
  if (!state.run_id) throw new Error("desktop control plane not reachable — launch the app with TACHYON_DEBUG=1");
  await control("POST", "/control/navigate", { route: "/login" });
  await new Promise((r) => setTimeout(r, 4000));
  await control("POST", "/control/eval", { js: "1" }); // warmup (first-eval drop)
  await new Promise((r) => setTimeout(r, 800));

  // login if needed
  const route = (await control("GET", "/control/state")).route;
  if (route === "/login" || route === "") {
    await control("POST", "/control/type", { selector: "input[name=server_url]", text: API });
    await control("POST", "/control/type", { selector: "input[name=username]", text: "admin" });
    await control("POST", "/control/type", { selector: "input[name=password]", text: "admin123" });
    await control("POST", "/control/eval", {
      js: `var b=Array.from(document.querySelectorAll("button")).find(x=>x.textContent.trim()==="Sign in"); b?b.click():"NO_BTN"`,
    });
    await new Promise((r) => setTimeout(r, 6000));
  }

  await control("POST", "/control/navigate", { route: `/documents/${docId}/edit` });
  await new Promise((r) => setTimeout(r, 9000));

  // 3. run contract
  const result = await evalJs(`window.__TAURI__ ? ${CONTRACT_JS} : "NO_TAURI"`);
  console.log(result);
  const out = JSON.parse(result.replaceAll("\n", ""));

  // 4. cleanup — delete scratch doc
  await fetch(`${API}/api/v1/documents/${docId}`, { method: "DELETE", headers: auth });

  if (out.fail > 0) {
    console.error(`CONTRACT: ${out.fail} FAILURES`);
    process.exit(1);
  }
  console.log(`CONTRACT: all ${out.pass} checks PASS`);
}

main().catch((e) => {
  console.error("contract run failed:", e.message);
  process.exit(1);
});
