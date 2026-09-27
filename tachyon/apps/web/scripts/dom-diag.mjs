import { chromium } from "playwright";

const BASE = process.env.PROBE_URL ?? "http://127.0.0.1:4426/";
const browser = await chromium.launch({ args: ["--disable-dev-shm-usage"] });
const page = await browser.newPage();
page.on("console", (m) => console.log("[console]", m.type(), m.text().slice(0, 150)));
page.on("pageerror", (e) => console.log("[pageerror]", String(e).slice(0, 200)));
page.on("requestfailed", (r) => console.log("[reqfail]", r.url().slice(0, 100), r.failure()?.errorText));
page.on("response", (r) => {
  if (r.status() >= 400) console.log("[http]", r.status(), r.url().slice(0, 100));
});

await page.goto(`${BASE}login`, { timeout: 30000, waitUntil: "domcontentloaded" });
await page.waitForTimeout(6000);
const root = await page.locator("#root").innerHTML();
console.log("ROOT innerHTML length:", root.length);
console.log("ROOT:", root.slice(0, 300) || "(empty)");
await browser.close();
