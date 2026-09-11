#!/usr/bin/env node
import assert from "node:assert/strict";
import { mkdir } from "node:fs/promises";
import { resolve } from "node:path";
import { chromium } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";

const base = "http://127.0.0.1:8765/docs/design/v2/index.html";
const output = resolve("docs/design/v2/screenshots");
const views = [{ name: "1440", width: 1440, height: 900 }, { name: "1280", width: 1280, height: 800 }];
const states = [
  ["discover-grain", "#discover"],
  ["overview-telemetry", "?concepts=1#overview"],
  ["first-run-grain", "?step=choose#first-run"],
];

await mkdir(output, { recursive: true });
const browser = await chromium.launch({ channel: "chrome", headless: true });
try {
  for (const view of views) {
    const context = await browser.newContext({ viewport: view });
    const page = await context.newPage();
    for (const [name, suffix] of states) {
      await page.goto(`${base}${suffix}`, { waitUntil: "networkidle" });
      const violations = (await new AxeBuilder({ page }).analyze()).violations;
      assert.deepEqual(violations.map((item) => item.id), [], `${name} ${view.name}: ${violations.map((item) => item.id).join(", ")}`);
      const overflow = await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth);
      assert.ok(overflow <= 0, `${name} ${view.name}: ${overflow}px horizontal overflow`);
      if (view.name === "1440") await page.screenshot({ path: resolve(output, `${name}-1440.png`), fullPage: true });
    }
    await context.close();
  }

  const context = await browser.newContext({ viewport: views[0] });
  const page = await context.newPage();
  await page.goto(`${base}#discover`, { waitUntil: "networkidle" });
  assert.equal(await page.locator(".featured-recipe-card").count(), 3);
  await page.getByRole("button", { name: "Review n8n install" }).click();
  assert.equal(new URL(page.url()).hash, "#install");
  assert.match(await page.locator(".recipe-head h2").innerText(), /n8n/i);
  assert.match(await page.locator("body").innerText(), /localhost:5678/);

  await page.goto(`${base}?concepts=1#overview`, { waitUntil: "networkidle" });
  assert.equal(await page.locator(".telemetry-line").count(), 1);
  assert.match(await page.locator(".concept-telemetry").innerText(), /Concept telemetry · backend work/i);
  assert.match(await page.locator(".telemetry-notice").innerText(), /not collected by the current backend/i);
  await context.close();
} finally {
  await browser.close();
}

console.log("Verified V2 grain surfaces and concept telemetry at two laptop viewports, including reviewed-recipe routing and truth labeling.");
