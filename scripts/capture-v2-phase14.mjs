#!/usr/bin/env node
// Captures the screens changed by the Phase 14 corrections, for visual review.
// Serve the repository root first:  python -m http.server 8765 --bind 127.0.0.1
import { mkdirSync } from "node:fs";
import { chromium } from "@playwright/test";

const base = "http://127.0.0.1:8765/docs/design/v2/index.html";
const out = "docs/design/v2/screenshots";
mkdirSync(out, { recursive: true });
const shots = [
  ["overview", "#overview"], ["discover", "#discover"], ["my-apps", "#my-apps"],
  ["my-apps-error", "?app=linkding#my-apps"], ["activity", "#activity"], ["settings", "#settings"],
  ["install-review", "#install"], ["install-port-failure", "?state=failure&failure=port_in_use#install"],
  ["recovery", "#recovery"], ["first-run-welcome", "?step=welcome&preflight=ready#first-run"],
  ["first-run-choose", "?step=choose&preflight=ready#first-run"], ["discover-connect", "#discover"],
];
const browser = await chromium.launch({ channel: "chrome", headless: true });
for (const viewport of [{ width: 1280, height: 800 }, { width: 1440, height: 900 }]) {
  const page = await (await browser.newContext({ viewport, reducedMotion: "reduce" })).newPage();
  for (const [name, suffix] of shots) {
    await page.goto(`${base}${suffix}`, { waitUntil: "networkidle" });
    await page.evaluate(() => document.fonts.ready);
    if (name === "discover-connect") { await page.click('[data-open-connect="immich"]'); await page.waitForSelector("#connect-address"); }
    await page.screenshot({ path: `${out}/phase14-${name}-${viewport.width}.png` });
  }
  await page.context().close();
}
await browser.close();
console.log(`Captured ${shots.length * 2} Phase 14 screenshots in ${out}`);
