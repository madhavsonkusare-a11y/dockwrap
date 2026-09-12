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
  ["activity", "#activity"], ["activity-loading", "?state=loading#activity"], ["activity-empty", "?state=empty#activity"], ["activity-live", "?state=busy#activity"], ["activity-failure", "?state=failure#activity"], ["activity-detail", "?event=op-102#activity"],
  ["settings", "#settings"], ["settings-loading", "?state=loading#settings"], ["settings-empty", "?state=empty#settings"], ["settings-busy", "?state=busy#settings"], ["settings-failure", "?state=failure#settings"],
  ["recovery", "#recovery"], ["recovery-loading", "?recovery=loading#recovery"], ["recovery-empty", "?recovery=empty#recovery"], ["recovery-no-containers", "?recovery=no-containers#recovery"], ["recovery-keep", "?recovery=confirm-keep#recovery"], ["recovery-delete", "?recovery=confirm-delete#recovery"], ["recovery-working", "?recovery=working#recovery"], ["recovery-success", "?recovery=success-delete#recovery"], ["recovery-mismatch", "?recovery=mismatch#recovery"], ["recovery-scan-failure", "?recovery=scan-failure#recovery"],
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
      if (view.name === "1440" && ["activity", "settings", "recovery", "recovery-delete", "recovery-mismatch"].includes(name)) await page.screenshot({ path: resolve(output, `phase12-${name}-1440.png`), fullPage: true });
    }
    await context.close();
  }

  const context = await browser.newContext({ viewport: views[0] });
  const page = await context.newPage();
  await page.goto(`${base}#activity`, { waitUntil: "networkidle" });
  await page.locator('[data-activity-filter="failed"]').click();
  assert.equal(await page.locator(".activity-event").count(), 2);
  await page.locator("#activity-search").fill("Memos");
  await page.getByRole("button", { name: "Clear filters" }).click();
  assert.equal(await page.locator(".activity-event").count(), 5);
  await page.locator('[data-activity-event="op-102"]').click();
  assert.equal(await page.locator(".activity-event-detail").count(), 1);

  await page.goto(`${base}#settings`, { waitUntil: "networkidle" });
  await page.getByRole("button", { name: "Run Docker check" }).click();
  assert.match(await page.locator(".system-summary h2").innerText(), /Checking Docker/);
  await page.waitForTimeout(650);
  assert.match(await page.locator(".system-summary h2").innerText(), /Ready for managed apps/);
  await page.getByRole("button", { name: "Scan again" }).click();
  assert.equal(new URL(page.url()).hash, "#recovery");
  assert.match(await page.locator("h2").first().innerText(), /Inspecting retained setups/);
  await page.waitForTimeout(650);
  assert.match(await page.locator(".recovery-identity h2").innerText(), /Memos/);

  await page.getByRole("button", { name: "Review deletion" }).click();
  const destructive = page.getByRole("button", { name: "Delete setup and data" });
  assert.equal(await destructive.isDisabled(), true);
  await page.locator("#recovery-confirmation").fill("Memos");
  assert.equal(await destructive.isEnabled(), true);
  await destructive.click();
  assert.match(await page.locator("h2").first().innerText(), /Re-verifying/);
  await page.waitForTimeout(750);
  assert.match(await page.locator(".recovery-result h2").innerText(), /data deleted/i);

  await page.goto(`${base}?recovery=mismatch#recovery`, { waitUntil: "networkidle" });
  assert.equal(await page.locator("[data-recovery-action='confirm-delete']").count(), 0);
  assert.match(await page.locator(".failure-status").innerText(), /blocked/i);
  await context.close();
} finally {
  await browser.close();
}

console.log(`Verified Phase 12 secondary surfaces: ${states.length} states at ${views.length} laptop viewports plus filtering, diagnostics, recovery confirmation, deletion, and refusal paths.`);
