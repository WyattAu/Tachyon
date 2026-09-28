// Diagnostic: dump editor page state
import { chromium } from "playwright";

const BASE = process.env.PROBE_URL ?? "http://127.0.0.1:4441/";
const browser = await chromium.launch({ args: ["--disable-dev-shm-usage"] });
const page = await browser.newPage();
page.on("console", (m) => console.log("[console]", m.type(), m.text().slice(0, 180)));
page.on("pageerror", (e) => console.log("[pageerror]", String(e).slice(0, 250)));
page.on("requestfailed", (r) => console.log("[reqfail]", r.url().slice(0, 120)));
page.on("response", (r) => {
  if (r.status() >= 400 || r.url().includes("api")) console.log("[http]", r.status(), r.url().slice(0, 100));
});
await page.addInitScript(() => localStorage.setItem("tachyon_token", "mock-token"));
await page.goto(`${BASE}documents/test-doc/edit`, { timeout: 30000, waitUntil: "domcontentloaded" });
await page.waitForTimeout(8000);
const body = await page.locator("body").innerText();
console.log("BODY:\n" + body.slice(0, 400));
await browser.close();
