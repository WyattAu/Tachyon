// Pixel-level check of the sigma canvas on the mock graph page.
import { chromium } from "playwright";

const BASE = process.env.PROBE_URL ?? "http://127.0.0.1:4509/";
const browser = await chromium.launch({ args: ["--disable-dev-shm-usage"] });
const page = await browser.newPage();
await page.addInitScript(() => localStorage.setItem("tachyon_token", "mock-token"));
await page.goto(`${BASE}graph`, { waitUntil: "domcontentloaded", timeout: 30000 });
await page.waitForTimeout(6000);
const px = await page.evaluate(() => {
  const c = document.querySelector("canvas");
  if (!c) return "no canvas";
  const g = c.getContext("webgl2") || c.getContext("webgl");
  if (!g) return "no gl on canvas";
  const buf = new Uint8Array(4 * 10000);
  const w = 100, h = 100;
  g.readPixels(Math.floor(c.width / 2 - w / 2), Math.floor(c.height / 2 - h / 2), w, h, g.RGBA, g.UNSIGNED_BYTE, buf);
  const nonzero = Array.from(buf).filter((v) => v > 0).length;
  return `canvas ${c.width}x${c.height}, center ${w}x${h} non-zero bytes: ${nonzero}/40000`;
});
console.log("PIXELS:", px);
await browser.close();
