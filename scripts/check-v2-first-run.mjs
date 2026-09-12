#!/usr/bin/env node
import assert from "node:assert/strict";
import { mkdir } from "node:fs/promises";
import { resolve } from "node:path";
import { chromium } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";

const base = "http://127.0.0.1:8765/docs/design/v2/index.html";
const output = resolve("docs/design/v2/screenshots");
const views = [
  { name: "1440", width: 1440, height: 900 },
  { name: "1280", width: 1280, height: 800 },
];
const states = [
  ["welcome", "#first-run"],
  ["welcome-ready", "?preflight=ready#first-run"],
  ["choose", "?step=choose&preflight=ready#first-run"],
  ["recipes-empty", "?state=empty&step=choose#first-run"],
  ["docker-missing", "?preflight=missing#first-run"],
  ["ready", "?step=ready&starter=memos#first-run"],
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
      assert.deepEqual(violations.map((item) => item.id), [], `${name} ${view.name}: ${violations.map((item) => `${item.id} (${item.nodes.map((node) => node.target.join(" ")).join(", ")})`).join("; ")}`);
      const overflow = await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth);
      assert.ok(overflow <= 0, `${name} ${view.name}: ${overflow}px horizontal overflow`);
      if (view.name === "1440" && ["welcome", "choose", "docker-missing", "ready"].includes(name)) {
        await page.screenshot({ path: resolve(output, `phase11-first-run-${name}-1440.png`), fullPage: true });
      }
    }
    await context.close();
  }

  const context = await browser.newContext({ viewport: views[0] });
  const page = await context.newPage();
  await page.goto(`${base}#first-run`, { waitUntil: "networkidle" });
  await page.getByRole("button", { name: "Get started" }).click();
  assert.match(await page.locator("h1").innerText(), /reviewed app/i);
  assert.equal(await page.locator("[data-first-recipe]").count(), 3);
  await page.locator('[data-first-recipe="n8n"]').click();
  assert.equal(await page.locator("#first-run-port").inputValue(), "5678");
  await page.locator("#first-run-port").fill("5680");
  await page.getByRole("button", { name: /Review n8n install/ }).click();
  assert.equal(new URL(page.url()).hash, "#install");
  assert.match(await page.locator(".recipe-head h2").innerText(), /n8n/i);
  assert.equal(await page.locator("#install-port").inputValue(), "5680");
  await page.locator(".lab-trigger").click();
  await page.locator('[data-state="success"]').click();
  await page.locator(".panel-close").click();
  await page.getByRole("button", { name: "Finish setup" }).click();
  assert.match(await page.locator("h1").innerText(), /n8n has a home/i);

  await page.goto(`${base}?preflight=missing#first-run`, { waitUntil: "networkidle" });
  await page.getByRole("button", { name: "Check again" }).click();
  assert.match(await page.locator("h1").innerText(), /reviewed app/i);

  await page.goto(`${base}#first-run`, { waitUntil: "networkidle" });
  await page.getByRole("button", { name: "Skip for now" }).click();
  assert.equal(new URL(page.url()).hash, "#overview");
  await page.goto(`${base}#settings`, { waitUntil: "networkidle" });
  await page.getByRole("button", { name: "Start" }).click();
  assert.equal(new URL(page.url()).hash, "#first-run");
  assert.match(await page.locator("h1").innerText(), /self-hosted apps/i);
  await context.close();
} finally {
  await browser.close();
}

console.log(`Verified Phase 11 First Run: ${states.length} states at ${views.length} laptop viewports plus the full starter, recovery, skip, and re-entry paths.`);
