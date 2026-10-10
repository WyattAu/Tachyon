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

let server;
if (REGISTRY_DIR) {
  server = spawn("python3", ["-m", "http.server", "4701", "--bind", "127.0.0.1"], {
    cwd: REGISTRY_DIR,
    stdio: "ignore",
  });
  await sleep(1200);
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

const marker = await p.getByText("echo-dogfood").count();
console.log("remote marker plugin visible:", marker > 0 ? "PASS" : "FAIL");

await b.close();
server?.kill();
process.exit(remote && marker > 0 ? 0 : 1);
