#!/usr/bin/env node
// Behavioural proof for the Phase 14 findings register. Each block drives the
// prototype the way a user would and asserts the corrected behaviour.
//
// Serve the repository root first:  python -m http.server 8765 --bind 127.0.0.1
import assert from "node:assert/strict";
import { chromium } from "@playwright/test";

const base = "http://127.0.0.1:8765/docs/design/v2/index.html";
const browser = await chromium.launch({ channel: "chrome", headless: true });
const page = await (await browser.newContext({ viewport: { width: 1280, height: 800 }, reducedMotion: "reduce" })).newPage();
// A hash-only change to the current URL is a same-document navigation, which would
// carry state (an open panel, say) from one check into the next. Always load fresh.
const open = async (suffix) => { await page.goto("about:blank"); await page.goto(`${base}${suffix}`, { waitUntil: "networkidle" }); };
const passed = [];

try {
  // F-02 - connecting from a catalog entry never pre-fills a fabricated address.
  // Drive the real path (the card's Connect button), not the prototype's deep link.
  await open("#discover");
  await page.click('[data-open-connect="immich"]');
  await page.waitForSelector("#connect-address");
  assert.equal(await page.inputValue("#connect-address"), "", "F-02: address must start empty");
  assert.match(await page.getAttribute("#connect-address", "placeholder"), /^e\.g\. /, "F-02: guess shown as an example");
  assert.equal(await page.evaluate(() => document.activeElement?.id), "connect-address", "F-02: address is focused");
  passed.push("F-02 no fabricated address");

  // F-07 - an empty name is explained visibly, not only flagged.
  await page.fill("#connect-name", "");
  await page.fill("#connect-address", "http://photos.home:2283");
  await page.click('.connect-form button[type="submit"]');
  assert.equal(await page.getAttribute("#connect-name", "aria-invalid"), "true");
  assert.match(await page.textContent("#connect-feedback"), /name/i, "F-07: visible reason");
  assert.equal(await page.isVisible("#connect-feedback"), true);
  passed.push("F-07 name validation is visible");

  // F-08 - the global Connect path does not land on an unrelated app.
  await open("#my-apps");
  await page.click('[data-open-connect=""]');
  await page.fill("#connect-name", "Photos");
  await page.fill("#connect-address", "http://photos.home:2283");
  await page.click('.connect-form button[type="submit"]');
  await page.waitForSelector(".condition-banner.success");
  assert.match(await page.textContent(".condition-banner.success"), /Photos saved as a linked app/, "F-08: names what was saved");
  passed.push("F-08 global connect is honest about selection");

  // F-09 - a refused port cannot be retried unchanged, and the refusal is explained.
  await open("?state=failure&failure=port_in_use#install");
  assert.equal(await page.getAttribute("#failure-port", "aria-invalid"), "true", "F-09: failing port is marked");
  assert.equal(await page.isVisible("#failure-port-error"), true);
  await page.click('[data-install-action="review"]');
  assert.equal(await page.isVisible("#failure-port"), true, "F-09: stays on the failure until the port changes");
  await page.fill("#failure-port", "5231");
  assert.equal(await page.isVisible("#failure-port-error"), false, "F-09: message clears for a new port");
  await page.click('[data-install-action="review"]');
  assert.equal(await page.inputValue("#install-port"), "5231", "F-09: the new port carries into review");
  passed.push("F-09 port retry requires a different port");

  // F-10 - an error is a fault, not a pause.
  await open("?app=linkding#my-apps");
  assert.equal(await page.locator('.app-row[aria-selected="true"] .row-status').evaluate((el) => el.classList.contains("danger")), true, "F-10");
  passed.push("F-10 error status uses danger");

  // F-11 - one ember-filled action on My Apps.
  await open("#my-apps");
  assert.equal(await page.locator(".screen .button.primary:visible").count(), 1, "F-11");
  passed.push("F-11 one primary on My Apps");

  // F-12 - buttons never wrap.
  await open("#settings");
  const wrapped = await page.evaluate(() => [...document.querySelectorAll(".button")].filter((b) => b.getClientRects().length && b.getBoundingClientRect().height > parseFloat(getComputedStyle(b).minHeight) + 2).map((b) => b.textContent.trim()));
  assert.deepEqual(wrapped, [], `F-12: ${wrapped.join(", ")}`);
  passed.push("F-12 no wrapped buttons");

  // F-13 - the search concept is labelled before activation.
  assert.match(await page.textContent(".command-button"), /Concept/, "F-13");
  passed.push("F-13 search labelled as concept");

  // F-20 - focused tasks keep their parent marked.
  await open("#recovery");
  assert.equal(await page.getAttribute('.nav-item[data-route="settings"]', "aria-current"), "location", "F-20 recovery");
  await open("#install");
  assert.equal(await page.getAttribute('.nav-item[data-route="discover"]', "aria-current"), "location", "F-20 install");
  assert.match(await page.textContent("#route-label"), /Discover \/ Install/);
  passed.push("F-20 parent section stays marked");

  // F-06 - Tab never leaves an open modal.
  await open("#overview");
  await page.click(".lab-trigger");
  for (let i = 0; i < 14; i += 1) {
    await page.keyboard.press("Tab");
    assert.equal(await page.evaluate(() => document.querySelector("#prototype-panel").contains(document.activeElement)), true, `F-06: focus left the panel on Tab ${i + 1}`);
  }
  passed.push("F-06 panel traps focus");

  // F-25 - the rail's Docker card is the link its chevron promises.
  await open("#overview");
  await page.click(".environment-card");
  await page.waitForFunction(() => location.hash === "#settings");
  passed.push("F-25 Docker card opens Settings");

  // Narrow-viewport truncation introduced by the 13px floor stays fixed.
  await open("#my-apps");
  const clipped = await page.evaluate(() => [...document.querySelectorAll(".app-row-copy strong, .environment-card small")].filter((el) => el.scrollWidth > el.clientWidth + 1).map((el) => el.textContent.trim()));
  assert.deepEqual(clipped, [], `truncated at 1280: ${clipped.join(", ")}`);
  passed.push("No truncated app names or engine version at 1280");

  // F-23 - the spacing scale is complete; the welcome groups are separated.
  await open("?step=welcome&preflight=ready#first-run");
  const gap = await page.evaluate(() => parseFloat(getComputedStyle(document.querySelector(".welcome-principles")).marginTop));
  assert.equal(gap, 28, "F-23: --v2-space-7 resolves");
  passed.push("F-23 welcome groups separated");
} finally {
  await browser.close();
}
console.log(`Verified ${passed.length} Phase 14 corrections:\n  ${passed.join("\n  ")}`);
