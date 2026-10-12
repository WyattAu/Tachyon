// Remote-registry dogfood E2E: the Plugins page must consume the
// tachyon-plugins companion repo's registry over HTTP and show its
// echo-dogfood marker — proving the marketplace is no longer limited to the
// bundled copy shipped inside Tachyon.
//
// Run: REGISTRY_URL=http://127.0.0.1:4701/registry.json node scripts/registry-e2e.mjs
import { chromium } from "playwright";
import { spawn } from "node:child_process";

const REGISTRY_URL = process.env.REGISTRY_URL;
const REGISTRY_DIR = process.env.REGISTRY_DIR; // served via python http.server
const BASE = process.env.PROBE_URL ?? "http://127.0.0.1:18080/";
const TOKEN = process.env.CRDT_TOKEN ?? "";
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

// A real remote registry must serve CORS (the browser blocks a cross-origin
// fetch without Access-Control-Allow-Origin, and the client correctly falls
// back to the bundled copy). python's http.server sends no CORS headers, so
// the test serves the registry from a tiny node server that does — exactly
// what a production registry host must configure.
let server;
if (REGISTRY_DIR) {
  const { readFile } = await import("node:fs/promises");
  const { createServer } = await import("node:http");
  const { join } = await import("node:path");
  server = createServer(async (rq, rs) => {
    if (rq.url?.startsWith("/registry.json")) {
      try {
        const body = await readFile(join(REGISTRY_DIR, "registry.json"));
        rs.writeHead(200, {
          "Content-Type": "application/json",
          "Access-Control-Allow-Origin": "*",
        });
        rs.end(body);
      } catch {
        rs.writeHead(404);
        rs.end();
      }
      return;
    }
    rs.writeHead(404);
    rs.end();
  });
  await new Promise((resolve) => server.listen(4701, "127.0.0.1", resolve));
  await sleep(300);
}
const url = REGISTRY_URL ?? "http://127.0.0.1:4701/registry.json";
console.log("registry url:", url);

const b = await chromium.launch({ args: ["--disable-dev-shm-usage", "--no-sandbox"] });
const p = await b.newPage({ viewport: { width: 1100, height: 800 } });
await p.addInitScript(
  ({ t, r }) => {
    localStorage.setItem("tachyon_token", t);
    localStorage.setItem("tachyon_registry_url", r);
  },
  { t: TOKEN, r: url }
);

await p.goto(`${BASE}plugins`, { waitUntil: "commit", timeout: 90000 });
// The registry source indicator lives in the marketplace tab; wait for the
// page shell to finish booting before clicking (goto only waits for commit).
const marketplaceTab = p.getByRole("button", { name: "Marketplace" });
await marketplaceTab.waitFor({ timeout: 60000 });
await marketplaceTab.click();
await p.getByTestId("registry-source").waitFor({ timeout: 60000 });
await sleep(1500);

const source = await p.getByTestId("registry-source").innerText();
console.log("source indicator:", JSON.stringify(source));
const remote = source.includes("remote");

// The marketplace hides already-installed plugins, so "echo-dogfood" is not
// a stable marker (it was installed in an earlier step). The version is:
// the remote registry ships word-count 0.2.0 while the bundled copy has
// 0.1.0, so seeing 0.2.0 can only come from the tachyon-plugins repo.
const remoteVersion = await p.getByText("0.2.0").count();
console.log("remote-only version visible (word-count 0.2.0):", remoteVersion > 0 ? "PASS" : "FAIL");
const marker = remoteVersion > 0;

await b.close();
if (server) server.close();
process.exit(remote && marker > 0 ? 0 : 1);
