// Editor round-trip E2E against the local mock, with retries under load.
// Run: node tachyon/apps/web/scripts/editor-e2e.mjs (repo root)
import { chromium } from "playwright";

const BASE = process.env.PROBE_URL ?? "http://127.0.0.1:4441/";
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

async function attempt(n) {
  const browser = await chromium.launch({ args: ["--disable-dev-shm-usage"] });
  const page = await browser.newPage();
  page.on("pageerror", (e) => console.log(`[a${n} pageerror]`, String(e).slice(0, 200)));
  try {
    await page.addInitScript(() => localStorage.setItem("tachyon_token", "mock-token"));
    await page.goto(`${BASE}documents/test-doc/edit`, {
      timeout: 45000,
      waitUntil: "domcontentloaded",
    });

    await page.getByTestId("editor-surface").waitFor({ timeout: 30000 });
    await page.getByText("# Hello Editor").waitFor({ timeout: 15000 });
    console.log("editor boots with document content: PASS");

    await page.getByTestId("editor-surface").click();
    await page.keyboard.press("End");
    await page.keyboard.press("Control+End").catch(() => {});
    await page.keyboard.press("End");
    await page.keyboard.type("Typed-by-E2E");
    await page.getByText("Typed-by-E2E").waitFor({ timeout: 8000 });
    console.log("typing routes into wasm model: PASS");

    const status = await page.locator("[data-testid=editor-status]").innerText();
    if (!status.includes("words") || !status.includes("Ln")) throw new Error("status bar incomplete");
    console.log("status bar (words/Ln/Col): PASS");

    await page.keyboard.press("Control+s");
    await page.getByText("Saved", { exact: true }).waitFor({ timeout: 10000 });
    const put = await (await fetch(`${BASE}__mock/last-put`)).json();
    if (!put.content || !String(put.content).includes("Typed-by-E2E")) {
      throw new Error(`mock last-put missing typed content: ${JSON.stringify(put).slice(0, 150)}`);
    }
    console.log("Ctrl+S saves via PUT (round-trip): PASS");
    await browser.close();
    return true;
  } catch (e) {
    console.log(`attempt ${n} failed: ${String(e).split("\n")[0].slice(0, 160)}`);
    await browser.close();
    return false;
  }
}

let ok = false;
for (let i = 1; i <= 3 && !ok; i++) {
  ok = await attempt(i);
  if (!ok) await sleep(2500);
}
process.exit(ok ? 0 : 1);
