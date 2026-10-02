// Parity screenshot harness — captures the migrated routes on whichever
// frontend the Tachyon server currently serves.
//
// Usage: BASE=http://127.0.0.1:18080 OUT=/tmp/parity/leptos \
//        node tachyon/apps/web/scripts/parity-shots.mjs
//
// Logs in via the real form, then screenshots each route into $OUT.
import { chromium } from "playwright";
import { mkdirSync } from "node:fs";

const BASE = process.env.BASE ?? "http://127.0.0.1:18080";
const OUT = process.env.OUT ?? "/tmp/parity/out";
mkdirSync(OUT, { recursive: true });

const ROUTES = [
  ["login", "/login", null],
  ["dashboard", "/dashboard", "Dashboard"],
  ["documents", "/documents", "Documents"],
  ["spaces", "/spaces", "Spaces"],
  ["tags", "/tags", "Tags"],
  ["search", "/search", "Search"],
  ["teams", "/teams", "Teams"],
  ["templates", "/templates", "Templates"],
  ["graph", "/graph", "Graph"],
  ["settings", "/settings", "Settings"],
  ["billing", "/billing", "Billing"],
  ["audit", "/audit", "Audit"],
  ["plugins", "/plugins", "Plugins"],
  ["catalog", "/catalog", "Catalog"],
  ["import", "/import", "Import"],
  ["profile", "/profile", "Profile"],
  ["flashcards", "/flashcards", "Flashcards"],
  ["servers", "/servers", "Server Connections"],
];

const browser = await chromium.launch({ args: ["--disable-dev-shm-usage"] });
const page = await browser.newPage({ viewport: { width: 1280, height: 800 } });

// login (Leptos + Solid share the field names)
await page.goto(`${BASE}/login`, { waitUntil: "domcontentloaded", timeout: 30000 });
await page.locator('input[name=username]').fill("admin");
await page.locator('input[name=password]').fill("admin123");
await page.getByRole("button", { name: /Sign in/i }).first().click();
await page.waitForTimeout(4000);

let ok = 0;
for (const [name, route, marker] of ROUTES) {
  try {
    await page.goto(`${BASE}${route}`, { waitUntil: "domcontentloaded", timeout: 30000 });
    await page.waitForTimeout(marker ? 3500 : 2000);
    await page.screenshot({ path: `${OUT}/${name}.png` });
    if (marker) {
      const body = await page.locator("body").innerText();
      const found = body.toLowerCase().includes(marker.toLowerCase());
      console.log(`${name}: ${found ? "MARKER OK" : "MARKER MISS"} (${marker})`);
      if (found) ok++;
    } else {
      console.log(`${name}: captured (login page)`);
      ok++;
    }
  } catch (e) {
    console.log(`${name}: ERROR ${String(e).split("\n")[0].slice(0, 100)}`);
  }
}
console.log(`DONE: ${ok}/${ROUTES.length} routes captured with markers -> ${OUT}`);
await browser.close();
