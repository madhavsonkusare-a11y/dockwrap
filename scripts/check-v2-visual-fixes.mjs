#!/usr/bin/env node
// Behavioural proof for the Phase 15 visual register (docs/design/v2/visual-verification.md).
// Each block loads the state the finding came from and asserts the corrected result.
//
// Serve the repository root first:  python -m http.server 8765 --bind 127.0.0.1
import assert from "node:assert/strict";
import { chromium } from "@playwright/test";

const base = "http://127.0.0.1:8765/docs/design/v2/index.html";
const browser = await chromium.launch({ channel: "chrome", headless: true });
const context = await browser.newContext({ viewport: { width: 1280, height: 800 }, reducedMotion: "reduce" });
const page = await context.newPage();
const open = async (suffix, target = page) => { await target.goto("about:blank"); await target.goto(`${base}${suffix}`, { waitUntil: "networkidle" }); };
// Text cut by its own box, horizontally or vertically, for every match of a selector.
const clipped = (selector, target = page) => target.evaluate((s) => [...document.querySelectorAll(s)]
  .filter((el) => el.getClientRects().length && (el.scrollWidth > el.clientWidth + 1 || el.scrollHeight > el.clientHeight + 1))
  .map((el) => el.textContent.trim().slice(0, 60)), selector);
const inView = (selector, target = page) => target.evaluate((s) => {
  const el = document.querySelector(s); const r = el.getBoundingClientRect();
  const hit = document.elementFromPoint(r.left + r.width / 2, r.top + r.height / 2);
  return r.top >= 0 && r.bottom <= innerHeight && r.left >= 0 && r.right <= innerWidth && el.contains(hit);
}, selector);
const passed = [];

