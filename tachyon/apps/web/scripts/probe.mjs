// Combined probe: wait for green, dump diagnostics if not.
import { chromium } from "playwright";

const BASE = process.env.PROBE_URL ?? "http://127.0.0.1:4401/";
const browser = await chromium.launch();
const page = await browser.newPage();
page.on("pageerror", (e) => console.log("[page-error]", String(e).slice(0, 300)));
page.on("requestfailed", (r) => console.log("[req-failed]", r.url().slice(0, 120)));
page.on("response", (r) => {
  if (r.url().includes("/api/")) console.log("[api]", r.status(), r.url().slice(0, 90));
});
try {
  await page.goto(BASE, { timeout: 60000, waitUntil: "domcontentloaded" });
  try {
    await page.getByText("all green").waitFor({ timeout: 45000 });
  } catch {
    console.log("[warn] 'all green' not reached; dumping state:");
  }
  const body = await page.locator("body").innerText();
  console.log("BODY:\n" + body.slice(0, 600));
} finally {
  await browser.close();
}
