#!/usr/bin/env node
import assert from "node:assert/strict";
import { chromium } from "@playwright/test";

const base = "http://127.0.0.1:8765/docs/design/v2/index.html";
const browser = await chromium.launch({ channel: "chrome", headless: true });

try {
  const context = await browser.newContext({ viewport: { width: 1440, height: 900 } });
  const page = await context.newPage();
  page.on("pageerror", (error) => console.error(`pageerror: ${error.message}`));

  await page.goto(`${base}#discover`, { waitUntil: "networkidle" });
  const buttonMotion = await page.locator(".button").first().evaluate((node) => {
    const style = getComputedStyle(node);
    return { property: style.transitionProperty, duration: style.transitionDuration };
  });
  assert.match(buttonMotion.property, /transform/);
  assert.match(buttonMotion.duration, /0\.12s/);

  await page.locator("[data-toggle-filters]").first().click();
  const popoverMotion = await page.locator(".filter-popover").evaluate((node) => {
    const style = getComputedStyle(node);
    return { property: style.transitionProperty, duration: style.transitionDuration, origin: style.transformOrigin };
  });
  assert.match(popoverMotion.property, /opacity, transform/);
  assert.match(popoverMotion.duration, /0\.16s/);
  assert.match(popoverMotion.origin, /px 0px$/);
  await page.locator(".filter-popover [data-toggle-filters]").click();
  assert.equal(await page.locator(".filter-popover.is-closing").count(), 1);
  await page.waitForTimeout(420);
  assert.notEqual(await page.locator(".filter-popover").getAttribute("hidden"), null);

  await page.locator('[data-open-project="memos"]').click();
  const drawerMotion = await page.locator(".project-drawer").evaluate((node) => {
    const style = getComputedStyle(node);
    return { property: style.transitionProperty, duration: style.transitionDuration, timing: style.transitionTimingFunction };
  });
  assert.match(drawerMotion.property, /opacity, transform/);
  assert.match(drawerMotion.duration, /0\.22s/);
  assert.match(drawerMotion.timing, /cubic-bezier\(0\.32, 0\.72, 0, 1\)/);
  await page.locator(".project-drawer [data-dismiss-discover]").click();
  assert.equal(await page.locator(".project-drawer.is-closing").count(), 1);
  await page.waitForTimeout(420);
  assert.equal(await page.locator(".project-drawer").count(), 0);

  await page.locator(".lab-trigger").click();
  assert.equal(await page.locator("#prototype-panel").getAttribute("hidden"), null);
  await page.locator(".panel-close").click();
  assert.equal(await page.locator("#prototype-panel.is-closing").count(), 1);
  await page.waitForTimeout(420);
  assert.notEqual(await page.locator("#prototype-panel").getAttribute("hidden"), null);

  await page.goto(`${base}?app=memos&section=manage#my-apps`, { waitUntil: "networkidle" });
  await page.locator('[data-open-app-dialog="uninstall"]').click();
  const dialogMotion = await page.locator(".app-dialog").evaluate((node) => {
    const style = getComputedStyle(node);
    return { property: style.transitionProperty, duration: style.transitionDuration, origin: style.transformOrigin };
  });
  assert.match(dialogMotion.property, /opacity, transform/);
  assert.match(dialogMotion.duration, /0\.22s/);
  assert.match(dialogMotion.origin, /px/);
  await page.locator(".app-dialog [data-close-app-dialog]").click();
  assert.equal(await page.locator(".app-dialog.is-closing").count(), 1);
  await page.waitForTimeout(420);
  assert.equal(await page.locator(".app-dialog").count(), 0);

  await page.goto(`${base}#install`, { waitUntil: "networkidle" });
  const installFeedback = await page.evaluate(() => {
    const start = performance.now();
    document.querySelector('[data-install-action="start"]').click();
    return { elapsed: performance.now() - start, busy: Boolean(document.querySelector('.install-progress-layout [aria-busy="true"]')) };
  });
  assert.ok(installFeedback.busy);
  assert.ok(installFeedback.elapsed < 100, `install feedback took ${installFeedback.elapsed}ms`);

  await page.goto(`${base}#settings`, { waitUntil: "networkidle" });
  const doctorFeedback = await page.evaluate(() => {
    const start = performance.now();
    document.querySelector('[data-settings-action="doctor"]').click();
    return { elapsed: performance.now() - start, busy: document.querySelector('.system-status h2')?.textContent === 'Checking Docker' };
  });
  assert.ok(doctorFeedback.busy);
  assert.ok(doctorFeedback.elapsed < 100, `doctor feedback took ${doctorFeedback.elapsed}ms`);
  await context.close();

  const reducedContext = await browser.newContext({ viewport: { width: 1280, height: 800 }, reducedMotion: "reduce" });
  const reducedPage = await reducedContext.newPage();
  await reducedPage.goto(`${base}#discover`, { waitUntil: "networkidle" });
  await reducedPage.locator('[data-open-project="memos"]').click();
  assert.equal(await reducedPage.locator(".project-drawer").evaluate((node) => getComputedStyle(node).transitionProperty), "opacity");
  await reducedPage.keyboard.press("Escape");
  assert.equal(await reducedPage.locator(".project-drawer").count(), 0, "keyboard dismissal should be instant");

  await reducedPage.goto(`${base}?state=busy#settings`, { waitUntil: "networkidle" });
  assert.equal(await reducedPage.locator(".busy-orbit").evaluate((node) => getComputedStyle(node).animationName), "none");
  const reducedButtonProperties = await reducedPage.locator(".button").first().evaluate((node) => getComputedStyle(node).transitionProperty);
  assert.doesNotMatch(reducedButtonProperties, /transform/);
  await reducedPage.keyboard.press("Control+k");
  assert.equal(await reducedPage.locator(".toast").evaluate((node) => getComputedStyle(node).transitionDuration), "0s");
  await reducedContext.close();
} finally {
  await browser.close();
}

console.log("Verified Phase 13 motion: press, popover, drawer, panel, dialog, toast, immediate feedback, keyboard exits, and reduced-motion substitutions.");