try {
  // V-01 / V-06 - install evidence is machine truth; with a digest-pinned image and a
  // long Docker error it wraps rather than truncating.
  await open("?stress=1&evidence=1#install");
  assert.deepEqual(await clipped(".evidence-grid code, .change-list code"), [], "V-01: evidence truncated");
  assert.match(await page.textContent(".evidence-grid"), /@sha256:[0-9a-f]{12}/, "V-06: digest shown");
  passed.push("V-01/V-06 install evidence wraps with a pinned digest");

  // V-02 - starter version and licence fit their card.
  await open("?step=choose&preflight=ready#first-run");
  assert.deepEqual(await clipped(".starter-card em"), [], "V-02: starter facts truncated");
  passed.push("V-02 starter facts wrap");

  // V-09 - an unselected starter card answers hover.
  const card = page.locator(".starter-card:not(.selected)").first();
  const before = await card.evaluate((el) => getComputedStyle(el).backgroundColor);
  await card.hover();
  await page.waitForTimeout(250);
  assert.notEqual(await card.evaluate((el) => getComputedStyle(el).backgroundColor), before, "V-09: no hover state");
  passed.push("V-09 starter card hover");

  // V-03 / V-05 / V-04 - long operational messages and names stay readable in full.
  await open("?stress=1#overview");
  const attention = await page.$$eval(".attention-row small", (els) => els.map((el) => [el.textContent.trim(), el.title]));
  assert.ok(attention.length > 0);
  for (const [textValue, title] of attention) assert.equal(title, textValue, "V-03: attention message has its full value");
  await open("?stress=1#my-apps");
  const names = await page.$$eval(".app-row-copy strong", (els) => els.map((el) => [el.textContent.trim(), el.title]));
  for (const [textValue, title] of names) assert.equal(title, textValue, "V-04: app name has its full value");
  passed.push("V-03/V-04/V-05 long messages and names recoverable");

  // V-07 / V-08 - catalog names keep their value; recipe-card mono lines keep their glyphs.
  await open("?stress=1#discover");
  const cards = await page.$$eval(".catalog-card h2, .catalog-card .category", (els) => els.every((el) => el.title === el.textContent.trim()));
  assert.equal(cards, true, "V-07: catalog name or category missing its title");
  const shaved = await page.$$eval(".featured-recipe-card small", (els) => els.filter((el) => el.scrollHeight > el.clientHeight + 1).length);
  assert.equal(shaved, 0, "V-08: recipe card glyphs clipped");
  passed.push("V-07/V-08 catalog names recoverable, recipe glyphs whole");

  // V-13 / V-15 - real logos from the widest to the tallest stay inside their frame,
  // and a project with no stated architecture shows no dangling separator.
  const spilled = await page.$$eval(".app-icon img", (imgs) => imgs.filter((img) => {
    const i = img.getBoundingClientRect(); const f = img.parentElement.getBoundingClientRect();
    return i.top < f.top - 1 || i.bottom > f.bottom + 1 || i.left < f.left - 1 || i.right > f.right + 1;
  }).map((img) => img.getAttribute("src").split("/").pop()));
  assert.deepEqual(spilled, [], "V-13: logo outside its frame");
  assert.equal(await page.$$eval(".card-facts span", (spans) => spans.filter((s) => !s.textContent.trim()).length), 0, "V-15: empty fact rendered");
  passed.push("V-13/V-15 logos contained at every aspect ratio; no empty facts");

  // V-10 - a search with no matches offers no pages and speaks product language.
  await open("?q=zzzz-no-match#discover");
  assert.equal(await page.isVisible("#catalog-no-results"), true);
  assert.equal(await page.isVisible("#catalog-pagination"), false, "V-10: pagination shown for zero results");
  assert.doesNotMatch(await page.textContent("#catalog-no-results"), /fixture/i, "V-10: prototype language");
  await page.fill("#catalog-search", "memos");
  assert.equal((await page.textContent("#catalog-page-count")).trim(), "Showing 1 matching project · Page 1", "V-10: narrowed count");
  await page.fill("#catalog-search", "");
  assert.match(await page.textContent("#catalog-page-count"), /^Showing 6 of 1,672/, "V-10: unfiltered count");
  passed.push("V-10 no-results hides pagination; counts follow the results");

  // V-11 - status and recovery guidance wraps instead of truncating.
  for (const [suffix, selector] of [["?state=busy#overview", ".environment-card small"], ["?state=failure#overview", ".environment-card small"], ["?state=success#overview", ".calm-state small"], ["?state=failure#settings", ".doctor-results small"]]) {
    await open(suffix);
    assert.deepEqual(await clipped(selector), [], `V-11: ${selector} truncated in ${suffix}`);
  }
  passed.push("V-11 status guidance wraps");

  // V-12 / V-14 - at the minimum viewport and at a 1920x1080 laptop at 150% (maximised,
  // 1280x640 CSS px), More filters opens wholly on screen and a drawer's commit stays
  // visible and clickable.
  const short = await (await browser.newContext({ viewport: { width: 1280, height: 640 }, reducedMotion: "reduce" })).newPage();
  for (const target of [page, short]) {
    const size = `${target.viewportSize().width}x${target.viewportSize().height}`;
    await open("#discover", target);
    await target.click(".filter-trigger");
    assert.equal(await inView("#catalog-filter-panel", target), true, `V-12: filter popover off screen at ${size}`);
    assert.equal(await target.evaluate(() => document.activeElement?.closest("#catalog-filter-panel") !== null), true, "V-12: focus moved into the popover");
    await open("?project=memos&drawer=detail#discover", target);
    assert.equal(await inView(".project-drawer .drawer-actions .button.primary", target), true, `V-14: drawer commit hidden at ${size}`);
  }
  passed.push("V-12 filter popover on screen; V-14 drawer commit pinned (1280x800, 1280x640)");

  // V-16 - in the short window, a focused task's commit is on screen and clickable, and
  // a refused port's field is focused clear of the pinned row.
  for (const suffix of ["?state=failure&failure=port_in_use#install", "?recovery=success-keep#recovery", "?recovery=success-delete#recovery", "?recovery=mismatch#recovery", "?recovery=scan-failure#recovery", "?step=welcome&preflight=missing#first-run"]) {
    await open(suffix, short);
    assert.equal(await inView(".screen .button.primary", short), true, `V-16: commit off screen at 1280x640 in ${suffix}`);
  }
  await open("?state=failure&failure=port_in_use#install", short);
  await short.click(".result-actions .button.primary");
  const field = await short.evaluate(() => ({ id: document.activeElement.id, clear: document.activeElement.getBoundingClientRect().bottom <= document.querySelector(".result-actions").getBoundingClientRect().top }));
  assert.deepEqual(field, { id: "failure-port", clear: true }, "V-16: refused port hidden under the pinned row");
  await open("?state=failure&failure=port_in_use#install");
  assert.equal(await page.evaluate(() => getComputedStyle(document.querySelector(".result-actions")).position), "static", "V-16: the row pins only in short windows");
  passed.push("V-16 focused commits pinned at 1280x640; unchanged at 1280x800");
} finally {
  await browser.close();
}
console.log(`Verified ${passed.length} Phase 15 corrections:\n  ${passed.join("\n  ")}`);
